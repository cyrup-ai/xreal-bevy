use anyhow::Result;
use bevy::prelude::*;
use imu_fusion::{Fusion, FusionAhrsSettings, FusionVector};
use ar_drivers::GlassesEvent;
use crate::driver::poll_events;
use instant::Instant;
use std::time::Duration;
use quaternion_core::slerp;
use crossbeam_channel::{Receiver, Sender};

#[derive(Copy, Clone, Default, Resource)]
pub struct Orientation {
    pub quat: Quat,
}

#[derive(Copy, Clone, Resource)]
pub enum CalibrationState {
    Idle,
    Calibrating { start_time: Instant, gyro_count: usize, accel_count: usize, mag_count: usize, gyro_samples: [[f32; 3]; 5000], accel_samples: [[f32; 3]; 5000], mag_samples: [[f32; 3]; 5000] },
    Calibrated { gyro_bias: [f32; 3], accel_bias: [f32; 3], mag_bias: [f32; 3] },
}

impl Default for CalibrationState {
    fn default() -> Self {
        CalibrationState::Idle
    }
}
#[derive(Copy, Clone)]
pub enum Command {
    SetRollLock(bool),
    StartCalibration,
    SetBrightness(u8),
    SetDisplayMode(bool),
}

#[derive(Copy, Clone)]
pub enum Data {
    Orientation(Quat),
    CalState(CalibrationState),
}


/// Bevy AsyncComputeTaskPool compatible IMU polling function
pub async fn poll_imu_bevy(rx_command: Receiver<Command>, tx_data: Sender<Data>) -> Result<()> {
    
    // Create a separate async task for the actual IMU polling
    let imu_task = async move {
        let glasses = crate::driver::init_glasses()?;
        let ahrs_settings = FusionAhrsSettings::new();
        let mut fusion = Fusion::new(1000, ahrs_settings);
        let mut last_ts: u64 = 0;
        let mut last_accel = [0.0f32; 3];
        let mut last_gyro = [0.0f32; 3];
        let mut last_mag = [0.0f32; 3];
        let mut roll_lock = false;
        let mut gyro_bias = [0.0f32; 3];
        let mut accel_bias = [0.0f32; 3];
        let mut mag_bias = [0.0f32; 3];
        let alpha = 0.05f32;
        let mut smoothed_q: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
        let mut cal_state = CalibrationState::default();
        
        loop {
            // Check for commands from the main thread
            if let Ok(cmd) = rx_command.try_recv() {
                match cmd {
                    Command::SetRollLock(b) => roll_lock = b,
                    Command::StartCalibration => cal_state = CalibrationState::Calibrating { 
                        start_time: Instant::now(), 
                        gyro_count: 0, 
                        accel_count: 0, 
                        mag_count: 0, 
                        gyro_samples: [[0.0; 3]; 5000], 
                        accel_samples: [[0.0; 3]; 5000], 
                        mag_samples: [[0.0; 3]; 5000] 
                    },
                    Command::SetBrightness(b) => { let _ = glasses.set_brightness(b); }
                    Command::SetDisplayMode(m) => { let _ = glasses.set_display_mode(m); }
                }
            }

            let events = poll_events(&glasses)?;
            let mut has_accel_gyro = false;
            let mut has_mag = false;        
            for event in events {
                match event {
                    GlassesEvent::AccGyro { accelerometer, gyroscope, timestamp } => {
                        let accel = [accelerometer.x - accel_bias[0], accelerometer.y - accel_bias[1], accelerometer.z - accel_bias[2]];
                        let gyro = [gyroscope.x - gyro_bias[0], gyroscope.y - gyro_bias[1], gyroscope.z - gyro_bias[2]];
                        last_accel = accel;
                        last_gyro = gyro;
                        let dt = if last_ts > 0 { (timestamp - last_ts) as f32 / 1_000_000.0 } else { 0.001 };
                        last_ts = timestamp;
                        
                        let gyro_vec = FusionVector { x: last_gyro[0], y: last_gyro[1], z: last_gyro[2] };
                        let accel_vec = FusionVector { x: last_accel[0], y: last_accel[1], z: last_accel[2] };
                        let mag_vec = FusionVector { x: last_mag[0], y: last_mag[1], z: last_mag[2] };
                        
                        fusion.update(gyro_vec, accel_vec, mag_vec, dt);
                        has_accel_gyro = true;
                    }
                    GlassesEvent::Magnetometer { magnetometer, .. } => {
                        last_mag = [magnetometer.x - mag_bias[0], magnetometer.y - mag_bias[1], magnetometer.z - mag_bias[2]];
                        has_mag = true;
                    }
                    _ => {}
                }
            }
            
            if has_accel_gyro {
                let fusion_quat = fusion.quaternion();
                let mut q = Quat::from_xyzw(fusion_quat.x, fusion_quat.y, fusion_quat.z, fusion_quat.w);
                if roll_lock {
                    let euler = q.to_euler(EulerRot::YXZ);
                    q = Quat::from_euler(EulerRot::YXZ, euler.0, euler.1, 0.0);
                }
                
                let curr_q = (q.w, [q.x, q.y, q.z]);
                let smooth_q = (smoothed_q[0], [smoothed_q[1], smoothed_q[2], smoothed_q[3]]);
                let result_q = slerp(smooth_q, curr_q, alpha);
                smoothed_q = [result_q.0, result_q.1[0], result_q.1[1], result_q.1[2]];
                let send_q = Quat::from_xyzw(smoothed_q[1], smoothed_q[2], smoothed_q[3], smoothed_q[0]);
                
                // Send data using sync channel
                if tx_data.send(Data::Orientation(send_q)).is_err() {
                    return Err(anyhow::anyhow!("Failed to send orientation"));
                }
            }
            
            // Handle calibration state updates
            match &mut cal_state {
                CalibrationState::Calibrating { start_time, gyro_count, accel_count, mag_count, gyro_samples, accel_samples, mag_samples } => {
                    if has_accel_gyro && *gyro_count < 5000 {
                        gyro_samples[*gyro_count] = last_gyro;
                        accel_samples[*accel_count] = last_accel;
                        *gyro_count += 1;
                        *accel_count += 1;
                    }
                    if has_mag && *mag_count < 5000 {
                        mag_samples[*mag_count] = last_mag;
                        *mag_count += 1;
                    }
                    if start_time.elapsed() > Duration::from_secs(5) {
                        let g_count = *gyro_count as f32;
                        if g_count > 0.0 {
                            let gyro_bias_x = gyro_samples[0..*gyro_count].iter().map(|s| s[0]).sum::<f32>() / g_count;
                            let gyro_bias_y = gyro_samples[0..*gyro_count].iter().map(|s| s[1]).sum::<f32>() / g_count;
                            let gyro_bias_z = gyro_samples[0..*gyro_count].iter().map(|s| s[2]).sum::<f32>() / g_count;
                            gyro_bias = [gyro_bias_x, gyro_bias_y, gyro_bias_z];
                        }

                        let a_count = *accel_count as f32;
                        if a_count > 0.0 {
                            let accel_bias_x = accel_samples[0..*accel_count].iter().map(|s| s[0]).sum::<f32>() / a_count;
                            let accel_bias_y = accel_samples[0..*accel_count].iter().map(|s| s[1]).sum::<f32>() / a_count;
                            let accel_bias_z = accel_samples[0..*accel_count].iter().map(|s| s[2]).sum::<f32>() / a_count - 9.81;
                            accel_bias = [accel_bias_x, accel_bias_y, accel_bias_z];
                        }

                        let m_count = *mag_count as f32;
                        if m_count > 0.0 {
                            let mag_bias_x = mag_samples[0..*mag_count].iter().map(|s| s[0]).sum::<f32>() / m_count;
                            let mag_bias_y = mag_samples[0..*mag_count].iter().map(|s| s[1]).sum::<f32>() / m_count;
                            let mag_bias_z = mag_samples[0..*mag_count].iter().map(|s| s[2]).sum::<f32>() / m_count;
                            mag_bias = [mag_bias_x, mag_bias_y, mag_bias_z];
                        }

                        cal_state = CalibrationState::Calibrated { gyro_bias, accel_bias, mag_bias };
                        if tx_data.send(Data::CalState(cal_state)).is_err() {
                            return Err(anyhow::anyhow!("Failed to send cal state"));
                        }
                    }
                }
                _ => {}
            }

            // Use async_std sleep for compatibility with AsyncComputeTaskPool
            async_std::task::sleep(Duration::from_millis(1)).await;
        }
    };
    
    // Run the IMU task
    imu_task.await
}
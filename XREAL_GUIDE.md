# XREAL AR Glasses Development Guide

## Overview

This guide provides comprehensive technical documentation for developing applications with XREAL AR glasses, covering hardware protocols, driver integration, software architecture, and performance optimization techniques.

## Hardware Architecture

### USB-C Communication Protocol

XREAL glasses use a sophisticated USB-C communication architecture:

- **Dual Bus Design**: Utilizes both USB2 and USB3 buses simultaneously
- **DisplayPort Integration**: 2-lane DisplayPort support, limiting bandwidth to 1080p@60
- **HID Communication**: Uses Human Interface Device protocol for device control and sensor data

### IMU Data Communication

The IMU system operates through a specialized HID-based protocol:

#### Activation Command
```rust
// Command to enable IMU streaming
let enable_imu = [0x02, 0x19, 0x01];
```

#### Data Packet Structure
- **Timestamps**: Separate timestamps for gyroscope and accelerometer
- **Precision Control**: Multiplier and divisor values for precise sensor calculations
- **Sensor Readings**: Raw accelerometer, gyroscope, and magnetometer data

#### Conversion Formulas
```rust
// Gyroscope (rad/s)
let gyro_rad_per_sec = (raw_value * multiplier) / divisor;

// Accelerometer (m/s²)
let accel_m_per_s2 = (raw_value * multiplier) / divisor;
```

### Display Control Protocol

Display management uses ASCII-based commands with specific formatting:

- **Command Format**: Start/end text markers
- **Mode Control**: Support for 2D, 3D, and 3D@72Hz modes
- **Heartbeat Requirement**: Periodic "SDK works" messages to maintain display functionality
- **Calibration Integration**: Retrieval of device-specific calibration files

## Driver Layer (ar-drivers-rs)

### Supported Devices
- XREAL Air, Air 2, Air 2 Pro
- XREAL Light
- Rokid Air and Max
- Grawoow G530 (Metavision M53)
- Mad Gaze Glow

### Architecture Principles

#### Device-Agnostic Design
```rust
// Unified interface across different AR glasses
pub trait ARGlasses {
    fn get_sensor_data(&self) -> Result<SensorData>;
    fn set_display_mode(&self, mode: DisplayMode) -> Result<()>;
    fn set_brightness(&self, level: u8) -> Result<()>;
}
```

#### Performance Optimizations
- **HIDAPI Usage**: More performant than libusb for HID communication
- **Static Linking**: Enables portable executables
- **libudev Integration**: Efficient device detection and management

### Core Capabilities
- Basic sensor data retrieval
- Display setup and configuration
- Mode switching (particularly 3D SBS mode)
- Device enumeration and identification

## Software Architecture

### Bevy ECS Integration

#### AsyncComputeTaskPool Pattern
```rust
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::ecs::world::CommandQueue;

#[derive(Component)]
struct IMUTask(Task<CommandQueue>);

fn spawn_imu_task(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        // Async IMU processing
        if let Ok(orientation) = poll_imu_data().await {
            command_queue.push(move |world: &mut World| {
                // Update ECS world with new orientation
            });
        }
        
        command_queue
    });
    
    commands.spawn(IMUTask(task));
}
```

#### Resource Management
```rust
#[derive(Resource)]
struct Orientation {
    pub quat: Quat,
}

#[derive(Resource)]
struct CalibrationState {
    // Calibration data and state
}
```

### Thread Safety Considerations

#### Channel Communication
```rust
// Use crossbeam for Sync compatibility with Bevy resources
use crossbeam_channel::{bounded, Receiver, Sender};

#[derive(Resource, Deref)]
struct DataReceiver(Receiver<OrientationData>);

#[derive(Resource)]
struct CommandSender(Sender<IMUCommand>);
```

#### Async-to-ECS Communication
```rust
fn handle_imu_tasks(
    mut commands: Commands,
    mut tasks: Query<&mut IMUTask>,
) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for mut task in &mut tasks {
        // Non-blocking poll
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
        }
    }
}
```

## Performance & Jitter Elimination

### Async-First Design Principles

#### Eliminate Blocking Operations
- **No spawn_blocking**: Use AsyncComputeTaskPool exclusively
- **No block_on** (except for polling): All operations must be truly async
- **No synchronous file I/O**: Convert all fs operations to async

#### Optimal Threading Model
```rust
// Main thread: Bevy systems and rendering
// AsyncComputeTaskPool: IMU polling, screen capture, system calls
// Communication: Lock-free channels for data flow
```

### Sensor Fusion Optimization

#### IMU Processing Pipeline
```rust
use imu_fusion::{Fusion, FusionAhrsSettings, FusionVector};

async fn process_imu_data() -> Result<Quat> {
    let ahrs_settings = FusionAhrsSettings::new();
    let mut fusion = Fusion::new(1000, ahrs_settings); // 1000Hz sample rate
    
    // Process sensor data with bias compensation
    let gyro_vec = FusionVector { 
        x: gyro_x - gyro_bias[0], 
        y: gyro_y - gyro_bias[1], 
        z: gyro_z - gyro_bias[2] 
    };
    
    fusion.update(gyro_vec, accel_vec, mag_vec, dt);
    let quat = fusion.quaternion();
    
    Ok(Quat::from_xyzw(quat.x, quat.y, quat.z, quat.w))
}
```

#### Quaternion Smoothing
```rust
use quaternion_core::slerp;

// Apply smoothing to reduce jitter
let alpha = 0.05f32; // Smoothing factor
let smooth_q = slerp(previous_quat, current_quat, alpha);
```

#### Roll Lock Implementation
```rust
fn apply_roll_lock(quat: Quat) -> Quat {
    let euler = quat.to_euler(EulerRot::YXZ);
    // Lock roll to 0 for text stability
    Quat::from_euler(EulerRot::YXZ, euler.0, euler.1, 0.0)
}
```

### Screen Capture Optimization

#### Zero-Allocation Patterns
```rust
pub struct ScreenCaptures {
    // Pre-allocated buffer pool
    rgba_buffer: Vec<u8>,
    buffer_capacity: usize,
}

impl ScreenCaptures {
    fn new() -> Result<Self> {
        // Pre-allocate for 4K RGBA to avoid hot-path allocations
        const MAX_BUFFER_SIZE: usize = 3840 * 2160 * 4;
        let rgba_buffer = Vec::with_capacity(MAX_BUFFER_SIZE);
        
        Ok(Self {
            rgba_buffer,
            buffer_capacity: MAX_BUFFER_SIZE,
        })
    }
}
```

#### Adaptive Frame Rate Detection
```rust
fn detect_optimal_framerate() -> u32 {
    // Priority: 120Hz (XREAL 2 Pro), 90Hz (XREAL 2), 72Hz (Air), 60Hz (fallback)
    if let Ok(output) = std::process::Command::new("system_profiler")
        .args(&["SPDisplaysDataType"])
        .output() 
    {
        let display_info = String::from_utf8_lossy(&output.stdout);
        
        if display_info.contains("120") {
            return 120; // XREAL 2 Pro
        } else if display_info.contains("90") {
            return 90;  // XREAL 2
        } else if display_info.contains("72") {
            return 72;  // XREAL Air series
        }
    }
    
    60 // Safe fallback
}
```

## Implementation Details

### IMU Calibration System

#### Calibration Process
```rust
#[derive(Copy, Clone, Resource)]
pub enum CalibrationState {
    Idle,
    Calibrating { 
        start_time: Instant, 
        gyro_count: usize, 
        accel_count: usize, 
        mag_count: usize, 
        gyro_samples: [[f32; 3]; 5000], 
        accel_samples: [[f32; 3]; 5000], 
        mag_samples: [[f32; 3]; 5000] 
    },
    Calibrated { 
        gyro_bias: [f32; 3], 
        accel_bias: [f32; 3], 
        mag_bias: [f32; 3] 
    },
}
```

#### Bias Calculation
```rust
// Collect 5000 samples over 5 seconds
if start_time.elapsed() > Duration::from_secs(5) {
    let gyro_bias_x = gyro_samples[0..gyro_count].iter()
        .map(|s| s[0]).sum::<f32>() / gyro_count as f32;
    
    // Compensate for gravity in Z-axis
    let accel_bias_z = accel_samples[0..accel_count].iter()
        .map(|s| s[2]).sum::<f32>() / accel_count as f32 - 9.81;
    
    cal_state = CalibrationState::Calibrated { gyro_bias, accel_bias, mag_bias };
}
```

### Error Handling Patterns

#### Robust Error Propagation
```rust
use anyhow::Result;

// All fallible operations use anyhow::Result
async fn poll_imu() -> Result<()> {
    let glasses = init_glasses()?;
    
    loop {
        let events = poll_events(&glasses).await?;
        
        for event in events {
            match event {
                GlassesEvent::AccGyro { .. } => {
                    // Process with error handling
                    if tx_data.send(orientation).await.is_err() {
                        return Err(anyhow::anyhow!("Failed to send orientation"));
                    }
                }
                _ => {}
            }
        }
    }
}
```

#### Channel Failure Management
```rust
// Bounded channels prevent memory accumulation
let (tx, rx) = crossbeam_channel::bounded::<Data>(1);

// Handle channel failures gracefully
if let Err(_) = tx.try_send(data) {
    warn!("Channel full, dropping frame");
    // Continue processing rather than failing
}
```

## Development Best Practices

### Async Programming Guidelines

#### CommandQueue Pattern
```rust
async fn async_task() -> CommandQueue {
    let mut command_queue = CommandQueue::default();
    
    // Perform async work
    let result = perform_async_operation().await;
    
    // Use CommandQueue for deferred world updates
    command_queue.push(move |world: &mut World| {
        // Apply results to ECS world
        world.resource_mut::<SomeResource>().update(result);
    });
    
    command_queue
}
```

#### Resource Lifecycle Management
```rust
impl Drop for ScreenCaptures {
    fn drop(&mut self) {
        if let Some(ref mut capturer) = self.capturer {
            capturer.stop_capture();
        }
    }
}
```

### Testing Strategies

#### Unit Testing Async Functions
```rust
#[tokio::test]
async fn test_imu_processing() {
    let (tx, rx) = crossbeam_channel::bounded(10);
    
    // Test async IMU processing
    let result = process_test_imu_data(tx).await;
    
    assert!(result.is_ok());
    assert!(rx.try_recv().is_ok());
}
```

#### Integration Testing
```rust
#[test]
fn test_bevy_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .insert_resource(Orientation::default())
       .add_systems(Update, update_camera_from_orientation);
    
    app.update();
    
    // Verify system integration
}
```

### Performance Monitoring

#### FPS Diagnostics
```rust
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

fn log_fps(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.average() {
            trace!("FPS: {}", value);
        }
    }
}
```

#### Fixed Timestep Scheduling
```rust
// 1ms fixed timestep for consistent updates
app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(1)))
   .add_systems(FixedUpdate, (update_orientation, update_camera));
```

## Common Issues and Solutions

### Thread Safety
**Issue**: `std::sync::mpsc` channels are `!Sync`
**Solution**: Use `crossbeam_channel` for Bevy resource compatibility

### Jitter Reduction
**Issue**: Sensor data jitter affecting display stability
**Solution**: Apply quaternion smoothing with slerp interpolation

### Performance Bottlenecks
**Issue**: Blocking operations causing frame drops
**Solution**: Convert all operations to use AsyncComputeTaskPool

### Memory Management
**Issue**: Hot-path allocations during screen capture
**Solution**: Pre-allocate buffer pools for maximum expected data sizes

## Future Considerations

### Hardware Evolution
- Support for newer XREAL models with higher refresh rates
- Integration with upcoming AR glasses from other manufacturers
- Adaptation to new USB-C protocol versions

### Software Optimization
- GPU-accelerated sensor fusion
- Advanced prediction algorithms for motion compensation
- Real-time performance profiling and adjustment

### Platform Expansion
- Windows and Linux compatibility layers
- Mobile platform integration (iOS/Android)
- VR headset protocol adaptation

This guide provides the foundation for building jitter-free, high-performance XREAL AR applications using modern Rust async patterns and Bevy ECS architecture.
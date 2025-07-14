use anyhow::Result;
use ar_drivers::{any_glasses, GlassesEvent, ARGlasses, DisplayMode};
use std::sync::{Arc, Mutex};

pub struct GlassesDevice {
    inner: Arc<Mutex<Box<dyn ARGlasses>>>,
}

impl GlassesDevice {
    #[inline]
    pub fn new() -> Result<Self> {
        let glasses = any_glasses()?;
        Ok(Self {
            inner: Arc::new(Mutex::new(glasses)),
        })
    }
    
    #[inline]
    pub fn set_display_mode(&self, enabled: bool) -> Result<()> {
        match self.inner.lock() {
            Ok(mut glasses) => {
                let mode = if enabled { DisplayMode::Stereo } else { DisplayMode::SameOnBoth };
                glasses.set_display_mode(mode).map_err(|e| anyhow::anyhow!("Display mode error: {}", e))
            }
            Err(_) => Err(anyhow::anyhow!("Failed to acquire glasses lock"))
        }
    }
    
    #[inline]
    pub fn set_brightness(&self, _level: u8) -> Result<()> {
        // Brightness control not available in ar-drivers API
        Ok(())
    }
    
    #[inline]
    pub fn poll_events(&self) -> Result<Vec<GlassesEvent>> {
        match self.inner.lock() {
            Ok(mut glasses) => {
                let mut events = Vec::with_capacity(16);
                for _ in 0..16 {
                    match glasses.read_event() {
                        Ok(event) => events.push(event),
                        Err(_) => break,
                    }
                }
                Ok(events)
            }
            Err(_) => Err(anyhow::anyhow!("Failed to acquire glasses lock"))
        }
    }
}

unsafe impl Send for GlassesDevice {}
unsafe impl Sync for GlassesDevice {}

#[inline]
pub fn init_glasses() -> Result<GlassesDevice> {
    let device = GlassesDevice::new()?;
    
    // Initialize display settings
    if let Err(e) = device.set_display_mode(true) {
        eprintln!("Warning: Could not set display mode: {}", e);
    }
    
    Ok(device)
}

#[inline]
pub fn configure_display() -> Result<()> {
    // Display configuration removed as requested
    // The XREAL glasses will work in mirror mode or be configured 
    // through system settings
    Ok(())
}

#[inline]
pub fn poll_events(device: &GlassesDevice) -> Result<Vec<GlassesEvent>> {
    device.poll_events()
}
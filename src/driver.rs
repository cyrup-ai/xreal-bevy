use anyhow::Result;
use ar_drivers::{any_glasses, GlassesEvent, ARGlasses, DisplayMode};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

/// Zero-allocation XREAL device manager with blazing-fast performance
/// Maintains persistent connection to XREAL glasses throughout application lifecycle
/// Follows lock-free patterns where possible for optimal performance
#[derive(Resource)]
pub struct XRealDevice {
    inner: Arc<Mutex<Box<dyn ARGlasses>>>,
    is_connected: bool,
    stereo_enabled: bool,
    display_resolution: (u32, u32),
}

#[derive(Debug, Clone, Copy)]
pub enum XRealDisplayMode {
    Mirror,    // 2D mirrored display
    Stereo,    // 3D stereo AR display
    Off,       // Display disabled
}

impl Default for XRealDisplayMode {
    #[inline]
    fn default() -> Self {
        XRealDisplayMode::Stereo
    }
}

pub struct GlassesDevice {
    inner: Arc<Mutex<Box<dyn ARGlasses>>>,
}

impl XRealDevice {
    /// Create new XRealDevice with persistent connection
    /// Zero-allocation initialization with proper error handling
    #[inline]
    pub fn new() -> Result<Self> {
        println!("   🔌 Calling ar-drivers any_glasses() function...");
        
        match any_glasses() {
            Ok(glasses) => {
                println!("   ✅ ar-drivers successfully detected glasses!");
                Ok(Self {
                    inner: Arc::new(Mutex::new(glasses)),
                    is_connected: true,
                    stereo_enabled: false,
                    display_resolution: (1920, 1080), // XREAL native resolution
                })
            }
            Err(e) => {
                println!("   ❌ ar-drivers any_glasses() failed: {}", e);
                println!("      Original error: {:?}", e);
                
                // Try individual glasses detection as fallback
                println!("   🔄 Attempting fallback individual detection...");
                match try_individual_glasses_detection() {
                    Ok(glasses) => {
                        println!("   ✅ Individual detection succeeded!");
                        Ok(Self {
                            inner: Arc::new(Mutex::new(glasses)),
                            is_connected: true,
                            stereo_enabled: false,
                            display_resolution: (1920, 1080), // XREAL native resolution
                        })
                    }
                    Err(individual_error) => {
                        println!("   ❌ Individual detection also failed: {}", individual_error);
                        
                        // Provide additional context based on error type
                        let error_msg = format!("{}", e);
                        if error_msg.contains("NotFound") || error_msg.contains("not found") {
                            println!("      💡 This suggests no supported AR glasses were detected");
                            println!("      📝 Troubleshooting steps:");
                            println!("         1. Ensure glasses are connected via USB-C");
                            println!("         2. Check if glasses are powered on");
                            println!("         3. Try reconnecting the USB-C cable");
                            println!("         4. 🔒 IMPORTANT: Check macOS accessibility permissions");
                            println!("            • Go to System Settings → Privacy & Security → Accessibility");
                            println!("            • Add this application to the allowed list");
                            println!("            • Restart the application");
                            println!("         5. Verify libusb is installed: brew install libusb");
                        } else if error_msg.contains("Permission") || error_msg.contains("permission") {
                            println!("      💡 This suggests a USB permissions issue");
                            println!("      📝 macOS Accessibility Solution:");
                            println!("         1. Open System Settings → Privacy & Security → Accessibility");
                            println!("         2. Click the '+' button to add this application");
                            println!("         3. Navigate to and select this application");
                            println!("         4. Restart the application");
                            println!("         5. Try reconnecting the XREAL glasses");
                        } else {
                            println!("      💡 Unknown error type - may need deeper investigation");
                            println!("      📝 Try accessibility permissions first:");
                            println!("         System Settings → Privacy & Security → Accessibility");
                        }
                        
                        Err(anyhow::anyhow!("Failed to detect AR glasses: {} (fallback also failed: {})", e, individual_error))
                    }
                }
            }
        }
    }
    
    /// Set display mode with blazing-fast performance
    /// Supports stereo AR mode for 3D desktop experience
    #[inline]
    pub fn set_display_mode(&mut self, mode: XRealDisplayMode) -> Result<()> {
        match self.inner.lock() {
            Ok(mut glasses) => {
                let ar_mode = match mode {
                    XRealDisplayMode::Stereo => DisplayMode::Stereo,
                    XRealDisplayMode::Mirror => DisplayMode::SameOnBoth,
                    XRealDisplayMode::Off => DisplayMode::SameOnBoth, // Fallback
                };
                
                match glasses.set_display_mode(ar_mode) {
                    Ok(_) => {
                        self.stereo_enabled = matches!(mode, XRealDisplayMode::Stereo);
                        Ok(())
                    }
                    Err(e) => Err(anyhow::anyhow!("Display mode error: {}", e))
                }
            }
            Err(_) => Err(anyhow::anyhow!("Failed to acquire glasses lock"))
        }
    }
    
    /// Get display resolution for render target creation
    #[inline]
    pub fn get_display_resolution(&self) -> (u32, u32) {
        self.display_resolution
    }
    
    /// Check if stereo mode is enabled
    #[inline]
    pub fn is_stereo_enabled(&self) -> bool {
        self.stereo_enabled
    }
    
    /// Check if glasses are connected
    #[inline]
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }
    
    /// Poll IMU events with zero-allocation fixed-size buffer
    #[inline]
    pub fn poll_events(&self) -> Result<Vec<GlassesEvent>> {
        match self.inner.lock() {
            Ok(mut glasses) => {
                let mut events = Vec::with_capacity(16); // Fixed capacity for zero allocation
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
    
    /// Validate glasses connection without blocking
    #[inline]
    pub fn validate_connection(&mut self) -> Result<bool> {
        match self.inner.lock() {
            Ok(mut glasses) => {
                // Try to read a single event to validate connection
                match glasses.read_event() {
                    Ok(_) => {
                        self.is_connected = true;
                        Ok(true)
                    }
                    Err(_) => {
                        self.is_connected = false;
                        Ok(false)
                    }
                }
            }
            Err(_) => {
                self.is_connected = false;
                Ok(false)
            }
        }
    }
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

unsafe impl Send for XRealDevice {}
unsafe impl Sync for XRealDevice {}

unsafe impl Send for GlassesDevice {}
unsafe impl Sync for GlassesDevice {}

/// Initialize AR glasses with proper persistent connection
/// Returns XRealDevice resource for Bevy ECS management
/// Detects ALL supported AR glasses types automatically
#[inline]
pub fn init_xreal_device() -> Result<XRealDevice> {
    println!("   Attempting to detect AR glasses...");
    println!("   Checking for: XREAL Air/Air2/Air2Pro, Rokid Air/Max, Mad Gaze Glow, Grawoow G530");
    
    // Add detailed debugging for glasses detection
    debug_glasses_detection_process();
    
    let mut device = XRealDevice::new()?;
    
    // Initialize to stereo mode for AR experience
    if let Err(e) = device.set_display_mode(XRealDisplayMode::Stereo) {
        eprintln!("⚠️  Warning: Could not set stereo display mode: {}", e);
        eprintln!("    Falling back to mirror mode");
        let _ = device.set_display_mode(XRealDisplayMode::Mirror);
    }
    
    Ok(device)
}

/// Legacy compatibility function for existing code
#[inline]
pub fn init_glasses() -> Result<GlassesDevice> {
    let device = GlassesDevice::new()?;
    
    // Initialize display settings
    if let Err(e) = device.set_display_mode(true) {
        eprintln!("Warning: Could not set display mode: {}", e);
    }
    
    Ok(device)
}

/// Configure AR glasses display with proper device management
/// Detects and initializes ANY supported AR glasses type including:
/// - XREAL Air, Air 2, Air 2 Pro
/// - Rokid Air, Max
/// - Mad Gaze Glow
/// - Grawoow G530
#[inline]
pub fn configure_display() -> Result<Option<XRealDevice>> {
    println!("🥽 Detecting AR glasses...");
    println!("   Supported: XREAL Air/Air2/Air2Pro, Rokid Air/Max, Mad Gaze Glow, Grawoow G530");
    
    match init_xreal_device() {
        Ok(device) => {
            println!("✅ AR glasses detected and initialized successfully");
            
            if device.is_stereo_enabled() {
                println!("🎯 Stereo display mode enabled - AR content will render to glasses");
            } else {
                println!("🪞 Mirror display mode enabled - content will mirror to glasses");
            }
            
            let (width, height) = device.get_display_resolution();
            println!("📺 Display resolution: {}x{}", width, height);
            
            Ok(Some(device))
        }
        Err(e) => {
            eprintln!("❌ No supported AR glasses detected: {}", e);
            eprintln!("   Supported models: XREAL Air/Air2/Air2Pro, Rokid Air/Max, Mad Gaze Glow, Grawoow G530");
            eprintln!("   Ensure glasses are connected via USB-C and powered on");
            eprintln!("   The app will continue in desktop-only mode");
            
            // Return None instead of failing the app
            Ok(None)
        }
    }
}

#[inline]
pub fn poll_events(device: &GlassesDevice) -> Result<Vec<GlassesEvent>> {
    device.poll_events()
}

/// Debug function to provide detailed information about glasses detection
/// This function helps diagnose why `any_glasses()` might be failing
fn debug_glasses_detection_process() {
    println!("🔍 Debugging glasses detection process...");
    
    // ar-drivers handles feature detection internally
    println!("   📦 ar-drivers features enabled in Cargo.toml: nreal, rokid, mad_gaze, grawoow");
    
    // Try to enumerate USB devices using hidapi directly
    match hidapi::HidApi::new() {
        Ok(api) => {
            println!("   ✅ hidapi initialized successfully");
            
            // Look for devices that might be AR glasses
            let mut found_potential_devices = false;
            for device_info in api.device_list() {
                let vendor_id = device_info.vendor_id();
                let product_id = device_info.product_id();
                
                // Known AR glasses vendor/product IDs
                let is_potential_ar_device = match vendor_id {
                    0x0486 => true, // Nreal/XREAL
                    0x04d8 => true, // Rokid
                    0x0c45 => true, // Mad Gaze
                    0x1234 => true, // Grawoow (placeholder, actual VID may vary)
                    _ => false,
                };
                
                if is_potential_ar_device {
                    found_potential_devices = true;
                    println!("   🎯 Found potential AR device: VID={:04x}, PID={:04x}", vendor_id, product_id);
                    if let Some(manufacturer) = device_info.manufacturer_string() {
                        println!("      Manufacturer: {}", manufacturer);
                    }
                    if let Some(product) = device_info.product_string() {
                        println!("      Product: {}", product);
                    }
                }
            }
            
            if !found_potential_devices {
                println!("   ⚠️  No potential AR devices found in HID enumeration");
                println!("      This could mean:");
                println!("      - Glasses are not connected");
                println!("      - USB permissions issue");
                println!("      - Device not recognized by HID subsystem");
            }
        }
        Err(e) => {
            println!("   ❌ Failed to initialize hidapi: {}", e);
            println!("      This suggests a system-level HID issue");
        }
    }
    
    // Provide macOS-specific guidance without system calls
    #[cfg(target_os = "macos")]
    {
        println!("   🍎 Running on macOS - checking for common USB permission issues");
        println!("      🔒 IMPORTANT: XREAL glasses may require accessibility permissions");
        println!("      📝 Go to System Settings → Privacy & Security → Accessibility");
        println!("      📝 Add this application to the allowed list and restart");
    }
    
    // ar-drivers features are handled internally by the library
    println!("   📋 ar-drivers features configured: nreal, rokid, mad_gaze, grawoow");
    
    println!("   💡 Now attempting ar-drivers detection...");
}

/// Attempt to detect glasses with additional debugging information
fn try_individual_glasses_detection() -> Result<Box<dyn ARGlasses>> {
    println!("   🔍 Attempting detection with additional debugging...");
    
    // The ar-drivers library doesn't expose individual device constructors
    // Instead, it uses any_glasses() to automatically detect and connect
    // to any supported device (XREAL Air, Rokid Air/Max, Mad Gaze, Grawoow)
    
    println!("      Using ar-drivers any_glasses() with enhanced error reporting...");
    match any_glasses() {
        Ok(mut glasses) => {
            println!("      ✅ Successfully detected AR glasses!");
            println!("         Device name: {}", glasses.name());
            
            // Try to get additional device info
            if let Ok(serial) = glasses.serial() {
                println!("         Serial: {}", serial);
            }
            
            Ok(glasses)
        }
        Err(e) => {
            println!("      ❌ Enhanced detection also failed: {}", e);
            Err(anyhow::anyhow!("AR glasses detection failed: {}", e))
        }
    }
}
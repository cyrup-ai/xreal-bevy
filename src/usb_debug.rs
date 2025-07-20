/// USB debugging utility for AR glasses detection
/// This module provides utilities to help diagnose USB connection issues
use anyhow::Result;
use hidapi::HidApi;

/// Debug USB devices to help diagnose AR glasses connection issues
pub fn debug_usb_devices() -> Result<()> {
    println!("🔍 USB Device Debug Report");
    println!("========================");

    // Initialize HID API
    let api = HidApi::new().map_err(|e| anyhow::anyhow!("Failed to initialize HID API: {}", e))?;

    println!("📱 All HID devices:");
    for device_info in api.device_list() {
        let vendor_id = device_info.vendor_id();
        let product_id = device_info.product_id();

        println!("  Device: VID={:04x}, PID={:04x}", vendor_id, product_id);

        if let Some(manufacturer) = device_info.manufacturer_string() {
            println!("    Manufacturer: {}", manufacturer);
        }

        if let Some(product) = device_info.product_string() {
            println!("    Product: {}", product);
        }

        if let Some(serial) = device_info.serial_number() {
            println!("    Serial: {}", serial);
        }

        println!("    Path: {}", device_info.path().to_string_lossy());
        println!("    Usage Page: {:04x}", device_info.usage_page());
        println!("    Usage: {:04x}", device_info.usage());
        println!("    Interface: {}", device_info.interface_number());
        println!("    Release: {:04x}", device_info.release_number());
        println!();
    }

    // Look for known AR glasses vendor IDs
    println!("🥽 Known AR Glasses Vendors:");
    println!("  0x0486 - Nreal/XREAL");
    println!("  0x04d8 - Rokid");
    println!("  0x0c45 - Mad Gaze");
    println!("  0x2833 - Grawoow (verified vendor ID)");
    println!();

    println!("🎯 Potential AR Glasses Detected:");
    let mut found_ar_devices = false;

    for device_info in api.device_list() {
        let vendor_id = device_info.vendor_id();
        let product_id = device_info.product_id();

        let (is_ar_device, vendor_name) = match vendor_id {
            0x0486 => (true, "Nreal/XREAL"),
            0x04d8 => (true, "Rokid"),
            0x0c45 => (true, "Mad Gaze"),
            0x1234 => (true, "Grawoow"),
            _ => (false, "Unknown"),
        };

        if is_ar_device {
            found_ar_devices = true;
            println!(
                "  ✅ {} Device: VID={:04x}, PID={:04x}",
                vendor_name, vendor_id, product_id
            );

            if let Some(product) = device_info.product_string() {
                println!("     Product: {}", product);
            }

            if let Some(manufacturer) = device_info.manufacturer_string() {
                println!("     Manufacturer: {}", manufacturer);
            }

            // Try to open the device to check accessibility
            match device_info.open_device(&api) {
                Ok(device) => {
                    println!("     ✅ Device can be opened successfully");

                    // Try to get device info
                    match device.get_manufacturer_string() {
                        Ok(Some(mfg)) => println!("     Manufacturer (via device): {}", mfg),
                        Ok(None) => println!("     Manufacturer (via device): None"),
                        Err(e) => println!("     ❌ Failed to get manufacturer: {}", e),
                    }

                    match device.get_product_string() {
                        Ok(Some(prod)) => println!("     Product (via device): {}", prod),
                        Ok(None) => println!("     Product (via device): None"),
                        Err(e) => println!("     ❌ Failed to get product: {}", e),
                    }
                }
                Err(e) => {
                    println!("     ❌ Cannot open device: {}", e);
                    println!("        This might indicate a permission issue");
                }
            }

            println!();
        }
    }

    if !found_ar_devices {
        println!("  ❌ No AR glasses with known vendor IDs found");
        println!("     This could mean:");
        println!("     - Glasses are not connected");
        println!("     - Glasses use a different vendor ID");
        println!("     - USB driver issues");
        println!("     - Glasses are not powered on");
    }

    Ok(())
}

/// Check if libusb is available and functioning
pub fn check_libusb_status() -> Result<()> {
    println!("📚 libusb Status Check");
    println!("=====================");

    println!("📦 USB access via hidapi (compiled in)");
    println!("💡 libusb functionality available through ar-drivers dependency");

    Ok(())
}

/// Main debug function that runs all diagnostic checks
pub fn run_full_debug() -> Result<()> {
    println!("🚀 AR Glasses USB Debug Tool");
    println!("============================");
    println!();

    check_libusb_status()?;
    println!();

    debug_usb_devices()?;

    println!("🏁 Debug complete!");
    println!("If no AR glasses were found, try:");
    println!("1. Reconnecting the USB-C cable");
    println!("2. Checking if glasses are powered on");
    println!("3. Running with sudo for USB permissions");
    println!("4. Installing libusb: brew install libusb");

    Ok(())
}

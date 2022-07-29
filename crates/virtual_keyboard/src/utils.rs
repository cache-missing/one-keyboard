use std::{fs::File, io, path::Path};

use evdev_rs::{
    enums::{BusType, EventCode, EventType, EV_KEY, EV_SYN},
    Device, DeviceWrapper, UninitDevice,
};

pub(crate) fn setup_uinit_device(uinit_device: &mut UninitDevice) -> io::Result<()> {
    // Setup device
    uinit_device.set_name("Virtual Keyboard");
    uinit_device.set_bustype(BusType::BUS_USB as u16);
    uinit_device.set_vendor_id(0xabcd);
    uinit_device.set_product_id(0xefef);

    uinit_device.enable(EventType::EV_KEY)?;
    if let Err(e) = uinit_device.enable(EventCode::EV_KEY(EV_KEY::KEY_K)) {
        println!("Failed to enable EV_KEY::KEY_K, error: {}", e);
    }
    uinit_device.enable(EventCode::EV_SYN(EV_SYN::SYN_REPORT))?;

    Ok(())
}

// scan all the input devices to find a real keyboard
pub(crate) fn open_a_valid_device() -> Option<Device> {
    println!("opening a valid device...");
    let d = find_valid_device();
    match d {
        Some(d) => {
            if let Some(n) = d.name() {
                println!(
                    "Connected to device: '{}' ({:04x}:{:04x})",
                    n,
                    d.vendor_id(),
                    d.product_id()
                );
            }
            Some(d)
        }
        None => None,
    }
}

pub(crate) fn find_valid_device() -> Option<Device> {
    println!("Scanning for keyboard devices...");
    let devices_home = Path::new("/dev/input");
    for entry in devices_home.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        // check if the file name start with "event"
        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("event")
        {
            // Connect to real keyboard, or fallback to default virtual keyboard(TODO)
            let f = match File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to open device, error: {}", e);
                    return None;
                }
            };
            let d = match Device::new_from_file(f) {
                Ok(d) => d,
                Err(e) => {
                    println!("Failed to create device, error: {}", e);
                    return None;
                }
            };
            // check if the device is a real keyboard
            if is_keyboard(&d) {
                return Some(d);
            }
        }
    }

    None
}

pub(crate) fn is_keyboard(device: &Device) -> bool {
    // when the device has EV_KEY and EV_REP, it is likly a real keyboard
    if device.has_event_type(&EventType::EV_KEY) && device.has_event_type(&EventType::EV_REP) {
        println!("{} is a real keyboard", device.name().unwrap());
        return true;
    }

    false
}

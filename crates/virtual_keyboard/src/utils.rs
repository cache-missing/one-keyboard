use std::{fs::File, io, path::Path};

use evdev_rs::{
    enums::{BusType, EventCode, EventType, EV_KEY, EV_SYN},
    Device, DeviceWrapper, UninitDevice,
};
use log::{debug, error, info};

pub(crate) fn setup_uinit_device(uinit_device: &mut UninitDevice) -> io::Result<()> {
    // Setup device
    uinit_device.set_name("Virtual Keyboard");
    uinit_device.set_bustype(BusType::BUS_USB as u16);
    uinit_device.set_vendor_id(0xabcd);
    uinit_device.set_product_id(0xefef);

    uinit_device.enable(EventType::EV_KEY)?;
    if let Err(e) = uinit_device.enable(EventCode::EV_KEY(EV_KEY::KEY_K)) {
        error!("Failed to enable EV_KEY::KEY_K, error: {}", e);
    }
    uinit_device.enable(EventCode::EV_SYN(EV_SYN::SYN_REPORT))?;

    Ok(())
}

// scan all the input devices to find a real keyboard
pub(crate) fn open_a_valid_device() -> Option<Device> {
    info!("opening a valid device...");
    let d = find_valid_device();
    match d {
        Some(d) => {
            if let Some(n) = d.name() {
                info!(
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
    info!("Scanning for keyboard devices...");
    let devices_home = Path::new("/dev/input");
    for entry in match devices_home.read_dir() {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to read devices home directory: {}", e);
            return None;
        }
    } {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to read entry: {}", e);
                return None;
            }
        };
        let path = entry.path();
        if path
            .file_name()
            .map(|name| name.to_str().map(|s| s.starts_with("event")))
            .is_some_and(|x| x.is_some_and(|y| *y))
        // check if the file name start with "event"
        {
            let f = File::open(&path).ok()?;
            let d = Device::new_from_file(f).ok()?;
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
        debug!("{:?} is a real keyboard", device.name());
        return true;
    }

    false
}

use std::io;

use evdev_rs::{
    enums::{BusType, EventCode, EventType, EV_KEY, EV_SYN},
    DeviceWrapper, UninitDevice,
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

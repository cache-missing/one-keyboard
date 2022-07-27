use std::fs::File;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use evdev_rs::enums::{BusType, EventCode, EventType, EV_KEY, EV_SYN};
use evdev_rs::{Device, DeviceWrapper, InputEvent, TimeVal, UInputDevice, UninitDevice};

pub fn virtual_keyboard() {
    // TODO: Remove hard-coding
    let device = "/dev/input/event2";

    // Connect to real keyboard
    let f = File::open(device).unwrap();
    let d = Device::new_from_file(f).unwrap();

    if let Some(n) = d.name() {
        println!(
            "Connected to device: '{}' ({:04x}:{:04x})",
            n,
            d.vendor_id(),
            d.product_id()
        );
    }

    // Create virtual device
    let u = UninitDevice::new().unwrap();

    // Setup device
    u.set_name("Virtual Keyboard");
    u.set_bustype(BusType::BUS_USB as u16);
    u.set_vendor_id(0xabcd);
    u.set_product_id(0xefef);

    // Note mouse keys have to be enabled for this to be detected
    // as a usable device, see: https://stackoverflow.com/a/64559658/6074942
    u.enable_event_type(&EventType::EV_KEY).unwrap();
    u.enable_event_code(&EventCode::EV_KEY(EV_KEY::KEY_K), None)
        .unwrap();

    u.enable_event_code(&EventCode::EV_SYN(EV_SYN::SYN_REPORT), None)
        .unwrap();

    // Attempt to create UInputDevice from UninitDevice
    let v = UInputDevice::create_from_device(&d).unwrap();
    println!("Virtual Keyboard: {}", v.syspath().unwrap());

    // devnode
    println!("devnode: {}", v.devnode().unwrap());

    loop {
        println!("send event");

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        v.write_event(&InputEvent {
            time: TimeVal {
                tv_sec: current_time.as_secs() as i64,
                tv_usec: 0,
            },
            event_code: EventCode::EV_KEY(EV_KEY::KEY_K),
            value: 37,
        })
        .unwrap();

        v.write_event(&InputEvent {
            time: TimeVal {
                tv_sec: current_time.as_secs() as i64,
                tv_usec: 1,
            },
            event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
            value: 0,
        })
        .unwrap();

        sleep(Duration::from_secs(1));
    }
}

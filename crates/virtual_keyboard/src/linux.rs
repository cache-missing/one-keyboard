use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

use evdev_rs::enums::{EventCode, EV_SYN};
use evdev_rs::Device;
use evdev_rs::{DeviceWrapper, InputEvent, TimeVal, UInputDevice, UninitDevice};

use crate::utils::setup_uinit_device;

enum ReadEventDevice {
    Device(Device),
    UInitDevice(UninitDevice),
}

// Global virtual keyboard
pub struct VirtualKeyboard {
    input_device: UInputDevice,
}

impl VirtualKeyboard {
    // new
    pub fn new() -> Self {
        let device = match find_device() {
            Some(d) => ReadEventDevice::Device(d),
            None => {
                // Create virtual device
                let mut u = match UninitDevice::new() {
                    Some(u) => u,
                    None => panic!("Failed to create virtual device"),
                };
                if let Err(e) = setup_uinit_device(&mut u) {
                    panic!("Failed to setup virtual device, error: {}", e)
                }
                ReadEventDevice::UInitDevice(u)
            }
        };
        let v = match device {
            ReadEventDevice::Device(d) => match UInputDevice::create_from_device(&d) {
                Ok(v) => v,
                Err(e) => panic!("Failed to create virtual device, error: {}", e),
            },
            ReadEventDevice::UInitDevice(u) => match UInputDevice::create_from_device(&u) {
                Ok(v) => v,
                Err(e) => panic!("Failed to create virtual device, error: {}", e),
            },
        };

        if let Some(syspath) = v.syspath() {
            println!("Created virtual device: {}", syspath);
        }

        // devnode
        if let Some(devnode) = v.devnode() {
            println!("devnode: {}", devnode);
        }

        VirtualKeyboard { input_device: v }
    }

    pub fn write_event(&self, event_buf: [u8; 4096]) {
        println!("send event");
        // deserialize event
        let event: InputEvent = match serde_json::from_slice(&event_buf) {
            Ok(event) => event,
            Err(e) => {
                println!("Failed to deserialize event, error: {}", e);
                return;
            }
        };

        if let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH) {
            if let Err(e) = self.input_device.write_event(&InputEvent {
                time: TimeVal {
                    tv_sec: current_time.as_secs() as i64,
                    tv_usec: 0,
                },
                event_code: event.event_code,
                value: event.value,
            }) {
                return println!("Failed to write event, event: {:?}, error: {}", event, e);
            }

            if let Err(e) = self.input_device.write_event(&InputEvent {
                time: TimeVal {
                    tv_sec: current_time.as_secs() as i64,
                    tv_usec: 1,
                },
                event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
                value: 0,
            }) {
                println!("Failed to write event, event: {:?}, error: {}", event, e)
            }
        }
    }
}

impl Default for VirtualKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

// scan all the input devices to find a real keyboard
pub(crate) fn find_device() -> Option<Device> {
    let device_path = "/dev/input/event10";
    // Connect to real keyboard, or fallback to default virtual keyboard(TODO)
    let f = match File::open(&device_path) {
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

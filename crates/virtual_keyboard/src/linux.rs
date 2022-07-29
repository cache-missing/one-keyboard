use std::time::{SystemTime, UNIX_EPOCH};

use evdev_rs::enums::{EventCode, EV_SYN};
use evdev_rs::Device;
use evdev_rs::{InputEvent, TimeVal, UInputDevice, UninitDevice};

use crate::utils::{open_a_valid_device, setup_uinit_device};

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
        let device = match open_a_valid_device() {
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

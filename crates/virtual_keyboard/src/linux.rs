use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

use evdev_rs::enums::{BusType, EventCode, EventType, EV_KEY, EV_SYN};
use evdev_rs::Device;
use evdev_rs::{DeviceWrapper, InputEvent, ReadFlag, TimeVal, UInputDevice, UninitDevice};

// Global virtual keyboard
pub struct VirtualKeyboard {
    device: Device,
    input_device: UInputDevice,
}

impl VirtualKeyboard {
    // new
    pub fn new() -> Self {
        // scan all the input devices to find a real keyboard
        let device = "/dev/input/event2";

        // Connect to real keyboard, or fallback to default virtual keyboard(TODO)
        let f = match File::open(device) {
            Ok(f) => f,
            Err(e) => panic!("Failed to open file {}, error: {}", device, e),
        };
        let d = match Device::new_from_file(f) {
            Ok(d) => d,
            Err(e) => panic!("Failed to open device {}, error: {}", device, e),
        };

        if let Some(n) = d.name() {
            println!(
                "Connected to device: '{}' ({:04x}:{:04x})",
                n,
                d.vendor_id(),
                d.product_id()
            );
        }

        // Create virtual device
        let u = match UninitDevice::new() {
            Some(u) => u,
            None => panic!("Failed to create virtual device"),
        };

        // Setup device
        u.set_name("Virtual Keyboard");
        u.set_bustype(BusType::BUS_USB as u16);
        u.set_vendor_id(0xabcd);
        u.set_product_id(0xefef);

        if let Err(e) = u.enable(EventType::EV_KEY) {
            panic!("Failed to enable EV_KEY, error: {}", e)
        };
        if let Err(e) = u.enable(EventCode::EV_KEY(EV_KEY::KEY_K)) {
            println!("Failed to enable EV_KEY(KEY_K), error: {}", e)
        };
        if let Err(e) = u.enable(EventCode::EV_SYN(EV_SYN::SYN_REPORT)) {
            panic!("Failed to enable EV_SYN(SYN_REPORT), error: {}", e)
        };

        // Attempt to create UInputDevice from UninitDevice
        let v = match UInputDevice::create_from_device(&d) {
            Ok(v) => v,
            Err(e) => panic!("Failed to create virtual device, error: {}", e),
        };
        if let Some(syspath) = v.syspath() {
            println!("Created virtual device: {}", syspath);
        }

        // devnode
        if let Some(devnode) = v.devnode() {
            println!("devnode: {}", devnode);
        }

        VirtualKeyboard {
            device: d,
            input_device: v,
        }
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

    pub fn read_event(self) -> Option<InputEvent> {
        // read event
        let (_read_status, event) = match self.device.next_event(ReadFlag::NORMAL) {
            Ok((read_status, event)) => (read_status, event),
            Err(e) => {
                println!("Failed to read event, error: {}", e);
                return None;
            }
        };
        Some(event)
    }
}

impl Default for VirtualKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

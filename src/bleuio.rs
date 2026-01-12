// use dioxus::prelude::*;
use serialport5::*;
// use std::io::{self, Read, Write};
// use std::thread::sleep;
// use std::time::Duration;

const BLUEIO_VID: u16 = 0x2dcf;
const BLUEIO_PID: u16 = 0x6002;

// #[derive(Debug)]
// pub struct BleuIOState {
//     port: SerialPort,
// }

// impl BleuIOState {
//     pub fn new(path: &String) -> BleuIOState {
//         let port = SerialPort::builder()
//             .baud_rate(115200)
//             .read_timeout(Some(Duration::from_secs(5)))
//             .open(path);
//         BleuIOState { port: port.unwrap() }
//     }

//     pub fn info(&self) -> String {
//         let res: String;
//         match (&self.port).write_all(&"ATI\r".as_bytes()) {
//             Ok(_) => {
//                 sleep(Duration::from_millis(2000));
//                 let mut serial_buf: Vec<u8> = vec![0; 1000];
//                 match (&self.port).read(serial_buf.as_mut_slice()) {
//                     Ok(t) => {
//                         let v = Vec::from(&serial_buf[..t]);
//                         res = String::from_utf8(v).unwrap();
//                     },
//                     Err(ref e) if e.kind() == io::ErrorKind::TimedOut => res = "Timeout".to_string(),
//                     Err(e) => res = "Error".to_string(),
//                 }
//             },
//             Err(ref e) if e.kind() == io::ErrorKind::TimedOut => res = "Timeout".to_string(),
//             Err(e) => res = "Error".to_string(),
//         }

//         res
//     }
// }

fn is_bleuio(info: &UsbPortInfo) -> bool {
    (info.vid == BLUEIO_VID) && (info.pid == BLUEIO_PID)
}

pub fn find_bleuio() -> String {
    let mut pl: Vec<SerialPortInfo> = Vec::new();
    let mut blueio_port: String = "".to_string();

    // Get vector of serialport infos. Have to use another serialport crate
    // to get the SerialPortInfo stuff. Not available from serial2_tokio unfortunately.
    match serialport5::available_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                pl = ports;
            } else {
                pl = ports;
            }
        },
        Err(_) => {
        },
    };

    if pl.len() > 0 {
        for x in pl.into_iter() {
            match x.port_type {
                SerialPortType::UsbPort(info) => {
                    if is_bleuio(&info) {
                        if cfg!(target_os = "macos") {
                            blueio_port = x.port_name.replace("/dev/tty.", "/dev/cu.");
                        } else {
                            blueio_port = x.port_name;
                        }
                        // println!("{}", blueio_port);
                    }
                }
                _ => {
                }
            }
        }
    }

    blueio_port
}

// pub fn open_bleuio(path: String) -> Result<BleuIOState> {
//     let mut port = SerialPort::builder().baud_rate(115200).timeout(Duration::from_secs(5)).open(path);
//     let mut bleuio_state: BleuIOState
//     if port.is_ok() {

//     }

// }

// pub fn set_central_role() {

// }

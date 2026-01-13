use serialport5::*;
use serde_json::{Result, Value};
use std::convert::TryFrom;

const BLUEIO_VID: u16 = 0x2dcf;
const BLUEIO_PID: u16 = 0x6002;

#[derive(PartialEq)]
pub enum BleuIOResponseType {
    UnknownResponse,            // Unknown command found in JSON string
    CommandResponse,            // {"C":Command Index,"cmd":"command"}
    AcknowledgementResponse,    // {"A":Command Index,"err":error code in hex,"errMsg":"Error Message String"}
    ReplyResponse,              // {"R":Command Index,Reply data}
    EndResponse,                // {"E":Command Index,"nol":number of lines belonging to this command (excluding scan responses))}
    ScanDataResponse,           // {"S":Command Index,"rssi":rssi value,"addr":"mac address",(if available)"name":"device name"}
    ScanFindDataResponse,       // {"SF":Command Index,(if AT+SHOWRSSI turned on)"rssi":rssi value,"addr":"mac address","type":advertising type,"data":"data in hex"}
    ScanTargetResponse,	        // {"ST":Command Index,(if AT+SHOWRSSI turned on)"rssi":rssi value,"addr":"mac address","type":advertising type,"data":"data in hex"}
    ScanEndedResponse,	        // {"SE":Command Index,"action":"scan completed"}
    EventResponse, 	            // {event code:"Connection Index in hex if any otherwise 0xFFFF",Event response data}
}

#[derive(Debug)]
pub enum BleuIOErrorCode {
	Success,                        // 0x00
	GenericFailure,                 // 0x01
	AlreadyDone,                    // 0x02
	OperationAlreadyInProgress,     // 0x03
	InvalidParameter,               // 0x04
	NotAllowed,                     // 0x05
	NotConnected,                   // 0x06
	NotSupported,                   // 0x07
	NotAccepted,                    // 0x08
	Busy,                           // 0x09
	RequestTimedOut,                // 0x0A
	NotSupportedByPeer,             // 0x0B
	CanceledByUser,                 // 0x0C
	EncryptionKeyMissing,           // 0x0D
	InsufficientResources,          // 0x0E
	NotFound,                       // 0x0F
	NoCreditsAvailableOnL2CAPCoC,   // 0x10
	MTUExceededOnL2CAPCoC,          // 0x11
	InsufficientBandwidth,          // 0x12
    UnknownError,
}

impl TryFrom<i64> for BleuIOErrorCode {
    type Error = &'static str;

    fn try_from(value: i64) -> std::result::Result<Self, Self::Error> {
        match value {
            0x00 => Ok(BleuIOErrorCode::Success),
            0x01 => Ok(BleuIOErrorCode::GenericFailure),
            0x02 => Ok(BleuIOErrorCode::AlreadyDone),
            0x03 => Ok(BleuIOErrorCode::OperationAlreadyInProgress),
            0x04 => Ok(BleuIOErrorCode::InvalidParameter),
            0x05 => Ok(BleuIOErrorCode::NotAllowed),
            0x06 => Ok(BleuIOErrorCode::NotConnected),
            0x07 => Ok(BleuIOErrorCode::NotSupported),
            0x08 => Ok(BleuIOErrorCode::NotAccepted),
            0x09 => Ok(BleuIOErrorCode::Busy),
            0x0A => Ok(BleuIOErrorCode::RequestTimedOut),
            0x0B => Ok(BleuIOErrorCode::NotSupportedByPeer),
            0x0C => Ok(BleuIOErrorCode::CanceledByUser),
            0x0D => Ok(BleuIOErrorCode::EncryptionKeyMissing),
            0x0E => Ok(BleuIOErrorCode::InsufficientResources),
            0x0F => Ok(BleuIOErrorCode::NotFound),
            0x10 => Ok(BleuIOErrorCode::NoCreditsAvailableOnL2CAPCoC),
            0x11 => Ok(BleuIOErrorCode::MTUExceededOnL2CAPCoC),
            0x12 => Ok(BleuIOErrorCode::InsufficientBandwidth),
            _ => Err("fel"),
        }
    }
}

// Checks if given UsbPortInfo is associated with an attached BleuIO device.
fn is_bleuio(info: &UsbPortInfo) -> bool {
    (info.vid == BLUEIO_VID) && (info.pid == BLUEIO_PID)
}

// Scan list of available USB devices and return device path of first detected BleuIO device.
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

/* Test result strings:
    {"C":38,"cmd":"AT+FINDSCANDATA=FF5B07=2"}
    {"A":38,"err":0,"errMsg":"ok"}
    {"R":38,"action":"scanning"}
    {"E":38,"nol":4}
    {"SF":38,"addr":"F5:50:35:CF:B1:ED","type":0,"data":"0201061BFF5B07050422013FBD007D27E000BB00F419000000000000020A02"}
    {"SF":38,"addr":"D2:B1:28:3F:42:D4","type":0,"data":"0201061BFF5B070504220049880B7F27EE00AB000A01000000000000024503"}
    {"SE":38,"action":"scan completed"}
*/
pub fn get_bleuio_result_type(v: &Value) -> BleuIOResponseType {
    let result_type: BleuIOResponseType;

    if v.get("C").is_some() {
        result_type = BleuIOResponseType::CommandResponse;
    } else if v.get("A").is_some() {
        result_type = BleuIOResponseType::AcknowledgementResponse;
    } else if v.get("R").is_some() {
        result_type = BleuIOResponseType::ReplyResponse;
    } else if v.get("E").is_some() {
        result_type = BleuIOResponseType::EndResponse;
    } else if v.get("S").is_some() {
        result_type = BleuIOResponseType::ScanDataResponse;
    } else if v.get("SF").is_some() {
        result_type = BleuIOResponseType::ScanFindDataResponse;
    } else if v.get("ST").is_some() {
        result_type = BleuIOResponseType::ScanTargetResponse;
    } else if v.get("SE").is_some() {
        result_type = BleuIOResponseType::ScanEndedResponse;
    } else {
        result_type = BleuIOResponseType::UnknownResponse;
    }

    result_type
}

pub fn parse_bleuio_result(json: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(json)?; 
    Ok(v)
}

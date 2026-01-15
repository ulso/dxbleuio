use std::collections::HashMap;

use dioxus::html::div;
use dioxus::prelude::*;
use serial2_tokio::SerialPort;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
// use std::path::PathBuf;
// use std::io::{self, Read, Write};
// use std::thread::sleep;
// use std::time::Duration;
use futures_util::StreamExt;
use bleuio::*;

pub mod bleuio;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");

static CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[derive(Debug, Clone, PartialEq, Copy)]
struct HibouAir {
    mfid: u16,          // the manufacturer id of the device
    beacon_nr: u8,      // type of beacon
    board_type: u8,     // type of device
    board_id: [u8;3],   // unique board id
    als: u16,           // ambient light sensor
    bar: u16,           // pressure
    temp: i16,          // temperature
    hum: u16,           // humidity
    voc: u16,           // volatile organic compounds
    pm1_0: u16,         // particle matter PM1.0
    pm2_5: u16,         // particle matter PM2.5
    pm10: u16,          // particle matter PM10.0
    co2: u16,           // carbon dioxide
    voc_type: u8,       // 0 = old, 1 = resistance, 2 = ppm, 3 = IAQ
}

impl HibouAir {
    fn new(data: &str)  -> Self {
        // Parse the scan data string and populate the struct fields.
        // Return None if parsing fails.
        Self {
            mfid: data.get(10..14).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            beacon_nr: data.get(14..16).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(0),
            board_type: data.get(16..18).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(0),
            board_id: data.get(18..24).and_then(|s| {
                if s.len() == 6 {
                    let b1 = u8::from_str_radix(&s[0..2], 16).ok()?;
                    let b2 = u8::from_str_radix(&s[2..4], 16).ok()?;
                    let b3 = u8::from_str_radix(&s[4..6], 16).ok()?;
                    Some([b1, b2, b3])
                } else {
                    None
                }
            }).unwrap_or([0,0,0]),
            als: data.get(24..28).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            bar: data.get(28..32).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            temp: data.get(32..36).and_then(|s| i16::from_str_radix(s, 16).ok()).unwrap_or(0),
            hum: data.get(36..40).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            voc: data.get(40..44).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            pm1_0: data.get(44..48).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            pm2_5: data.get(48..52).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            pm10: data.get(52..56).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            co2: data.get(56..60).and_then(|s| u16::from_str_radix(s, 16).ok()).unwrap_or(0),
            voc_type: data.get(60..61).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(0),
        }
    }

    fn to_string(&self) -> String {
        format!(
            "HibouAir(mfid: {}, beacon_nr: {}, board_type: {}, board_id: {:02X?}, als: {}, bar: {}, temp: {}, hum: {}, voc: {}, pm1_0: {}, pm2_5: {}, pm10: {}, co2: {}, voc_type: {})",
            self.mfid,
            self.beacon_nr,
            self.board_type,
            self.board_id,
            self.als,
            self.bar,
            self.temp,
            self.hum,
            self.voc,
            self.pm1_0,
            self.pm2_5,
            self.pm10,
            self.co2,
            self.voc_type
        )
    }

    fn get_id(&self) -> u32 {
        ((self.board_id[0] as u32) << 16) | ((self.board_id[1] as u32) << 8) | (self.board_id[2] as u32)
    }

    fn get_board_id_string(&self) -> String {
        format!("{:02X}", self.get_id())
    }

    fn get_board_type(&self) -> u8 {
        self.board_type
    }

    fn get_board_type_string(&self) -> String {
        match self.board_type {
            0x03 => "PPM".to_string(),
            0x04 => "CO2".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn get_co2(&self) -> u16 {
        self.co2
    }

    fn get_pm2_5(&self) -> u16 {
        self.pm2_5
    }
}

 
pub enum BleuIOCommand {
    At,
    AtI,
    AtCentral,
    AtFindscandata,
}

#[derive(Clone, PartialEq)]
struct SensorData {
    value: u32,
    label: &'static str,
    status: &'static str, // t.ex. "Excellent", "Good"
    unit: &'static str, // t.ex. "ppm"
    bg_color: &'static str, // Tailwind-klass, t.ex. "bg-green-500"
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let port_name = find_bleuio();
    rsx! {
        // Länka till den kompilerade filen (dx lägger den i assets/main.css som standard)
        document::Stylesheet { href: CSS }

        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Hero { port_name }
    }
}

/// Utility function for sending text messages to the 'log' pane.
fn logga(mut log: Signal<String>, msg: &str) {
    log.with_mut(|l| l.push_str(&format!("{}", msg)));
}

fn add_sensor(mut sens: Signal<HashMap<u32, HibouAir>>, sensor: HibouAir) {
    sens.with_mut(|s| {
        s.insert(sensor.get_id(), sensor);
    });
}

#[component]
fn SensorPanel(sensor: HibouAir) -> Element {
    rsx! {
        div { class: "p-4 rounded-lg shadow-md text-white flex justify-between items-center",
            div { class: "flex items-center",
                // Här kan du lägga till en ikon (använd asset! macro för bilder eller en ikonfont)
                span { class: "mr-2", "{sensor.get_board_type_string()}" }
                        // span { class: "text-sm opacity-80", "{data.status}" }
            }
            if sensor.get_board_type() == 0x04 {
                div { class: "text-3xl font-bold",
                    "{sensor.get_co2()}"
                    span { class: "text-lg ml-1", "ppm" }
                }
            } else if sensor.get_board_type() == 0x03 {

            }
            div { class: "text-3xl font-bold",
                "{sensor.get_pm2_5()}"
                span { class: "text-lg ml-1", "ppm" }
            }
        }
    }
}

const AT: &[u8; 4] = b"AT\r\n";
const ATE0: &[u8; 6] = b"ATE0\r\n";
const ATV1: &[u8; 6] = b"ATV1\r\n";
const AT_FINDSCANDATA: &[u8;  24] = b"AT+FINDSCANDATA=FF5B07\r\n";

#[component]
pub fn Hero(port_name: String) -> Element {
    //     let co2_data = SensorData {
    //     value: 524,
    //     label: "CO2",
    //     status: "Excellent",
    //     unit: "ppm",
    //     bg_color: "bg-green-600",
    // };

    // let voc_data = SensorData {
    //     value: 1,
    //     label: "VOC",
    //     status: "Good",
    //     unit: "ppm",
    //     bg_color: "bg-green-600", // Anpassa färg beroende på statuslogik
    // };

    let mut sensor_hash: HashMap<u32, HibouAir> = HashMap::new();
    let mut sens = sensor_hash.clone();
    let mut hibs = use_signal(|| sensor_hash.clone());
    let mut log = use_signal(|| String::new());
    
    let serial_task = use_coroutine(move |mut external_rx: UnboundedReceiver<BleuIOCommand>| {
        let port_name_for_async = port_name.clone();
        let log_handle = log;
        let mut sensors = sensor_hash.clone();

        async move {
            // let mut app_state: AppState = AppState::OpenPort;

            // logga(log_handle, &format!("Försöker öppna {}\n", port_name_for_async));
            let port = match SerialPort::open(port_name_for_async, 115200) {
                Ok(p) => {
                    p.set_dtr(true).ok();
                    p.set_rts(true).ok();
                    p},
                Err(e) => {
                    logga(log_handle, &format!("Error: {}\n", e));
                    return;
                }
            };

            // Dela upp porten i läsare och skrivare för att kunna använda båda i select!
            let (reader, mut writer) = tokio::io::split(port);
            let mut buffered_reader = BufReader::new(reader);
            let mut read_buffer = String::new();

            // Current coomunicating state with the BleuIO dongle.
            let mut last_cmd: &[u8] = AT;

            logga(log_handle, "Port öppen. Väntar...\n");

            // Skapa en intern kanal
            let (internal_tx, mut internal_rx) = futures_channel::mpsc::unbounded::<BleuIOCommand>();
            let initial_tx = internal_tx.clone();

            // 1. Skicka initialt kommando direkt
            // initial_tx.unbounded_send(BleuIOCommand::At).ok();
            // writer.write_all(b"ATE0\r\n").await.ok();
            writer.write_all(ATE0).await.ok();
            last_cmd = ATE0;
            let mut last_error: i64 = 0;

            // let mut sensors: HashMap<u32, HibouAir> = HashMap::new();

            loop {
                tokio::select! {
                    // GREN 1: Läs inkommande data från USB (fram till LF)
                    res = timeout(Duration::from_secs(5), buffered_reader.read_line(&mut read_buffer)) => {
                        match res {
                            Ok(Ok(0)) => break, // Porten stängdes
                            Ok(Ok(_)) => {
                                let clean_line = read_buffer.trim_end_matches(['\r', '\n']).to_string();
                                read_buffer.clear();
                                // logga(log_handle, &format!("{}\n", clean_line));
                                match parse_bleuio_result(&clean_line) {
                                    Ok(v) => {
                                        let t = get_bleuio_result_type(&v);
                                        match &t {
                                            BleuIOResponseType::AcknowledgementResponse => {
                                                // Received line with possible error code - let's hope it is success!
                                                // In any case, save it for later.
                                                last_error = v["err"].as_i64().unwrap_or(-1); 
                                                let ec = BleuIOErrorCode::try_from(last_error);
                                                // logga(log_handle, &format!("Error code: {}, msg: {}, ec: {:?}\n", last_error, &v["errMsg"], &ec));
                                            },
                                            BleuIOResponseType::EndResponse => {
                                                // Last line of response received.
                                                if last_error == 0 {
                                                    // logga(log_handle, "Operation slutförd utan fel.\n");
                                                    if last_cmd == ATE0 {
                                                        // Echo off successful
                                                        // logga(log_handle, "Echo avstängt\n");
                                                        writer.write_all(ATV1).await.ok();
                                                        last_cmd = ATV1;
                                                    } else if last_cmd == ATV1 {
                                                        // logga(log_handle, "Verbose läge aktiverat\n");
                                                        writer.write_all(AT_FINDSCANDATA).await.ok();
                                                        last_cmd = AT_FINDSCANDATA;
                                                    }
                                                } else {
                                                    logga(log_handle, &format!("Operation slutförd med felkod {}\n", last_error));
                                                }
                                            },
                                            BleuIOResponseType::ScanFindDataResponse => {
                                                // Scan completed.
                                                // logga(log_handle, &format!("address: {} data: {}\n", &v["addr"], &v["data"]));
                                                let data = &v["data"].as_str().unwrap_or("");
                                                let hibou = HibouAir::new(data);
                                                let id = hibou.get_id();
                                                sensors.insert(id, hibou);
                                                add_sensor(hibs, hibou);
                                                // let hibou2 = sensors.get(&hibou.get_id()).unwrap();
                                                // logga(log_handle, &format!("HibouAIR data: {}\n", hibou2.get_board_id_string()));
                                                let n = sensors.clone().len();
                                                logga(log_handle, &format!("HibouAIR-enheter funna: {}\n", n));
                                            },
                                            _ => {}
                                        }
                                    }
                                    Err(e) => {
                                        // We may end up here for a couple of reasons:
                                        // 1. The line is not JSON (e.g. "OK" or "ERROR")
                                        // 2. The line is malformed JSON
                                        // logga(log_handle, &format!("JSON error: {}\n", e));
                                        // logga(log_handle, &format!("Rådata: {}\n", clean_line));
                                        if last_cmd == ATE0 {
                                            if clean_line == "ECHO OFF" {
                                                // Echo off successful
                                                // logga(log_handle, "Echo avstängt\n");
                                                writer.write_all(ATV1).await.ok();
                                                last_cmd = ATV1;
                                            // } else {
                                            //     logga(log_handle, "Fel vid avstängning av echo\n");
                                            }
                                        } else if last_cmd == ATV1 {
                                            if clean_line == "VERBOSE ON" {
                                                // logga(log_handle, "Verbose läge aktiverat\n");
                                                writer.write_all(AT_FINDSCANDATA).await.ok();
                                                last_cmd = AT_FINDSCANDATA;
                                            // } else {
                                            //     logga(log_handle, "Fel vid aktivering av verbose läge\n");
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                logga(log_handle, &format!("Läsfel: {}\n", e));
                                break;
                            }
                            Err(_) => {
                                // Detta händer om 5 sekunder går utan att read_line blir klar
                                logga(log_handle, "Timeout.\n");
                            }
                        }
                    }

                    // GREN 2: Lyssna på kommandon från Dioxus UI (rx)
                    ext_msg = external_rx.next() => {
                        if let Some(cmd) = ext_msg {
                            internal_tx.unbounded_send(cmd).ok();
                        } else {
                            break; // Avsluta om UI-kanalen dör
                        }
                    }

                    // GREN 3: Här körs ALL logik (både från UI och interna triggers)
                    cmd_to_exec = internal_rx.next() => {
                        if let Some(cmd) = cmd_to_exec {
                            // logga(log_handle, &format!("Kör kommando: {:?}", cmd));
                            match cmd {
                                BleuIOCommand::At => {writer.write_all(b"AT\r\n").await.ok();},
                                BleuIOCommand::AtI=> {writer.write_all(b"ATI\r\n").await.ok();},
                                BleuIOCommand::AtCentral => {writer.write_all(b"AT+CENTRAL\r\n").await.ok();},
                                BleuIOCommand::AtFindscandata => {writer.write_all(b"AT+FINDSCANDATA\r\n").await.ok();},
                            }
                        }
                    }
                }
            }
        }
    });

    let mut show_log = use_signal(|| false);

    rsx! {
        div {
            // img { src: HEADER_SVG, id: "header" }
            // style: "font-family: monospace; padding: 20px;",
            h1 { "HibouAIR Monitor" }

            if show_log() {
                div { style: "background: rgb(31, 28, 28); height: 300px; overflow-y: scroll; margin-bottom: 10px;",
                    pre { "{log}" }
                }
            }

            button {
                class: "border p-1 rounded-md bg-gray-500 mr-2",
                onclick: move |_| show_log.toggle(),
                {if show_log() { "Hide log" } else { "Show log" }}
            }

            if show_log() {
                button {
                    class: "border p-1 rounded-md bg-gray-500",
                    onclick: move |_| log.set(String::new()),
                    "Clear log"
                }
            }

            div {
                // Horizontal container for all panel groups
                class: "flex flex-col gap-8 p-4",
                // Note: flex-row is the default for 'flex', but explicit is fine.
                // gap-8 (2rem/32px) adds space between each group of 3.
                for sensor in hibs.read().values() {
                    {
                        // Detta är nu ett vanligt Rust-block
                        let s = format!("{:?}", sensor);

                        // Returnera rsx! från blocket
                        rsx! {
                            div {
                                // pre { "{s}" }
                                SensorPanel { sensor: sensor.clone() }
                            }
                        }
                    }
                }
            }
        }

        // div { class: "grid grid-cols-1 gap-4 lg:grid-cols-[120px_1fr] lg:gap-8",
        //     div { class: "h-32 rounded bg-gray-300" }
        //     div { class: "h-32 rounded bg-gray-300" }
        // }
    }
}

#![allow(non_snake_case)]
use std::collections::HashMap;
use dioxus::html::div;
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder, LogicalSize};
use serial2_tokio::SerialPort;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use hex;
use zerocopy::{FromBytes, Unaligned, Immutable, KnownLayout};
// use zerocopy::byteorder::little_endian::U16;
use futures_util::StreamExt;
use bleuio::*;

pub mod bleuio;

use hibouair::*;

pub mod hibouair;

const FAVICON: Asset = asset!("/assets/favicon.ico");
// const HEADER_SVG: Asset = asset!("/assets/header.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");

static CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

const AT: &[u8; 4] = b"AT\r\n";
const ATE0: &[u8; 6] = b"ATE0\r\n";
const ATV1: &[u8; 6] = b"ATV1\r\n";
const AT_FINDSCANDATA: &[u8;  24] = b"AT+FINDSCANDATA=FF5B07\r\n";

 
pub enum BleuIOCommand { // not used yet
    At,
    AtI,
    AtCentral,
    AtFindscandata,
}

fn main() {
    // 1. Define your window configuration
    let window = WindowBuilder::new()
        .with_title("Sensor Dashboard")
        .with_inner_size(LogicalSize::new(1100.0, 600.0)); // Width, Height

    // 2. Launch with the custom config
    LaunchBuilder::new()
        .with_cfg(Config::new().with_window(window))
        .launch(App);

    // dioxus::launch(App);
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

// Utility function for sending text messages to the 'log' pane.
fn logga(mut log: Signal<String>, msg: &str) {
    log.with_mut(|l| l.push_str(&format!("{}", msg)));
}

fn add_sensor(mut sens: Signal<HashMap<u32, HibouAir>>, sensor: HibouAir) {
    sens.with_mut(|s| {
        s.insert(sensor.get_id(), sensor);
        // println!("Sensor added: {}", sensor.to_string());
    });
}


#[component]
fn SensorPanelCO2(sensor: HibouAir) -> Element {
    rsx! {
        div {
            class: "p-4 bg-green-700 rounded-lg shadow-md text-white flex justify-between items-center",
            style: "display: grid; grid-template-columns: repeat(8, 1fr); gap: 4px 20px;",

            // Headers #1
            div { style: "font-weight: bold;", "CO2 Sensor" }
            div { "ID: {sensor.get_board_id_string()}" }
            div { "" }
            div { "" }
            div { "" }
            div { "" }
            div { "" }
            div { "" }
            hr { class: "col-span-8 border-white/20 my-2" }

            // Headers #2
            div { style: "font-weight: bold;", "CO2" }
            div { style: "font-weight: bold;", "" }
            div { style: "font-weight: bold;", "" }
            div { style: "font-weight: bold;", "VOC" }
            div { style: "font-weight: bold;", "Humidity" }
            div { style: "font-weight: bold;", "Temp" }
            div { style: "font-weight: bold;", "Pressure" }
            div { style: "font-weight: bold;", "Light" }

            // Data Row
            div { "{sensor.get_co2()} ppm" }
            div { "" }
            div { "" }
            div { "{sensor.get_voc_view()}" }
            div { "{sensor.get_hum():.0} %rh" }
            div { "{sensor.get_temp()} °C" }
            div { "{sensor.get_bar():.0} hPA" }
            div { "{sensor.get_als()} Lux" }
        }
    }
}

#[component]
fn SensorPanelPM(sensor: HibouAir) -> Element {
    rsx! {
        div {
            class: "p-4 bg-green-700 rounded-lg shadow-md text-white flex justify-between items-center",
            style: "display: grid; grid-template-columns: repeat(8, 1fr); gap: 4px 20px;",

            // Headers #1
            div { style: "font-weight: bold;", "PM Sensor" }
            div { "ID: {sensor.get_board_id_string()}" }
            div { "" }
            div { "" }
            div { "" }
            div { "" }
            div { "" }
            div { "" }
            hr { class: "col-span-8 border-white/20 my-2" }

            // Headers #2
            div { style: "font-weight: bold;", "PM10" }
            div { style: "font-weight: bold;", "PM2.5" }
            div { style: "font-weight: bold;", "PM1.0" }
            div { style: "font-weight: bold;", "VOC" }
            div { style: "font-weight: bold;", "Humidity" }
            div { style: "font-weight: bold;", "Temp" }
            div { style: "font-weight: bold;", "Pressure" }
            div { style: "font-weight: bold;", "Light" }

            // Data Row
            div { "{sensor.get_pm10()} μg/m³" }
            div { "{sensor.get_pm2_5()} μg/m³" }
            div { "{sensor.get_pm1_0()} μg/m³" }
            div { "{sensor.get_voc_view()}" }
            div { "{sensor.get_hum():.0} %rh" }
            div { "{sensor.get_temp()} °C" }
            div { "{sensor.get_bar():.0} hPa" }
            div { "{sensor.get_als()} lux" }
        }
    }
}   

#[component]
fn SensorPanel(sensor: HibouAir) -> Element {
    rsx! {
        div {
            if sensor.get_board_type() == SensorType::CO2_SENSOR {
                SensorPanelCO2 { sensor: sensor.clone() }
            } else if sensor.get_board_type() == SensorType::PM_SENSOR {
                SensorPanelPM { sensor: sensor.clone() }
            } else {
                div { "Unknown sensor type" }
            }
        }
    }
}   


#[component]
pub fn Hero(port_name: String) -> Element {
    let sensor_hash: HashMap<u32, HibouAir> = HashMap::new();
    let hibs = use_signal(|| sensor_hash.clone());
    let mut log = use_signal(|| String::new());
    
    let _serial_task = use_coroutine(move |mut external_rx: UnboundedReceiver<BleuIOCommand>| {
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
                                                if data.len() > 60 {
                                                    match HibouAir::from_hex(data) {
                                                        Ok(hibou) => {
                                                            let id = hibou.get_id();
                                                            let voc_type = hibou.get_voc_type();
                                                            // if voc_type == 2 || voc_type == 3 {
                                                                sensors.insert(id, hibou);
                                                                add_sensor(hibs, hibou);
                                                                // let hibou2 = sensors.get(&hibou.get_id()).unwrap();
                                                                // logga(log_handle, &format!("HibouAIR data: {}\n", hibou2.get_board_id_string()));
                                                                let n = sensors.clone().len();
                                                                logga(log_handle, &format!("HibouAIR-enheter funna: {}\n", n));
                                                            // }
                                                        },
                                                        Err(e) => {
                                                            logga(log_handle, &format!("Fel vid tolkning av HibouAIR-data: {}\n", e));
                                                        }
                                                    }
                                                }
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
                                BleuIOCommand::AtFindscandata => {writer.write_all(b"AT+FINDSCANDATA=FF5B07\r\n").await.ok();},
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
            // h1 { "HibouAIR Monitor" }
            if show_log() {
                div { style: "background: rgb(128, 128, 128); height: 300px; overflow-y: scroll; margin-bottom: 10px;",
                    pre { "{log}" }
                }
            }

            // button {
            //     class: "border p-1 rounded-md bg-gray-500 mr-2",
            //     onclick: move |_| show_log.toggle(),
            //     {if show_log() { "Hide log" } else { "Show log" }}
            // }
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
                        rsx! {
                            div {
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

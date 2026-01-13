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

#[component]
fn SensorPanel(data: SensorData) -> Element {
    rsx! {
        div { class: "{data.bg_color} p-4 rounded-lg shadow-md text-white flex justify-between items-center",
            div { class: "flex items-center",
                // Här kan du lägga till en ikon (använd asset! macro för bilder eller en ikonfont)
                span { class: "mr-2", "{data.label}" }
                span { class: "text-sm opacity-80", "{data.status}" }
            }
            div { class: "text-3xl font-bold",
                "{data.value}"
                span { class: "text-lg ml-1", "{data.unit}" }
            }
        }
    }
}

const AT: &[u8; 4] = b"AT\r\n";
const ATE0: &[u8; 6] = b"ATE0\r\n";
const ATV1: &[u8; 6] = b"ATV1\r\n";
const AT_SCANFILTER: &[u8;  29] = b"AT+SCANFILTER=NAME=HibouAIR\r\n";
const AT_FINDSCANDATA: &[u8;  17] = b"AT+FINDSCANDATA\r\n";

#[component]
pub fn Hero(port_name: String) -> Element {
        let co2_data = SensorData {
        value: 524,
        label: "CO2",
        status: "Excellent",
        unit: "ppm",
        bg_color: "bg-green-600",
    };

    let voc_data = SensorData {
        value: 1,
        label: "VOC",
        status: "Good",
        unit: "ppm",
        bg_color: "bg-green-600", // Anpassa färg beroende på statuslogik
    };


    let mut log = use_signal(|| String::new());
    
    let serial_task = use_coroutine(move |mut external_rx: UnboundedReceiver<BleuIOCommand>| {
        let port_name_for_async = port_name.clone();
        let log_handle = log;

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

            // let cmd: String = "AT".to_string();
            // writer.write_all(cmd.as_bytes()).await.ok();

            loop {
                tokio::select! {
                    // GREN 1: Läs inkommande data från USB (fram till LF)
                    res = timeout(Duration::from_secs(5), buffered_reader.read_line(&mut read_buffer)) => {
                        match res {
                            Ok(Ok(0)) => break, // Porten stängdes
                            Ok(Ok(_)) => {
                                let clean_line = read_buffer.trim_end_matches(['\r', '\n']).to_string();
                                read_buffer.clear();
                                logga(log_handle, &format!("{}\n", clean_line));
                                match parse_bleuio_result(&clean_line) {
                                    Ok(v) => {
                                        let t = get_bleuio_result_type(&v);
                                        match &t {
                                            BleuIOResponseType::AcknowledgementResponse => {
                                                // Received line with possible error code - let's hope it is success!
                                                // In any case, save it for later.
                                                last_error = v["err"].as_i64().unwrap_or(-1); 
                                                let ec = BleuIOErrorCode::try_from(last_error);
                                                logga(log_handle, &format!("Error code: {}, msg: {}, ec: {:?}\n", last_error, &v["errMsg"], &ec));
                                            },
                                            BleuIOResponseType::EndResponse => {
                                                // Last line of response received.
                                                if last_error == 0 {
                                                    // logga(log_handle, "Operation slutförd utan fel.\n");
                                                    if last_cmd == ATE0 {
                                                        // Echo off successful
                                                        logga(log_handle, "Echo avstängt\n");
                                                        writer.write_all(ATV1).await.ok();
                                                        last_cmd = ATV1;
                                                    } else if last_cmd == ATV1 {
                                                        logga(log_handle, "Verbose läge aktiverat\n");
                                                        writer.write_all(AT_SCANFILTER).await.ok();
                                                        last_cmd = AT_SCANFILTER;
                                                    } else if last_cmd == AT_SCANFILTER {
                                                        logga(log_handle, "Scan filter satt. Startar skanning...\n");
                                                        writer.write_all(AT_FINDSCANDATA).await.ok();
                                                        last_cmd = AT_FINDSCANDATA;
                                                    }
                                                } else {
                                                    logga(log_handle, &format!("Operation slutförd med felkod {}\n", last_error));
                                                }
                                            },
                                            BleuIOResponseType::ScanFindDataResponse => {
                                                // Scan completed.
                                                logga(log_handle, &format!("{}\n", &v["data"]));
                                            },
                                            _ => {}
                                        }
                                    }
                                    Err(e) => {
                                        // We may end up here for a couple of reasons:
                                        // 1. The line is not JSON (e.g. "OK" or "ERROR")
                                        // 2. The line is malformed JSON
                                        logga(log_handle, &format!("JSON error: {}\n", e));
                                        logga(log_handle, &format!("Rådata: {}\n", clean_line));
                                        if last_cmd == ATE0 {
                                            if clean_line == "ECHO OFF" {
                                                // Echo off successful
                                                logga(log_handle, "Echo avstängt\n");
                                                writer.write_all(ATV1).await.ok();
                                                last_cmd = ATV1;
                                            } else {
                                                logga(log_handle, "Fel vid avstängning av echo\n");
                                            }
                                        } else if last_cmd == ATV1 {
                                            if clean_line == "VERBOSE ON" {
                                                logga(log_handle, "Verbose läge aktiverat\n");
                                                writer.write_all(AT_SCANFILTER).await.ok();
                                                last_cmd = AT_SCANFILTER;
                                            } else {
                                                logga(log_handle, "Fel vid aktivering av verbose läge\n");
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

    rsx! {
        div {
            // img { src: HEADER_SVG, id: "header" }
            // style: "font-family: monospace; padding: 20px;",
            h1 { "HibouAIR Monitor" }

            div { style: "background: rgb(31, 28, 28); height: 300px; overflow-y: scroll; margin-bottom: 10px;",
                pre { "{log}" }
            }

            button {
                class: "border p-1 rounded-md bg-gray-500 mr-2",
                onclick: move |_| serial_task.send(BleuIOCommand::At),
                "Skicka AT"
            }
            button {
                class: "border p-1 rounded-md bg-gray-500 mr-2",
                onclick: move |_| serial_task.send(BleuIOCommand::AtI),
                "Skicka ATI"
            }
            button {
                class: "border p-1 rounded-md bg-gray-500 mr-2",
                onclick: move |_| log.set(String::new()),
                "Rensa logg"
            }

            div {
                // Horizontal container for all panel groups
                class: "flex flex-row gap-8 p-4",
                // Note: flex-row is the default for 'flex', but explicit is fine.
                // gap-8 (2rem/32px) adds space between each group of 3.
                for _ in 0..3 {
                    div {
                        SensorPanel { data: co2_data.clone() }
                        SensorPanel { data: voc_data.clone() }
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

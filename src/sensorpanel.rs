#![allow(non_snake_case)]
use dioxus::{html::feComponentTransfer, prelude::*};
use crate::hibouair::*;


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
fn SensorPanelUnknown(sensor: HibouAir) -> Element {
    rsx! {
        div { class: "p-4 bg-gray-700 rounded-lg shadow-md text-white",
            "Unknown sensor type for board ID: {sensor.get_board_id_string()}"
        }
    }
}

#[component]
pub fn SensorPanel(sensor: HibouAir) -> Element {
    match sensor.get_board_type() {
        SensorType::CO2_SENSOR => rsx! {
            SensorPanelCO2 { sensor: sensor.clone() }
        },
        SensorType::PM_SENSOR => rsx! {
            SensorPanelPM { sensor: sensor.clone() }
        },
        _ => rsx! {
            SensorPanelUnknown { sensor: sensor.clone() }
        }
    }
}   

pub fn render_sensor_panel(sensor: &HibouAir) -> Element {
    rsx! {
        SensorPanel { sensor: sensor.clone() }
    }
}

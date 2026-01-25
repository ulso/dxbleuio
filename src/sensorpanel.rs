#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::hibouair::*;


#[component]
fn SensorPanelCO2(sensor: HibouAir) -> Element {
    rsx! {
        div {
            class: "p-4 bg-green-700 rounded-lg shadow-md text-white flex justify-between items-center",
            style: "display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px 20px;",

            // Headers #1
            div { class: "col-span-1", "ID: {sensor.get_board_id_string()}" }
            div { class: "col-span-4 text-left", "CO2: {sensor.get_co2()} ppm" }
            // hr { class: "col-span-5 border-white/20 my-2" }

            // Data Row
            div { "VOC: {sensor.get_voc_view()}" }
            div { "Humid: {sensor.get_hum():.0} %rh" }
            div { "Temp: {sensor.get_temp()} °C" }
            div { "Press: {sensor.get_bar():.0} hPA" }
            div { "Light: {sensor.get_als()} Lux" }
        }
    }
}

#[component]
fn SensorPanelPM(sensor: HibouAir) -> Element {
    rsx! {
        div {
            class: "p-4 bg-green-700 rounded-lg shadow-md text-white flex justify-between items-center",
            style: "display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px 20px;",

            // Headers #1
            div { "ID: {sensor.get_board_id_string()}" }
            div { "PM10: {sensor.get_pm10()} μg/m³" }
            div { "PM2.5: {sensor.get_pm2_5()} μg/m³" }
            div { "PM1.0: {sensor.get_pm1_0()} μg/m³" }
            div { "" }
            // hr { class: "col-span-5 border-white/20 my-2" }

            // Data Row
            div { "VOC: {sensor.get_voc_view()}" }
            div { "Humid: {sensor.get_hum():.0} %rh" }
            div { "Temp: {sensor.get_temp()} °C" }
            div { "Press: {sensor.get_bar():.0} hPA" }
            div { "Light: {sensor.get_als()} Lux" }
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

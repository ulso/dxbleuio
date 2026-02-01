#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder, LogicalSize, LogicalPosition};

// Modules
pub mod components;
pub mod models;
pub mod hooks;

use crate::components::dashboard::*;
use crate::hooks::use_window_config::use_window_config;
use crate::models::bleuio::find_bleuio;
use crate::models::config::AppConfig;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
static CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(target_os = "macos")]
    macos_app_nap::prevent(); 

    // Load saved configuration
    let config = AppConfig::load().unwrap_or_default();
    
    // Debug: Show what config was loaded
    println!("Loading config: {}x{} at {:?}", 
        config.window.width, config.window.height,
        (config.window.x, config.window.y));

    // Build window with saved size and position
    // Force reasonable size constraints
    let width = config.window.width.clamp(800.0, 2000.0);
    let height = config.window.height.clamp(500.0, 1500.0);
    
    println!("Creating window with size: {}x{}", width, height);
    
    let mut window = WindowBuilder::new()
        .with_title("Sensor Dashboard")
        .with_inner_size(LogicalSize::new(width, height))
        .with_min_inner_size(LogicalSize::new(800.0, 500.0))
        .with_max_inner_size(LogicalSize::new(2000.0, 1500.0));
    
    // Apply saved position if available
    if let (Some(x), Some(y)) = (config.window.x, config.window.y) {
        println!("Setting window position to: {}, {}", x, y);
        window = window.with_position(LogicalPosition::new(x, y));
    } else {
        println!("No saved position found, using default");
    }

    // Launch with the custom config
    LaunchBuilder::new()
        .with_cfg(Config::new().with_window(window))
        .launch(App);
}

#[component]
fn App() -> Element {
    // Save window position/size when it changes
    use_window_config();
    
    let port_name = find_bleuio();
    rsx! {
        // Länka till den kompilerade filen (dx lägger den i assets/main.css som standard)
        document::Stylesheet { href: CSS }

        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Dashboard { port_name }
    }
}

use dioxus::prelude::*;
use crate::models::config::AppConfig;

/// Hook to automatically save window configuration when the app exits
pub fn use_window_config() {
    // Save window state when the component (app) is dropped/unmounted
    use_drop(move || {
        let window = dioxus::desktop::window();
        save_window_state(&window);
    });
}

fn save_window_state(window: &dioxus::desktop::DesktopContext) {
    // Get scale factor for Retina displays
    let scale_factor = window.scale_factor();
    
    // Get current window position
    // On macOS: with_position() sets inner position, so we must save inner_position()
    // On Windows/Linux: with_position() sets outer position, so we'd use outer_position()
    #[cfg(target_os = "macos")]
    let position = window.inner_position().ok();
    
    #[cfg(not(target_os = "macos"))]
    let position = window.outer_position().ok();
    
    // Position - divide by scale_factor to get logical coordinates
    let logical_x = position.as_ref().map(|p| p.x as f64 / scale_factor);
    let logical_y = position.as_ref().map(|p| p.y as f64 / scale_factor);
    
    let mut config = AppConfig::load().unwrap_or_default();
    
    // Update position only (size saving disabled due to Dioxus API bug)
    config.window.x = logical_x;
    config.window.y = logical_y;
    
    // Save configuration
    if let Err(e) = config.save() {
        eprintln!("Failed to save config: {}", e);
    }
}

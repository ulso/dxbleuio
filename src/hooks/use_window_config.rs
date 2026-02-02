use dioxus::{desktop::use_window, prelude::*};
use crate::models::config::AppConfig;

/// Hook to automatically save window configuration when the app exits
pub fn use_window_config() {
    // Save window state when the component (app) is dropped/unmounted
    use_drop(move || {
        let window = use_window();
        // let window = dioxus::desktop::window();
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
    
    // Get window size - inner_size() is broken in Dioxus 0.7 and always returns default size
    // So we use outer_size() which works correctly
    let outer_physical = window.outer_size();
    let width = outer_physical.width as f64 / scale_factor;
    let height = outer_physical.height as f64 / scale_factor;
    
    // WORKAROUND: Since inner_size() is broken, we calculate inner size from outer_size()
    // by subtracting the titlebar height. This is platform-specific and may break if
    // Apple changes the titlebar height in future macOS versions.
    // 
    // Ideally, this would be fixed in Dioxus by making inner_size() return correct values.
    #[cfg(target_os = "macos")]
    let titlebar_height = 32.0; // macOS standard titlebar as of macOS 14 Sonoma
    
    #[cfg(target_os = "windows")]
    let titlebar_height = 31.0; // Windows 11 standard titlebar
    
    #[cfg(target_os = "linux")]
    let titlebar_height = 35.0; // Typical Linux titlebar (varies by window manager)
    
    let save_width = width;
    let save_height = height - titlebar_height;
    
    println!("Saving window size: {}x{} (outer: {}x{})", 
        save_width, save_height, width, height);
    
    let mut config = AppConfig::load().unwrap_or_default();
    
    // Update position
    config.window.x = logical_x;
    config.window.y = logical_y;

    // Save dimensions
    config.window.width = save_width;
    config.window.height = save_height;
    
    // Save configuration
    if let Err(e) = config.save() {
        eprintln!("Failed to save config: {}", e);
    }
}

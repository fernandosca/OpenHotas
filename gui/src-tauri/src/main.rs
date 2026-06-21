//! src-tauri/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::DeviceState;

fn main() {
    tauri::Builder::default()
        .manage(DeviceState::new())
        .invoke_handler(tauri::generate_handler![
            // Serial port
            commands::list_ports,
            commands::connect,
            commands::disconnect,
            // System
            commands::get_info,
            commands::reboot,
            commands::factory_reset,
            // Config
            commands::get_config,
            commands::set_config,
            commands::save_config,
            commands::load_defaults,
            // Diagnostics
            commands::get_raw_axes,
            commands::get_processed_axes,
            commands::get_button_states,
            commands::get_sensor_status,
            commands::get_runtime_stats,
            commands::get_error_counters,
            // Calibration
            commands::start_calibration,
            commands::capture_calibration_point,
            commands::finish_calibration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

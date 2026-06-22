//! openhotas-cli — PC-side configuration and diagnostics tool.
//!
//! Usage:
//!   openhotas-cli <command> [options]
//!
//! Commands:
//!   info              Show device identity and version
//!   get-config        Show active configuration
//!   raw-axes          Show raw sensor values (u16)
//!   processed-axes    Show processed axis values (i16)
//!   buttons           Show button states
//!   stats             Show runtime statistics
//!   sensor-status     Show per-sensor health
//!   errors            Show error counters
//!   set-axis          Modify axis config (see --help)
//!   save              Persist config to flash
//!   load-defaults     Load factory defaults (runtime only)
//!   reboot            Trigger software reboot
//!   factory-reset     Erase stored config and reboot
//!   calibrate         Interactive axis calibration
//!
//! Options:
//!   --port <PATH>     Specify CDC port (auto-detect by default)

mod commands;
mod display;
mod transport;

use commands::SetAxisOptions;
use openhotas_protocol::request::AxisId;
use openhotas_protocol::version::{PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR};
use transport::OpenHotasTransport;

fn print_help() {
    println!(
        "OpenHOTAS CLI protocol v{major}.{minor}\n\
         \n\
         USAGE:\n  openhotas-cli <command> [options]\n\
         \n\
         COMMANDS:\n\
           info                  Show device identity and version\n\
           get-config            Show active configuration\n\
           raw-axes              Show raw sensor values\n\
           processed-axes        Show processed axis values\n\
           buttons               Show button states\n\
           stats                 Show runtime statistics\n\
           sensor-status         Show per-sensor health\n\
           errors                Show error counters\n\
           set-axis              Modify axis config\n\
           save                  Persist config to flash\n\
           load-defaults         Load factory defaults (runtime only)\n\
           reboot                Trigger software reboot\n\
           factory-reset         Erase stored config and reboot\n\
           calibrate            Interactive axis calibration\n\
         \n\
         SET-AXIS OPTIONS:\n\
           --axis <x|y|twist>    Axis to modify (required)\n\
           --deadzone <0-20>     Deadzone percentage\n\
           --invert <true|false> Invert axis\n\
           --enabled <true|false> Enable/disable axis\n\
           --curve-preset <name> Response curve preset (linear|smooth|center|s)\n\
           --ema <1-100>         EMA filter percentage\n\
           --travel-limit <1-100> Symmetric travel limit from center (%)\n\
           --center-offset <-20..20> Center offset percentage\n\
           --axis-to-button <opts> Axis-to-button mapping\n\
                                 Format: enabled=true,threshold=80,direction=positive,button=28\n\
         \n\
         CALIBRATE OPTIONS:\n\
           --axis <x|y|twist>    Axis to calibrate (required)\n\
         \n\
         OPTIONS:\n\
           --port <PATH>         Specify CDC port (auto-detect by default)\n\
           --help, -h            Show this help\n\
         ",
        major = PROTOCOL_VERSION_MAJOR,
        minor = PROTOCOL_VERSION_MINOR,
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_help();
        return;
    }

    let command = args[1].as_str();
    let rest: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    // Extract --port if present
    let port_path = extract_flag_value(&rest, "--port");
    let filtered_args: Vec<&str> = rest
        .iter()
        .enumerate()
        .filter(|(i, a)| **a != "--port" && (*i == 0 || rest[i - 1] != "--port"))
        .map(|(_, a)| *a)
        .collect();

    let mut t = match &port_path {
        Some(path) => match OpenHotasTransport::connect_to(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        },
        None => match OpenHotasTransport::connect() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        },
    };

    let result = match command {
        "info" => commands::cmd_info(&mut t),
        "get-config" => commands::cmd_get_config(&mut t),
        "raw-axes" => commands::cmd_raw_axes(&mut t),
        "processed-axes" => commands::cmd_processed_axes(&mut t),
        "buttons" => commands::cmd_buttons(&mut t),
        "stats" => commands::cmd_stats(&mut t),
        "sensor-status" => commands::cmd_sensor_status(&mut t),
        "errors" => commands::cmd_errors(&mut t),
        "save" => commands::cmd_save(&mut t),
        "load-defaults" => commands::cmd_load_defaults(&mut t),
        "reboot" => commands::cmd_reboot(&mut t),
        "factory-reset" => commands::cmd_factory_reset(&mut t),
        "set-axis" => {
            let axis = extract_flag_value(&filtered_args, "--axis");
            match axis {
                Some(name) => match AxisId::parse(&name) {
                    Some(ax) => {
                        let deadzone = extract_flag_value_u8(&filtered_args, "--deadzone");
                        let invert = extract_flag_value_bool(&filtered_args, "--invert");
                        let enabled = extract_flag_value_bool(&filtered_args, "--enabled");
                        let ema = extract_flag_value_u8(&filtered_args, "--ema");
                        let travel_limit = extract_flag_value_u8(&filtered_args, "--travel-limit");
                        let curve_preset = extract_flag_value(&filtered_args, "--curve-preset");
                        let center_offset =
                            extract_flag_value_i16(&filtered_args, "--center-offset");
                        let axis_to_button = extract_flag_value(&filtered_args, "--axis-to-button");

                        let options = SetAxisOptions {
                            deadzone_pct: deadzone,
                            invert,
                            enabled,
                            ema_pct: ema,
                            travel_limit_pct: travel_limit,
                            curve_preset,
                            center_offset,
                            axis_to_button,
                        };

                        commands::cmd_set_axis(&mut t, ax, options)
                    }
                    None => Err(format!("Invalid axis: {name}. Use x, y, or twist.").into()),
                },
                None => Err("Missing --axis. Use --axis x|y|twist".to_string().into()),
            }
        }
        "calibrate" => {
            let axis = extract_flag_value(&filtered_args, "--axis");
            match axis {
                Some(name) => match AxisId::parse(&name) {
                    Some(ax) => commands::cmd_calibrate(&mut t, ax),
                    None => Err(format!("Invalid axis: {name}. Use x, y, or twist.").into()),
                },
                None => Err("Missing --axis. Use --axis x|y|twist".to_string().into()),
            }
        }
        _ => {
            eprintln!("Unknown command: {command}");
            eprintln!("Use --help for usage.");
            std::process::exit(1);
        }
    };

    match result {
        Ok(output) => {
            println!("{output}");
        }
        Err(e) => {
            // Special-case transport errors that wrap strings
            if let transport::TransportError::PortError(_)
            | transport::TransportError::SendError(_)
            | transport::TransportError::ReadError(_) = &e
            {
            } else {
                // For Display-formatted errors, use Display
            }
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

// ── Argument Helpers ─────────────────────────────────────────────────────

fn extract_flag_value(args: &[&str], flag: &str) -> Option<String> {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == flag {
            return Some(args[i + 1].to_string());
        }
    }
    None
}

fn extract_flag_value_u8(args: &[&str], flag: &str) -> Option<u8> {
    extract_flag_value(args, flag).and_then(|v| v.parse().ok())
}

fn extract_flag_value_i16(args: &[&str], flag: &str) -> Option<i16> {
    extract_flag_value(args, flag).and_then(|v| v.parse().ok())
}

fn extract_flag_value_bool(args: &[&str], flag: &str) -> Option<bool> {
    extract_flag_value(args, flag).and_then(|v| match v.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    })
}

// Allow string errors in Result
impl From<String> for transport::TransportError {
    fn from(s: String) -> Self {
        transport::TransportError::PortError(s)
    }
}

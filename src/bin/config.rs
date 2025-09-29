use anyhow::Result;
use std::env;
use zmk_battery_monitor::Config;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "generate" {
        // Generate template config
        println!("{}", Config::generate_template());
    } else {
        // Show current config location and status
        let config_path = Config::config_path()?;

        if config_path.exists() {
            println!("Config file exists at: {}", config_path.display());

            // Try to load and display current config
            match Config::load() {
                Ok(config) => {
                    println!("\nCurrent configuration:");
                    println!(
                        "  Update interval: {} seconds",
                        config.general.update_interval
                    );
                    println!("  Log level: {}", config.general.log_level);
                    println!("\nDevices:");
                    for device in &config.devices {
                        let status = if device.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        println!("  - {} ({}) [{}]", device.name, device.address, status);
                        println!(
                            "    Low battery threshold: {}%",
                            device.low_battery_threshold
                        );
                    }
                    println!("\nTray:");
                    println!("  Enabled: {}", config.tray.enabled);
                    println!("  Show percentage: {}", config.tray.show_percentage_in_tray);
                }
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                }
            }
        } else {
            println!("No config file found at: {}", config_path.display());
            println!("\nRun with 'generate' to create a template:");
            println!("  {} generate > config.toml", args[0]);
            println!("\nOr run the main program to create a default config automatically.");
        }
    }

    Ok(())
}

use anyhow::Result;
use zmk_battery_monitor::{Config, ZmkBatteryReader};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Get the primary enabled device
    let device = match config.get_primary_device() {
        Some(d) => d,
        None => {
            eprintln!("No enabled devices found in config!");
            eprintln!(
                "Please edit the config file at: {}",
                Config::config_path()?.display()
            );
            eprintln!("\nAvailable devices from bluetoothctl:");

            // List available devices to help user
            let reader = ZmkBatteryReader::new().await?;
            if let Ok(devices) = reader.list_devices().await {
                for (name, address) in devices {
                    println!("  {name} - {address}");
                }
            }
            return Ok(());
        }
    };

    let reader = ZmkBatteryReader::new().await?;

    println!("Reading battery for: {} ({})", device.name, device.address);

    match reader.read_battery_levels(&device.address).await {
        Ok(batteries) => {
            if batteries.is_empty() {
                println!("No battery services found");
                println!("Make sure:");
                println!("  1. The keyboard is connected");
                println!("  2. Battery reporting is enabled in ZMK firmware");
                println!("  3. The device address is correct in the config");
                println!(
                    "\nConfig file location: {}",
                    Config::config_path()?.display()
                );
            } else {
                println!("\n=== Battery Levels ===");
                for battery in batteries {
                    println!("{}: {}%", battery.name, battery.level);

                    // Check low battery threshold
                    if battery.level <= device.low_battery_threshold {
                        println!("  ⚠ Low battery warning!");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading battery levels: {e}");
            eprintln!(
                "\nConfig file location: {}",
                Config::config_path()?.display()
            );

            // List available devices to help debug
            println!("\nAvailable Bluetooth devices:");
            if let Ok(devices) = reader.list_devices().await {
                for (name, address) in devices {
                    println!("  {name} - {address}");
                }
            }
        }
    }

    Ok(())
}

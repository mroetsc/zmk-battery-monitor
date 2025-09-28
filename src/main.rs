use anyhow::Result;
use zmk_battery_monitor::ZmkBatteryReader;

const DEVICE_ADDRESS: &str = "";

#[tokio::main]
async fn main() -> Result<()> {
    let reader = ZmkBatteryReader::new().await?;

    println!("Scanning for battery information on {DEVICE_ADDRESS}...");

    match reader.read_battery_levels(DEVICE_ADDRESS).await {
        Ok(batteries) => {
            if batteries.is_empty() {
                println!("No battery services found");
                println!("Make sure:");
                println!("  1. The keyboard is connected");
                println!("  2. Battery reporting is enabled in ZMK firmware");
                println!("  3. The device address is correct");
            } else {
                println!("\n=== Battery Levels ===");
                for battery in batteries {
                    println!("{}: {}%", battery.name, battery.level);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading battery levels: {e}");

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

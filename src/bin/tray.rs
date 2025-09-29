use anyhow::Result;
use ksni::menu::StandardItem;
use ksni::{MenuItem, Tray, TrayService};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use zmk_battery_monitor::{Config, ZmkBatteryReader};

enum Command {
    Refresh,
    Quit,
}

struct BatteryTray {
    battery_info: Arc<Mutex<String>>,
    tx: mpsc::UnboundedSender<Command>,
    device_name: String,
}

impl BatteryTray {
    fn new(
        battery_info: Arc<Mutex<String>>,
        tx: mpsc::UnboundedSender<Command>,
        device_name: String,
    ) -> Self {
        Self {
            battery_info,
            tx,
            device_name,
        }
    }
}

impl Tray for BatteryTray {
    fn icon_name(&self) -> String {
        "battery".to_string()
    }

    fn title(&self) -> String {
        format!("ZMK Battery - {}", self.device_name)
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let info = self.battery_info.lock().unwrap();
        ksni::ToolTip {
            title: format!("{} Battery", self.device_name),
            description: info.clone(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            MenuItem::Standard(StandardItem {
                label: "Refresh".to_string(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.send(Command::Refresh);
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quit".to_string(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.send(Command::Quit);
                }),
                ..Default::default()
            }),
        ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        // Click on tray icon refreshes battery status
        let _ = self.tx.send(Command::Refresh);
    }
}

async fn update_battery_info(
    battery_info: Arc<Mutex<String>>,
    device_address: &str,
    low_threshold: u8,
) {
    match read_battery(device_address, low_threshold).await {
        Ok(info) => {
            let mut data = battery_info.lock().unwrap();
            *data = info;
        }
        Err(e) => {
            let mut data = battery_info.lock().unwrap();
            *data = format!("Error: {e}");
        }
    }
}

async fn read_battery(device_address: &str, low_threshold: u8) -> Result<String> {
    let reader = ZmkBatteryReader::new().await?;
    let batteries = reader.read_battery_levels(device_address).await?;

    if batteries.is_empty() {
        Ok("No battery data available".to_string())
    } else {
        let info = batteries
            .iter()
            .map(|b| {
                let warning = if b.level <= low_threshold { " ⚠" } else { "" };
                format!("{}: {}%{}", b.name, b.level, warning)
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(info)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Check if tray is enabled
    if !config.tray.enabled {
        eprintln!(
            "Tray is disabled in config. Enable it in: {}",
            Config::config_path()?.display()
        );
        return Ok(());
    }

    // Get the primary enabled device
    let device = match config.get_primary_device() {
        Some(d) => d,
        None => {
            eprintln!("No enabled devices found in config!");
            eprintln!(
                "Please edit the config file at: {}",
                Config::config_path()?.display()
            );
            return Ok(());
        }
    };

    let device_address = device.address.clone();
    let device_name = device.name.clone();
    let low_threshold = device.low_battery_threshold;
    let update_interval = Duration::from_secs(config.general.update_interval);

    let battery_info = Arc::new(Mutex::new("Loading...".to_string()));

    // Initial battery read
    update_battery_info(Arc::clone(&battery_info), &device_address, low_threshold).await;

    // Create channel for commands
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Create tray service
    let tray = BatteryTray::new(Arc::clone(&battery_info), tx, device_name.clone());
    let service = TrayService::new(tray);
    let handle = service.handle();
    service.spawn();

    println!(
        "Battery monitor tray started for: {} ({})",
        device_name, device_address
    );
    println!("Update interval: {} seconds", update_interval.as_secs());
    println!("Config file: {}", Config::config_path()?.display());

    // Handle commands and periodic updates
    let info_clone = Arc::clone(&battery_info);
    let mut interval = tokio::time::interval(update_interval);
    let device_addr_clone = device_address.clone();

    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                match cmd {
                    Command::Refresh => {
                        update_battery_info(Arc::clone(&battery_info), &device_address, low_threshold).await;
                        handle.update(|_| {});
                    }
                    Command::Quit => {
                        std::process::exit(0);
                    }
                }
            }
            _ = interval.tick() => {
                update_battery_info(Arc::clone(&info_clone), &device_addr_clone, low_threshold).await;
                handle.update(|_| {});
            }
        }
    }
}

use anyhow::Result;
use ksni::menu::StandardItem;
use ksni::{MenuItem, Tray, TrayService};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use zmk_battery_monitor::ZmkBatteryReader;

const DEVICE_ADDRESS: &str = "";
const REFRESH_INTERVAL: Duration = Duration::from_secs(60); // Update every minute

enum Command {
    Refresh,
    Quit,
}

struct BatteryTray {
    battery_info: Arc<Mutex<String>>,
    tx: mpsc::UnboundedSender<Command>,
}

impl BatteryTray {
    fn new(battery_info: Arc<Mutex<String>>, tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { battery_info, tx }
    }
}

impl Tray for BatteryTray {
    fn icon_name(&self) -> String {
        "battery".to_string()
    }

    fn title(&self) -> String {
        "ZMK Battery Monitor".to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let info = self.battery_info.lock().unwrap();
        ksni::ToolTip {
            title: "ZMK Keyboard Battery".to_string(),
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

async fn update_battery_info(battery_info: Arc<Mutex<String>>) {
    match read_battery().await {
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

async fn read_battery() -> Result<String> {
    let reader = ZmkBatteryReader::new().await?;
    let batteries = reader.read_battery_levels(DEVICE_ADDRESS).await?;

    if batteries.is_empty() {
        Ok("No battery data available".to_string())
    } else {
        let info = batteries
            .iter()
            .map(|b| format!("{}: {}%", b.name, b.level))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(info)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let battery_info = Arc::new(Mutex::new("Loading...".to_string()));

    // Initial battery read
    update_battery_info(Arc::clone(&battery_info)).await;

    // Create channel for commands
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Create tray service
    let tray = BatteryTray::new(Arc::clone(&battery_info), tx);
    let service = TrayService::new(tray);
    let handle = service.handle();
    service.spawn();

    println!("Battery monitor tray started. Hover over the tray icon to see battery levels.");
    println!(
        "The tray will refresh every {} seconds.",
        REFRESH_INTERVAL.as_secs()
    );

    // Handle commands and periodic updates
    let info_clone = Arc::clone(&battery_info);
    let mut interval = tokio::time::interval(REFRESH_INTERVAL);

    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                match cmd {
                    Command::Refresh => {
                        update_battery_info(Arc::clone(&battery_info)).await;
                        handle.update(|_| {});
                    }
                    Command::Quit => {
                        std::process::exit(0);
                    }
                }
            }
            _ = interval.tick() => {
                update_battery_info(Arc::clone(&info_clone)).await;
                handle.update(|_| {});
            }
        }
    }
}

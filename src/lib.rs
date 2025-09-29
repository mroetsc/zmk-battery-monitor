use anyhow::{Context, Result};
use std::collections::HashMap;
use zbus::{zvariant, Connection};

pub mod config;
pub use config::Config;

pub const BATTERY_UUID: &str = "0000180f-0000-1000-8000-00805f9b34fb";
pub const BATTERY_LEVEL_UUID: &str = "00002a19-0000-1000-8000-00805f9b34fb";
pub const BATTERY_USER_DESC: &str = "00002901-0000-1000-8000-00805f9b34fb";

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String,
    pub level: u8,
}

pub struct ZmkBatteryReader {
    conn: Connection,
}

impl ZmkBatteryReader {
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self { conn })
    }

    pub async fn read_battery_levels(&self, device_address: &str) -> Result<Vec<BatteryInfo>> {
        let device_path = format!(
            "/org/bluez/hci0/dev_{}",
            device_address.replace([':', '-'], "_")
        );

        // Get all managed objects
        let proxy = zbus::Proxy::new(
            &self.conn,
            "org.bluez",
            "/",
            "org.freedesktop.DBus.ObjectManager",
        )
        .await?;

        let reply = proxy.call_method("GetManagedObjects", &()).await?;
        let managed_objects: HashMap<
            zvariant::OwnedObjectPath,
            HashMap<String, HashMap<String, zvariant::OwnedValue>>,
        > = reply.body().deserialize()?;

        let mut batteries = Vec::new();

        // Find battery services
        for (path, interfaces) in managed_objects.iter() {
            let path_str = path.as_str();
            if !path_str.starts_with(&device_path) {
                continue;
            }

            if let Some(service_props) = interfaces.get("org.bluez.GattService1") {
                if let Some(uuid_value) = service_props.get("UUID") {
                    let service_uuid: String = uuid_value.try_to_owned()?.try_into()?;

                    if service_uuid == BATTERY_UUID {
                        // Find battery characteristics
                        if let Some(battery_info) = self
                            .read_battery_from_service(path_str, &managed_objects)
                            .await?
                        {
                            batteries.push(battery_info);
                        }
                    }
                }
            }
        }

        Ok(batteries)
    }

    async fn read_battery_from_service(
        &self,
        service_path: &str,
        managed_objects: &HashMap<
            zvariant::OwnedObjectPath,
            HashMap<String, HashMap<String, zvariant::OwnedValue>>,
        >,
    ) -> Result<Option<BatteryInfo>> {
        for (char_path, char_interfaces) in managed_objects.iter() {
            let char_path_str = char_path.as_str();
            if !char_path_str.starts_with(service_path) || char_path_str == service_path {
                continue;
            }

            if let Some(char_props) = char_interfaces.get("org.bluez.GattCharacteristic1") {
                if let Some(char_uuid_value) = char_props.get("UUID") {
                    let char_uuid: String = char_uuid_value.try_to_owned()?.try_into()?;

                    if char_uuid == BATTERY_LEVEL_UUID {
                        // Read battery level
                        let char_proxy = zbus::Proxy::new(
                            &self.conn,
                            "org.bluez",
                            char_path_str,
                            "org.bluez.GattCharacteristic1",
                        )
                        .await?;

                        let options: HashMap<String, zvariant::Value> = HashMap::new();
                        let reply = char_proxy
                            .call_method("ReadValue", &(options,))
                            .await
                            .context("Failed to read battery value")?;

                        let battery_data: Vec<u8> = reply.body().deserialize()?;
                        let level = battery_data.first().copied().unwrap_or(0);

                        // Get battery name from descriptor
                        let name = self
                            .read_battery_name(char_path_str, managed_objects)
                            .await?
                            .unwrap_or_else(|| "Battery".to_string());

                        return Ok(Some(BatteryInfo { name, level }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn read_battery_name(
        &self,
        char_path: &str,
        managed_objects: &HashMap<
            zvariant::OwnedObjectPath,
            HashMap<String, HashMap<String, zvariant::OwnedValue>>,
        >,
    ) -> Result<Option<String>> {
        for (desc_path, desc_interfaces) in managed_objects.iter() {
            let desc_path_str = desc_path.as_str();
            if !desc_path_str.starts_with(char_path) || desc_path_str == char_path {
                continue;
            }

            if let Some(desc_props) = desc_interfaces.get("org.bluez.GattDescriptor1") {
                if let Some(desc_uuid_value) = desc_props.get("UUID") {
                    let desc_uuid: String = desc_uuid_value.try_to_owned()?.try_into()?;

                    if desc_uuid == BATTERY_USER_DESC {
                        let desc_proxy = zbus::Proxy::new(
                            &self.conn,
                            "org.bluez",
                            desc_path_str,
                            "org.bluez.GattDescriptor1",
                        )
                        .await?;

                        let desc_options: HashMap<String, zvariant::Value> = HashMap::new();
                        if let Ok(reply) =
                            desc_proxy.call_method("ReadValue", &(desc_options,)).await
                        {
                            if let Ok(desc_data) = reply.body().deserialize::<Vec<u8>>() {
                                if let Ok(desc_str) = String::from_utf8(desc_data) {
                                    return Ok(Some(desc_str.trim_end_matches('\0').to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn list_devices(&self) -> Result<Vec<(String, String)>> {
        let proxy = zbus::Proxy::new(
            &self.conn,
            "org.bluez",
            "/",
            "org.freedesktop.DBus.ObjectManager",
        )
        .await?;

        let reply = proxy.call_method("GetManagedObjects", &()).await?;
        let managed_objects: HashMap<
            zvariant::OwnedObjectPath,
            HashMap<String, HashMap<String, zvariant::OwnedValue>>,
        > = reply.body().deserialize()?;

        let mut devices = Vec::new();

        for (_path, interfaces) in managed_objects.iter() {
            if let Some(device_props) = interfaces.get("org.bluez.Device1") {
                if let (Some(name_value), Some(address_value)) =
                    (device_props.get("Name"), device_props.get("Address"))
                {
                    if let (Ok(name), Ok(address)) = (
                        name_value.try_to_owned()?.try_into(),
                        address_value.try_to_owned()?.try_into(),
                    ) {
                        devices.push((name, address));
                    }
                }
            }
        }

        Ok(devices)
    }
}

use anyhow::{Context, Result};
use std::collections::HashMap;
use zbus::{zvariant, Connection};

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

impl ZmkBatteryReader {}

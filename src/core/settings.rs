use serde::{Serialize, Deserialize};
use crate::core::error::{Result, AgrgErr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSettings {
    pub working_mode: WorkingMode,
    pub pinpad_mode: PinpadMode,
    pub reader_mode: ReaderMode,
    pub access_mode: AccessMode,
    pub admin_password: [u8; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkingMode {
    CardReader,
    Controller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinpadMode {
    Wiegand6,
    Wiegand26Hex,
    Wiegand26Dec,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReaderMode {
    Wiegand26,
    Wiegand34,
    Off,
    Wiegand58
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessMode {
    PinOrCard,
    Pin,
    Card,
    PinAndCard
}

impl DeviceSettings {
    const SETTINGS_SIZE: usize = 16;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SETTINGS_SIZE {
            return Err(AgrgErr::InvalidInput(format!("Settings byte vector of incorrect length (Expected: 16; Given: {})", bytes.len())));
        }

        Ok(Self {
            working_mode: WorkingMode::from_byte(bytes[0])?,
            pinpad_mode: PinpadMode::from_byte(bytes[1])?,
            reader_mode: ReaderMode::from_byte(bytes[2])?,
            access_mode: AccessMode::from_byte(bytes[3])?,
            admin_password: bytes[4..10].try_into().unwrap(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::SETTINGS_SIZE);
        bytes.push(self.working_mode.as_byte());
        bytes.push(self.pinpad_mode.as_byte());
        bytes.push(self.reader_mode.as_byte());
        bytes.push(self.access_mode.as_byte());
        bytes.extend(&self.admin_password);
        bytes.resize(Self::SETTINGS_SIZE, 0xFF);
        bytes
    }
}
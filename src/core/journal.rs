use crate::core::error::Result;
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: DateTime<Local>,
    pub event_type: EventType,
    pub user_id: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    PowerOn,
    RegisteredUser,
    UnknownUid,
    UnknownPin,
    HandleUnlocked,
    ForcedUnlock,
    HandleLocked,
    ProgrammingMode,
    WrongAdminPassword,
    Custom(String),
}

impl JournalEntry {
    const ENTRY_SIZE: usize = 16;

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::ENTRY_SIZE {
            return Err(Error::InvalidInput(format!(
                "Journal entry must be {} bytes",
                Self::ENTRY_SIZE
            )));
        }

        let timestamp = parse_timestamp(&bytes[0..7])?;
        let event_type = EventType::from_byte(bytes[7]);
        let user_id = bytes[8];
        let data = bytes[9..16].to_vec();

        Ok(Self {
            timestamp,
            event_type,
            user_id,
            data,
        })
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{}",
            self.timestamp.to_rfc3339(),
            self.event_type,
            self.user_id,
            hex::encode(&self.data)
        )
    }
}


// TODO: implement that!!!!
fn parse_timestamp(bytes: &[u8]) -> Result<DateTime<Local>> {
    Ok(Local::now())
}

impl EventType {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::PowerOn,
            0x01 => Self::RegisteredUser,
            0x02 => Self::UnknownUid,
            0x03 => Self::UnknownPin,
            0x06 => Self::HandleUnlocked,
            0x07 => Self::ForcedUnlock,
            0x08 => Self::HandleLocked,
            0x09 => Self::ProgrammingMode,
            0x0A => Self::WrongAdminPassword,
            _ => Self::Custom(format!("Unknown(0x{:02X})", byte)),
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PowerOn => write!(f, "Power On"),
            Self::RegisteredUser => write!(f, "Registered User"),
            Self::UnknownUid => write!(f, "Unknown UID"),
            Self::UnknownPin => write!(f, "Unknown PIN"),
            Self::HandleUnlocked => write!(f, "Handle Unlocked"),
            Self::ForcedUnlock => write!(f, "Break-in"),
            Self::HandleLocked => write!(f, "Handle Locked"),
            Self::ProgrammingMode => write!(f, "Entered Programming Mode"),
            Self::WrongAdminPassword => write!(f, "incorrect Admin Password"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}
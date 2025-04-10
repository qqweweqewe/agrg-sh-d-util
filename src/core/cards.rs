use crate::core::error::{Error, Result};
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub uid: String,
    pub pin: String,
}

impl Card {
    const UID_LENGTH: usize = 20;
    const PIN_LENGTH: usize = 6;
    const BINARY_SIZE: usize = 16;

    pub fn new(uid: String, pin: String) -> Result<Self> {
        if uid.len() > Self::UID_LENGTH {
            return Err(Error::InvalidInput(format!(
                "UID must be {} hex characters",
                Self::UID_LENGTH
            )));
        }

        if pin.len() != Self::PIN_LENGTH {
            return Err(Error::InvalidInput(format!(
                "PIN must be {} digits",
                Self::PIN_LENGTH
            )));
        }

        Ok(Self { uid, pin })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let uid_bytes = hex::decode(&self.uid)
            .map_err(|_| Error::Parse("Invalid UID hex format".into()))?;

        let pin_bytes: Vec<u8> = self.pin
            .chars()
            .map(|c| c.to_digit(10)
                .ok_or(Error::Parse("Invalid PIN character".into()))
                .and_then(|d| u8::try_from(d).map_err(|_| Error::Parse("PIN out of range".into()))))
            .collect::<Result<_>>()?;

        let mut bytes = Vec::with_capacity(Self::BINARY_SIZE);
        bytes.extend(uid_bytes);
        bytes.extend(pin_bytes);
        bytes.resize(Self::BINARY_SIZE, 0xFF);  // Pad with 0xFF

        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::BINARY_SIZE {
            return Err(Error::InvalidInput(format!(
                "Card data must be exactly {} bytes",
                Self::BINARY_SIZE
            )));
        }

        let uid_bytes = Self::trim_ff_padding(&bytes[..10]);
        let pin_bytes = Self::trim_ff_padding(&bytes[10..16]);

        let uid = hex::encode(uid_bytes);
        let pin = pin_bytes.iter()
            .map(|b| {
                if b > &9 {
                    Err(Error::Parse("Invalid PIN byte value".into()))
                } else {
                    Ok((b'0' + b) as char)
                }
            })
            .collect::<Result<String>>()?;

        Self::new(uid, pin)
    }

    fn trim_ff_padding(data: &mut Vec<u8>) -> &[u8] {
        let end = data.iter()
            .rposition(|b| b != &0xFF)
            .map(|pos| pos + 1)
            .unwrap_or(0);
        &data[..end]
    }

    fn pad_ff(data: &mut Vec<u8>, len: usize) -> &mut Vec<u8> {
        data.resize(len, 0xff);
        data
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Card(UID: {}, PIN: {})", self.uid, self.pin)
    }
}
mod port;

use crate::core::error::{ Result, AgrgErr };
use std::time::Duration;

const DEFAULT_BAUD_RATE: u32 = 38400;
const READ_TIMEOUT: Duration = Duration::from_millis(100);
const WRITE_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: serialport::DataBits,
    pub stop_bits: serialport::StopBits,
    pub parity: serialport::Parity,
    pub flow_control: serialport::FlowControl,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baud_rate: DEFAULT_BAUD_RATE,
            data_bits: serialport::DataBits::Eight,
            stop_bits: serialport::StopBits::One,
            parity: serialport::Parity::None,
            flow_control: serialport::FlowControl::None,
        }
    }
}

pub fn available_ports() -> Result<Vec<String>> {
    Ok(match serialport::available_ports() {
        Ok(thing) => {
            thing.into_iter()
            .map(|p| p.port_name)
            .collect()
        }
        Err(_) => Vec::new()
    })
}
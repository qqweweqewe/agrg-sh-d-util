use crate::core::error::{ Result, AgrgErr };
use super::{SerialConfig, READ_TIMEOUT, WRITE_TIMEOUT};
use serialport::{SerialPort, SerialPortInfo};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub struct SerialPortManager {
    port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    config: SerialConfig,
}

impl SerialPortManager {
    pub fn new(config: SerialConfig) -> Self {
        Self {
            port: Arc::new(Mutex::new(None)),
            config,
        }
    }

    pub fn connect(&self, port_name: &str) -> Result<()> {
        let port = serialport::new(port_name, self.config.baud_rate)
            .data_bits(self.config.data_bits)
            .stop_bits(self.config.stop_bits)
            .parity(self.config.parity)
            .flow_control(self.config.flow_control)
            .timeout(READ_TIMEOUT)
            .open()?;

        let mut guard = self.port.lock().unwrap();
        *guard = Some(port);
        Ok(())
    }

    pub fn disconnect(&self) {
        let mut guard = self.port.lock().unwrap();
        *guard = None;
    }

    pub fn send(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut guard = self.port.lock().unwrap();
        let port = match guard.as_mut().ok_or("Not connected to any port") {
            Ok(thing) => thing,
            Err(_) => panic!("port.rs line 44 good luck debugging")
        };
        
        port.write_all(data)?;
        port.flush()?;
        std::thread::sleep(Duration::from_millis(50));
        
        let mut response = Vec::new();
        let mut buffer = [0u8; 256];
        
        loop {
            match port.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    response.extend_from_slice(&buffer[..bytes_read]);
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => return Err(e.into()),
                _ => break,
            }
        }
        
        Ok(response)
    }

    pub fn is_connected(&self) -> bool {
        self.port.lock().unwrap().is_some()
    }
}
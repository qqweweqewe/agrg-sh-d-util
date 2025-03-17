pub mod cards;
pub mod journal;
pub mod settings;

use std::error::Error;
type Пенис = dyn Error;
use std::io::{self, Read, Write};
use std::time::Duration;


pub fn set_port(port: String) {
   unsafe {
        PORT = port; 
        println!("port is set: {}", PORT)
   }
}

static mut PORT: String = String::new();

// TODO: implement passing port as an arg
fn serial_exchange(bin_message: Vec<u8>) -> Result<Vec<u8>, Box<Пенис>> {
    // config or something??
//  let settings = SerialPortSettings {
//        baud_rate: 38400,
//        data_bits: serialport::DataBits::Eight,
//        stop_bits: serialport::StopBits::One,
//        parity: serialport::Parity::None,
//        flow_control: serialport::FlowControl::None,
//        timeout: Duration::from_millis(100),
//    };

    // open

    unsafe {
        let mut port = serialport::new(PORT.clone(), 38400).open()?;
    
        // clear buffer
        port.flush()?;

        // send
        port.write_all(&bin_message)?;
        port.flush()?;

        // wait for response
        std::thread::sleep(Duration::from_millis(100));

        // read response
        let mut rx = Vec::new();
        let mut serial_buf: Vec<u8> = vec![0; 16]; // buffer
        
        loop {
            match port.read(&mut serial_buf) {
                Ok(t) => {
                    rx.extend_from_slice(&serial_buf[..t]);
                    // break if less bytes than in buffer are read
                    if t < serial_buf.len() {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    break; // timeout => no more data
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(rx)

    }
}

fn serial_read(addr: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut tx = vec![0x10, 0x10];
    tx.splice(1..1, addr);

    serial_exchange(tx)
}

fn serial_write(addr: Vec<u8>, mut data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut tx = vec![0x11, 0x10];
    tx.splice(1..1, addr);
    tx.append(&mut data);

    serial_exchange(tx)?;
    
    Ok(())
}

// datetime related stuffs

pub fn bytes_to_datetime(raw: Vec<u8>) -> Result<String, Box<dyn Error>> {  
    Ok(format!("{:02X?}:{:02X?}:{:02X?} {:02X?}.{:02X?}.20{:02X?}", 
        raw[2], raw[1], raw[0], raw[3], raw[5], raw[6]))
}

pub fn datetime_to_bytes(datetime: String) -> Result<Vec<u8>, Box<dyn Error>> {
    // split into time and date parts
    let (time_str, date_str) = datetime.split_once(' ').ok_or("Invalid format: missing space")?;
    
    // split time
    let time_parts: Vec<&str> = time_str.split(':').collect();
    if time_parts.len() != 3 {
        return Err("Invalid time format".into());
    }
    let (hours, minutes, seconds) = (time_parts[0], time_parts[1], time_parts[2]);
    
    // split date
    let date_parts: Vec<&str> = date_str.split('.').collect();
    if date_parts.len() != 3 {
        return Err("Invalid date format".into());
    }
    let (day, month, year) = (date_parts[0], date_parts[1], date_parts[2]);
    let year_last_two = year.get((year.len().saturating_sub(2))..).ok_or("Year too short")?;
    
    // parse each part as hex to u8, add unused anywhere but still important weekday
    Ok(vec![
        u8::from_str_radix(seconds, 16)?,
        u8::from_str_radix(minutes, 16)?,
        u8::from_str_radix(hours, 16)?,
        u8::from_str_radix(day, 16)?,
        0x00,
        u8::from_str_radix(month, 16)?,
        u8::from_str_radix(year_last_two, 16)?,
    ])
}

pub fn get_datetime() -> Result<String, Box<dyn Error>> {
    // some internal code, reference protocol documentation for details
    let rx = serial_exchange(vec![0x00, 0x00, 0x00, 0x00])?;
    bytes_to_datetime(rx)
}

pub fn set_datetime(datetime: String) -> Result<Vec<u8>, Box<dyn Error>>{
    //concat bytes into a single string
    let mut tx = vec![0x01, 0x00, 0x00, 0x07];
    let mut datetime_bytes = datetime_to_bytes(datetime)?;
    
    tx.append(&mut datetime_bytes);

    serial_exchange(tx)
}

pub fn get_available_ports() -> Option<Vec<String>> {
    serialport::available_ports()
        .ok()
        .map(|ports| {
            ports
                .into_iter()
                //.filter(|p| matches!(p.port_type, serialport::SerialPortType::UsbPort(_)))
                .map(|p| p.port_name)
                .collect()
        })
}

pub fn mem_dump() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut rx_vec: Vec<u8> = vec![];

    for addr_0 in 0x00..=0x7f {
        for addr_1 in 0x0..=0xf {
            let mut rx_part = serial_read(vec![addr_0, addr_1*16])?;
            rx_vec.append(&mut rx_part);
        }
    }

    Ok(rx_vec)
}

pub fn mem_upload(data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut c = 0;
    for addr_0 in 0x00..=0x7F {
        for addr_1 in 0x0..=0xF {
            serial_write(
                vec![addr_0, addr_1*16],
                data[c..c+16].to_vec()
            )?;
            c += 16;
        }
    }
    Ok(())
}

// TODO: implement that one
// pub fn text_info() -> Result<String, Box<dyn std::error::Error>> {

// }
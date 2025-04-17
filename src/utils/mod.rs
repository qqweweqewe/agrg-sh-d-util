pub mod cards;
pub mod journal;
pub mod settings;

use std::error::Error;
type Пенис = dyn Error;

use std::{io::{self, Read, Write},
    time::Duration,
    sync::{
        Arc, Mutex
    }
};

use lazy_static::lazy_static;

lazy_static! {
    static ref PORT: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}


pub fn set_port(port: String) {
    let mut global_port = PORT.lock().unwrap();
    *global_port = port;
    println!("Port set to: {}", *global_port);
}

pub fn check_handle(port: String) -> bool {
    set_port(port);
    match get_datetime() {
        Ok(val) => {
            val != []
        },
        Err(_) => false,
    }
}

pub fn scan_ports() -> Option<String> {
    println!("Scanning ports");
    let ports = get_available_ports();
    println!("Found ports: {:?}", &ports);

    match ports {
        Some(ports) => {
            
            if ports.len() == 1 {
                if check_handle(ports[0].clone()) { return Some(ports[0].clone()) }
            }
            
            for port in ports {
                if check_handle(port.clone()) {
                    println!("found handle on port {}", port);
                    return Some(port)
                    
                };
            }
            set_port(String::new());
            None
        },
        None => { println!("handle not found"); None }
    }
}

// static mut PORT: String = String::new();

//TODO: implement passing port as an arg
fn atomic_serial_exchange(bin_message: Vec<u8>) -> Result<Vec<u8>, Box<Пенис>> {
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

    let port_name = PORT.lock().unwrap().clone();


    
        let mut port = serialport::new(
            port_name, 
            38400 
        ).open()?; 
    
        // clear buffer
        port.flush()?;

        // send
        port.write_all(&bin_message)?;
        port.flush()?;

        // wait for response
        std::thread::sleep(Duration::from_millis(50));

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

// fn atomic_serial_exchange(message: Vec<u8>) -> Result<Vec<u8>, Box<Пенис>>{
//     let mut file = OpenOptions::new()
//         .append(true)
//         .create(true)
//         .open("log.txt")?;

//         for byte in &message {
//             write!(file, "{:02x}", byte)?;
//         }

//         writeln!(file, "\n");
    
//     Ok(Vec::new())
// }

fn serial_write(addr: Vec<u8>, mut data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut tx = vec![0x02, 0x10];
    tx.splice(1..1, addr);
    tx.append(&mut data);

    atomic_serial_exchange(tx)?;
    
    Ok(())
}

// datetime related stuffs

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

pub fn get_datetime() -> Result<Vec<u8>, Box<dyn Error>> {
    // some internal code, reference protocol documentation for details
    atomic_serial_exchange(vec![0x01, 0x00, 0x00, 0x00])
    
}

pub fn set_datetime(datetime: String) -> Result<Vec<u8>, Box<dyn Error>>{
    //concat bytes into a single string
    let mut tx = vec![0x00, 0x00, 0x00, 0x07];
    let mut datetime_bytes = datetime_to_bytes(datetime)?;
    
    tx.append(&mut datetime_bytes);

    atomic_serial_exchange(tx)
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

    for base_addr in (0x0000..=0x7FFF).step_by(32) {
        let addr_bytes = (base_addr as u16).to_be_bytes();
        let command = vec![0x03, addr_bytes[0], addr_bytes[1], 0x20];

        let mut rx_part = atomic_serial_exchange(command)?;
        println!("{:04X}: {:X?}", base_addr, &rx_part);
        if (base_addr >= 0x1000 && rx_part == vec![0xff; 32]) || (rx_part) == vec![] {
           return Ok(rx_vec);
        };
        rx_vec.append(&mut rx_part);
    }
    println!("len:{:?}", rx_vec.len());
    println!("{:X?}", rx_vec[0]);
    Ok(rx_vec)
}

pub fn mem_upload(data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {

    for base_addr in (0x0000..0x1000).step_by(16) {
        if base_addr == 0x0000 {
            let mut settings1 = data[0x0000..0x0008].to_vec();
            let mut settings2 = data[0xA..=0xF].to_vec();
            let mut message1 = vec![0x02, 0x00, 0x00, 0x08];
            let mut message2 = vec![0x02, 0x00, 0x0A, 0x06];
            
            message1.append(&mut settings1);
            message2.append(&mut settings2);

            atomic_serial_exchange(message1);
            atomic_serial_exchange(message2);
            

        } else {
            let addr = (base_addr as u16).to_be_bytes();
            println!("{:4X}: {:X?}", base_addr, &data[(base_addr as usize)..(base_addr as usize)+16].to_vec());
            serial_write(
                vec![addr[0], addr[1]],
                data[(base_addr as usize)..(base_addr as usize)+16].to_vec()
            )?;
        }
    }
    Ok(())
}

// no prog mode here
// pub fn get_text() -> Option<String> {
//     println!("getting text");
//     match atomic_serial_exchange(vec![131, 0x00, 0x00, 64]) {
//         Ok(res) => match res.as_slice() {
//             [] => { println!("no response on text"); return None },
//             _ => Some(
//                 match String::from_utf8(cards::trim_empty(res)) {
//                     Ok(thing) => { println!("Пришло: {}", &thing); thing },
//                     Err(_) => { println!("Че реально в utf-8 не перевелось?"); String::new() }
//                 }
//             )
//         }
//         Err(_) => {println!("error getting text"); None }
//     }
// }

pub fn get_text() -> Option<String> {
    println!("getting text");
    match atomic_serial_exchange(vec![131, 0x00, 0x00, 64]) {
        Ok(res) => {
            let cleaned = cards::trim_empty(res);
            if cleaned.is_empty() {
                println!("no response on text");
                return None;
            }
            
            let s = cleaned.iter()
                .filter_map(|&b| {
                    // Keep only ASCII characters (0-127)
                    if b <= 127 {
                        Some(b as char)
                    } else {
                        None  // Discard non-ASCII bytes
                    }
                })
                .collect::<String>();
            
            println!("Пришло: {}", &s);
            Some(s)
        }
        Err(_) => {
            println!("error getting text"); 
            None
        }
    }
}

pub fn set_text(input: Vec<u8>) {
    let mut append = input;
    let mut tx = vec![0x82, 0x00, 0x00, 0x40];
    
    tx.append(&mut append);

    match atomic_serial_exchange(tx) {
        Ok(_) => {},
        Err(_) => {}
    }
}

// no prog mode here
// pub fn agrg_text_info() -> Option<String> {
//     match atomic_serial_exchange(vec![0x11, 0x00, 0x00, 0xFF]) {
//         Ok(res) => match res.as_slice() {
//             [] => { return None },
//             _ => Some(
//                 match String::from_utf8(trim_empty(res)) {
//                     Ok(thing) => thing,
//                     Err(_) => {
//                         println!("агрг тоже хуета или его просто нет?"); 
//                         String::new()
//                     }
//                 }
//             )
//         }
//         Err(_) => { None }
//     }
// }

pub fn agrg_text_info() -> Option<String> {
    match atomic_serial_exchange(vec![0x11, 0x00, 0x00, 0xFF]) {
        Ok(res) => {
            if res.is_empty() {
                println!("Empty response after trimming");
                return None;
            }

            let s = res.iter()
                .filter_map(|&b| {
                    // Keep only ASCII characters (0-127)
                    if b <= 127 {
                        Some(b as char)
                    } else {
                        None  // Silently discard non-ASCII bytes
                    }
                })
                .collect::<String>();

            if s.is_empty() {
                println!("Filtered out all characters as non-ASCII");
                None
            } else {
                Some(s)
            }
        }
        Err(_) => {
            println!("Communication error");
            None
        }
    }
}




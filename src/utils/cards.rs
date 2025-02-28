use std::{error::Error, path::Path, result, str::Chars};
use super::{serial_read, serial_write};
use serde::{Serialize, Deserialize};
use chrono::Local;

#[derive(Serialize, Deserialize)]
pub struct Card {
    pub rfid: String,
    pub pin: String,
}

#[derive(Serialize, Deserialize)]
struct CsvRecord {
    addr: usize,
    rfid: String,
    pin: String,
}

pub fn get_card(addr: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    if addr.len() !=2 { return Err("address invalid".into()); }
    match u16::from_be_bytes([addr[0], addr[1]]) {
        0x0010..=0x0FFF => serial_read(addr),
        _ => Err("invalid address".into())
    } 
}

pub fn bulk_read() -> Result<Vec<(Vec<u8>, Vec<u8>)>, Box<dyn Error>> {
    let mut rx_vec: Vec<Vec<u8>> = vec![];

    for addr_0 in 0x00..=0x0F {
        for addr_1 in 0x10..=0xFF {
            rx_vec.push(serial_read(vec![addr_0, addr_1])?);
        }
    }

    Ok(rx_vec)
}

pub fn bulk_write(entries: Vec<Vec<u8>>) -> Result<(), Box<dyn Error>> {
    let mut rx_vec: Vec<Vec<u8>> = vec![];

    for addr_0 in 0x00..=0x0F {
        for addr_1 in 0x10..=0xFF {
            rx_vec.push(serial_write(vec![addr_0, addr_1],  )?);
        }
    }

    Ok(())
}

pub fn parse(card_bytes: Vec<u8>) -> Result<Card, Box<dyn Error>> {
    if card_bytes.len() != 16 { 
        return Err(format!("incorrect entry length: must be 16, got {}", card_bytes.len()).into())
    }

    let rfid = trim_empty(hex::encode_upper(&card_bytes[..10]));
    let pin = trim_leading_zero(trim_empty(hex::encode_upper(&card_bytes[10..16])));

    Ok(Card {
        rfid,
        pin
    })

}


pub fn export_csv(cards: Vec<Vec<u8>>) -> Result<(), Box<dyn Error>> {

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("cards_{}.csv", timestamp);
    
    let mut writer = csv::Writer::from_path(filename)?;
    
    for (index, card_bytes) in cards.iter().enumerate() {
        let card = parse(card_bytes.clone())?;
        
        writer.serialize(CsvRecord {
            addr: index + 1, 
            rfid: card.rfid,
            pin: card.pin,
        })?;
    }
    
    writer.flush()?;
    Ok(())
}


pub fn import_csv(filename: &str) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(filename)?;
    let mut records = Vec::new();

    // Read and parse all records
    for result in reader.deserialize() {
        let record: CsvRecord = result?;
        records.push(record);
    }

    // Sort by original address (1-based index)
    records.sort_by_key(|r| r.addr);

    let mut card_data = Vec::new();

    for record in records {
        // reconstruct RFID
        let rfid_bytes = {
            let mut hex_str = record.rfid.to_uppercase();
            // Pad/cut to 20 characters (10 bytes)
            hex_str.truncate(20);
            while hex_str.len() < 20 {
                hex_str.push('F');
            }
            hex::decode(hex_str)?
        };

        // reconstruct PIN
        let pin_bytes = {
            let mut hex_str = String::new();
            
            for c in record.pin.chars() {
                hex_str.push('0');
                hex_str.push(c);
            }
            
            hex_str.truncate(12);
            while hex_str.len() < 12 {
                hex_str.push('F');
            }
            hex::decode(hex_str)?
        };

        // combine to 16 bytes
        let mut card_bytes = Vec::with_capacity(16);
        card_bytes.extend(rfid_bytes);
        card_bytes.extend(pin_bytes);
        
        card_data.push(card_bytes);
    }

    Ok(card_data)
}


fn trim_leading_zero(s: String) -> String {

    let mut res: Vec<char> = Vec::new();
    let mut c = 0;

    for i in s.chars() {
        if c % 2 != 0 {
            res.push(i);
        }
        c += 1;
    };

    res.into_iter().collect()
}

fn trim_empty(s: String) -> String {
    // "FF" bytes are usually mean they're empty thats a usual memory mechanism in sh-d
    // check if the entire string is "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    if s.chars().all(|c| c == 'F') {
        return String::new();
    }

    if !s.chars().any(|c| c == 'F') {
        return s
    }

    // count trailing "FF"s
    let mut count = 0;
    let chars = s.chars().rev().collect::<Vec<_>>();
    let mut i = 0;
    while i + 1 < chars.len() {
        if chars[i] == 'F' && chars[i + 1] == 'F' {
            count += 1;
            i += 2;
        } else {
            break;
        }
    }

    // determine how many "FF"s keep
    let desired = if count >= 3 {
        7
    } else if count >= 6 {
        4
    } else {
        0
    };

    // calculate new length and trim the string
    
    s[..(s.len() - (desired*2) +2)].to_string()
}

pub fn delete(addr: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let empty: Vec<u8> = vec![0xff; 16];

    serial_write(addr, empty)?;
    
    Ok(())
}

pub fn bulk_delete(addrs: Vec<Vec<u8>>) -> Result<(), Box<dyn Error>>{
    
    for addr in addrs {
        delete(addr)?;
    }
    

    Ok(())
}
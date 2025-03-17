use std::error::Error;
use serde::{Serialize, Deserialize};
use chrono::Local;
use super::{serial_read, serial_write};

type Address = Vec<u8>;  // 2-byte address 
type CardData = Vec<u8>; // 16-byte card data

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub rfid: String,
    pub pin: String,
}

#[derive(Serialize, Deserialize)]
pub struct CsvRecord {
    #[serde(rename = "Address")]
    address: String,
    #[serde(rename = "RFID")]
    rfid: String,
    #[serde(rename = "PIN")]
    pin: String,
}

// core operations
pub fn bulk_read() -> Result<Vec<(Address, CardData)>, Box<dyn Error>> {
    let mut cards = Vec::new();

    for addr_high in 0x00..=0x0F {
        for addr_low in 0x10..=0xFF {
            let addr = vec![addr_high, addr_low];
            match get_card(addr.clone()) {
                Ok(data) => cards.push((addr, data)),
                Err(e) => eprintln!("Skipped address {:02X}{:02X}: {}", addr_high, addr_low, e),
            }
        }
    }
    
    Ok(cards)
}

pub fn bulk_write(entries: Vec<(Address, CardData)>) -> Result<(), Box<dyn Error>> {
    for (addr, data) in entries {
        validate_address(&addr)?;
        validate_card_data(&data)?;
        serial_write(addr, data)?;
    }
    Ok(())
}

pub fn export_bin(cards: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    // let mut writer = csv::Writer::from_path()?;

    // for (i, card_bytes) in cards.iter().enumerate() {
        // let card = parse(*card_bytes)?;
        // let address_num = u16::from_be_bytes([addr[0], addr[1]]);

    std::fs::write(format!("cards_{}.bin", timestamp), cards)?;
    Ok(())
}



// pub fn import_csv(filename: &str) -> Result<Vec<(Address, CardData)>, Box<dyn Error>> {
//     let mut reader = csv::Reader::from_path(filename)?;
//     let mut entries = Vec::new();

//     for result in reader.deserialize() {
//         let record: CsvRecord = result?;
        
//         let address = parse_csv_address(&record.address)?;
//         let data = reconstruct_card_data(&record)?;
        
//         entries.push((address, data));
//     }

//     Ok(entries)
// }

// helper functions
fn parse_csv_address(addr_str: &str) -> Result<Address, Box<dyn Error>> {
    let addr_num: u16 = addr_str.parse()?;
    Ok(addr_num.to_be_bytes().to_vec())
}

fn reconstruct(card: Card) -> Result<CardData, Box<dyn Error>> {
    let mut bytes = Vec::with_capacity(16);
    
    // reconstruct RFID
    let rfid_bytes = hex::decode(pad_hex(&card.rfid, 20))?;
    bytes.extend(rfid_bytes);
    
    // reconstruct PIN
    let pin_encoded = card.pin.chars()
        .flat_map(|c| ['0', c])
        .collect::<String>();
    let pin_bytes = hex::decode(pad_hex(&pin_encoded, 12))?;
    bytes.extend(pin_bytes);

    Ok(bytes)
}

pub fn reconstruct_pin(pin_str: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let pin_encoded = pin_str.chars()
        .flat_map(|c| ['0', c])
        .collect::<String>();
    Ok(hex::decode(pad_hex(&pin_encoded, 12))?)
}

pub fn reconstruct_rfid(rfid_str: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(hex::decode(pad_hex(&rfid_str, 20))?)
}
 
fn validate_address(addr: &Address) -> Result<(), Box<dyn Error>> {
    if addr.len() != 2 {
        Err(format!("Invalid address length: {} bytes", addr.len()).into())
    } else {
        Ok(())
    }
}

fn validate_card_data(data: &CardData) -> Result<(), Box<dyn Error>> {
    if data.len() != 16 {
        Err(format!("Invalid card data length: {} bytes", data.len()).into())
    } else {
        Ok(())
    }
}

fn pad_hex(s: &str, target_len: usize) -> String {
    let mut padded = s.to_uppercase();
    padded.truncate(target_len);
    while padded.len() < target_len {
        padded.push('F');
    }
    padded
}

pub fn get_card(addr: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    if addr.len() !=2 { return Err("address invalid".into()); }
    match u16::from_be_bytes([addr[0], addr[1]]) {
        0x0010..=0x0FFF => serial_read(addr),
        _ => Err("invalid address".into())
    } 
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
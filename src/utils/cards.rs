use std::error::Error;
use serde::{Serialize, Deserialize};
use chrono::Local;
use std::fs;
use rfd::FileDialog;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub rfid: String,
    pub pin: String,
}


pub fn export_bin(cards: Vec<u8>, uid: String) -> Result<(), Box<dyn Error>> {

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    
    let file_path = FileDialog::new()
        .set_title("Сохранить данные пользователей")
        .set_file_name(format!("cards_{}_{}.agrg", uid, timestamp))
        .save_file();

    if let Some(path) = file_path {
        fs::write(path, cards)?;
    }

    Ok(())
}


pub fn rfid_to_bytes(hex_str: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = [0xFF; 10];

    let hex_even = match hex_str.len()%2 {
        0 => hex_str,
        _ => format!("{}{}", hex_str, "f")
    };

    let bytes = hex::decode(hex_even)?;
    
    if bytes.len() > 10 {
        return Err(format!(
            "RFID too long: {} bytes (max 10)",
            bytes.len()
        ).into());
    }
    
    buffer[..bytes.len()].copy_from_slice(&bytes);
    Ok(buffer.to_vec())
}

pub fn pin_to_bytes(pin_str: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = [0xFF; 6];
    let digits: Result<Vec<u8>, _> = pin_str.chars()
        .map(|c| {
            c.to_digit(10)
                .and_then(|d| u8::try_from(d).ok())
                .ok_or_else(|| format!("Invalid PIN character: '{}'", c))
        })
        .collect();

    let digits = digits?;
    
    if digits.len() > 6 {
        return Err(format!(
            "PIN too long: {} digits (max 6)",
            digits.len()
        ).into());
    }
    
    buffer[..digits.len()].copy_from_slice(&digits);
    Ok(buffer.to_vec())
}


pub fn parse(card_bytes: Vec<u8>) -> Result<Card, Box<dyn Error>> {
    if card_bytes.len() != 16 {
        return Err(format!("incorrect entry length: must be 16, got {}", card_bytes.len()).into());
    }

    // trim trailing FF
    let trimmed_rfid = trim_empty(card_bytes[..10].to_vec());
    let trimmed_pin = trim_empty(card_bytes[10..16].to_vec());

    // convert RFID to hex
    let rfid = hex::encode(trimmed_rfid);

    // convert PIN bytes to numeric string
    let pin = trimmed_pin.iter()
        .map(|&b| {
            if b > 9 {
                return Err(format!("Invalid PIN byte: {} (must be 0-9)", b));
            }
            Ok((b'0' + b) as char)
        })
        .collect::<Result<String, _>>()?;

    Ok(Card { rfid, pin })
}


pub fn trim_empty(data: Vec<u8>) -> Vec<u8> {
    let mut end = data.len();
    // iterate backward to find the first non-0xFF byte
    while end > 0 && data[end - 1] == 0xFF {
        end -= 1;
    }
    data[..end].to_vec()
}

pub fn import_bin() -> Result<Vec<u8>, std::io::Error> {
    let file_path = FileDialog::new()
        .set_title("Импортировать данные пользователей").pick_file();

    if let Some(path) = file_path {
        fs::read(path)
    }
    else {
        Ok(Vec::new())
    }


}
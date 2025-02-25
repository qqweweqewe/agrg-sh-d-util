use std::error::Error;
use super::{serial_read, serial_write};

pub struct Card {
    pub rfid: String,
    pub pin: String,
}

pub fn get_card(addr: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    if addr.len() !=2 { return Err("address invalid".into()); }
    match u16::from_be_bytes([addr[0], addr[1]]) {
        0x0010..=0x0FFF => serial_read(addr),
        _ => Err("invalid address".into())
    } 
}

pub fn bulk_read() -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let mut rx_vec: Vec<Vec<u8>> = vec![];

    for addr_0 in 0x00..=0x0F {
        for addr_1 in 0x10..=0xFF {
            rx_vec.push(serial_read(vec![addr_0, addr_1])?);
        }
    }

    Ok(rx_vec)
}

// TODO: fix removing trailing FF, remove leading zero from pin
pub fn parse(card_bytes: Vec<u8>) -> Result<Card, Box<dyn Error>> {
    if card_bytes.len() != 16 { 
        return Err(format!("incorrect entry length: must be 16, got {}", card_bytes.len()).into())
    }

    let rfid = trim_empty(hex::encode_upper(&card_bytes[..10]));
    let pin = trim_empty(hex::encode_upper(&card_bytes[10..16]));

    Ok(Card {
        rfid,
        pin
    })

}

fn trim_empty(s: String) -> String {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // check if the entire string is 'FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF'
    let is_all_ff = len % 2 == 0 && bytes.iter().all(|&b| b == b'F');
    if is_all_ff {
        return String::new();
    }

    // count trailing 'FF'
    let mut count = 0;
    let mut i = len;
    while i >= 2 {
        if bytes[i - 2] == b'F' && bytes[i - 1] == b'F' {
            count += 1;
            i -= 2;
        } else {
            break;
        }
    }

    // determine how many 'FF' to keep
    let desired = if count >= 7 {
        7
    } else if count >= 4 {
        4
    } else {
        0
    };

    // calc new length and create a new string
    let chars_to_remove = (count - desired) * 2;
    let new_len = len.saturating_sub(chars_to_remove);
    String::from_utf8_lossy(&bytes[..new_len]).into_owned()
}

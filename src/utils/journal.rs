use iced::widget::shader::wgpu::hal::empty;
use serde::{Serialize, Deserialize};
use chrono::Local;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct JournalEntry {
    #[serde(rename = "Timestamp")]
    timestamp: String,
    #[serde(rename = "Event Type")]
    event_type: String,
    #[serde(rename = "User ID")]
    user_id: String,
    #[serde(rename = "Data")]
    data: String,
}

const EVENT_TYPES: [&str; 11] = [
    "Power On", // 0x00
    "Registered User", // 0x01
    "Unknown UID", // 0x02
    "Unknown PIN", // 0x03
    "Unknown", // 0x04
    "Unknown", // 0x05
    "Handle Released", // 0x06
    "Forced Release/Break In", // 0x07
    "Fixed Handle Pos", // 0x08
    "Programming Mode", // 0x09
    "Wrong Admin Password", // 0x0A
];

pub fn journal_entry_to_string(entry: JournalEntry) -> Option<(String, String)> {

    let empty_entry = JournalEntry { 
        timestamp: String::from("20FF-FF-FF FF:FF:FF"), 
        event_type: String::from("Unknown"), 
        user_id: String::from("255"), 
        data: String::from("ffffffffffffff") 
    };
    
    if entry == empty_entry {
        return None
    }


   // info construction
   let info = match entry.data.as_str() {
       "ffffffffffffffff" => {
            format!("user_id:{}", entry.user_id)
       },

       _ => {
            format!("entered:{}{}", entry.user_id, entry.data)
       } 
   };

   let info = format!("{} {}", entry.event_type, info);

   Some((entry.timestamp, info))
}

pub fn parse_journal_entry(raw: Vec<u8>) -> Result<JournalEntry, Box<dyn Error>> {
    if raw.len() != 16 {
        return Err("Invalid journal entry length".into());
    }

    // parse timestamp
    let time = format!(
        "{:02X}:{:02X}:{:02X}",
        raw[0], raw[1], raw[2]  // ss, mm, hh
    );

    // parse date 
    let date = format!(
        "20{:02X}-{:02X}-{:02X}", 
        raw[3], raw[5], raw[6]     
    );

    // parse event type
    let event_byte = raw[7];
    let event_type = EVENT_TYPES
        .get(event_byte as usize)
        .unwrap_or(&"Unknown")
        .to_string();

    // parse user ID
    let user_id = format!("{:03}", raw[8]); 

    // parse data
    let data = hex::encode(&raw[9..16]);


    let res = JournalEntry {
        timestamp: format!("{} {}", date, time),
        event_type,
        user_id,
        data,
    };
    Ok(res)
}

pub fn bulk_journal_read() -> Result<Vec<JournalEntry>, Box<dyn Error>> {
    let mut entries = Vec::with_capacity(1792);

    // journal spans addresses 0x1000-0x7FFF
    for addr_high in 0x10..=0x7F {
        for addr_low in 0x00..=0xFF {
            let addr = vec![addr_high, addr_low];
            let data = super::serial_read(addr)?;
            
            match parse_journal_entry(data) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse entry at {:02X}{:02X}: {}", 
                    addr_high, addr_low, e),
            }
        }
    }

    Ok(entries)
}

pub fn export_journal_csv(entries: Vec<JournalEntry>) -> Result<(), Box<dyn Error>> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("journal_{}.csv", timestamp);
    
    let mut writer = csv::Writer::from_path(filename)?;

    for entry in entries {
        writer.serialize(entry)?;
    }

    writer.flush()?;
    Ok(())
}

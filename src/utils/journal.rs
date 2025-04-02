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
    user_id: u8,
    #[serde(rename = "Data")]
    data: Vec<u8>,
}

const EVENT_TYPES: [&str; 11] = [
    "Power On", // 0x00
    "Registered User", // 0x01
    "Unknown UID", // 0x02
    "Unknown PIN", // 0x03
    "Unknown", // 0x04
    "Unknown", // 0x05
    "Handle Unlocked", // 0x06
    "Forced Unlock/Break In", // 0x07
    "Handle Locked", // 0x08
    "Programming Mode", // 0x09
    "Wrong Admin Password", // 0x0A
];

pub fn journal_entry_to_string(entry: JournalEntry) -> Option<(String, String)> {

    let empty_entry = JournalEntry { 
        timestamp: String::from("20FF-FF-FF FF:FF:FF"), 
        event_type: String::from("Unknown"), 
        user_id: 0xFF, 
        // data: String::from("ffffffffffffff")
        data: vec![0xFF; 7] 
    };
    
    if entry == empty_entry {
        return None
    }

    // bytes to string
    let bytestring: String = entry.data.iter().map(|b| format!("{:2X}", b)).collect();

   // info construction
    let info = match entry.event_type.as_str() {
        "Registered User" => { format!("{}", entry.user_id) },
        "Unknown UID" => { format!("{:X}{}", entry.user_id, &bytestring[..bytestring.len()-2]) },
        "Unknown PIN" => { format!("{:01x}{:01}", entry.user_id, &bytestring[..10]) },

        _ => String::new(),
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
        raw[2], raw[1], raw[0]  // ss, mm, hh
    );

    // parse date 
    let date = format!(
        "20{:02X}-{:02X}-{:02X}", 
        raw[6], raw[5], raw[3]     
    );

    // parse event type
    let event_byte = raw[7];
    let event_type = EVENT_TYPES
        .get(event_byte as usize)
        .unwrap_or(&"Unknown")
        .to_string();

    // parse user ID
    let user_id = raw[8]; 

    // parse data
    let data = raw[9..16].to_vec();


    let res = JournalEntry {
        timestamp: format!("{} {}", date, time),
        event_type,
        user_id,
        data,
    };
    Ok(res)
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

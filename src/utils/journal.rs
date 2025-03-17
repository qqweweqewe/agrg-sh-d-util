use serde::{Serialize, Deserialize};
use chrono::Local;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    "Power On",
    "Registered User",
    "Unknown UID",
    "Unknown PIN",
    "Unlock",
    "Lock",
    "Handle Released",
    "Forced Release/Break in",
    "Handle Fixed",
    "Programming Mode",
    "Wrong Admin Password",
];

pub fn journal_entry_to_string(entry: JournalEntry) -> Vec<String> {
   let mut out = vec![entry.timestamp];

   // info construction
   let info = match entry.data.as_str() {
       "ffffffffffffffff" => {
            format!("user_id:{}", entry.user_id)
       },

       _ => {
            format!("entered:{}{}", entry.user_id, entry.data)
       } 
   };

   out.push(format!("{} {}", entry.event_type, info));

   out
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

    Ok(JournalEntry {
        timestamp: format!("{} {}", date, time),
        event_type,
        user_id,
        data,
    })
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

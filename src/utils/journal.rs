
use chrono::Local;
use std::{error::Error, io::Write};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JournalEntry {
    timestamp: String,
    event_type: String,
    user_id: u8,
    data: Vec<u8>,
}

const EVENT_TYPES: [&str; 11] = [
    "Подача питания", // 0x00
    "Пользователь", // 0x01
    "Незарегистрированный UID", // 0x02
    "Незарегистрированный PIN", // 0x03
    "Unknown", // 0x04
    "Unknown", // 0x05
    "Рукоятка замка освобождена", // 0x06
    "Принудительное освобождение замка/Взлом", // 0x07
    "Рукоятка замка зафиксирована", // 0x08
    "Вход в режим программирования", // 0x09
    "Неверный PIN администратора", // 0x0A
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
        "Пользователь" => { format!("{}", entry.user_id) },
        "Незарегистрированный UID" => { format!("{:X}{}", entry.user_id, &bytestring[..bytestring.len()-2]) },
        "Незарегистрированный PIN" => { format!("{}{}", entry.user_id, &bytestring[..10]) },

        _ => String::new(),
    };


   let info = format!("{} {}", entry.event_type, info);

   Some((entry.timestamp, info))
}


pub fn serializer(entry_vec: Vec<Option<(String, String)>>) -> Result<(), Box<dyn Error>> {

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    
    let filename = format!("journal_{}.csv", timestamp);

    let file_path = rfd::FileDialog::new()
        .set_title("Сохранение журнала событий")
        .set_file_name(&filename)
        .save_file();

    print!("{:?}", file_path);

    match file_path {
        Some(path) => {
            let mut file = std::fs::File::create(path)
                .expect("Failed to create file");

            file.write(b"Timestamp, Data\n")?;

            for entry in entry_vec {
                match entry {
                    Some(tuple) => {
                        let data = format!("{},{}\n", tuple.0, tuple.1); 
                        file.write(data.as_bytes())?;
                    },
                    None => {}
                }
            };
        },
        None => {}
    };
    Ok(())
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

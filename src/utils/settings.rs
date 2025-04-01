use chrono::Local;

struct DeviceSettings {
    main_mode: u8,
    numpad_wiegand_mode: u8,
    cardreader_wiegand_mode: u8,
    auto_access_mode: u8,
    admin_password: Vec<u8>
}

pub fn parse(data: Vec<u8>) -> Result<DeviceSettings, Box<dyn std::error::Error>> {
    
    match data.len() {
        16 => {
            Ok(DeviceSettings {
                main_mode: data[0],
                numpad_wiegand_mode: data[1],
                cardreader_wiegand_mode: data[2],
                auto_access_mode: data[3],
                admin_password: data[10..16].to_vec() 
            })
        }
        _ => {Err("invalid settings data!".into())}
    }
        
        
}

pub fn export_bin(cards: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    // let mut writer = csv::Writer::from_path()?;

    // for (i, card_bytes) in cards.iter().enumerate() {
        // let card = parse(*card_bytes)?;
        // let address_num = u16::from_be_bytes([addr[0], addr[1]]);

    std::fs::write(format!("settings_{}.agrg", timestamp), cards)?;
    Ok(())
}

pub fn import_bin() -> Result<Vec<u8>, std::io::Error> {
    let file_path = rfd::FileDialog::new()
        .set_title("Import File")
        .save_file();
    
    std::fs::read(file_path.expect("Invalid filepath"))
    
}

pub fn serialize(settings: DeviceSettings) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut res = vec![
        settings.main_mode, 
        settings.numpad_wiegand_mode,
        settings.cardreader_wiegand_mode,
        settings.auto_access_mode
    ];

    let mut append = settings.admin_password;

    res.append(&mut append);

    Ok(vec![])
}
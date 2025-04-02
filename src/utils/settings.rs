use chrono::Local;


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
use chrono::Local;
use rfd::FileDialog;


pub fn export_bin(settings: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    
    let file_path = FileDialog::new()
        .set_title("Save File")
        .set_file_name(format!("cards_{}.agrg", timestamp))
        .save_file();

    if let Some(path) = file_path {
        std::fs::write(path, settings)?;
    }

    Ok(())

}

pub fn import_bin() -> Result<Vec<u8>, std::io::Error> {
    let file_path = rfd::FileDialog::new()
        .set_title("Import File")
        .pick_file();
    
    if let Some(path) = file_path {
        std::fs::read(path)
    }
    else {
        Ok(Vec::new())
    }

    
}
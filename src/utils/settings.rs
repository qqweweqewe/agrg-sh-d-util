use chrono::Local;
use rfd::FileDialog;


pub fn export_bin(settings: Vec<u8>, uid: String) -> Result<(), Box<dyn std::error::Error>> {
    
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    
    let file_path = FileDialog::new()
        .set_title("Сохранить настройки")
        .set_file_name(format!("settings_{}_{}.agrg", uid, timestamp))
        .save_file();

    if let Some(path) = file_path {
        std::fs::write(path, settings)?;
    }

    Ok(())

}

pub fn import_bin() -> Result<Vec<u8>, std::io::Error> {
    let file_path = rfd::FileDialog::new()
        .set_title("Импортировать настройки")
        .pick_file();
    
    if let Some(path) = file_path {
        std::fs::read(path)
    }
    else {
        Ok(vec![0x00; 16])
    }

    
}
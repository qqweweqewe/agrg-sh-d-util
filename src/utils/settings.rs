struct DeviceSettings {
    is_autonomous: u8,
    numpad_wiegand_mode: u8,
    cardreader_wiegand_mode: u8,
    auto_access_mode: u8,
    admin_password: Vec<u8>
}

pub fn parse(data: Vec<u8>) -> Result<DeviceSettings, Box<dyn std::error::Error>> {
    
    match data.len() {
        16 => {
            Ok(DeviceSettings {
                is_autonomous: data[0],
                numpad_wiegand_mode: data[1],
                cardreader_wiegand_mode: data[2],
                auto_access_mode: data[3],
                admin_password: data[10..16].to_vec() 
            })
        }
        _ => {Err("invalid settings data!".into())}
    }
        
        
}
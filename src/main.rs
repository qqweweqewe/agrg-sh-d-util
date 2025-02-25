mod utils;
use utils::cards::{parse, Card};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let v: Vec<u8> = vec![0x12, 0x31, 0xf3, 0x23, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 
                            0x01, 0x03, 0x05, 0x07, 0x09, 0x00];

    let card: Card = parse(v)?;

    println!("rfid: {:?}\npin: {:?}", card.rfid, card.pin);
    
    Ok(())
}

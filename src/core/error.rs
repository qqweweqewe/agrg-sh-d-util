#[derive(Debug)]
pub enum AgrgErr {
    Serial(String),
    Io(String),
    Parse(String),
    InvalidInput(String),
    Csv(String),
    Hex(String),
}

impl std::fmt::Display for AgrgErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgrgErr::Serial(thing) => write!(f, "Serial Error: {}", thing) ,
            AgrgErr::Io(thing) => write!(f, "IO Error: {}", thing),
            AgrgErr::Parse(thing) =>  write!(f, "Parsing Error: {}", thing),
            AgrgErr::InvalidInput(thing) =>  write!(f, "Input Error: {}", thing),
            AgrgErr::Csv(thing) =>  write!(f, "CSV Error whatever that means: {}", thing),
            AgrgErr::Hex(thing) =>  write!(f, "HEX error: {}", thing),
        }
    }
}

impl From<serialport::Error> for AgrgErr {
    fn from(value: serialport::Error) -> Self {
        Self::Serial(value.description)
    }
}

impl From<std::io::Error> for AgrgErr {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AgrgErr>;
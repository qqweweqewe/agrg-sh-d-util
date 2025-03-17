mod utils;

use iced::{
    alignment::{Horizontal, Vertical}, 
    widget::{text_input, button, column, container, pick_list, row, text, Text, Row, Column, Container, Space}, 
    Alignment, Length, Sandbox, Settings
};

fn main() -> iced::Result {
    Agrg::run(Settings::default())
}

#[derive(Clone, Copy)]
enum Tab {
    Settings,
    Cards,
    Journal
}

#[derive(Debug, Clone)]
enum AgrgMsg {
    SettingsTab,
    JournalTab,
    SerialChoice(String),
    CardsTab,
    ExportJournal,
    ExportCards,
    MemDump,
    MemUpload,
    ImportBin,
    ExportBin,
    CardEdited(usize, bool, String), // (index, UID(0) or PIN(1), new_value)
}

struct Agrg {
    tab: Tab,
    ports: Vec<String>,
    port: Option<String>,
    data: Vec<u8>,
}

impl Sandbox for Agrg {
    type Message = AgrgMsg;

    fn new() -> Self {
        Self {
            tab: Tab::Journal,
            ports: utils::get_available_ports()
                .expect("No Ports Found! Connect your device and restart the program"),
            port: None,
            data: Vec::new(),
        }
    }

    fn title(&self) -> String {
        "AGRG SH-D Utility".into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxDark
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            AgrgMsg::CardsTab => self.tab = Tab::Cards,
            AgrgMsg::CardEdited(chunk_index, is_part1, value) => {
                let base_address = 0x0010 + chunk_index * 16;
                
                // Validate hex input
                let clean_value: String = value.chars()
                    .filter(|c| c.is_ascii_hexdigit())
                    .collect();
    
                // Get target byte range
                let (start, end) = if is_part1 {
                    (base_address, base_address + 10)
                } else {
                    (base_address + 10, base_address + 16)
                };
    
                // Convert hex string to bytes
                if let Ok(parsed_bytes) = hex::decode(&clean_value) {
                    let required_length = if is_part1 { 10 } else { 6 };
                    
                    if parsed_bytes.len() == required_length {
                        // Update the data vector directly
                        for (i, byte) in parsed_bytes.iter().enumerate() {
                            if let Some(pos) = self.data.get_mut(start + i) {
                                *pos = *byte;
                            }
                        }
                    }
                }
            },
            AgrgMsg::JournalTab => self.tab = Tab::Journal,
            AgrgMsg::SettingsTab => self.tab = Tab::Settings,
            AgrgMsg::SerialChoice(s) => { 
                self.port = Some(s); 
                utils::set_port(self.port.clone().expect("no()")) 
            },
            AgrgMsg::ExportJournal => {

            },
            AgrgMsg::ExportCards => {

            },
            AgrgMsg::ExportBin => {

            },
            AgrgMsg::ImportBin => {

            },
            AgrgMsg::MemDump => {
                self.data = match utils::mem_dump() {
                    Ok(data) => data,
                    Err(_) => {
                        println!("ERR WRONG/INVALID PORT");
                        Vec::new()
                    }
                } 
            },
            AgrgMsg::MemUpload => {
                match self.data.as_slice() {
                    [] => println!("ERR WRONG/INVALID PORT"),
                    _ => {
                        match utils::mem_upload(self.data.clone()) {
                            Ok(_) => println!("Uploading.."),
                            Err(_) => {
                                println!("ERR WRONG/INVALID PORT")
                            }
                        }
                    }
                }
            }
        }
    } 

    fn view(&self) -> iced::Element<Self::Message> {
        column![
            pick_list(
                self.ports.clone(),
                self.port.clone(),
                AgrgMsg::SerialChoice
            ).placeholder("Select a port").width(200),

            row![
                button("Load data").on_press(AgrgMsg::MemDump),
                Space::new(10, 0),
                button("Upload data").on_press(AgrgMsg::MemUpload)
            ],

            Space::new(0, 20),
            
            row![
                button("Journal").on_press(AgrgMsg::JournalTab),
                button("Cards").on_press(AgrgMsg::CardsTab),
                button("Settings").on_press(AgrgMsg::SettingsTab)
            ].width(Length::Fill)
            .align_items(Alignment::Center).spacing(10),
            
            match self.tab {
                Tab::Journal => {
                    journal(self.data.clone())
                },
                
                Tab::Cards => {
                    cards(self.data.clone())
                },

                Tab::Settings => {
                    settings(self.data.clone())
                }
            }
        ].width(Length::Fill)
        .into()
    }   
}

// tab ui functions

fn journal(data: Vec<u8>) -> iced::Element<'static, AgrgMsg> {

    match data.as_slice() {
        [] => "No journal loaded".into(),
        _ => {    
            let journal_entries: Vec<Vec<String>> = data[0x1000..0x8000]
            .chunks(16)  // Split into 16-byte chunks
            .map(|chunk| {
                let parsed_entry = utils::journal::parse_journal_entry(chunk.to_vec()).expect("no journal");
                utils::journal::journal_entry_to_string(parsed_entry)
            })
            .collect();
            



            // two columns for data
            let mut left_col: Column<AgrgMsg> = Column::new()
                .spacing(10)
                .align_items(Alignment::Start);
            let mut right_col: Column<AgrgMsg> = Column::new()
                .spacing(10)
                .align_items(Alignment::Start);

            // headers
            left_col = left_col.push(Text::new("Date").width(Length::Fill));
            right_col = right_col.push(Text::new("Info").width(Length::Fill));

            // populate the columns
            for row in journal_entries {
                if row.len() >= 2 {
                    left_col = left_col.push(Text::new(row[0].clone()).width(Length::Fill));
                    right_col = right_col.push(Text::new(row[1].clone()).width(Length::Fill));
                }
            }

            // combine columns into a row
            let data_row: Row<AgrgMsg> = Row::new()
                .spacing(20)
                .push(left_col)
                .push(right_col)
                .into();

            container(
                column![
                    // exporn btn
                    button("Export CSV").on_press(AgrgMsg::ExportJournal),

                    //row of headers
                    row![
                        "Date",
                        "Info"
                    ],

                    // row of coluimns with content
                    data_row
                ]
            ).padding(10)
            .into()
        }
    }
}   

fn cards(data: Vec<u8>) -> iced::Element<'static, AgrgMsg> {
    match data.as_slice() {
        [] => "No Card data available".into(),
        _ => {

            let chunks: Vec<(String, String)> = data[0x0010..0x0fff]
                .chunks(16)
                .map(|chunk| {
                    let card = utils::cards::parse(chunk.to_vec()).expect("wow, something is REALLY off..");
                    (card.pin, card.rfid)
                })
                .collect();
        
            let mut address_col = Column::new().spacing(10);
            let mut uid_col = Column::new().spacing(10);
            let mut pin_col = Column::new().spacing(10);
        
            // headers
            address_col = address_col.push(Text::new("Address"));
            uid_col = uid_col.push(Text::new("Card"));
            pin_col = pin_col.push(Text::new("PIN"));
        
            for (index, chunk) in chunks.iter().enumerate() {
                let address = 0x0010 + index * 16;
                let address_text = format!("{:04X}", address);
        
                address_col = address_col.push(Text::new(address_text));
                
                uid_col = uid_col.push(
                    text_input(chunk.0.as_str(), chunk.0.as_str())
                        .on_input(move |v| AgrgMsg::CardEdited(index, true, v))
                        .width(200)
                );
                
                pin_col = pin_col.push(
                    text_input(chunk.1.as_str(), chunk.1.as_str())
                        .on_input(move |v| AgrgMsg::CardEdited(index, false, v))
                        .width(120)
                );
            }
        
            container(
                column![
                    button("Save Changes").on_press(AgrgMsg::MemUpload),
                    row!["Address", "Card Data", "PIN Data"],
                    Row::new()
                        .spacing(20)
                        .push(address_col)
                        .push(uid_col)
                        .push(pin_col)
                ]
            ).padding(10).into()
        }
    }
}


fn settings(data: Vec<u8>) -> iced::Element<'static, AgrgMsg>{
    container(    
        "WIP"
    ).padding(10)
    .into()
}


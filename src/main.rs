mod utils;

use iced::{
    alignment::{Horizontal, Vertical}, 
    widget::{button, column, container, pick_list, row, text, Text, Row, Column, Container, Space}, 
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
    MemDump,
    MemUpload
}

struct Agrg {
    tab: Tab,
    ports: Vec<String>,
    port: Option<String>,
    data: Vec<u8>
}

impl Sandbox for Agrg {
    type Message = AgrgMsg;

    fn new() -> Self {
        Self {
            tab: Tab::Journal,
            ports: utils::get_available_ports()
                .expect("No Ports Found! Connect your device and restart the program"),
            port: None,
            data: Vec::new()
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
            AgrgMsg::JournalTab => self.tab = Tab::Journal,
            AgrgMsg::SettingsTab => self.tab = Tab::Settings,
            AgrgMsg::SerialChoice(s) => { 
                self.port = Some(s); 
                utils::set_port(self.port.clone().expect("no()")) 
            },
            AgrgMsg::ExportJournal => {

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

            button("Load").on_press(AgrgMsg::MemDump),
            
            Space::new(0, 20),
            
            row![
                button("Journal").on_press(AgrgMsg::JournalTab),
                button("Cards").on_press(AgrgMsg::CardsTab),
                button("Settings").on_press(AgrgMsg::SettingsTab)
            ].width(Length::Fill)
            .align_items(Alignment::Center),
            
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

fn cards(data: Vec<u8>) -> iced::Element<'static, AgrgMsg>{
    match data.as_slice() {
        [] => "No data loaded".into(),
        _ => {    
            let card_entries: Vec<Vec<String>> = data[0x1000..0x8000]
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

fn settings(data: Vec<u8>) -> iced::Element<'static, AgrgMsg>{
    container(    
        "It works!!!"
    ).padding(10)
    .into()
}


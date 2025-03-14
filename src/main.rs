mod utils;

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{text, button, column, row, pick_list, container, Container},
    Sandbox, Settings, Alignment, Length
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
    ExportJournal
}

struct Agrg {
    tab: Tab,
    ports: Vec<String>,
    port: Option<String>,
}

impl Sandbox for Agrg {
    type Message = AgrgMsg;

    fn new() -> Self {
        Self {
            tab: Tab::Journal,
            ports: utils::get_available_ports()
                .expect("No Ports Found! Connect your device and restart the program"),
            port: None,
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
                button("Journal").on_press(AgrgMsg::JournalTab),
                button("Cards").on_press(AgrgMsg::CardsTab),
                button("Settings").on_press(AgrgMsg::SettingsTab)
            ].width(Length::Fill)
            .align_items(Alignment::Center),
            
            match self.tab {
                Tab::Journal => {
                    journal()
                },
                
                Tab::Cards => {
                    cards()
                },

                Tab::Settings => {
                    settings()
                }
            }
        ].width(Length::Fill)
        .into()
    }   
}

// tab ui functions

fn journal() -> iced::Element<'static, AgrgMsg> {

    let journal: Vec<Vec<String>> = utils::journal::bulk_journal_read()
        .expect("something not gud happened :(")
        .into_iter()
        .map(utils::journal::journal_entry_to_string)
        .collect();
    
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
            row![
                
                column![
                    "test1",
                    "test2",
                    "test3"
                ],
                column![
                    "test1",
                    "test2",
                    "test3"
                ]
            ]
        ]
    ).padding(10)
    .into()
}   

fn cards() -> iced::Element<'static, AgrgMsg>{
    container(    
        "Wow surprisingly enough"
    ).padding(10)
    .into()
}

fn settings() -> iced::Element<'static, AgrgMsg>{
    container(    
        "It works!!!"
    ).padding(10)
    .into()
}


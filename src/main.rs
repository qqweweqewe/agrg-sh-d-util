mod utils;

use iced::{
    widget::{text, button, column, row, pick_list},
    Sandbox, Settings, Alignment,
};
use utils::get_available_ports;

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
    DoNothing,
    SettingsTab,
    JournalTab,
    SerialChoice(String),
    CardsTab,
}

struct Agrg {
    tab: Tab,
    port: Option<String>,
}

impl Sandbox for Agrg {
    type Message = AgrgMsg;

    fn new() -> Self {
        Self {
            tab: Tab::Journal,
            port: Some(String::from("COM0")),
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
            AgrgMsg::SerialChoice(s) => self.port = Some(s), 
            AgrgMsg::DoNothing => {},
        }
    } 

    fn view(&self) -> iced::Element<Self::Message> {
        column![
            pick_list(
                get_available_ports().expect("No Available ports")
                .into_iter()
                .map(|port| Some(port.port_name))
                .collect(),
                None,
                |choice| AgrgMsg::SerialChoice(choice)
            ).placeholder("Ports Selection"),

            row![
                button("Journal").on_press(AgrgMsg::JournalTab),
                button("Cards").on_press(AgrgMsg::CardsTab),
                button("Settings").on_press(AgrgMsg::SettingsTab)
            ].align_items(Alignment::Center),
            
            match self.tab {
                Tab::Journal => {
                    text("Journal here")
                },
                
                Tab::Cards => {
                    text("Cards here")
                },

                Tab::Settings => {
                    text("Settings menu")
                }
            }
        ]
        .into()
    }
}

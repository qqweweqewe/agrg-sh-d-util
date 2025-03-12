mod utils;

use iced::{
    widget::{text, button, column, row},
    Sandbox, Settings,
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
    DoNothing,
    SettingsTab,
    JournalTab,
    CardsTab,
}

struct Agrg {
    tab: Tab,
}

impl Sandbox for Agrg {
    type Message = AgrgMsg;

    fn new() -> Self {
        Self {
            tab: Tab::Journal
        }
    }

    fn title(&self) -> String {
        "AGRG SH-D Utility".into()
    }

    fn update(&mut self, message: Self::Message) {
        self.tab = match message {
            AgrgMsg::CardsTab => Tab::Cards,
            AgrgMsg::JournalTab => Tab::Journal,
            AgrgMsg::SettingsTab => Tab::Settings,
            AgrgMsg::DoNothing => self.tab,
        }
    } 

    fn view(&self) -> iced::Element<Self::Message> {
        column![
            row![
                button("Journal").on_press(AgrgMsg::JournalTab),
                button("Cards").on_press(AgrgMsg::CardsTab),
                button("Settings").on_press(AgrgMsg::SettingsTab)
            ],
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

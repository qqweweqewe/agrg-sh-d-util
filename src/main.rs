mod utils;
use iced::Theme;
use utils::cards;
use utils::journal;
use utils::get_available_ports;

use iced::{
    Alignment, Color, Command, Element, Length, Application, Settings, executor,
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Column}
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
enum Message {
    PortSelected(String),
    ConnectPressed,
    TabSelected(Tab),
    Cards(CardsMessage),
}

#[derive(Debug, Clone)]
enum CardsMessage {
    LoadPressed,
    CardsLoaded(Result<Vec<CardEntry>, String>),  
    EntrySelected(usize),
    RfidChanged(usize, String),
    PinChanged(usize, String),
    ImportPressed,
    ExportPressed,
    SavePressed,
    RevertPressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Journal,
    Cards,
    Settings,
}

#[derive(Debug, Clone)]
struct CardEntry {
    original: cards::Card,
    current: cards::Card,
    address: String,
    modified: bool,
}

struct RFIDApp {
    selected_port: Option<String>,
    ports: Vec<String>,
    connected: bool,
    active_tab: Tab,
    
    // Cards tab state
    cards_loading: bool,
    card_entries: Arc<Mutex<Vec<CardEntry>>>,
    selected_entry: Option<usize>,
}

impl Application for RFIDApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                selected_port: None,
                ports: utils::get_available_ports()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| p.port_name)
                    .collect(),
                connected: false,
                active_tab: Tab::Cards,
                cards_loading: false,
                card_entries: Arc::new(Mutex::new(Vec::new())),
                selected_entry: None,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("AGRG SH-D Utility")
    }


    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::PortSelected(port) => {
                self.selected_port = Some(port);
            }
            Message::ConnectPressed => {
                self.connected = !self.connected;
            }
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
            Message::Cards(cards_msg) => {
                let mut entries = self.card_entries.lock().unwrap();
                match cards_msg {
                    CardsMessage::LoadPressed => {
                        self.cards_loading = true;
                        return Command::perform(load_cards(), |result| {
                            Message::Cards(CardsMessage::CardsLoaded(result))
                        });
                    }
                    CardsMessage::EntrySelected(index) => {
                        self.selected_entry = Some(index);
                    }
                    CardsMessage::RfidChanged(index, value) => {
                        if let Some(entry) = entries.get_mut(index) {
                            entry.current.rfid = value;
                            entry.modified = entry.current != entry.original;
                        }
                    }
                    CardsMessage::PinChanged(index, value) => {
                        if let Some(entry) = entries.get_mut(index) {
                            entry.current.pin = value;
                            entry.modified = entry.current != entry.original;
                        }
                    }
                    CardsMessage::ImportPressed => { /* CSV import logic */ }
                    CardsMessage::ExportPressed => { /* CSV export logic */ }
                    CardsMessage::SavePressed => { /* Save to device logic */ }
                    CardsMessage::RevertPressed => {
                        if let Some(index) = self.selected_entry {
                            if let Some(entry) = entries.get_mut(index) {
                                entry.current = entry.original.clone();
                                entry.modified = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let connection_bar = row![
            pick_list(
                &self.ports[..],
                self.selected_port.clone(),
                Message::PortSelected
            )
            .width(200),
            button(if self.connected { "Disconnect" } else { "Connect" })
                .on_press(Message::ConnectPressed)
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let tab_bar = row![
            tab_button("Journal", Tab::Journal, self.active_tab),
            tab_button("Cards", Tab::Cards, self.active_tab),
            tab_button("Settings", Tab::Settings, self.active_tab),
        ]
        .spacing(20);

        let content = match self.active_tab {
            Tab::Cards => self.cards_tab(),
            _ => column![text("Coming soon...")].into(),
        };

        container(column![connection_bar, tab_bar, content].spacing(20))
            .padding(20)
            .into()
    }
}

impl RFIDApp {
    fn cards_tab(&self) -> Element<Message> {
        let controls = row![
            button("Load from Device")
                .on_press(Message::Cards(CardsMessage::LoadPressed)),
            button("Import CSV").on_press(Message::Cards(CardsMessage::ImportPressed)),
            button("Export CSV").on_press(Message::Cards(CardsMessage::ExportPressed)),
            button("Save to Device").on_press(Message::Cards(CardsMessage::SavePressed)),
            button("Revert Changes").on_press(Message::Cards(CardsMessage::RevertPressed)),
        ]
        .spacing(10);

        let entries = self.card_entries.lock().unwrap();
        let list = Column::with_children(
            entries.iter().enumerate().map(|(i, entry)| {
                let bg_color = if Some(i) == self.selected_entry {
                    Color::from_rgb8(220, 240, 255)
                } else if entry.modified {
                    Color::from_rgb8(255, 240, 220)
                } else {
                    Color::WHITE
                };

                row![
                    text(&entry.address).width(80),
                    text_input("RFID", &entry.current.rfid)
                        .on_input(move |v| Message::Cards(CardsMessage::RfidChanged(i, v))),
                    text_input("PIN", &entry.current.pin)
                        .on_input(move |v| Message::Cards(CardsMessage::PinChanged(i, v))),
                ]
                .spacing(10)
                .padding(5)
                .into()
            })
        )
        .spacing(2);

        column![
            controls,
            scrollable(
                container(list)
                    .width(Length::Fill)
                    .height(Length::FillPortion(1))
            )
        ]
        .into()
    }
}

fn tab_button<'a>(label: &str, tab: Tab, current_tab: Tab) -> Element<'a, Message> {
    let button = button(text(label))
        .style(if tab == current_tab {
            iced::theme::Button::Primary
        } else {
            iced::theme::Button::Secondary
        })
        .on_press(Message::TabSelected(tab));

    container(button).into()
}

async fn load_cards() -> Result<Vec<CardEntry>, String> {
    let raw_cards = cards::bulk_read().map_err(|e| e.to_string())?;
    
    Ok(raw_cards.into_iter().map(|(addr, data)| {
        let card = cards::parse(data).unwrap();
        CardEntry {
            original: card.clone(),
            current: card,
            address: format!("{}", u16::from_be_bytes([addr[0], addr[1]])),
            modified: false,
        }
    }).collect())
}

fn main() -> iced::Result {
    RFIDApp::run(Settings::default())
}
//#![cfg_attr(windows, windows_subsystem = "windows")]

mod utils;
mod styles;

use std::time::Duration;

use iced::{
    alignment::Horizontal, widget::{button, column, container, pick_list, row, scrollable, text_input, Column, Container, Row, Space, Text, Toggler}, Alignment, Application, Length, Settings
};
use chrono::Local;

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
    CustomDataChange(String),
    SaveCustomData,
    PingKeepAlive,
    ToggleKeepAlive,
    SettingsUpdate(usize, String),
    AdminPasswdEdited(String),
    SettingsTab,
    JournalTab,
    CardsTab,
    SerialChoice(String),
    RefreshPorts,
    MemDump,
    ExportJournal,
    ExportCards,
    ImportCards,
    ExportSettings,
    ImportSettings,
    MemUpload,
    TimeSync,
    CardEdited(usize, bool, String), // index / UID(0) or PIN(1) / new_value
}

struct Agrg {
    keepalive: bool,
    tab: Tab,
    ports: Vec<String>,
    port: Option<String>,
    data: Vec<u8>,
    time: String,
    settings_map: Vec<Vec<String>>,

    connected: bool,

    agrg: Option<String>,
    custom_desc: Option<String>
}

impl Application for Agrg {
    type Message = AgrgMsg;
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let mut v = vec![0x00; 16];
        v.resize(0x1000, 0xff);

        let port = utils::scan_ports();
        (
            Self {
                keepalive: false,
                agrg: match port {
                    None => None,
                    Some(_) => utils::agrg_text_info()
                },
                custom_desc: match port {
                    None => None,
                    Some(_) => utils::get_text()
                },

                connected: port.is_some(),

                tab: Tab::Journal,
                ports: match utils::get_available_ports() {
                    None => vec![String::from("Нет доступных портов")],
                    Some(ports) => ports
                },
                port: port,
                data: v,
                //time: String::new()
                time: Local::now().format("%H:%M:%S %d.%m.%Y").to_string(),
                settings_map: vec![
                    // mode
                    vec![
                        "Считыватель".into(),
                        "Автономный контроллер".into()
                    ],
                    
                    // pinpad mode
                    vec![
                        "Wiegand6".into(),
                        "Wiegand26(hex)".into(),
                        "Wiegand26(dec)".into(),
                        "OFF".into()
                    ],

                    // card reader mode
                    vec![
                        "Wiegand26".into(),
                        "Wiegand34".into(),
                        "OFF".into()
                    ],

                    // auto access mode
                    vec![
                        "PIN или UID".into(),
                        "PIN".into(),
                        "UID".into(),
                        "PIN и UID".into()
                    ]
                ]
            },
            iced::Command::none()
        )
    }

    fn title(&self) -> String {
        "Программа настройки AGRG SH-D, v.1.0, 2025".into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::SolarizedDark
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AgrgMsg::SaveCustomData => {
                let mut replacements: Vec<u8> = self.custom_desc.clone().unwrap().into_bytes();
                
                // if more that 64 bytes, take the first 64
                if replacements.len() > 64 {
                    replacements = replacements[..64].to_vec();
                }
                else {
                    // pad replacements to 64 bytes
                    replacements.resize(64, 0xff);
                }

                // send using utils::set_text()
                utils::set_text(replacements)

            },
            AgrgMsg::CustomDataChange(str) => {
                self.custom_desc = Some(str);
            },
            AgrgMsg::PingKeepAlive => {
                println!("попытка пинга..");
                if self.keepalive {
                    println!("успешно");
                    _ = utils::get_datetime();
                }
            }
            AgrgMsg::ToggleKeepAlive => {
                println!("переключение режима сохранения соединения: {}", !&self.keepalive);
                let current = self.keepalive;
                self.keepalive = !current;
            },
            AgrgMsg::AdminPasswdEdited(str) => {
                let replacements: Vec<u8> = str.chars()
                    .map(|c| c.to_digit(10).expect("Некорректный символ") as u8)
                    .collect();
            
                for (i, &new_byte) in replacements.iter().enumerate() {
                    if i < 6 { // Ensure we only write 6 digits
                        self.data[0x0A + i] = new_byte;
                    }
                }
            }
            
            AgrgMsg::CardsTab => self.tab = Tab::Cards,
            AgrgMsg::CardEdited(chunk_index, is_uid, value) => {
                let base_address = 0x0010 + chunk_index * 16;
    
                // get target byte range
                let start = if is_uid {
                    base_address
                } else {
                    base_address + 10
                };
    
                // convert hex string to bytes
                let required_length = if is_uid { 10 } else { 6 };
                let parsed_bytes = if is_uid {
                    utils::cards::rfid_to_bytes(value).expect("некорректный формат")
                } else {
                    utils::cards::pin_to_bytes(value).expect("некорректный формат")
                };

                if parsed_bytes.len() == required_length {
                    // update the data vector directly
                    for (i, byte) in parsed_bytes.iter().enumerate() {
                        if let Some(pos) = self.data.get_mut(start + i) {
                            *pos = *byte;
                        }
                    }
                }
            },
            AgrgMsg::ImportCards => {
                let new_data = match utils::cards::import_bin() {
                    Ok(res) => res,
                    Err(_) => { 
                        println!("Не удалось импортировать данные пользователей");
                        Vec::new()
                    }
                };
                
                if new_data.len() != 255*16 {
                    panic!("Некорректный/Поврежденный файл");
                }

                if self.data.len() < 256*16 {
                    self.data.resize(256*16, 0xff);
                }

                self.data[0x0010..0x1000].copy_from_slice(&new_data);
                
            },
            AgrgMsg::JournalTab => self.tab = Tab::Journal,
            AgrgMsg::SettingsTab => self.tab = Tab::Settings,
            AgrgMsg::ExportSettings => {
                _ = utils::settings::export_bin(self.data[0x0000..=0x000f].to_vec(), self.custom_desc.clone().unwrap());
            },
            AgrgMsg::ImportSettings => {
                let new_data = match utils::settings::import_bin() {
                    Ok(res) => res,
                    Err(_) => { 
                        println!("Не удалось импортировать настройки");
                        vec![0; 16]
                    }
                };

                if new_data.len() != 16 {
                    panic!("Некорректный/Поврежденный файл настроек");
                }

                if self.data.len() < 16 {
                    self.data.resize(16, 0xff);
                }

                self.data[0x0000..0x0010].copy_from_slice(&new_data);
            },
            AgrgMsg::SettingsUpdate(addr, val) => {
                self.data[addr] = self.search(&val);
            },
            AgrgMsg::SerialChoice(s) => { 
                self.port = Some(s); 
                utils::set_port(self.port.clone().expect("Порты не найдены")); 
                self.agrg = utils::agrg_text_info();
                self.custom_desc = utils::get_text();
                self.connected = utils::check_handle(self.port.clone().unwrap())
            },
            AgrgMsg::RefreshPorts => {
                self.ports = match utils::get_available_ports() {
                    None => vec![String::from("Порты не найдены")],
                    Some(ports) => ports
                }
            },
            AgrgMsg::ExportJournal => {
                let journal_entries: Vec<Option<(String, String)>> = self.data[0x1000..self.data.len()]
                    .chunks(16) 
                    .map(|chunk| utils::journal::journal_entry_to_string(
                        utils::journal::parse_journal_entry(chunk.to_vec()).expect("ошибка при обработке строки журнала")
                        )
                    )
                    .collect();

                _ = utils::journal::serializer(journal_entries);
            },
            AgrgMsg::ExportCards => {
                _ = utils::cards::export_bin(self.data[0x0010..=0x0fff].to_vec(), self.custom_desc.clone().unwrap());
            },
            AgrgMsg::MemDump => {
                // self.time = match utils::get_datetime() {
                //     Ok(res) => res,
                //     Err(_) => "Error".to_string()
                // };
                let current = self.keepalive.clone();
                if current {
                    self.keepalive = false;
                }

                self.data = vec![];

                self.data = match utils::mem_dump() {
                    Ok(data) => data,
                    Err(_) => {
                        println!("Порт неверный - нет данных");
                        Vec::new()
                    }
                };
                // self.data = utils::mock::get_data()

                if current {
                    self.keepalive = true;
                }
            },
            AgrgMsg::MemUpload => {
                let current = self.keepalive.clone();
                if current {
                    self.keepalive = false;
                }

                match self.data.as_slice() {
                    [] => println!("Порт неверный - нет данных"),
                    _ => {
                        match utils::mem_upload(self.data[0x0000..0x1000].to_vec()) {
                            Ok(_) => println!("Обновление данных ручки.."),
                            Err(_) => {
                                println!("Порт неверный - нет данных")
                            }
                        }
                    }
                }
                
                if current {
                    self.keepalive = true;
                }
            }, 
            AgrgMsg::TimeSync => {
                self.time = Local::now().format("%H:%M:%S %d.%m.%Y").to_string(); 
                _ = utils::set_datetime(self.time.clone())
            }
        }
        iced::Command::none()
    } 

    fn view(&self) -> iced::Element<Self::Message> {
        println!("{}", self.connected);
        column![
            // connection header
            row![
                row![
                    Text::new("o").style(
                        if self.connected { 
                            iced::Color::from_rgb(0.0, 1.0, 0.0) 
                        } else { 
                            iced::Color::from_rgb(1.0, 0.0, 0.0)
                        }
                    ),

                    pick_list(
                        self.ports.clone(),
                        self.port.clone(),
                        AgrgMsg::SerialChoice
                    ).placeholder("Выбор COM").width(200),
                    button("Обновить").on_press(AgrgMsg::RefreshPorts)
                ].spacing(20).padding(10).width(Length::Fill),
                column![
                    // agrg info
                    Container::new(
                        Text::new(
                            match &self.agrg {
                                Some(thing) => thing.clone().replace("\n", " "),
                                None => String::new()
                            }
                        )
                    ).width(400).padding(10).style(iced::theme::Container::Box),
                    // custom description
                    Container::new(
                        Text::new(
                            match &self.custom_desc {
                                Some(thing) => thing.clone().replace("\n", " "),
                                None => String::new()
                            }
                        )
                    ).width(400).padding(10).style(iced::theme::Container::Box),
                    
                ].spacing(20).width(Length::Fill)
            ].spacing(20),
            
            Toggler::new(Some("KeepAlive".into()), self.keepalive, |_| { AgrgMsg::ToggleKeepAlive }),
            
            Space::new(0, 20),

            row![
                button("Выгрузка v").on_press_maybe(if self.connected { Some(AgrgMsg::MemDump) } else { None } ),

                button("Загрузка ^").on_press_maybe(if self.connected { Some(AgrgMsg::MemUpload) } else { None } )
            ].spacing(20),

            Space::new(0, 20),
            
            container(
                row![
                    button("Журнал").on_press(AgrgMsg::JournalTab),
                    button("Пользователи").on_press(AgrgMsg::CardsTab),
                    button("Параметры").on_press(AgrgMsg::SettingsTab)
                ].spacing(20),
            ).width(Length::Fill).align_x(Horizontal::Center),

            Space::new(0, 20),

            match self.tab {
                Tab::Journal => {
                    journal(self.data.clone())
                },
                
                Tab::Cards => {
                    cards(self.data.clone())
                },

                Tab::Settings => {
                    settings(self.data.clone(), &self.settings_map, self.time.clone(), self.custom_desc.clone())
                }
            },
        ].width(Length::Fill).padding(20)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(Duration::from_secs(20)).map(|_| AgrgMsg::PingKeepAlive)
    }   
}

impl Agrg {
    fn search(&self, val: &String) -> u8 {
        for setting in self.settings_map.clone() {
            for (id, entry) in setting.iter().enumerate() {
                if val == entry { return id as u8 };
            }
        };
        
        0
    }    
}

// tab ui functions
fn journal(data: Vec<u8>) -> iced::Element<'static, AgrgMsg> {
    match data.len() {
        0..0x1000 => Text::new("Нет данных").height(Length::Fill).into(),
        _ => {    
            let journal_entries: Vec<(String, String)> = data[0x1000..data.len()]
                .chunks(16)
                .map(|chunk| {
                    utils::journal::journal_entry_to_string(utils::journal::parse_journal_entry(chunk.to_vec()).expect("asdasd"))
                })
                .filter(|chunk| {
                    chunk.is_some()
                })
                .map(|chunk| {
                    chunk.unwrap()
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
            left_col = left_col.push(Text::new("Дата").width(Length::Fill));
            right_col = right_col.push(Text::new("Событие").width(Length::Fill));

            // populate the columns
            for row in journal_entries {
                left_col = left_col.push(Text::new(row.0.clone()).width(Length::Fill));
                right_col = right_col.push(Text::new(row.1.clone()).width(Length::Fill));
            }

            // combine columns into a row
            let data_row: Row<AgrgMsg> = Row::new()
                .spacing(30)
                .push(left_col)
                .push(right_col)
                .into();

            container(
                column![
                    // exporn btn
                    button("Экспорт CSV").on_press(AgrgMsg::ExportJournal),

                    // row of coluimns with content
                    scrollable(
                        data_row
                    ).height(Length::Fill)
                ]
            ).padding(10)
            .into()
        }
    }
}   

fn cards(data: Vec<u8>) -> iced::Element<'static, AgrgMsg> {
    match data.as_slice() {
        [] => column![ 
                Text::new("Нет данных").height(Length::Fill),
                button("Импортировать").on_press(AgrgMsg::ImportCards)
            ].into(),
        _ => {
            let chunks: Vec<(String, String)> = data[0x0010..=0x0fff]
                .chunks(16)
                .map(|chunk| {
                    let card = utils::cards::parse(chunk.to_vec()).expect("Invalid card data");
                    (card.rfid, card.pin)
                })
                .collect();

            // header row
            let header = row![
                Text::new("№").width(20),
                Text::new("UID").width(200),
                Text::new("PIN").width(120),
            ].spacing(20);

            // card rows
            let mut card_rows = Column::new()
                .spacing(10)
                .push(header);

            for (index, chunk) in chunks.iter().enumerate() {
                let address = 0x0010 + index * 16;
                let address_text = format!("{}", address/16);

                let card_row = row![
                    Text::new(address_text).width(20),
                    text_input(&chunk.0, &chunk.0)
                        .on_input(move |v| {
                            let cleaned = sanitize_hex_input(&v, 20);
                            AgrgMsg::CardEdited(index, true, cleaned)
                        })
                        .width(200),
                    text_input(&chunk.1, &chunk.1)
                        .on_input(move |v| {
                            let cleaned = sanitize_pin(&v, 6);
                            AgrgMsg::CardEdited(index, false, cleaned)
                        })
                        .width(120),
                ].spacing(20);

                card_rows = card_rows.push(card_row);
            }

            container(
                column![
                    row![
                        button("Экспорт").on_press(AgrgMsg::ExportCards),
                        button("Импорт").on_press(AgrgMsg::ImportCards)
                    ].spacing(20),
                    scrollable(card_rows).height(Length::Fill).width(Length::Fill)
                ].spacing(10)
            ).padding(10).into()
        }
    }
}

fn sanitize_hex_input(input: &str, max_length: usize) -> String {
    let cleaned: String = input.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    cleaned.chars().take(max_length).collect()
}

fn sanitize_pin(input: &str, max_length: usize) -> String {
    let cleaned: String = input.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    cleaned.chars().take(max_length).collect()
}


fn settings(data: Vec<u8>, option_map: &Vec<Vec<String>>, time: String, custom_data: Option<String>) -> iced::Element<'static, AgrgMsg> {
    match data.as_slice() {
        [] => column![
            Text::new("No Data loaded").height(Length::Fill),
            button("Import").on_press(AgrgMsg::ImportSettings)
        ].into(),
        _ => {    
            // Convert admin password bytes to string, ensuring we only process valid digits
            let admin_passwd: String = data[0x000A..0x0010]
                .iter()
                .map(|&b| {
                    if b <= 9 { // Only map valid single digits
                        (b'0' + b) as char
                    } else {
                        '0' // Default to '0' for invalid values
                    }
                })
                .collect();

            let placeholder = custom_data.unwrap_or("".to_string());
            let mut row = Column::new();
            let headers = ["Режим работы", "Формат кодонаборной панели", "Формат считывателя", "Режим доступа"];

            // pick list for each byte
            for (index, &byte) in data[0..4].iter().enumerate() {
                let selected = option_map[index][byte as usize].clone();
                let on_select = move |new_selection: String| AgrgMsg::SettingsUpdate(index, new_selection);
                let pick_list = pick_list(
                    option_map[index].clone(),
                    Some(selected),
                    on_select
                );
                row = row.push(column![headers[index], pick_list]);
            }

            column![
                // import export
                row![
                    button("Импорт").on_press(AgrgMsg::ImportSettings),
                    button("Экспорт").on_press(AgrgMsg::ExportSettings)
                ].spacing(20),
                Space::new(0, 20),
                
                // MAIN BODY
                column![

                    row![
                        row.spacing(20),
                        Space::new(20, 0),
                        column![
                            Text::new("PIN Администратора"),
                            text_input(&admin_passwd, &admin_passwd)
                                .on_input(move |v| {
                                    let cleaned = sanitize_admin_passwd(&v, 6);
                                    AgrgMsg::AdminPasswdEdited(cleaned)
                                })
                                .width(120)
                                .padding(5),
                            container(
                                row![
                                    Text::new(time),
                                    button("Sync").on_press(AgrgMsg::TimeSync)
                                ].spacing(20)
                            ),
                            Space::new(0, 20),
                        ],
                    ].spacing(20),
                    Space::new(0, 30),
                    // custom data input field with a save button
                    row![
                        text_input(&placeholder, &placeholder)
                            .on_input(move |v| {
                                AgrgMsg::CustomDataChange(v)
                            })
                            .width(500),
                        button("Сохранить").on_press(AgrgMsg::SaveCustomData)
                    ],
                ]
                
            ].width(Length::Fill).align_items(Alignment::Center).into()
        }
    }
}



// fn settings(data: Vec<u8>, option_map: &Vec<Vec<String>>) -> iced::Element<'static, AgrgMsg> {
//     match data.as_slice() {
//         [] => Text::new("No Data loaded").height(Length::Fill).into(),
//         _ => {    // row!["WIP"].into()

//             let admin_passwd: String = data[0x000A..0x0010].iter()
//                 .map(|&b| {
//                     (b'0' + b) as char
//                 })
//                 .collect();

//             let mut row = Row::new();
//             let headers = ["Working mode", "Pinpad mode", "Card reader mode", "Access mode"];

//             // pick list for each byte
//             for (index, &byte) in data[0..4].iter().enumerate() {
//                 // currently selected option
//                 let selected = option_map[index][byte as usize].clone();

//                 // closure with captured index
//                 let on_select = move |new_selection: String| AgrgMsg::SettingsUpdate(index, new_selection);

//                 // pick list widget
//                 let pick_list = pick_list(
//                     option_map[index].clone(),
//                     Some(selected),
//                     on_select
//                 );

//                 row = row.push(column![headers[index], pick_list]);
//             }


//             column![
//                 row![
//                     button("Import").on_press(AgrgMsg::ImportSettings),
//                     button("Export").on_press(AgrgMsg::ExportSettings)
//                 ].spacing(20),
//                 row,
//                 text_input(&admin_passwd, &admin_passwd)
//                         .on_input(move |v| {
//                             let cleaned = sanitize_admin_passwd(&v, 6);
//                             AgrgMsg::AdminPasswdEdited(cleaned)
//                         })
//                         .width(120),
//             ].spacing(20).height(Length::Fill).into()
//         }
//     }
// }


fn sanitize_admin_passwd(input: &str, max_length: usize) -> String {
    let cleaned: String = input.chars()
        .filter(|c| c.is_ascii_digit())
        .take(max_length)
        .collect();

    if cleaned.len() < max_length {
        format!("{:0>6}", cleaned)
    } else {
        cleaned
    }
}


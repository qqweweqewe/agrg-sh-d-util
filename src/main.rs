use iced::{
    Alignment, Sandbox, Settings, Theme, Element
};

//main entrypoint
fn main() -> iced::Result {
    AgrgUI::run(Settings::default())
}

//define a new struict for U
struct AgrgUI {
    //main variables
    theme: Theme,
    tab: Tab,
}

// tabs enum
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tab {
    Cards, 
    Journal,
    Settings
}

// possible messages
#[derive(Debug, Clone)]
enum Message {
    SwitchTo(Tab),
}

// implement sandbox trait for app (simplified Application trait)

impl Sandbox for AgrgUI {
    type Message = Message;

    // constructor
    fn new() -> Self {
        Self {
            theme: Theme::GruvboxDark,
            tab: Tab::Cards,

        }
    }

    //title
    fn title(&self) -> String {
        String::from("AGRG SH-D utility")
    }

    // theme
    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    // update
    fn update(&mut self, message: Message) {
        match message {
            Message::SwitchTo(next_tab) => {}
        }

    } 

    // view
    fn view(&self) -> Element<Message> {
        "Test test".into()
    }
    

}

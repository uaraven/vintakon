use iced::widget::{button, column, text};
use iced::{Element, Settings, Task};

mod exif;

#[derive(Default)]
struct VintakonApp {
    path: Option<String>,
    counter: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}

impl VintakonApp {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn title(&self) -> String {
        String::from("Vintakon")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.counter += 1,
            Message::Decrement => self.counter -= 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        column![
            text(format!("Counter: {}", self.counter)).size(30),
            button("Increment").on_press(Message::Increment),
            button("Decrement").on_press(Message::Decrement),
        ]
        .padding(20)
        .spacing(10)
        .into()
    }
}

fn main() -> iced::Result {
    iced::application("Vintakon", VintakonApp::update, VintakonApp::view)
        .settings(Settings::default())
        .run_with(VintakonApp::new)
}

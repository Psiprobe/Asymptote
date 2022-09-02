use iced_wgpu::Renderer;
use iced_winit::widget::text_input::{TextInput};
use iced_winit::widget::{Row};
use iced_winit::{Alignment, Command, Element, Length, Program};

pub struct Controls {
    text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            text: Default::default(),
        }
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TextChanged(text) => {
                self.text = text;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message, Renderer> {
        let text = &self.text;

        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Start)
            .padding(20)
            .spacing(20)
            .push(
                TextInput::new(
                ">",
                text,
                Message::TextChanged,
            ))
        .into()
    }
}

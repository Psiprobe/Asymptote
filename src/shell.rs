use std::time::Duration;
use iced_wgpu::Renderer;
use iced_winit::widget::text_input::{TextInput};
use iced_winit::widget::{Row,Text,Column};
use iced_winit::{Alignment, Command, Element, Length, Program, Color,time::Instant};

pub struct Controls {
    pub text: String,
    pub fps: i32,
    pub last_render_time: Instant,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
    FrameRate(i32),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            text: Default::default(),
            fps: 0,
            last_render_time: Instant::now(),
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
            Message::FrameRate(fps) =>{
                self.fps = fps;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message, Renderer> {
        let text = &self.text;

        let cli:TextInput<Message,Renderer> = TextInput::new(
            ">",
            text,
            Message::TextChanged,
        );
        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Start)
            .padding(20)
            .spacing(5)
            .push(cli)
            .push(
                Text::new("Current FPS: ".to_owned() + &self.fps.to_string())
                    .style(Color::new(0.0,1.0,0.0,1.0)),
            )
            .into()
    }
}

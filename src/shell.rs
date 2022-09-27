use iced_wgpu::Renderer;
use iced_winit::widget::text_input::{TextInput};
use iced_winit::widget::{Text,Column};
use iced_winit::{Alignment, Command, Element, Length, Program, Color,time::Instant};

pub struct TextColumn {
    pub text: String,
    pub timer: f32,
    pub alpha: f32,
}

impl TextColumn {
    pub fn new(text: String) -> TextColumn{
        Self { text: text, timer: (5.0) , alpha: (1.0)}
    }
}
pub struct Controls {
    pub text: String,
    pub fps: i32,
    pub last_render_time: Instant,
    pub text_column:Vec<TextColumn>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
    OnSubmit,
    FrameUpdate(i32),
    Update,
    ServerLog(String),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            text: Default::default(),
            fps: 0,
            last_render_time: Instant::now(),
            text_column: Default::default(),
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
            
            Message::OnSubmit => {

                self.text_column.push(TextColumn::new(("Niggra: ".to_owned() + &self.text).to_string()));
                self.text = String::from("");
                
            }
            
            Message::FrameUpdate(fps) =>{
                self.fps = fps;
                for x in self.text_column.iter_mut() {
                    x.timer -= 1.0;
                }
            }

            Message::Update =>{
                
                if self.text_column.len() > 0 && self.text_column[0].timer < 0.0 {
                    if self.text_column[0].alpha - 0.01 > 0.0{
                        self.text_column[0].alpha -= 0.01
                    }else{
                        let _ = self.text_column.remove(0);
                    }
                }

            }
            Message::ServerLog(text) => {
                self.text_column.push(TextColumn::new(text));
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
            
        ).on_submit(Message::OnSubmit);

        let text_columns = Column::with_children(
            self.text_column
                .iter()
                .map(|event| Text::new(format!("{}", event.text))
                .style(Color::new(0.0,1.0,0.0,event.alpha))
                .size(20))
                .map(Element::from)
                .collect(),
        );

        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Start)
            .padding(20)
            .spacing(5)
            .push(cli)
            .push(
                Text::new("FPS: ".to_owned() + &self.fps.to_string())
                    .style(Color::new(0.0,1.0,0.0,1.0)).size(20),
            )
            .push(text_columns)
            .into()
    }
}

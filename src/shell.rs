use iced_wgpu::Renderer;
use iced_winit::widget::text_input::{TextInput};
use iced_winit::widget::{Text,Column,Row, slider};
use iced_winit::{Alignment, Command, Element, Length, Program, Color,time::Instant};


pub struct TextColumn {
    pub text: String,
    pub timer: f32,
    pub alpha: f32,
}

impl TextColumn {
    pub fn new(text: String) -> TextColumn{
        Self { text: text, timer: (3.0) , alpha: (0.7)}
    }
}
pub struct Controls {
    pub text: String,

    pub indicator_text: String,
    pub indicator_overdose: bool,

    pub test: [f32;3],
    pub fps: i32,
    pub last_render_time: Instant,
    pub text_column: Vec<TextColumn>,
    pub parse_flag: bool,

    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum Message {
    TextChanged(String),
    UsrIndicator(String,bool),
    ServerLog(String),
    FrameUpdate(i32),
    OnSubmit,
    Update,
    CommandParsed,
    ChatMessage,
    Coordinate([f32;3]),
    BackgroundColorChanged(Color),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            text: Default::default(),

            indicator_text: Default::default(),
            indicator_overdose: Default::default(),

            test: Default::default(),
            fps: 0,
            last_render_time: Instant::now(),
            text_column: Default::default(),
            parse_flag: false,

            color: Color::WHITE,
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
                self.parse_flag = true;
            }
            
            
            Message::FrameUpdate(fps) => {
                self.fps = fps;
                //for x in self.text_column.iter_mut() {
                //    x.timer -= 1.0;
                //}
            }

            Message::Update =>{

                self.text_column.iter_mut().for_each(|t|{
                    t.timer -= 0.01;
                });

                self.text_column.iter_mut().filter(|t| t.timer <= 0.0).for_each(|t|{
                    if t.timer > - 0.1 {
                        t.alpha = 1.0;
                    }
                    t.alpha -= 0.1;
                });
                
                self.text_column.retain(|t|{
                    t.alpha > 0.0
                });

            }
            Message::ServerLog(text) => {
                self.text_column.push(TextColumn::new(text));
            }
            Message::CommandParsed => {
                self.text = String::from("");
                self.parse_flag = false;
            }
            Message::ChatMessage =>{
                self.text_column.push(TextColumn::new(("Admin: ".to_owned() + &self.text).to_string()));
            }
            Message::Coordinate(coordinate) =>{
                self.test[0] = coordinate[0];
                self.test[1] = coordinate[1];
                self.test[2] = coordinate[2];
            }
            Message::UsrIndicator(text,overdose) =>{
                self.indicator_text = text;
                self.indicator_overdose = overdose;
            }

            Message::BackgroundColorChanged(color) =>{
                self.color = color;
            }

        }

        Command::none()
    }
    

    fn view(&self) -> Element<Message, Renderer> {

        let background_color = self.color;
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
                .style(Color::new(1.0,1.0,1.0,event.alpha))
                .size(20))
                .map(Element::from)
                .collect(),
        );

        let sliders:Row<'_, _, Renderer> = Row::new()
        .width(Length::Units(500))
        .spacing(20)
        .push(
            slider(0.0..=1.0, background_color.r, move |r| {
                Message::BackgroundColorChanged(Color {
                    r,
                    ..background_color
                })
            })
            .step(0.01),
        )
        .push(
            slider(0.0..=1.0, background_color.g, move |g| {
                Message::BackgroundColorChanged(Color {
                    g,
                    ..background_color
                })
            })
            .step(0.01)
            ,
        )
        .push(
            slider(0.0..=1.0, background_color.b, move |b| {
                Message::BackgroundColorChanged(Color {
                    b,
                    ..background_color
                })
            })
            
            .step(0.01),
        );


        
        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Start)
            .padding(20)
            .spacing(5)
            .push(cli)
            .push(
                Text::new(&self.indicator_text)
                    .style(
                        if self.indicator_overdose{
                            Color::from_rgb(1.0,1.0,1.0)
                        }
                        else{
                            Color::from_rgb(0.0,1.0,0.0)
                        }
                        
                    ).size(20),
            )
            .push(sliders)
            .push(
                Text::new("FPS: ".to_owned() + &self.fps.to_string())
                    .style(Color::from_rgb(1.0,1.0,1.0)).size(20),
            )
            .push(
                Text::new("TPS: ".to_owned() + &'0'.to_string())
                    .style(Color::from_rgb(1.0,1.0,1.0)).size(20),
            )

            .push(
                Text::new(self.color.r.to_string())
                    .style(Color::from_rgb(self.color.r,self.color.g,self.color.b)).size(20),
            )

            .push(
                Text::new(self.color.g.to_string())
                    .style(Color::from_rgb(self.color.r,self.color.g,self.color.b)).size(20),
            )
            .push(
                Text::new(self.color.b.to_string())
                    .style(Color::from_rgb(self.color.r,self.color.g,self.color.b)).size(20),
            )

            .push(text_columns)  
            .into()
    }
}
  
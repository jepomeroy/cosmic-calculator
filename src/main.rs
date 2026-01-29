use cosmic::app::{Settings, Task};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, row, text};
use cosmic::{executor, ApplicationExt, Element};

const APP_ID: &str = "com.github.jepomeroy.CosmicCalculator";

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<Calculator>(Settings::default(), ())?;
    Ok(())
}

#[derive(Default)]
struct Calculator {
    core: cosmic::Core,
    display: String,
    current_value: f64,
    operator: Option<Operator>,
    new_number: bool,
}

#[derive(Debug, Clone, Copy)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
enum Message {
    Number(u8),
    Operator(Operator),
    Decimal,
    Equals,
    Clear,
    Backspace,
}

impl cosmic::Application for Calculator {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut app = Calculator {
            core,
            display: String::from("0"),
            current_value: 0.0,
            operator: None,
            new_number: true,
        };

        let command = app.update_title();

        (app, command)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Number(n) => {
                if self.new_number {
                    self.display = n.to_string();
                    self.new_number = false;
                } else if self.display.len() < 15 {
                    if self.display == "0" {
                        self.display = n.to_string();
                    } else {
                        self.display.push_str(&n.to_string());
                    }
                }
            }
            Message::Decimal => {
                if self.new_number {
                    self.display = String::from("0.");
                    self.new_number = false;
                } else if !self.display.contains('.') && self.display.len() < 14 {
                    self.display.push('.');
                }
            }
            Message::Operator(op) => {
                let display_value = self.display.parse::<f64>().unwrap_or(0.0);

                if let Some(operator) = self.operator {
                    self.current_value = self.calculate(self.current_value, display_value, operator);
                    self.display = self.format_result(self.current_value);
                } else {
                    self.current_value = display_value;
                }

                self.operator = Some(op);
                self.new_number = true;
            }
            Message::Equals => {
                if let Some(operator) = self.operator {
                    let display_value = self.display.parse::<f64>().unwrap_or(0.0);
                    self.current_value = self.calculate(self.current_value, display_value, operator);
                    self.display = self.format_result(self.current_value);
                    self.operator = None;
                    self.new_number = true;
                }
            }
            Message::Clear => {
                self.display = String::from("0");
                self.current_value = 0.0;
                self.operator = None;
                self.new_number = true;
            }
            Message::Backspace => {
                if !self.new_number && self.display.len() > 1 {
                    self.display.pop();
                } else if !self.new_number {
                    self.display = String::from("0");
                    self.new_number = true;
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let display = container(
            text(&self.display)
                .size(48)
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fixed(100.0))
        .align_x(Horizontal::Right)
        .align_y(Vertical::Center);

        let button_row = |buttons: Vec<(String, Message)>| {
            let mut r = row().spacing(8);
            for (label, msg) in buttons {
                r = r.push(
                    button::text(label)
                        .on_press(msg)
                        .width(Length::Fill)
                        .height(Length::Fixed(60.0))
                );
            }
            r
        };

        let buttons = column()
            .spacing(8)
            .push(button_row(vec![
                (String::from("C"), Message::Clear),
                (String::from("⌫"), Message::Backspace),
                (String::from("÷"), Message::Operator(Operator::Divide)),
                (String::from("×"), Message::Operator(Operator::Multiply)),
            ]))
            .push(button_row(vec![
                (String::from("7"), Message::Number(7)),
                (String::from("8"), Message::Number(8)),
                (String::from("9"), Message::Number(9)),
                (String::from("−"), Message::Operator(Operator::Subtract)),
            ]))
            .push(button_row(vec![
                (String::from("4"), Message::Number(4)),
                (String::from("5"), Message::Number(5)),
                (String::from("6"), Message::Number(6)),
                (String::from("+"), Message::Operator(Operator::Add)),
            ]))
            .push(button_row(vec![
                (String::from("1"), Message::Number(1)),
                (String::from("2"), Message::Number(2)),
                (String::from("3"), Message::Number(3)),
                (String::from("="), Message::Equals),
            ]))
            .push(button_row(vec![
                (String::from("0"), Message::Number(0)),
                (String::from("."), Message::Decimal),
            ]));

        let content = column()
            .spacing(16)
            .padding(16)
            .push(display)
            .push(buttons)
            .align_x(Alignment::Center)
            .width(Length::Fixed(400.0));

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

impl Calculator {
    fn calculate(&self, left: f64, right: f64, operator: Operator) -> f64 {
        match operator {
            Operator::Add => left + right,
            Operator::Subtract => left - right,
            Operator::Multiply => left * right,
            Operator::Divide => {
                if right != 0.0 {
                    left / right
                } else {
                    0.0
                }
            }
        }
    }

    fn format_result(&self, value: f64) -> String {
        if value.is_infinite() || value.is_nan() {
            String::from("Error")
        } else if value.fract() == 0.0 && value.abs() < 1e10 {
            format!("{}", value as i64)
        } else {
            let formatted = format!("{:.10}", value);
            formatted.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }

    fn update_title(&mut self) -> Task<Message> {
        self.set_header_title(String::from("Calculator"));
        self.set_window_title(String::from("Calculator"))
    }
}

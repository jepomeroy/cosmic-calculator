// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::fl;
use calclib::evaluator::evaluate;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Color, Length, Padding, clipboard, keyboard};
use cosmic::prelude::*;
use cosmic::widget::{
    self, Id, about::About, autosize::autosize, button, icon, menu, nav_bar, svg, text, text_input,
};
use std::collections::HashMap;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");
const INPUT_ID: &str = "calculator-input";
const HISTORY_ID: &str = "history-scrollable";

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Handle to the config context for persisting changes.
    config_handler: Option<cosmic_config::Config>,
    /// Calculator history (expression, result) pairs
    history: Vec<(String, String)>,
    /// Calculator input
    input: String,
    /// Calculator result
    result: String,
    /// Cursor position set by function buttons (e.g. inside `abs()`); None means append to end
    cursor_pos: Option<usize>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    KeyPressed(String),
    ModeSelected(String),
    CopyResultToInput(String),
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    ArrowLeft,
    ArrowRight,
    Home,
    End,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.jepomeroy.cosmic-calculator";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        nav.insert().data::<Page>(Page::Basic).activate();

        nav.insert().data::<Page>(Page::Advanced);

        nav.insert().data::<Page>(Page::Developer);

        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        // Load configuration from disk.
        let (config, config_handler) =
            match cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                Ok(context) => {
                    let config = match Config::get_entry(&context) {
                        Ok(config) => config,
                        Err((_errors, config)) => config,
                    };
                    (config, Some(context))
                }
                Err(_) => (Config::default(), None),
            };

        // Activate the saved page from config.
        if let Some(page) = Page::from_str(&config.page) {
            let target = nav.iter().find(|&id| {
                nav.data::<Page>(id)
                    .map(|data| std::mem::discriminant(data) == std::mem::discriminant(&page))
                    .unwrap_or(false)
            });
            if let Some(id) = target {
                nav.activate(id);
            }
        }

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            config,
            config_handler,
            history: Vec::new(),
            input: "".to_string(),
            result: "0".to_string(),
            cursor_pos: None,
        };

        // Create a startup command that sets the window title and size.
        let command = app.on_nav_select(app.nav.active());

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        cosmic::iced::event::listen_with(|event, _status, _window_id| {
            if let cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        Some(Message::ArrowLeft)
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        Some(Message::ArrowRight)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Home) => Some(Message::Home),
                    keyboard::Key::Named(keyboard::key::Named::End) => Some(Message::End),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let space_s = cosmic::theme::spacing().space_s;

        // Build history list from entries
        let history_items: Vec<Element<'_, Self::Message>> = self
            .history
            .iter()
            .map(|(expr, result)| {
                widget::row::with_capacity(2)
                    .push(
                        text(format!("{} = {}", expr, result))
                            .size(14)
                            .width(Length::Fill)
                            .align_x(Horizontal::Right),
                    )
                    .push(widget::tooltip(
                        button::icon(icon::from_name("edit-copy-symbolic").size(14))
                            .extra_small()
                            .on_press(Message::CopyResultToInput(result.clone())),
                        text("Copy to input"),
                        widget::tooltip::Position::Left,
                    ))
                    .align_y(Alignment::Center)
                    .spacing(8)
                    .into()
            })
            .collect();

        let history_column = widget::column::with_children(history_items)
            .spacing(4)
            .width(Length::Fill);

        let history = widget::container(
            widget::scrollable(history_column)
                .id(Id::new(HISTORY_ID))
                .height(Length::Fill),
        )
        .height(Length::Fixed(120.0))
        .width(Length::Fill)
        .padding(Padding::new(8.0))
        .class(cosmic::theme::Container::Card);

        let input = widget::row::with_capacity(1)
            .push(
                text_input("", &self.input)
                    .id(Id::new(INPUT_ID))
                    .on_input(Message::InputChanged)
                    .on_submit(|_| Message::KeyPressed("=".to_string()))
                    .always_active()
                    .size(24)
                    .padding(Padding::new(20.0)),
            )
            .align_y(Alignment::End)
            .spacing(space_s);

        let advanced_keyboard: Element<_> = widget::column::with_capacity(1)
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("Log", None))
                    .push(make_button("Ln", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("1/x", None))
                    .push(make_button("Log₂x", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("√", None))
                    .push(make_button("∛", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("x²", None))
                    .push(make_button("x³", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("xʸ", None))
                    .push(make_button("Abs", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("π", None))
                    .push(make_button("e", None))
                    .spacing(space_s),
            )
            .spacing(space_s)
            .into();

        let hexidecimal_keyboard: Element<_> = widget::column::with_capacity(1)
            .push(
                widget::row::with_capacity(1)
                    .push(make_button("A", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(1)
                    .push(make_button("B", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(1)
                    .push(make_button("C", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(1)
                    .push(make_button("D", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(1)
                    .push(make_button("E", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(1)
                    .push(make_button("F", None))
                    .spacing(space_s),
            )
            .spacing(space_s)
            .into();

        let developer_keyboard: Element<_> = widget::column::with_capacity(1)
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("AND", None))
                    .push(make_button("OR", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("NAND", None))
                    .push(make_button("NOR", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("XNOR", None))
                    .push(make_button("XOR", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("<<", None))
                    .push(make_button(">>", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(2)
                    .push(make_button("MOD", None))
                    .push(make_button("NOT", None))
                    .spacing(space_s),
            )
            .spacing(space_s)
            .into();

        let basic_keyboard: Element<_> = widget::column::with_capacity(1)
            .push(
                widget::row::with_capacity(4)
                    .push(make_button("AC", None))
                    .push(make_button("C", None))
                    .push(make_button("⌫", None))
                    .push(make_button("Ans", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(4)
                    .push(make_button("(", None))
                    .push(make_button(")", None))
                    .push(make_button("±", None))
                    .push(make_button("!", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(4)
                    .push(make_button("7", None))
                    .push(make_button("8", None))
                    .push(make_button("9", None))
                    .push(make_button("×", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(4)
                    .push(make_button("4", None))
                    .push(make_button("5", None))
                    .push(make_button("6", None))
                    .push(make_button("÷", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(4)
                    .push(make_button("1", None))
                    .push(make_button("2", None))
                    .push(make_button("3", None))
                    .push(make_button("+", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(4)
                    .push(make_button(".", None))
                    .push(make_button("0", None))
                    .push(make_button("=", None))
                    .push(make_button("-", None))
                    .spacing(space_s),
            )
            .spacing(space_s)
            .into();

        let calculator_mode: Element<_> = widget::column::with_capacity(1)
            .push(
                widget::row::with_capacity(3)
                    .push(icon_button_view(
                        "basic".to_string(),
                        include_bytes!("../resources/basic.svg"),
                    ))
                    .push(icon_button_view(
                        "advanced".to_string(),
                        include_bytes!("../resources/advanced.svg"),
                    ))
                    .push(icon_button_view(
                        "developer".to_string(),
                        include_bytes!("../resources/developer.svg"),
                    ))
                    .spacing(space_s),
            )
            .spacing(space_s)
            .into();

        let result = widget::row::with_capacity(1)
            .push(
                text(self.result.as_str())
                    .size(24)
                    .width(Length::Fill)
                    .align_x(Horizontal::Right),
            )
            .align_y(Alignment::End)
            .spacing(space_s);

        let content: Element<_> = match self.nav.active_data::<Page>().unwrap() {
            Page::Basic => widget::column::with_capacity(6)
                .push(history)
                .push(input)
                .push(result)
                .push(
                    widget::container(basic_keyboard)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center),
                )
                .push(widget::vertical_space().height(25))
                .push(calculator_mode)
                .spacing(space_s)
                .into(),

            Page::Advanced => widget::column::with_capacity(6)
                .push(history)
                .push(input)
                .push(result)
                .push(
                    widget::container(
                        widget::row::with_capacity(2)
                            .push(basic_keyboard)
                            .push(advanced_keyboard)
                            .spacing(space_s),
                    )
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
                )
                .push(widget::vertical_space().height(25))
                .push(calculator_mode)
                .spacing(space_s)
                .into(),

            Page::Developer => widget::column::with_capacity(6)
                .push(history)
                .push(input)
                .push(result)
                .push(
                    widget::container(
                        widget::row::with_capacity(3)
                            .push(hexidecimal_keyboard)
                            .push(basic_keyboard)
                            .push(developer_keyboard)
                            .spacing(space_s),
                    )
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
                )
                .push(widget::vertical_space().height(25))
                .push(calculator_mode)
                .spacing(space_s)
                .into(),
        };

        autosize(
            widget::container(content).padding(20).width(Length::Fill),
            Id::new("calculator-autosize"),
        )
        .min_width(660.0)
        .max_width(660.0)
        .min_height(800.0)
        .max_height(800.0)
        .into()
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::InputChanged(value) => {
                self.cursor_pos = None;
                // println!("Input changed: {value}");
                if value.contains('=') || value.contains('\n') {
                    return self.evaluate_input();
                }

                let substituted = substitute(value);
                // Reject a closing paren that would leave more closing than opening,
                // matching the same rule applied by the ")" button.
                if get_paren_count(&substituted) < 0 {
                    // Revert: leave self.input unchanged; the widget will re-sync on
                    // the next frame.
                } else {
                    self.input = substituted;
                }
            }
            Message::CopyResultToInput(result) => {
                self.input.push_str(&result);
                return Task::batch([
                    clipboard::write(result),
                    text_input::move_cursor_to_end(Id::new(INPUT_ID)),
                ]);
            }
            Message::KeyPressed(value) => {
                match value.as_str() {
                    "AC" => {
                        self.history.clear();
                        self.input.clear();
                        self.result = "0".to_string();
                        self.cursor_pos = None;
                        return text_input::move_cursor_to(Id::new(INPUT_ID), 0);
                    }
                    "C" => {
                        self.input.clear();
                        self.result = "0".to_string();
                        self.cursor_pos = None;
                        return text_input::move_cursor_to(Id::new(INPUT_ID), 0);
                    }
                    "⌫" => {
                        // Button backspace always removes the last character
                        let mut chars = self.input.chars();
                        chars.next_back();
                        self.input = chars.as_str().to_string();
                        return text_input::move_cursor_to_end(Id::new(INPUT_ID));
                    }
                    "±" => {
                        if self.input.starts_with('−') || self.input.starts_with('-') {
                            let mut chars = self.input.chars();
                            chars.next();
                            self.input = chars.as_str().to_string();
                        } else {
                            self.input.insert(0, '−');
                        }
                    }
                    "=" => {
                        let scroll_task = self.evaluate_input();
                        return Task::batch([
                            scroll_task,
                            text_input::move_cursor_to_end(Id::new(INPUT_ID)),
                        ]);
                    }
                    "Ans" => {
                        if let Some((_, last_result)) = self.history.last().cloned() {
                            let new_pos =
                                insert_at_cursor(&mut self.input, &last_result, self.cursor_pos);
                            self.cursor_pos = Some(new_pos);
                            return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                        }
                    }
                    ")" => {
                        if get_paren_count(&self.input) > 0 {
                            self.input.push(')');
                        }
                    }
                    "Log" => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "log()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    "Ln" => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "ln()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    "Log₂x" => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "log₂()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    "1/x" => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "1/()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    "√" => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "√()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    "∛" => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "∛()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    "x²" => self.input.push('²'),
                    "x³" => self.input.push('³'),
                    "xʸ" => self.input.push('^'),
                    "Abs" => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "abs()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    _ => {
                        let text = substitute(value);
                        let new_pos = insert_at_cursor(&mut self.input, &text, self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                }
                return text_input::move_cursor_to_end(Id::new(INPUT_ID));
            }
            Message::ModeSelected(mode) => {
                if let Some(page) = Page::from_str(&mode) {
                    let target = self.nav.iter().find(|&id| {
                        self.nav
                            .data::<Page>(id)
                            .map(|data| {
                                std::mem::discriminant(data) == std::mem::discriminant(&page)
                            })
                            .unwrap_or(false)
                    });

                    if let Some(id) = target {
                        return self.on_nav_select(id);
                    }
                }
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::ArrowLeft => {
                let len = self.input.chars().count();
                self.cursor_pos = Some(match self.cursor_pos {
                    None => len.saturating_sub(1),
                    Some(pos) => pos.saturating_sub(1),
                });
            }
            Message::ArrowRight => {
                let len = self.input.chars().count();
                self.cursor_pos = match self.cursor_pos {
                    None => None,
                    Some(pos) if pos + 1 >= len => None,
                    Some(pos) => Some(pos + 1),
                };
            }
            Message::Home => {
                self.cursor_pos = Some(0);
            }
            Message::End => {
                self.cursor_pos = None;
            }
        }
        Task::none()
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        // Persist the selected page to config.
        if let Some(page) = self.nav.active_data::<Page>() {
            self.config.page = page.as_str().to_string();
            if let Some(ref handler) = self.config_handler {
                let _ = self.config.write_entry(handler);
            }
        }

        Task::batch(vec![self.update_title()])
    }
}

fn get_paren_count(input: &String) -> i32 {
    let mut opening = 0;
    let mut closing = 0;

    for c in input.chars() {
        match c {
            '(' => opening += 1,
            ')' => closing += 1,
            _ => (),
        };
    }

    println!("Parens count: open: {opening}, close: {closing}");

    opening - closing
}

/// Inserts `text` into `input` at `cursor_pos` (char index), or appends if None.
/// Returns the new cursor position (after the inserted text).
fn insert_at_cursor(input: &mut String, text: &str, cursor_pos: Option<usize>) -> usize {
    match cursor_pos {
        Some(pos) => {
            let byte_pos = input
                .char_indices()
                .nth(pos)
                .map(|(i, _)| i)
                .unwrap_or(input.len());
            input.insert_str(byte_pos, text);
            pos + text.chars().count()
        }
        None => {
            input.push_str(text);
            input.chars().count()
        }
    }
}

/// Substitute certain characters with their calc lib equivalents
fn substitute(input: String) -> String {
    input.replace('*', "×").replace('/', "÷").replace('-', "−")
}

fn make_button(label: &str, handler: Option<Message>) -> Element<'_, Message> {
    let text_handler = handler.unwrap_or(Message::KeyPressed(label.to_string()));

    button::custom(
        text(label)
            .size(18)
            .font(cosmic::font::bold())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
    )
    .width(70)
    .height(40)
    .on_press(text_handler)
    .into()
}

// Function to create the button with an SVG icon
fn icon_button_view(id: String, svg_data: &'static [u8]) -> Element<'static, Message> {
    // 1. Load the SVG data from memory at compile time
    let handle = svg::Handle::from_memory(svg_data);

    // 2. Create the Svg widget
    let svg_widget = svg(handle)
        .width(32)
        .class(cosmic::theme::Svg::custom(|_theme| svg::Style {
            color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
        }));

    // 3. Combine the Svg with optional content in a Row
    let content = widget::row::with_capacity(2)
        .push(svg_widget)
        .align_y(Alignment::Center)
        .spacing(8);

    // 4. Wrap the content in a Button and add behavior
    button::custom(content)
        .on_press(Message::ModeSelected(id))
        .padding(10)
        .into()
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let window_title = fl!("app-title");
        self.set_header_title(window_title.clone());
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    /// Evaluate the current input and update the result and history
    pub fn evaluate_input(&mut self) -> Task<cosmic::Action<Message>> {
        let expression = self
            .input
            .replace('×', "*")
            .replace('÷', "/")
            .replace('−', "-")
            .replace(
                'π',
                format!("({})", &std::f64::consts::PI.to_string()).as_str(),
            )
            .replace(
                'e',
                format!("({})", std::f64::consts::E.to_string()).as_str(),
            )
            .replace('²', "^2")
            .replace('³', "^3")
            .replace('√', "sqrt")
            .replace('∛', "cbrt")
            .replace("log₂", "logtwo");

        match evaluate(expression) {
            Ok(result) => {
                self.result = result.value();
                self.history.push((self.input.clone(), self.result.clone()));
                self.input.clear();
                self.cursor_pos = None;
                cosmic::iced::widget::scrollable::snap_to(
                    Id::new(HISTORY_ID),
                    cosmic::iced::widget::scrollable::RelativeOffset::END,
                )
            }
            Err(err) => {
                self.result = err;
                Task::none()
            }
        }
    }
}

/// The page to display in the application.
pub enum Page {
    Basic,
    Advanced,
    Developer,
}

impl Page {
    fn as_str(&self) -> &str {
        match self {
            Page::Basic => "basic",
            Page::Advanced => "advanced",
            Page::Developer => "developer",
        }
    }

    fn from_str(s: &str) -> Option<Page> {
        match s {
            "basic" => Some(Page::Basic),
            "advanced" => Some(Page::Advanced),
            "developer" => Some(Page::Developer),
            _ => None,
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

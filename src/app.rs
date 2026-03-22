// SPDX-License-Identifier: MIT

use crate::calculator::get_paren_count;
use crate::calculator::insert_at_cursor;
use crate::calculator::substitute;
use crate::calculator::{EvalResult, evaluate_input, format_f64};
use crate::config::Config;
use crate::fl;
use crate::keyboard::{advanced_keyboard, basic_keyboard, developer_keyboard};
use crate::messages::{KeyPress, Message};
use calclib::numformat::NumberFormat;
use calclib::validator::validate;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::Horizontal;
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
    /// Calculator history as (expression, f64 result) pairs
    history: Vec<(String, f64)>,
    /// Calculator input
    input: String,
    /// Calculator result value, or an error string
    result: Option<Result<f64, String>>,
    /// Cursor position set by function buttons (e.g. inside `abs()`); None means append to end
    cursor_pos: Option<usize>,
    /// Character set used for input validation on the developer page
    number_format: NumberFormat,
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
                    .map(|data| data == &page)
                    .unwrap_or(false)
            });
            if let Some(id) = target {
                nav.activate(id);
            }
        }

        let number_format = NumberFormat::from_str(&config.number_format).unwrap_or_default();

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
            result: None,
            cursor_pos: None,
            number_format,
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
        cosmic::iced::event::listen_with(|event, _status, _window_id| -> Option<Message> {
            if let cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) = event
            {
                if modifiers.control() || modifiers.alt() {
                    return match key.as_ref() {
                        keyboard::Key::Character("d") => {
                            Some(Message::NumberFormatSelected(NumberFormat::Decimal))
                        }
                        keyboard::Key::Character("h") => {
                            Some(Message::NumberFormatSelected(NumberFormat::Hexadecimal))
                        }
                        keyboard::Key::Character("b") => {
                            Some(Message::NumberFormatSelected(NumberFormat::Binary))
                        }
                        _ => None,
                    };
                }
                match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        Some(Message::ArrowLeft)
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        Some(Message::ArrowRight)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Home) => Some(Message::Home),
                    keyboard::Key::Named(keyboard::key::Named::End) => Some(Message::End),
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        Some(Message::KeyPressed(KeyPress::Backspace))
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        Some(Message::KeyPressed(KeyPress::Equals))
                    }
                    _ => text.and_then(|t| match t.as_str() {
                        "=" => Some(Message::KeyPressed(KeyPress::Equals)),
                        s => Some(Message::KeyPressed(KeyPress::Insert(s.to_string()))),
                    }),
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
            .map(|(expr, value)| {
                let result_str = format_f64(*value, self.number_format);
                let input_str = format_f64(*value, self.number_format);
                widget::row::with_capacity(2)
                    .push(
                        text(format!("{} = {}", expr, result_str))
                            .size(14)
                            .width(Length::Fill)
                            .align_x(Horizontal::Right),
                    )
                    .push(widget::tooltip(
                        button::icon(icon::from_name("edit-copy-symbolic").size(14))
                            .extra_small()
                            .on_press(Message::CopyResultToInput(input_str)),
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
                    .on_paste(|s| Message::Paste(s))
                    .on_submit(|_| Message::KeyPressed(KeyPress::Equals))
                    .always_active()
                    .size(24)
                    .padding(Padding::new(20.0)),
            )
            .align_y(Alignment::End)
            .spacing(space_s);

        let charset_options: &'static [&'static str] = &["Decimal", "Hexadecimal", "Binary"];
        let charset_selected: Option<usize> = Some(match self.number_format {
            NumberFormat::Decimal => 0,
            NumberFormat::Hexadecimal => 1,
            NumberFormat::Binary => 2,
        });

        let calculator_mode: Element<_> = mode_buttons_row(space_s).into();

        let developer_mode_row: Element<_> = widget::row::with_capacity(3)
            .push(mode_buttons_row(space_s))
            .push(widget::horizontal_space())
            .push(widget::dropdown(charset_options, charset_selected, |idx| {
                Message::NumberFormatSelected(match idx {
                    1 => NumberFormat::Hexadecimal,
                    2 => NumberFormat::Binary,
                    _ => NumberFormat::Decimal,
                })
            }))
            .align_y(Alignment::Center)
            .into();

        let result_display = match &self.result {
            Some(Ok(value)) => format_f64(*value, self.number_format),
            Some(Err(err)) => err.clone(),
            None => "".to_string(),
        };

        let result = widget::row::with_capacity(1)
            .push(
                text(result_display)
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
                .push(basic_keyboard(self.number_format, space_s))
                .push(widget::vertical_space().height(25))
                .push(calculator_mode)
                .spacing(space_s)
                .into(),

            Page::Advanced => widget::column::with_capacity(6)
                .push(history)
                .push(input)
                .push(result)
                .push(advanced_keyboard(self.number_format, space_s))
                .push(widget::vertical_space().height(25))
                .push(calculator_mode)
                .spacing(space_s)
                .into(),

            Page::Developer => widget::column::with_capacity(6)
                .push(history)
                .push(input)
                .push(result)
                .push(developer_keyboard(self.number_format, space_s))
                .push(widget::vertical_space().height(25))
                .push(developer_mode_row)
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
            // Message::InputChanged(value) => {
            //     self.cursor_pos = None;
            //     if value.contains('=') || value.contains('\n') {
            //         return self.evaluate();
            //     }

            //     println!("Presses key: {value}");

            //     let substituted = substitute(&value);
            //     let number_format = self.number_format;
            //     let filtered: String = substituted
            //         .chars()
            //         .filter(|c| validate(c, number_format))
            //         .collect();
            //     // Reject a closing paren that would leave more closing than opening,
            //     // matching the same rule applied by the ")" button.
            //     if get_paren_count(&filtered) < 0 {
            //         // Revert: leave self.input unchanged; the widget will re-sync on
            //         // the next frame.
            //     } else {
            //         self.input = filtered;
            //     }
            // }
            Message::Paste(value) => {
                let filtered: String = value
                    .chars()
                    .filter(|c| validate(c, self.number_format))
                    .collect();

                let new_pos = insert_at_cursor(&mut self.input, &filtered, self.cursor_pos);
                self.cursor_pos = Some(new_pos);
                return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
            }
            Message::CopyResultToInput(result) => {
                self.input.push_str(&result);
                return Task::batch([
                    clipboard::write(result),
                    text_input::move_cursor_to_end(Id::new(INPUT_ID)),
                ]);
            }
            Message::KeyPressed(key) => {
                match key {
                    KeyPress::AllClear => {
                        self.history.clear();
                        self.input.clear();
                        self.result = None;
                        self.cursor_pos = None;
                        return text_input::move_cursor_to(Id::new(INPUT_ID), 0);
                    }
                    KeyPress::Clear => {
                        self.input.clear();
                        self.result = None;
                        self.cursor_pos = None;
                        return text_input::move_cursor_to(Id::new(INPUT_ID), 0);
                    }
                    KeyPress::Backspace => {
                        // Button backspace always removes the last character
                        let mut chars = self.input.chars();
                        chars.next_back();
                        self.input = chars.as_str().to_string();
                        return text_input::move_cursor_to_end(Id::new(INPUT_ID));
                    }
                    KeyPress::Negate => {
                        if self.input.starts_with('−') || self.input.starts_with('-') {
                            let mut chars = self.input.chars();
                            chars.next();
                            self.input = chars.as_str().to_string();
                        } else {
                            self.input.insert(0, '−');
                        }
                    }
                    KeyPress::Equals => {
                        let scroll_task = self.evaluate();
                        return Task::batch([
                            scroll_task,
                            text_input::move_cursor_to_end(Id::new(INPUT_ID)),
                        ]);
                    }
                    KeyPress::Ans => {
                        if let Some((_, last_value)) = self.history.last().cloned() {
                            let text = format_f64(last_value, self.number_format);
                            let new_pos = insert_at_cursor(&mut self.input, &text, self.cursor_pos);
                            self.cursor_pos = Some(new_pos);
                            return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                        }
                    }
                    KeyPress::CloseParen => {
                        if get_paren_count(&self.input) > 0 {
                            self.input.push(')');
                        }
                    }
                    KeyPress::Log => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "log()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    KeyPress::Ln => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "ln()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    KeyPress::Log2 => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "log₂()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    KeyPress::Reciprocal => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "1/()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    KeyPress::Sqrt => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "√()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    KeyPress::Cbrt => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "∛()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    KeyPress::And => {
                        let new_pos = insert_at_cursor(&mut self.input, " AND ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Or => {
                        let new_pos = insert_at_cursor(&mut self.input, " OR ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Nand => {
                        let new_pos = insert_at_cursor(&mut self.input, " NAND ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Nor => {
                        let new_pos = insert_at_cursor(&mut self.input, " NOR ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Xnor => {
                        let new_pos = insert_at_cursor(&mut self.input, " XNOR ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Xor => {
                        let new_pos = insert_at_cursor(&mut self.input, " XOR ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Lshift => {
                        let new_pos = insert_at_cursor(&mut self.input, " << ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Rshift => {
                        let new_pos = insert_at_cursor(&mut self.input, " >> ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Not => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "NOT()", self.cursor_pos);
                        let new_pos = inserted_end - 1;
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Mod => {
                        let new_pos = insert_at_cursor(&mut self.input, " MOD ", self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                    KeyPress::Abs => {
                        let inserted_end =
                            insert_at_cursor(&mut self.input, "abs()", self.cursor_pos);
                        let pos = inserted_end - 1;
                        self.cursor_pos = Some(pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
                    }
                    KeyPress::Insert(text) => {
                        let substituted = substitute(&text);
                        let number_format = self.number_format;
                        let validated: String = substituted
                            .chars()
                            .filter(|c| validate(c, number_format))
                            .collect();
                        if validated.is_empty() {
                            return Task::none();
                        }
                        let new_pos =
                            insert_at_cursor(&mut self.input, &validated, self.cursor_pos);
                        self.cursor_pos = Some(new_pos);
                        return text_input::move_cursor_to(Id::new(INPUT_ID), new_pos);
                    }
                }
                return text_input::move_cursor_to_end(Id::new(INPUT_ID));
            }
            Message::ModeSelected(page) => {
                let target = self
                    .nav
                    .iter()
                    .find(|&id| self.nav.data::<Page>(id) == Some(&page));
                if let Some(id) = target {
                    return self.on_nav_select(id);
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
                let pos = match self.cursor_pos {
                    None => len.saturating_sub(1),
                    Some(pos) => pos.saturating_sub(1),
                };
                self.cursor_pos = Some(pos);
                return text_input::move_cursor_to(Id::new(INPUT_ID), pos);
            }
            Message::ArrowRight => {
                let len = self.input.chars().count();
                self.cursor_pos = match self.cursor_pos {
                    None => None,
                    Some(pos) if pos + 1 >= len => None,
                    Some(pos) => Some(pos + 1),
                };
                return match self.cursor_pos {
                    Some(pos) => text_input::move_cursor_to(Id::new(INPUT_ID), pos),
                    None => text_input::move_cursor_to_end(Id::new(INPUT_ID)),
                };
            }
            Message::Home => {
                self.cursor_pos = Some(0);
                return text_input::move_cursor_to(Id::new(INPUT_ID), 0);
            }
            Message::End => {
                self.cursor_pos = None;
                return text_input::move_cursor_to_end(Id::new(INPUT_ID));
            }
            Message::NumberFormatSelected(number_format) => {
                self.number_format = number_format;
                self.save_number_format();
                // When triggered by keyboard shortcut, navigate to Developer page
                if !matches!(self.nav.active_data::<Page>(), Some(Page::Developer)) {
                    let dev_id = self
                        .nav
                        .iter()
                        .find(|&id| matches!(self.nav.data::<Page>(id), Some(Page::Developer)));
                    if let Some(id) = dev_id {
                        return self.on_nav_select(id);
                    }
                }
            }
        }
        Task::none()
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);
        if matches!(self.nav.active_data::<Page>(), Some(Page::Developer)) {
            self.number_format =
                NumberFormat::from_str(&self.config.number_format).unwrap_or_default();
        } else {
            self.number_format = NumberFormat::Decimal;
        }

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

fn mode_buttons_row(space_s: u16) -> widget::Row<'static, Message> {
    widget::row::with_capacity(3)
        .push(icon_button_view(
            Page::Basic,
            include_bytes!("../resources/basic.svg"),
        ))
        .push(icon_button_view(
            Page::Advanced,
            include_bytes!("../resources/advanced.svg"),
        ))
        .push(icon_button_view(
            Page::Developer,
            include_bytes!("../resources/developer.svg"),
        ))
        .spacing(space_s)
}

// Function to create the button with an SVG icon
fn icon_button_view(page: Page, svg_data: &'static [u8]) -> Element<'static, Message> {
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
        .on_press(Message::ModeSelected(page))
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

    fn save_number_format(&mut self) {
        self.config.number_format = self.number_format.as_str().to_string();
        if let Some(ref handler) = self.config_handler {
            let _ = self.config.write_entry(handler);
        }
    }

    fn evaluate(&mut self) -> Task<cosmic::Action<Message>> {
        match evaluate_input(&self.input, self.number_format) {
            EvalResult::Success { expression, value } => {
                self.result = Some(Ok(value));
                self.history.push((expression, value));
                self.input.clear();
                self.cursor_pos = None;
                cosmic::iced::widget::scrollable::snap_to(
                    Id::new(HISTORY_ID),
                    cosmic::iced::widget::scrollable::RelativeOffset::END,
                )
            }
            EvalResult::Failure(err) => {
                self.result = Some(Err(err.to_string()));
                Task::none()
            }
        }
    }
}

/// The page to display in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

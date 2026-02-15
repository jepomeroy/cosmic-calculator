// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::fl;
use calclib::evaluator::evaluate;
use calclib::validator::validate;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Padding};
use cosmic::prelude::*;
use cosmic::widget::{self, Id, about::About, button, icon, menu, nav_bar, text, text_input};
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
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    KeyPressed(String),
    CopyResultToInput(String),
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
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

        nav.insert()
            .text(fl!("basic"))
            .data::<Page>(Page::Basic)
            .icon(icon::from_name("accessories-calculator-symbolic"))
            .activate();

        nav.insert()
            .text(fl!("advanced"))
            .data::<Page>(Page::Advanced)
            .icon(icon::from_name("applications-system-symbolic"));

        nav.insert()
            .text(fl!("developer"))
            .data::<Page>(Page::Developer)
            .icon(icon::from_name(
                "preferences-desktop-remote-desktop-symbolic",
            ));

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
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

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

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
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

        let basic_keyboard: Element<_> = widget::column::with_capacity(1)
            .push(
                widget::row::with_capacity(5)
                    .push(make_button("AC", None))
                    .push(make_button("C", None))
                    .push(make_button("±", None))
                    .push(make_button("%", None))
                    .push(make_button("⌫", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(5)
                    .push(make_button("7", None))
                    .push(make_button("8", None))
                    .push(make_button("9", None))
                    .push(make_button("÷", None))
                    .push(make_button("(", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(5)
                    .push(make_button("4", None))
                    .push(make_button("5", None))
                    .push(make_button("6", None))
                    .push(make_button("×", None))
                    .push(make_button(")", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(4)
                    .push(make_button("1", None))
                    .push(make_button("2", None))
                    .push(make_button("3", None))
                    .push(make_button("−", None))
                    .push(make_button("!", None))
                    .spacing(space_s),
            )
            .push(
                widget::row::with_capacity(4)
                    .push(make_button("0", None))
                    .push(make_button(".", None))
                    .push(make_button("=", None))
                    .push(make_button("+", None))
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
            Page::Basic => widget::column::with_capacity(3)
                .push(history)
                .push(input)
                .push(result)
                .push(basic_keyboard)
                .spacing(space_s)
                .height(Length::Fill)
                .into(),

            Page::Advanced => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title2(fl!("advanced")))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                widget::column::with_capacity(1)
                    .push(header)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }

            Page::Developer => {
                let header = widget::row::with_capacity(2)
                    .push(widget::text::title1(fl!("developer")))
                    .align_y(Alignment::End)
                    .spacing(space_s);

                widget::column::with_capacity(1)
                    .push(header)
                    .spacing(space_s)
                    .height(Length::Fill)
                    .into()
            }
        };

        widget::container(content)
            .width(600)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::InputChanged(value) => {
                println!("input changed: {}", value);

                if value.chars().any(|c| c == '=' || c == '\n') {
                    return self.evaluate_input();
                }

                if value.chars().all(|c| validate(&c)) {
                    self.input = substitute(value);
                }
            }
            Message::CopyResultToInput(result) => {
                self.input.push_str(&result);
                return text_input::move_cursor_to_end(Id::new(INPUT_ID));
            }
            Message::KeyPressed(value) => {
                println!("key pressed: {}", value);

                match value.as_str() {
                    "AC" => {
                        self.history.clear();
                        self.input.clear();
                        self.result = "0".to_string();
                    }
                    "C" => {
                        self.input.clear();
                        self.result = "0".to_string();
                    }
                    "⌫" => {
                        self.input.pop();
                    }
                    "±" => {
                        if self.input.starts_with('-') {
                            self.input.remove(0);
                        } else {
                            self.input.insert(0, '-');
                        }
                    }
                    "=" => {
                        let scroll_task = self.evaluate_input();
                        return Task::batch([
                            scroll_task,
                            text_input::move_cursor_to_end(Id::new(INPUT_ID)),
                        ]);
                    }
                    _ => {
                        self.input.push_str(&value);
                    }
                }

                return text_input::move_cursor_to_end(Id::new(INPUT_ID));
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
            Message::UpdateConfig(config) => {
                println!("updating config: {:?}", config);
                self.config = config;
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
        }
        Task::none()
    }

    /// Called when a nav item is selected.
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

        self.update_title()
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
            .size(20)
            .font(cosmic::font::bold())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
    )
    .width(60)
    .height(40)
    .on_press(text_handler)
    .into()
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

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
            .replace('−', "-");
        match evaluate(expression) {
            Ok(result) => {
                self.result = result.value();
                self.history.push((self.input.clone(), self.result.clone()));
                self.input.clear();
                cosmic::iced::widget::scrollable::snap_to(
                    Id::new(HISTORY_ID),
                    cosmic::iced::widget::scrollable::RelativeOffset::END,
                )
            }
            Err(err) => {
                self.result = format!("{}", err);
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

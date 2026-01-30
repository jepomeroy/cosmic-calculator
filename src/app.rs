// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::fl;
use calclib::validator::validate;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Padding};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, icon, menu, nav_bar, text, text_editor, text_input};
use std::collections::HashMap;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

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
    /// Calculator history
    history: text_editor::Content,
    /// Calculator input
    input: String,
    /// Calculator result
    result: String,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    ActionPerformed(text_editor::Action),
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

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            nav,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            history: text_editor::Content::default(),
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
        let history = widget::row::with_capacity(1)
            .push(
                text_editor(&self.history)
                    .on_action(Message::ActionPerformed)
                    .wrapping(cosmic::iced_core::text::Wrapping::Word)
                    .height(Length::Fixed(120.0))
                    .padding(Padding::new(20.0)),
            )
            .align_y(Alignment::End)
            .spacing(space_s);

        let input = widget::row::with_capacity(1)
            .push(
                text_input("", &self.input)
                    .on_input(Message::InputChanged)
                    .always_active()
                    .size(24)
                    .padding(Padding::new(20.0)),
            )
            .align_y(Alignment::End)
            .spacing(space_s);

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
            Page::Basic => widget::column::with_capacity(2)
                .push(history)
                .push(input)
                .push(result)
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
            Message::ActionPerformed(action) => {
                if !action.is_edit() {
                    self.history.perform(action);
                }
            }
            Message::InputChanged(value) => {
                if value.chars().all(|c| validate(&c)) {
                    self.input = substitute(value);
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

            Message::UpdateConfig(config) => {
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

        self.update_title()
    }
}
/// Substitute certain characters with their calc lib equivalents
fn substitute(input: String) -> String {
    input.replace('*', "×").replace('/', "÷").replace('-', "−")
}

/// Validate input and allow only chars that the calc lib can handle
/// Returns true if the action is valid, false otherwise
fn validate_action(action: &text_editor::Action) -> bool {
    match action {
        text_editor::Action::Edit(edit) => match edit {
            text_editor::Edit::Insert(t) => validate(t),
            text_editor::Edit::Paste(t) => {
                for c in t.chars() {
                    if !validate(&c) {
                        return false;
                    }
                }
                true
            }
            // Enter, backspace, delete are always valid
            _ => true,
        },
        // Non-edit actions are always valid
        _ => true,
    }
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
}

/// The page to display in the application.
pub enum Page {
    Basic,
    Advanced,
    Developer,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_valid_insert_action() {
        // Valid insert action
        // Numbers and operators
        let valid_chars = vec![
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '*', '/', '(', ')', '.',
            '^', '%', '!', '=', '×', '÷', '−',
        ];

        for c in valid_chars {
            let action = text_editor::Action::Edit(text_editor::Edit::Insert(c));
            assert!(
                validate_action(&action),
                "Failed to validate insert action for char: {}",
                c
            );
        }
    }

    #[test]
    fn test_invalid_insert_action() {
        // Invalid insert action
        let invalid_chars = vec![
            'a', 'b', 'c', ' ', '@', '#', '$', '&', '_', '[', ']', '{', '}', ';', ':', '"', '\'',
            '<', '>', ',', '?', '\\', '|', '~', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
            'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ',
        ];

        for c in invalid_chars {
            let action = text_editor::Action::Edit(text_editor::Edit::Insert(c));
            assert!(
                !validate_action(&action),
                "Incorrectly validated insert action for char: {}",
                c
            );
        }
    }

    #[test]
    fn test_valid_paste_action() {
        // Valid Paste action
        let action =
            text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new("123+456".to_string())));
        assert!(validate_action(&action));
    }

    #[test]
    fn test_invalid_paste_action() {
        // Invalid insert action
        let action =
            text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new("123a456".to_string())));
        assert!(!validate_action(&action));
    }
}

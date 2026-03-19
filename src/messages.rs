use calclib::numformat::NumberFormat;

use crate::app::{ContextPage, Page};

/// Typed key actions emitted by calculator buttons.
#[derive(Debug, Clone)]
pub enum KeyPress {
    AllClear,
    Clear,
    Backspace,
    Negate,
    Equals,
    Ans,
    CloseParen,
    Log,
    Ln,
    Log2,
    Reciprocal,
    Sqrt,
    Cbrt,
    Abs,
    /// Insert text into the input (passed through `substitute` before insertion).
    Insert(&'static str),
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    KeyPressed(KeyPress),
    ModeSelected(Page),
    CopyResultToInput(String),
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    NumberFormatSelected(NumberFormat),
}

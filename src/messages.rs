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
    /// Advanced math
    Log,
    Ln,
    Log2,
    Reciprocal,
    Sqrt,
    Cbrt,
    Abs,
    /// Boolean math
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Lshift,
    Rshift,
    Mod,
    Not,
    /// Insert text into the input (passed through `substitute` before insertion).
    Insert(String),
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    // InputChanged(String),
    KeyPressed(KeyPress),
    Paste(String),
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

use calclib::numformat::NumberFormat;
use cosmic::{
    Element,
    iced::{
        Length,
        alignment::{Horizontal, Vertical},
    },
    widget::{self, button, text},
};

use crate::messages::{KeyPress, Message};

fn hexadecimal_keys(number_format: NumberFormat, space_s: u16) -> Element<'static, Message> {
    let hex_enabled = number_format == NumberFormat::Hexadecimal;
    let hexadecimal_keyboard: Element<_> = widget::column::with_capacity(1)
        .push(
            widget::row::with_capacity(1)
                .push(make_button_enabled(
                    "A",
                    KeyPress::Insert("A".to_string()),
                    hex_enabled,
                ))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(1)
                .push(make_button_enabled(
                    "B",
                    KeyPress::Insert("B".to_string()),
                    hex_enabled,
                ))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(1)
                .push(make_button_enabled(
                    "C",
                    KeyPress::Insert("C".to_string()),
                    hex_enabled,
                ))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(1)
                .push(make_button_enabled(
                    "D",
                    KeyPress::Insert("D".to_string()),
                    hex_enabled,
                ))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(1)
                .push(make_button_enabled(
                    "E",
                    KeyPress::Insert("E".to_string()),
                    hex_enabled,
                ))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(1)
                .push(make_button_enabled(
                    "F",
                    KeyPress::Insert("F".to_string()),
                    hex_enabled,
                ))
                .spacing(space_s),
        )
        .spacing(space_s)
        .into();

    hexadecimal_keyboard
}

fn advanced_keys(space_s: u16) -> Element<'static, Message> {
    let advanced_keyboard: Element<_> = widget::column::with_capacity(1)
        .push(
            widget::row::with_capacity(2)
                .push(make_button("Log", KeyPress::Log))
                .push(make_button("Ln", KeyPress::Ln))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("1/x", KeyPress::Reciprocal))
                .push(make_button("Log₂x", KeyPress::Log2))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("√", KeyPress::Sqrt))
                .push(make_button("∛", KeyPress::Cbrt))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("x²", KeyPress::Insert("²".to_string())))
                .push(make_button("x³", KeyPress::Insert("³".to_string())))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("xʸ", KeyPress::Insert("^".to_string())))
                .push(make_button("Abs", KeyPress::Abs))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("π", KeyPress::Insert("π".to_string())))
                .push(make_button("e", KeyPress::Insert("e".to_string())))
                .spacing(space_s),
        )
        .spacing(space_s)
        .into();

    advanced_keyboard
}

fn developer_keys(space_s: u16) -> Element<'static, Message> {
    let developer_keyboard: Element<_> = widget::column::with_capacity(1)
        .push(
            widget::row::with_capacity(2)
                .push(make_button("AND", KeyPress::And))
                .push(make_button("OR", KeyPress::Or))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("NAND", KeyPress::Nand))
                .push(make_button("NOR", KeyPress::Nor))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("XNOR", KeyPress::Xnor))
                .push(make_button("XOR", KeyPress::Xor))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("«", KeyPress::Lshift))
                .push(make_button("»", KeyPress::Rshift))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(2)
                .push(make_button("MOD", KeyPress::Mod))
                .push(make_button("NOT", KeyPress::Not))
                .spacing(space_s),
        )
        .spacing(space_s)
        .into();

    developer_keyboard
}

fn basic_keys(
    number_format: NumberFormat,
    space_s: u16,
    decimal_enabled: bool,
) -> Element<'static, Message> {
    let dec_enabled = number_format != NumberFormat::Binary;
    let basic_keyboard: Element<_> = widget::column::with_capacity(1)
        .push(
            widget::row::with_capacity(4)
                .push(make_button("AC", KeyPress::AllClear))
                .push(make_button("C", KeyPress::Clear))
                .push(make_button("⌫", KeyPress::Backspace))
                .push(make_button("Ans", KeyPress::Ans))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(4)
                .push(make_button("(", KeyPress::Insert("(".to_string())))
                .push(make_button(")", KeyPress::CloseParen))
                .push(make_button("±", KeyPress::Negate))
                .push(make_button("!", KeyPress::Insert("!".to_string())))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(4)
                .push(make_button_enabled(
                    "7",
                    KeyPress::Insert("7".to_string()),
                    dec_enabled,
                ))
                .push(make_button_enabled(
                    "8",
                    KeyPress::Insert("8".to_string()),
                    dec_enabled,
                ))
                .push(make_button_enabled(
                    "9",
                    KeyPress::Insert("9".to_string()),
                    dec_enabled,
                ))
                .push(make_button("×", KeyPress::Insert("×".to_string())))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(4)
                .push(make_button_enabled(
                    "4",
                    KeyPress::Insert("4".to_string()),
                    dec_enabled,
                ))
                .push(make_button_enabled(
                    "5",
                    KeyPress::Insert("5".to_string()),
                    dec_enabled,
                ))
                .push(make_button_enabled(
                    "6",
                    KeyPress::Insert("6".to_string()),
                    dec_enabled,
                ))
                .push(make_button("÷", KeyPress::Insert("÷".to_string())))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(4)
                .push(make_button("1", KeyPress::Insert("1".to_string())))
                .push(make_button_enabled(
                    "2",
                    KeyPress::Insert("2".to_string()),
                    dec_enabled,
                ))
                .push(make_button_enabled(
                    "3",
                    KeyPress::Insert("3".to_string()),
                    dec_enabled,
                ))
                .push(make_button("+", KeyPress::Insert("+".to_string())))
                .spacing(space_s),
        )
        .push(
            widget::row::with_capacity(4)
                .push(make_button_enabled(
                    ".",
                    KeyPress::Insert(".".to_string()),
                    decimal_enabled,
                ))
                .push(make_button("0", KeyPress::Insert("0".to_string())))
                .push(make_button("=", KeyPress::Equals))
                .push(make_button("-", KeyPress::Insert("-".to_string())))
                .spacing(space_s),
        )
        .spacing(space_s)
        .into();

    basic_keyboard
}

fn make_button(label: &'static str, key: KeyPress) -> Element<'static, Message> {
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
    .on_press(Message::KeyPressed(key))
    .into()
}

fn make_button_enabled(
    label: &'static str,
    key: KeyPress,
    enabled: bool,
) -> Element<'static, Message> {
    let btn = button::custom(
        text(label)
            .size(18)
            .font(cosmic::font::bold())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
    )
    .width(70)
    .height(40);

    if enabled {
        btn.on_press(Message::KeyPressed(key)).into()
    } else {
        btn.into()
    }
}

pub(crate) fn advanced_keyboard(
    number_format: NumberFormat,
    space_s: u16,
) -> Element<'static, Message> {
    widget::container(
        widget::row::with_capacity(2)
            .push(basic_keys(number_format, space_s, true))
            .push(advanced_keys(space_s))
            .spacing(space_s),
    )
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

pub(crate) fn basic_keyboard(
    number_format: NumberFormat,
    space_s: u16,
) -> Element<'static, Message> {
    widget::container(basic_keys(number_format, space_s, true))
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .into()
}

pub(crate) fn developer_keyboard(
    number_format: NumberFormat,
    space_s: u16,
) -> Element<'static, Message> {
    widget::container(
        widget::row::with_capacity(3)
            .push(hexadecimal_keys(number_format, space_s))
            .push(basic_keys(number_format, space_s, false))
            .push(developer_keys(space_s))
            .spacing(space_s),
    )
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

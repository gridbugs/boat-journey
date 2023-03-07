use crate::game_loop::{AppCF, State};
use gridbugs::chargrid::{
    control_flow::*,
    prelude::*,
    text::{StyledString, Text},
};

fn text_component(width: u32, text: Vec<StyledString>) -> CF<(), State> {
    Text::new(text).wrap_word().cf().set_width(width)
}

pub fn help(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    let b = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text().with_bold(true),
    };
    text_component(
        width,
        vec![
            b("Help\n\n"),
            b("On Foot\n"),
            t("Walk: Arrow Keys\n"),
            t("Drive Boat: e\n"),
            t("\n"),
            b("Driving Boat\n"),
            t("Move: Forward/Backward\n"),
            t("Turn: Left/Right\n"),
            t("Leave Boat: e\n"),
        ],
    )
    .press_any_key()
}

fn win_text(width: u32) -> CF<(), State> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("The ocean welcomes your return.")])
}
pub fn win(width: u32) -> AppCF<()> {
    // TODO: this is not ergonomic
    win_text(width)
        .delay(Duration::from_secs(2))
        .then(move || win_text(width).press_any_key())
}

fn game_over_text(width: u32) -> CF<(), State> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("You fail to reach the ocean.")])
}

pub fn game_over(width: u32) -> AppCF<()> {
    game_over_text(width)
        .delay(Duration::from_secs(2))
        .then(move || game_over_text(width).press_any_key())
}

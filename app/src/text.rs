use crate::game_loop::AppCF;
use gridbugs::chargrid::{
    prelude::*,
    text::{StyledString, Text},
};

fn text_component(width: u32, text: Vec<StyledString>) -> AppCF<()> {
    Text::new(text)
        .wrap_word()
        .cf()
        .set_width(width)
        .press_any_key()
}

pub fn help(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("Help")])
}

pub fn win(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("Win")])
}

pub fn game_over(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("Game Over")])
}

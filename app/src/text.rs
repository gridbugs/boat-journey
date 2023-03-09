use crate::game_loop::{AppCF, State};
use boat_journey_game::GameOverReason;
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

pub fn loading(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("Generating...")]).delay(Duration::from_millis(32))
}

pub fn saving(width: u32) -> AppCF<()> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    text_component(width, vec![t("Saving...")]).delay(Duration::from_millis(32))
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

fn game_over_text(width: u32, reason: GameOverReason) -> CF<(), State> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    let text = match reason {
        GameOverReason::OutOfFuel => vec! {
            t("You fail to reach the ocean.\n\n"),
            t("The boat sputters to a halt as the last dregs of fuel are consumed.\n\n"),
            t("Over time you make a home aboard the stationary boat and hope that one day someone will pick you up and take you to the ocean."),},
        GameOverReason::KilledByGhost => vec!{
            t("You fail to reach the ocean.\n\n"),
            t("At the icy touch of the ghost you lose your corporeal form.\n\n"),
            t("You lose sight of the boat as an unfamiliar figure drives it into the darkness."),
        },
    };
    text_component(width, text)
}

pub fn game_over(width: u32, reason: GameOverReason) -> AppCF<()> {
    game_over_text(width, reason)
        .delay(Duration::from_secs(2))
        .then(move || game_over_text(width, reason).press_any_key())
}

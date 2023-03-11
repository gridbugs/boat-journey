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
            b("Controls:\n\n"),
            b("General\n"),
            t("Wait: Space\n"),
            t("Ability: 1-9\n"),
            t("\n"),
            b("On Foot\n"),
            t("Walk: Arrow Keys\n"),
            t("Drive Boat: e\n"),
            t("\n"),
            b("Driving Boat\n"),
            t("Move: Forward/Backward\n"),
            t("Turn: Left/Right\n"),
            t("Leave Boat: e\n"),
            b("\n\nTips:\n\n"),
            t("- Walk into a door (+) to open it\n"),
            t("- Walk into the wall next to a door to close the door\n"),
            t("- Head to the inn when it gets dark\n"),
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
        GameOverReason::KilledByBeast => vec!{
            t("You fail to reach the ocean.\n\n"),
            t("You are wounded by the beast and unable to return to your boat.\n\n"),
            t("Over time your wounds heal but you no longer wish to travel to the ocean and don't understand why anyone would."),
        },
        GameOverReason::Abandoned => vec!{
            t("You fail to reach the ocean and decide to remain in the inn.\n\n"),
        },
        GameOverReason::KilledBySoldier => vec!{
            t("You fail to reach the ocean.\n\n"),
            t("You were caught in the soldier's blast.\n\n"),
        },
    };
    text_component(width, text)
}

pub fn game_over(width: u32, reason: GameOverReason) -> AppCF<()> {
    game_over_text(width, reason)
        .delay(Duration::from_secs(2))
        .then(move || game_over_text(width, reason).press_any_key())
}

fn sleep_text(width: u32, i: u32) -> CF<(), State> {
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: Style::plain_text(),
    };
    let _ = i;
    let text = vec![t(
        "Your passengers come in from the cold and you rest in the inn until morning.\n\n\
        The ghosts disappear and your health is restored.\n\n\
        Passenger actions are refreshed.\n\n\
        Press any key...",
    )];
    text_component(width, text)
}

pub fn sleep(width: u32, i: u32) -> AppCF<()> {
    sleep_text(width, i)
        .delay(Duration::from_millis(100))
        .then(move || sleep_text(width, i).press_any_key())
}

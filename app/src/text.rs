use crate::{colours, game_loop::CF};
use chargrid::{
    prelude::*,
    text::{StyledString, Text},
};

fn text_component(width: u32, text: Vec<StyledString>) -> CF<()> {
    Text::new(text)
        .wrap_word()
        .boxed_cf()
        .set_width(width)
        .press_any_key()
}

pub fn prologue(width: u32) -> CF<()> {
    let bold = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(true);
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let t = |text: &str, style| StyledString {
        string: text.to_string(),
        style,
    };
    text_component(width, vec![
        t("You tape over the flashing warning light. An overheating engine is the least of your worries. \
        Gotta focus.\n\n\
        The space station looms ahead. It's out of fuel, and about to come crashing down to Earth. \
        Unless you get to it first. \
        Special delivery: 1 hydrogen fuel cell with enough juice to kick the station out of this pesky \
        atmosphere and back into space where it belongs.\n\n\
        Back home your buddies are placing bets on whether you'll make it back alive. \
        Last you heard, odds were 5 to 1 against.\n\n\
        \"Docking complete,\" sounds a lifeless mechanical voice. No word yet from the station crew. Comms must be down. Figures. \
        Shouldering your pack containing the fuel cell, you trudge into the airlock. \
        Gotta lug this thing down the five flights of stairs to the fuel bay. Who designed this place?\n\n\
        A dim light flickers on in the airlock revealing words smeared in blood on the opposite door:\n", normal),
        t("DON'T OPEN! DEAD INSIDE!", bold),
        t("\n\n\
            Better make those odds 6 to 1...", normal),
            t("\n\n\n\n\n\nPress any key...", faint),
    ])
}

pub fn help(width: u32) -> CF<()> {
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new().with_foreground(colours::STRIPE);
    let f = |s: &str| StyledString {
        string: s.to_string(),
        style: faint,
    };
    let b = |s: &str| StyledString {
        string: s.to_string(),
        style: normal.with_bold(true),
    };
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: normal,
    };
    text_component(width, vec![
        b("Combat\n"),
        t("Each weapon has a DMG(♥) and PEN(♦) stat, and each enemy has heatlh(♥) and armour(♦). "),
        t("If an enemy is hit with a weapon that has a higher PEN than their armour, their health is "),
        t("reduced by the weapon's DMG. If a projectile's PEN exceeds an enemy's armour, it continues "),
        t("on its path with its PEN reduced by the enemy's armour. If it hits the hull, it has a "),
        t("chance to breach the hull (its HULL PEN stat).\n\n"),
        b("Hull Breaches\n"),
        t("If the hull is breached the air is sucked out of connected areas of the station. "),
        t("For several turns, characters and items in connected areas are pulled towards the breach. "),
        t("This is indicated by a red light. "),
        t("After the air is drained, the light will turn blue indicating vacuum. "),
        t("Your oxygen will start decreasing, and if it runs out then your health will start decreasing "),
        t("until you get back into a pressurised area.\n\n"),
        b("Keyboard Controls\n"),
        t("Movement/Aim: Arrows/WASD/HJKL\n"),
        t("Cancel Aim: Escape\n"),
        t("Wait: Space\n"),
        t("Examine: X\n"),
        t("Get Weapon: G\n"),
        t("Fire Ranged Weapon: 1-3\n\n"),
        b("Gamepad Controls\n"),
        t("Movement/Aim: D-Pad\n"),
        t("Cancel Aim: Select\n"),
        t("Wait: Select\n"),
        t("Examine: Right Bumper\n"),
        t("Get Weapon: Y/Triangle\n"),
        t("Fire Ranged Weapon Slot 1: X/Square\n"),
        t("Fire Ranged Weapon Slot 2: A/Cross\n"),
        t("Fire Ranged Weapon Slot 2: B/Circle\n"),
        f("\n\n\n\n\nPress any key..."),
    ])
}

fn epilogue1(width: u32) -> CF<()> {
    let bold = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(true);
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let t = |text: &str, style| StyledString {
        string: text.to_string(),
        style,
    };
    text_component(
        width,
        vec![
            t(
                "With its fuel supply restored, the station flies back into orbit. \
            On autopilot. Shame about the crew, but these things happen. Nobody said \
            space was a safe place to work.\n\n\
            You undock your shuttle and make for Earth. Easy does it. Gotta make it \
            back in one piece and collect on that ",
                normal,
            ),
            t("hefty wager", bold),
            t(
                " you placed on yourself. \
            Serves those suckers right for betting against you!\n\n\
            No doubt there'll be a ton of paperwork to complete before you can go home. \
            The company can't have this getting out. It's gonna be all NDA this and \
            sworn to secrecy that. Don't go running to the press about the ",
                normal,
            ),
            t(
                "undead space \
            station crew",
                bold,
            ),
            t(" you just put down. Now sign here.", normal),
            t("\n\n\n\n\n\nPress any key...", faint),
        ],
    )
}

fn epilogue2(width: u32) -> CF<()> {
    let bold = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(true);
    let normal = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let faint = Style::new()
        .with_foreground(colours::STRIPE)
        .with_bold(false);
    let t = |text: &str, style| StyledString {
        string: text.to_string(),
        style,
    };
    text_component(width, vec![
        t(
            "Now that you have time to think, something gives you pause. \
            Pretty big coincidence, the station running out of fuel at the ",
            normal,
        ),
        t("same time", bold),
        t(
            " that its crew transforms into a horde of ravenous bloodthirsty monsters. \
                The next scheduled resupply wasn't for months. They should have had plenty \
                of fuel!\n\n\
                And those words in the airlock: \"Don't open! Dead inside!\" Were they meant \
                for you? Who wrote them? How did they know the company would send a shuttle? The airlock was \
                deserted, so whoever wrote it must have gone back inside.\n\n\
                The airlock ",
                normal,
        ),
        t("was", bold),
        t(
            " empty. Yes. It was empty and you sealed the door behind you. There's no way any of those ",
            normal,
        ),
        t("things", bold),
        t(" could have snuck aboard your shuttle.\n\n\
            Everything is fine.",
            normal,
        ),
        t("\n\n\n\n\n\nPress any key...", faint),
    ])
}

pub fn epilogue(width: u32) -> CF<()> {
    epilogue1(width).and_then(move |()| epilogue2(width))
}

use crate::colours;
use chargrid::render::{ColModify, Coord, Frame, Rgb24, Style, View, ViewContext};
use chargrid::text::{
    wrap, RichStringViewSingleLine, RichTextPart, RichTextPartOwned, RichTextView,
    StringViewSingleLine,
};
use orbital_decay_game::{
    player::{self, Player, Weapon, WeaponAbility, WeaponName},
    CharacterInfo, Enemy, MeleeWeapon, Message, RangedWeapon,
};

pub struct Ui<'a> {
    pub player: &'a Player,
    pub player_info: &'a CharacterInfo,
}

pub struct UiView;

impl UiView {
    pub fn view<F: Frame, C: ColModify>(&mut self, ui: Ui, context: ViewContext<C>, frame: &mut F) {
        let plain = Style::new().with_foreground(Rgb24::new_grey(255));
        let mut text_view = RichTextView::new(wrap::None::new());
        text_view.view(
            vec![
                RichTextPart::new("Health: ", plain),
                RichTextPart::new(
                    format!(
                        "{}/{}",
                        ui.player_info.hit_points.current, ui.player_info.hit_points.max
                    )
                    .as_str(),
                    Style::new()
                        .with_foreground(crate::colours::HEALTH)
                        .with_bold(true),
                ),
                RichTextPart::new("\n", plain),
                RichTextPart::new("Oxygen: ", plain),
                RichTextPart::new(
                    format!(
                        "{}/{}",
                        ui.player_info.oxygen.current, ui.player_info.oxygen.max
                    )
                    .as_str(),
                    Style::new()
                        .with_foreground(crate::colours::OXYGEN)
                        .with_bold(true),
                ),
                RichTextPart::new("\n", plain),
                RichTextPart::new("Credit: ", plain),
                RichTextPart::new(
                    format!("${}", ui.player.credit).as_str(),
                    Style::new()
                        .with_foreground(colours::CREDIT_FOREGROUND)
                        .with_bold(true),
                ),
                RichTextPart::new("\n", plain),
            ],
            context,
            frame,
        );
        view_weapon(
            "Melee:",
            &ui.player.melee_weapon,
            &ui.player,
            context.add_offset(Coord { x: 0, y: 5 }),
            frame,
        );
        let context = context.add_offset(Coord { x: 0, y: 15 });
        for (i, ranged_slot) in ui.player.ranged_weapons.iter().enumerate() {
            if let Some(weapon) = ranged_slot {
                view_weapon(
                    format!("Ranged {}:", i + 1).as_str(),
                    weapon,
                    &ui.player,
                    context.add_offset(Coord {
                        x: 0,
                        y: i as i32 * 10,
                    }),
                    frame,
                );
            } else {
                view_empty_weapon_slot(
                    format!("Ranged {}:", i + 1).as_str(),
                    context.add_offset(Coord {
                        x: 0,
                        y: i as i32 * 10,
                    }),
                    frame,
                );
            }
        }
        view_upgrades(&ui, context.add_offset(Coord { x: 0, y: 32 }), frame);
    }
}

fn weapon_name_text(weapon_name: WeaponName) -> RichTextPartOwned {
    let t = |s: &str, c| {
        RichTextPartOwned::new(
            s.to_string(),
            Style::new().with_foreground(c).with_bold(true),
        )
    };
    match weapon_name {
        WeaponName::BareHands => t("Bare Hands", Rgb24::new_grey(255)),
        WeaponName::MeleeWeapon(MeleeWeapon::Chainsaw) => t(
            "Chainsaw",
            colours::CHAINSAW.saturating_scalar_mul_div(3, 2),
        ),
        WeaponName::RangedWeapon(RangedWeapon::Shotgun) => {
            t("Shotgun", colours::WOOD.saturating_scalar_mul_div(3, 2))
        }
        WeaponName::RangedWeapon(RangedWeapon::Railgun) => t("Railgun", colours::PLASMA),
        WeaponName::RangedWeapon(RangedWeapon::Rifle) => t("Rifle", colours::LASER),
        WeaponName::RangedWeapon(RangedWeapon::GausCannon) => {
            t("Gaus Cannon", colours::GAUS.saturating_scalar_mul_div(3, 2))
        }
        WeaponName::RangedWeapon(RangedWeapon::Oxidiser) => t("Oxidiser", colours::OXYGEN),
        WeaponName::RangedWeapon(RangedWeapon::LifeStealer) => t("Life Stealer", colours::HEALTH),
    }
}

fn enemy_text(enemy: Enemy) -> RichTextPartOwned {
    let t = |s: &str, c| {
        RichTextPartOwned::new(
            s.to_string(),
            Style::new().with_foreground(c).with_bold(true),
        )
    };
    match enemy {
        Enemy::Zombie => t("Zombie", colours::ZOMBIE.saturating_scalar_mul_div(3, 2)),
        Enemy::Skeleton => t("Skeleton", colours::SKELETON),
        Enemy::Boomer => t("Boomer", colours::BOOMER),
        Enemy::Tank => t("Tank", colours::TANK.saturating_scalar_mul_div(3, 2)),
    }
}

fn weapon_ability_text(weapon_ability: WeaponAbility) -> RichTextPartOwned {
    match weapon_ability {
        WeaponAbility::KnockBack => RichTextPartOwned::new(
            "Knocks Back".to_string(),
            Style::new().with_foreground(Rgb24::new(0xFF, 0x44, 0x00)),
        ),
        WeaponAbility::LifeSteal => RichTextPartOwned::new(
            "Restores Health".to_string(),
            Style::new().with_foreground(colours::HEALTH),
        ),
        WeaponAbility::Oxidise => RichTextPartOwned::new(
            "Restores Oxygen".to_string(),
            Style::new().with_foreground(colours::OXYGEN),
        ),
    }
}

fn view_empty_weapon_slot<F: Frame, C: ColModify>(
    title: &str,
    context: ViewContext<C>,
    frame: &mut F,
) {
    let plain = Style::new()
        .with_foreground(Rgb24::new_grey(255))
        .with_bold(false);
    let mut rich_view = RichStringViewSingleLine::new();
    let mut plain_view = StringViewSingleLine::new(plain);
    rich_view.view(
        RichTextPart::new(
            title,
            Style::new()
                .with_foreground(Rgb24::new_grey(255))
                .with_bold(false),
        ),
        context,
        frame,
    );
    plain_view.view("(empty)", context.add_offset(Coord::new(0, 1)), frame);
}

fn view_weapon<F: Frame, C: ColModify>(
    title: &str,
    weapon: &Weapon,
    player: &Player,
    context: ViewContext<C>,
    frame: &mut F,
) {
    let plain = Style::new()
        .with_foreground(Rgb24::new_grey(255))
        .with_bold(false);
    let mut rich_view = RichStringViewSingleLine::new();
    let mut plain_view = StringViewSingleLine::new(plain);
    rich_view.view(
        RichTextPart::new(
            title,
            Style::new()
                .with_foreground(Rgb24::new_grey(255))
                .with_bold(false),
        ),
        context,
        frame,
    );
    rich_view.view(
        weapon_name_text(weapon.name).as_rich_text_part(),
        context.add_offset(Coord::new(0, 1)),
        frame,
    );
    if let Some(ammo) = weapon.ammo.as_ref() {
        plain_view.view(
            format!("AMMO: {}/{}\n", ammo.current, ammo.max).as_str(),
            context.add_offset(Coord::new(0, 2)),
            frame,
        );
    } else {
        plain_view.view("AMMO: -", context.add_offset(Coord::new(0, 2)), frame);
    }
    plain_view.view(
        format!("PEN(♦): {}\n", weapon.pen).as_str(),
        context.add_offset(Coord::new(0, 3)),
        frame,
    );
    let extra = if player.traits.double_damage {
        "x2"
    } else {
        ""
    };
    plain_view.view(
        format!("DMG(♥): {}{}\n", weapon.dmg, extra).as_str(),
        context.add_offset(Coord::new(0, 4)),
        frame,
    );
    let extra = if player.traits.reduce_hull_pen {
        "/2"
    } else {
        ""
    };
    plain_view.view(
        format!("HULL PEN: {}%{}\n", weapon.hull_pen_percent, extra).as_str(),
        context.add_offset(Coord::new(0, 5)),
        frame,
    );
    for &ability in weapon.abilities.iter() {
        rich_view.view(
            weapon_ability_text(ability).as_rich_text_part(),
            context.add_offset(Coord::new(0, 6)),
            frame,
        );
    }
}

fn view_upgrades<F: Frame, C: ColModify>(ui: &Ui, context: ViewContext<C>, frame: &mut F) {
    let plain = Style::new()
        .with_foreground(Rgb24::new_grey(255))
        .with_bold(false);
    let mut view = StringViewSingleLine::new(plain);
    StringViewSingleLine::new(plain.with_bold(true)).view("Upgrades:", context, frame);
    let mut upgrades = Vec::new();
    use player::UpgradeLevel::*;
    if let Some(level) = ui.player.upgrade_table.toughness {
        upgrades.push("T1: Strong Back");
        if level == Level2 {
            upgrades.push("T2: Hardy");
        }
    }
    if let Some(level) = ui.player.upgrade_table.accuracy {
        upgrades.push("A1: Careful");
        if level == Level2 {
            upgrades.push("A2: Kill Shot");
        }
    }
    if let Some(level) = ui.player.upgrade_table.endurance {
        upgrades.push("E1: Sure-Footed");
        if level == Level2 {
            upgrades.push("E2: Big Lungs");
        }
    }
    if upgrades.is_empty() {
        view.view("(none)", context.add_offset(Coord { x: 0, y: 1 }), frame);
    } else {
        for (i, upgrade) in upgrades.into_iter().enumerate() {
            view.view(
                upgrade,
                context.add_offset(Coord {
                    x: 0,
                    y: i as i32 + 1,
                }),
                frame,
            );
        }
    }
}

pub fn view_message_log<F: Frame, C: ColModify>(
    messages: &[Message],
    context: ViewContext<C>,
    frame: &mut F,
) {
    const N: usize = 13;
    let plain = Style::new()
        .with_foreground(Rgb24::new_grey(255))
        .with_bold(false);
    let bold = Style::new()
        .with_foreground(Rgb24::new_grey(255))
        .with_bold(true);
    let start = messages.len().saturating_sub(N);
    let mut view = RichTextView::new(wrap::None::new());
    let t = |text: &str, style| RichTextPartOwned::new(text.to_string(), style);
    for (i, message) in messages[start..].iter().enumerate() {
        let offset = Coord { x: 0, y: i as i32 };
        let text = match message {
            Message::MaybeThisWasntSuchAGoodIdea => {
                vec![t("Maybe this wasn't such a good idea...", plain)]
            }
            Message::EquipWeapon(weapon) => {
                vec![
                    t("You equip the ", plain),
                    weapon_name_text(*weapon),
                    t(".", plain),
                ]
            }
            Message::PulledByVacuum => {
                vec![t("You are pulled towards the vacuum of space.", plain)]
            }
            Message::Descend => {
                vec![t("You descend to the next floor. Ammo refilled!", plain)]
            }
            Message::Suffocating => {
                vec![
                    t("Oxygen tank is empty. ", plain),
                    t(
                        "You are suffocating!",
                        bold.with_foreground(Rgb24::new(255, 0, 0)),
                    ),
                ]
            }
            Message::Heal => {
                vec![t("Health restored.", plain)]
            }
            Message::TakeCredit(1) => {
                vec![t("You gain $1 of credit.", plain)]
            }
            Message::TakeCredit(2) => {
                vec![t("You gain $2 of credit.", plain)]
            }
            Message::TakeCredit(_) => {
                vec![]
            }
            Message::BoomerExplodes => {
                vec![
                    t("The ", plain),
                    enemy_text(Enemy::Boomer),
                    t(" explodes!", plain),
                ]
            }
            Message::EnemyHitPlayer(enemy) => {
                vec![t("The ", plain), enemy_text(*enemy), t(" hits you!", plain)]
            }
            Message::PlayerHitEnemy { enemy, weapon } => {
                vec![
                    t("You hit the ", plain),
                    enemy_text(*enemy),
                    t(" with your ", plain),
                    weapon_name_text(*weapon),
                    t(".", plain),
                ]
            }
            Message::PlayerDies => {
                vec![t("You die!", bold.with_foreground(Rgb24::new(255, 0, 0)))]
            }
            Message::EnemyDies(enemy) => {
                vec![t("The ", plain), enemy_text(*enemy), t(" dies.", plain)]
            }
            Message::PlayerAdrift => {
                vec![t(
                    "You fall into the void!",
                    bold.with_foreground(Rgb24::new(0, 0, 255)),
                )]
            }
            Message::EnemyAdrift(enemy) => {
                vec![
                    t("The ", plain),
                    enemy_text(*enemy),
                    t(" falls into the void.", plain),
                ]
            }
            Message::MapTerminal => {
                vec![
                    t("You access the ", plain),
                    t(
                        "Map Terminal",
                        bold.with_foreground(colours::MAP_BACKGROUND),
                    ),
                    t(".", plain),
                ]
            }
        };
        let text = text.iter().map(|t| t.as_rich_text_part());
        view.view(text, context.add_offset(offset), frame);
    }
}

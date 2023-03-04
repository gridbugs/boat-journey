use crate::colours;
use gridbugs::chargrid::{
    prelude::*,
    text::{StyledString, Text},
};
use template2023_game::{
    player::{self, Player, Weapon, WeaponAbility, WeaponName},
    CharacterInfo, Enemy, MeleeWeapon, Message, RangedWeapon,
};

pub fn render_message_log(messages: &[Message], ctx: Ctx, fb: &mut FrameBuffer) {
    const N: usize = 13;
    let plain = Style::new()
        .with_foreground(Rgba32::new_grey(255))
        .with_bold(false);
    let bold = Style::new()
        .with_foreground(Rgba32::new_grey(255))
        .with_bold(true);
    let start = messages.len().saturating_sub(N);
    let t = |text: &str, style| StyledString {
        string: text.to_string(),
        style,
    };
    for (i, message) in messages[start..].iter().enumerate() {
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
                        bold.with_foreground(Rgba32::new_rgb(255, 0, 0)),
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
                vec![t(
                    "You die!",
                    bold.with_foreground(Rgba32::new_rgb(255, 0, 0)),
                )]
            }
            Message::EnemyDies(enemy) => {
                vec![t("The ", plain), enemy_text(*enemy), t(" dies.", plain)]
            }
            Message::PlayerAdrift => {
                vec![t(
                    "You fall into the void!",
                    bold.with_foreground(Rgba32::new_rgb(0, 0, 255)),
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
        Text::from(text).render(&(), ctx.add_y(i as i32), fb);
    }
}

pub fn weapon_name_text(weapon_name: WeaponName) -> StyledString {
    let t = |s: &str, c| StyledString {
        string: s.to_string(),
        style: Style::new().with_foreground(c).with_bold(true),
    };
    match weapon_name {
        WeaponName::BareHands => t("Bare Hands", Rgba32::new_grey(255)),
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

fn enemy_text(enemy: Enemy) -> StyledString {
    let t = |s: &str, c| StyledString {
        string: s.to_string(),
        style: Style::new().with_foreground(c).with_bold(true),
    };
    match enemy {
        Enemy::Zombie => t("Zombie", colours::ZOMBIE.saturating_scalar_mul_div(3, 2)),
        Enemy::Skeleton => t("Skeleton", colours::SKELETON),
        Enemy::Boomer => t("Boomer", colours::BOOMER),
        Enemy::Tank => t("Tank", colours::TANK.saturating_scalar_mul_div(3, 2)),
    }
}

pub fn render_hud(player: &Player, player_info: &CharacterInfo, ctx: Ctx, fb: &mut FrameBuffer) {
    let plain = Style::new().with_foreground(Rgba32::new_grey(255));
    let plain_str = |s: &str| StyledString {
        string: s.to_string(),
        style: plain,
    };
    let text = vec![
        plain_str("Health: "),
        StyledString {
            string: format!(
                "{}/{}",
                player_info.hit_points.current, player_info.hit_points.max
            ),
            style: Style::new()
                .with_foreground(crate::colours::HEALTH)
                .with_bold(true),
        },
        plain_str("\n"),
        plain_str("Oxygen: "),
        StyledString {
            string: format!("{}/{}", player_info.oxygen.current, player_info.oxygen.max),
            style: Style::new()
                .with_foreground(crate::colours::OXYGEN)
                .with_bold(true),
        },
        plain_str("\n"),
        plain_str("Credit: "),
        StyledString {
            string: format!("${}", player.credit),
            style: Style::new()
                .with_foreground(colours::CREDIT_FOREGROUND)
                .with_bold(true),
        },
        plain_str("\n"),
    ];
    Text::from(text).render(&(), ctx, fb);
    render_weapon("Melee:", &player.melee_weapon, &player, ctx.add_y(5), fb);
    let ctx = ctx.add_y(15);
    for (i, ranged_slot) in player.ranged_weapons.iter().enumerate() {
        if let Some(weapon) = ranged_slot {
            render_weapon(
                format!("Ranged {}:", i + 1).as_str(),
                weapon,
                &player,
                ctx.add_y(i as i32 * 10),
                fb,
            );
        } else {
            render_empty_weapon_slot(
                format!("Ranged {}:", i + 1).as_str(),
                ctx.add_y(i as i32 * 10),
                fb,
            );
        }
    }
    render_upgrades(player, ctx.add_y(32), fb);
}

fn weapon_ability_text(weapon_ability: WeaponAbility) -> StyledString {
    match weapon_ability {
        WeaponAbility::KnockBack => StyledString {
            string: "Knocks Back".to_string(),
            style: Style::new().with_foreground(Rgba32::new_rgb(0xFF, 0x44, 0x00)),
        },
        WeaponAbility::LifeSteal => StyledString {
            string: "Restores Health".to_string(),
            style: Style::new().with_foreground(colours::HEALTH),
        },
        WeaponAbility::Oxidise => StyledString {
            string: "Restores Oxygen".to_string(),
            style: Style::new().with_foreground(colours::OXYGEN),
        },
    }
}

fn render_empty_weapon_slot(title: &str, ctx: Ctx, fb: &mut FrameBuffer) {
    let style = Style::new()
        .with_foreground(Rgba32::new_grey(255))
        .with_bold(false);
    StyledString {
        string: title.to_string(),
        style,
    }
    .render(&(), ctx, fb);
    StyledString {
        string: "(empty)".to_string(),
        style,
    }
    .render(&(), ctx.add_y(1), fb);
}

fn render_weapon(title: &str, weapon: &Weapon, player: &Player, ctx: Ctx, fb: &mut FrameBuffer) {
    let plain = Style::new()
        .with_foreground(Rgba32::new_grey(255))
        .with_bold(false);
    StyledString {
        string: title.to_string(),
        style: plain,
    }
    .render(&(), ctx, fb);
    weapon_name_text(weapon.name).render(&(), ctx.add_y(1), fb);
    if let Some(ammo) = weapon.ammo.as_ref() {
        StyledString {
            string: format!("AMMO: {}/{}\n", ammo.current, ammo.max),
            style: plain,
        }
        .render(&(), ctx.add_y(2), fb);
    } else {
        StyledString {
            string: "AMMO: -".to_string(),
            style: plain,
        }
        .render(&(), ctx.add_y(2), fb);
    }
    StyledString {
        string: format!("PEN(♦): {}\n", weapon.pen),
        style: plain,
    }
    .render(&(), ctx.add_y(3), fb);
    let extra = if player.traits.double_damage {
        "x2"
    } else {
        ""
    };
    StyledString {
        string: format!("DMG(♥): {}{}\n", weapon.dmg, extra),
        style: plain,
    }
    .render(&(), ctx.add_y(4), fb);
    let extra = if player.traits.reduce_hull_pen {
        "/2"
    } else {
        ""
    };
    StyledString {
        string: format!("HULL PEN: {}%{}\n", weapon.hull_pen_percent, extra),
        style: plain,
    }
    .render(&(), ctx.add_y(5), fb);
    for &ability in weapon.abilities.iter() {
        weapon_ability_text(ability).render(&(), ctx.add_y(6), fb);
    }
}

fn render_upgrades(player: &Player, ctx: Ctx, fb: &mut FrameBuffer) {
    let plain = Style::new()
        .with_foreground(Rgba32::new_grey(255))
        .with_bold(false);
    StyledString {
        style: plain.with_bold(true),
        string: "Upgrades:".to_string(),
    }
    .render(&(), ctx, fb);
    let mut upgrades = Vec::new();
    use player::UpgradeLevel::*;
    if let Some(level) = player.upgrade_table.toughness {
        upgrades.push("T1: Strong Back");
        if level == Level2 {
            upgrades.push("T2: Hardy");
        }
    }
    if let Some(level) = player.upgrade_table.accuracy {
        upgrades.push("A1: Careful");
        if level == Level2 {
            upgrades.push("A2: Kill Shot");
        }
    }
    if let Some(level) = player.upgrade_table.endurance {
        upgrades.push("E1: Sure-Footed");
        if level == Level2 {
            upgrades.push("E2: Big Lungs");
        }
    }
    if upgrades.is_empty() {
        StyledString {
            style: plain,
            string: "(none)".to_string(),
        }
        .render(&(), ctx.add_y(1), fb);
    } else {
        for (i, upgrade) in upgrades.into_iter().enumerate() {
            StyledString {
                style: plain,
                string: upgrade.to_string(),
            }
            .render(&(), ctx.add_y(i as i32 + 1), fb);
        }
    }
}

use chargrid::render::{ColModify, Coord, Frame, Rgb24, Style, View, ViewContext};
use chargrid::text::{
    wrap, RichStringViewSingleLine, RichTextPart, RichTextPartOwned, RichTextView,
    StringViewSingleLine,
};
use orbital_decay_game::{
    player::{Player, Weapon, WeaponAbility, WeaponName},
    CharacterInfo, HitPoints,
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
                        .with_foreground(Rgb24::new(255, 0, 0))
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
                        .with_foreground(Rgb24::new(127, 127, 255))
                        .with_bold(true),
                ),
                RichTextPart::new("\n\n", plain),
            ],
            context,
            frame,
        );
        view_weapon(
            "Melee Weapon:",
            &ui.player.melee_weapon,
            context.add_offset(Coord { x: 0, y: 4 }),
            frame,
        );
    }
}

fn weapon_name_text(weapon_name: WeaponName) -> RichTextPartOwned {
    let string = match weapon_name {
        WeaponName::BareHands => "Bare Hands".to_string(),
    };
    let style = Style::new().with_foreground(Rgb24::new_grey(255));
    RichTextPartOwned::new(string, style)
}

fn weapon_ability_text(weapon_ability: WeaponAbility) -> RichTextPartOwned {
    match weapon_ability {
        WeaponAbility::KnockBack => RichTextPartOwned::new(
            "Knocks Back".to_string(),
            Style::new().with_foreground(Rgb24::new(0xFF, 0x44, 0x00)),
        ),
    }
}

fn view_weapon<F: Frame, C: ColModify>(
    title: &str,
    weapon: &Weapon,
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
                .with_bold(true),
        ),
        context,
        frame,
    );
    rich_view.view(
        weapon_name_text(weapon.name).as_rich_text_part(),
        context.add_offset(Coord::new(0, 1)),
        frame,
    );
    plain_view.view(
        format!("PEN(♦): {}\n", weapon.pen).as_str(),
        context.add_offset(Coord::new(0, 2)),
        frame,
    );
    plain_view.view(
        format!("DMG(♥): {}\n", weapon.dmg).as_str(),
        context.add_offset(Coord::new(0, 3)),
        frame,
    );
    plain_view.view(
        format!("HULL PEN: {}%\n", weapon.hull_pen_percent).as_str(),
        context.add_offset(Coord::new(0, 4)),
        frame,
    );
    for &ability in weapon.abilities.iter() {
        rich_view.view(
            weapon_ability_text(ability).as_rich_text_part(),
            context.add_offset(Coord::new(0, 5)),
            frame,
        );
    }
}

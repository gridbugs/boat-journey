use gridbugs::chargrid::{prelude::*, text::StyledString};
use orbital_decay_game::{CellVisibility, Game, Tile};

#[derive(Clone, Copy, Debug)]
enum MessageVerb {
    See,
    Remember,
}

pub fn examine(game: &Game, coord: Coord) -> Option<StyledString> {
    let vis_count = game.visibility_grid().count();
    let mut entity_under_cursor = None;
    if let Some(visibility_cell_under_cursor) = game.visibility_grid().get_cell(coord) {
        let verb = match visibility_cell_under_cursor.visibility(vis_count) {
            CellVisibility::CurrentlyVisibleWithLightColour(Some(_)) => Some(MessageVerb::See),
            CellVisibility::PreviouslyVisible => Some(MessageVerb::Remember),
            CellVisibility::NeverVisible
            | CellVisibility::CurrentlyVisibleWithLightColour(None) => None,
        };
        if let Some(verb) = verb {
            if let Some(floor) = visibility_cell_under_cursor.tile_layers().floor {
                entity_under_cursor = Some((floor.tile, verb));
            }
            if let Some(feature) = visibility_cell_under_cursor.tile_layers().feature {
                entity_under_cursor = Some((feature.tile, verb));
            }
            if let Some(character) = visibility_cell_under_cursor.tile_layers().character {
                entity_under_cursor = Some((character.tile, verb));
            }
            if let Some(item) = visibility_cell_under_cursor.tile_layers().item {
                entity_under_cursor = Some((item.tile, verb));
            }
        }
    }
    entity_under_cursor.and_then(|(tile, verb)| {
        tile_str(tile).map(|label| match label {
            TileLabel::Name(name) => {
                let verb_str = match verb {
                    MessageVerb::Remember => "remember seeing",
                    MessageVerb::See => "see",
                };
                StyledString::plain_text(format!("You {} {} here.", verb_str, name))
            }
            TileLabel::Literal(literal) => StyledString::plain_text(literal.to_string()),
        })
    })
}

enum TileLabel {
    Literal(&'static str),
    Name(&'static str),
}

fn tile_str(tile: Tile) -> Option<TileLabel> {
    let label = match tile {
        Tile::Player => TileLabel::Name("yourself"),
        Tile::DoorClosed(_) | Tile::DoorOpen(_) => TileLabel::Name("a door"),
        Tile::Wall | Tile::WallText0 | Tile::WallText1 | Tile::WallText2 | Tile::WallText3 => {
            TileLabel::Name("a wall")
        }
        Tile::Floor | Tile::FuelText0 | Tile::FuelText1 => TileLabel::Name("the floor"),
        Tile::FuelHatch => TileLabel::Name("the fuel bay"),
        Tile::Window(_) => TileLabel::Name("a window"),
        Tile::Stairs => TileLabel::Name("a staircase leading further down"),
        Tile::Zombie => TileLabel::Name("a zombie"),
        Tile::Skeleton => TileLabel::Name("a skeleton"),
        Tile::SkeletonRespawn => TileLabel::Name("a twitching pile of bones"),
        Tile::Boomer => TileLabel::Name("a boomer"),
        Tile::Tank => TileLabel::Name("a tank"),
        Tile::Bullet => return None,
        Tile::Credit1 => TileLabel::Name("a $1 credit chip"),
        Tile::Credit2 => TileLabel::Name("a $2 credit chip"),
        Tile::Upgrade => TileLabel::Name("an upgrade store"),
        Tile::Map => TileLabel::Name("a map terminal"),
        Tile::MapLocked => TileLabel::Name("a locked map terminal"),
        Tile::Medkit => TileLabel::Name("a medkit"),
        Tile::Chainsaw => {
            TileLabel::Literal("A chainsaw - melee weapon with high DMG and limited uses.")
        }
        Tile::Shotgun => TileLabel::Literal("A shotgun - high DMG, low PEN."),
        Tile::Railgun => TileLabel::Literal("A railgun - it can shoot through almost anything!"),
        Tile::Rifle => TileLabel::Literal("A rifle - general all-rounder. Boring."),
        Tile::GausCannon => TileLabel::Literal(
            "A gaus cannon - cooks organic matter leaving the hull intact. Ammo is scarce!",
        ),
        Tile::Oxidiser => {
            TileLabel::Literal("An oxidiser - converts organic matter into oxygen.")
        }
        Tile::LifeStealer => {
            TileLabel::Literal("A life stealer - converts the recently deceased into health like some kind of creepy vampire. And you thought the zombies were gross!")
        }
    };
    Some(label)
}

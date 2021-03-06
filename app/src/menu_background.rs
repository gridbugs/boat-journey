use crate::colours;
use chargrid::render::{grid_2d::Size, ColModify, Coord, Frame, Style, ViewCell, ViewContext};
use orbital_decay_game::{Config, Game, Omniscient, Tile, ToRenderEntity};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

pub struct MenuBackgroundData {
    game: Game,
    star_rng_seed: u64,
}

impl MenuBackgroundData {
    pub fn new() -> Self {
        let config = Config {
            omniscient: Some(Omniscient),
        };
        let mut rng = XorShiftRng::from_entropy();
        let game = Game::new(&config, &mut rng);
        let star_rng_seed = rng.gen();
        Self {
            game,
            star_rng_seed,
        }
    }

    pub fn render<F: Frame, C: ColModify>(&self, context: ViewContext<C>, frame: &mut F) {
        let mut rng = XorShiftRng::seed_from_u64(self.star_rng_seed);
        render_stars(&mut rng, context, frame);
        let context = context.add_offset(Coord { x: 38, y: 5 });
        for entity in self.game.to_render_entities() {
            match entity.tile {
                Tile::Wall
                | Tile::Floor
                | Tile::Window(_)
                | Tile::DoorClosed(_)
                | Tile::DoorOpen(_) => (),
                _ => continue,
            }
            render_entity(&entity, &self.game, context, frame);
        }
    }
}

fn render_entity<F: Frame, C: ColModify>(
    entity: &ToRenderEntity,
    game: &Game,
    context: ViewContext<C>,
    frame: &mut F,
) {
    let depth = crate::render::layer_depth(entity.layer);
    crate::tile_3x3::render_3x3(entity, game, context.add_depth(depth), frame);
}

pub fn render_stars<R: Rng, F: Frame, C: ColModify>(
    star_rng: &mut R,
    context: ViewContext<C>,
    frame: &mut F,
) {
    enum Star {
        None,
        Dim,
        Bright,
    }
    for coord in context.size.coord_iter_row_major() {
        let star = if star_rng.gen::<u32>() % 60 == 0 {
            Star::Bright
        } else if star_rng.gen::<u32>() % 60 == 0 {
            Star::Dim
        } else {
            Star::None
        };
        let (ch, style) = match star {
            Star::None => (' ', Style::new().with_background(colours::SPACE_BACKGROUND)),
            Star::Dim => (
                '.',
                Style::new()
                    .with_bold(false)
                    .with_foreground(colours::SPACE_FOREGROUND_DIM)
                    .with_background(colours::SPACE_BACKGROUND),
            ),
            Star::Bright => (
                '.',
                Style::new()
                    .with_bold(true)
                    .with_foreground(colours::SPACE_FOREGROUND)
                    .with_background(colours::SPACE_BACKGROUND),
            ),
        };
        frame.set_cell_relative(
            coord,
            0,
            ViewCell::new().with_character(ch).with_style(style),
            context,
        );
    }
}

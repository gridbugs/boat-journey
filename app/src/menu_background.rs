use crate::colours;
use chargrid::render::{ColModify, Coord, Frame, Style, ViewCell, ViewContext};
use orbital_decay_game::{Config, Game, Omniscient, ToRenderEntity};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::time::Duration;

pub struct MenuBackgroundData {
    game: Game,
    star_rng_seed: u64,
    duration: Duration,
}

impl MenuBackgroundData {
    pub fn new() -> Self {
        let config = Config {
            omniscient: Some(Omniscient),
            demo: true,
        };
        let mut rng = XorShiftRng::from_entropy();
        let game = Game::new(&config, &mut rng);
        let star_rng_seed = rng.gen();
        Self {
            game,
            star_rng_seed,
            duration: Duration::from_millis(0),
        }
    }

    pub fn render<F: Frame, C: ColModify>(&self, context: ViewContext<C>, frame: &mut F) {
        let mut rng = XorShiftRng::seed_from_u64(self.star_rng_seed);
        render_stars(&mut rng, context, frame);
        let context = context.add_offset(Coord { x: 38, y: 5 });
        for entity in self.game.to_render_entities() {
            render_entity(&entity, &self.game, context, frame);
        }
    }

    pub fn tick(&mut self, since_prev: Duration) {
        const NPC_TURN_PERIOD: Duration = Duration::from_millis(2000);
        self.duration += since_prev;
        if self.duration > NPC_TURN_PERIOD {
            self.duration -= NPC_TURN_PERIOD;
            self.game.handle_npc_turn();
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

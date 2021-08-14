use chargrid::{align::*, control_flow::*, core::*, text::*};
use orbital_decay_game::{Config, Game};
use rand::SeedableRng;
use rand_isaac::Isaac64Rng;

mod colours;
mod game;
mod stars;
mod tile_3x3;

pub fn app() -> impl Component<Output = app::Output, State = ()> {
    let config = Config {
        omniscient: None,
        demo: false,
        debug: true,
    };
    let mut rng = Isaac64Rng::from_entropy();
    let game = Game::new(&config, &mut rng);
    let stars = stars::Stars::new(&mut rng);
    render(move |ctx, fb| {
        stars.render(ctx, fb);
        game::render_game(&game, ctx, fb);
    })
    .press_any_key()
    .map(|()| app::Exit)
}

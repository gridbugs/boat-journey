use chargrid::{control_flow::*, core::*};
use chargrid_ansi_terminal::{col_encode, Context};
use grid_2d::Size;
use procgen::{generate, Spec, Terrain};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use rgb_int::Rgba32;

struct Args {
    size: Size,
    rng: Isaac64Rng,
}

impl Args {
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                rng_seed = opt_opt::<u64, _>("INT", 'r').name("rng-seed").desc("rng seed")
                    .with_default_lazy_general(|| rand::thread_rng().gen());
                width = opt_opt("INT", 'x').name("width").with_default(20);
                height = opt_opt("INT", 'y').name("height").with_default(14);
            } in {{
                eprintln!("RNG Seed: {}", rng_seed);
                let rng = Isaac64Rng::seed_from_u64(rng_seed);
                let size = Size::new(width, height);
                Self {
                    rng,
                    size,
                }
            }}
        }
    }
}

fn app(terrain: Terrain) -> App {
    render(move |ctx, fb| {
        /*
        let mut max_height = 0f64;
        for coord in terrain.land.cells.coord_iter() {
            max_height = max_height.max(terrain.land.get_height(coord).unwrap());
        }
        for (y, row) in terrain.land.cells.rows().enumerate() {
            for (x, _cell) in row.into_iter().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let height = terrain.land.get_height(coord).unwrap();
                let bg = Rgba32::new_grey(((height * 255.) / max_height) as u8);
                let render_cell = RenderCell::default().with_background(bg);
                fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
            }
        }
        for &coord in &terrain.river {
            let bg = Rgba32::new(0, 0, 255, 255);
            let render_cell = RenderCell::default().with_background(bg);
            fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
        }
        for (coord, &cell) in terrain.world1.enumerate() {
            use procgen::WorldCell1;
            let render_cell = match cell {
                WorldCell1::Land => RenderCell::default().with_character('#'),
                WorldCell1::Water => RenderCell::default()
                    .with_character('~')
                    .with_background(Rgba32::new_rgb(0, 255, 255)),
            };
            fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
        }*/
        /*
        for &coord in &terrain.blob.inside {
            let render_cell = RenderCell::default().with_character('#');
            fb.set_cell_relative_to_ctx(ctx, coord, 0, render_cell);
        }*/
        for offset in terrain.viz_size.coord_iter_row_major() {
            use procgen::WorldCell3;
            let coord = terrain.viz_coord + offset;
            if let Some(&cell) = terrain.world3.grid.get(coord) {
                let render_cell = match cell {
                    WorldCell3::Ground => RenderCell::default()
                        .with_character('.')
                        .with_background(Rgba32::new(0, 127, 0, 255)),
                    WorldCell3::Grave => RenderCell::default()
                        .with_character('!')
                        .with_background(Rgba32::new(127, 127, 127, 255)),
                    WorldCell3::TownGround => RenderCell::default()
                        .with_character('.')
                        .with_background(Rgba32::new(87, 127, 0, 255)),
                    WorldCell3::Floor => RenderCell::default()
                        .with_character('.')
                        .with_background(Rgba32::new(127, 127, 127, 255)),
                    WorldCell3::Water(_) => RenderCell::default()
                        .with_character('~')
                        .with_background(Rgba32::new_rgb(0, 0, 255)),
                    WorldCell3::Wall | WorldCell3::Gate => {
                        RenderCell::default().with_character('#')
                    }
                    WorldCell3::Door => RenderCell::default().with_character('+'),
                    WorldCell3::StairsDown => RenderCell::default().with_character('>'),
                    WorldCell3::StairsUp => RenderCell::default().with_character('<'),
                };
                fb.set_cell_relative_to_ctx(ctx, offset, 0, render_cell);
            }
            if coord == terrain.world3.spawn {
                let render_cell = RenderCell::default().with_character('@');
                fb.set_cell_relative_to_ctx(ctx, offset, 0, render_cell);
            }
        }
    })
    .press_any_key()
    .map(|()| app::Exit)
}

fn run(terrain: Terrain) {
    let context = Context::new().unwrap();
    let app = app(terrain);
    context.run(app, col_encode::XtermTrueColour);
}

fn main() {
    use meap::Parser;
    let Args { size, mut rng } = Args::parser().with_help_default().parse_env_or_exit();
    let spec = Spec {
        size,
        num_graves: 2,
    };
    let terrain = generate(&spec, &mut rng);
    run(terrain);
}

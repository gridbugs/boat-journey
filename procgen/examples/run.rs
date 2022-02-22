use grid_2d::Size;
use procgen::{generate, GameCell, Spec};
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;

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
                println!("RNG Seed: {}", rng_seed);
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

fn main() {
    use meap::Parser;
    let Args { size, mut rng } = Args::parser().with_help_default().parse_env_or_exit();
    let spec = Spec { size, small: false };
    let terrain = generate(spec, &mut rng);
    println!("    abcdefghijklmnopqrstuvwxyz\n");
    for (i, row) in terrain.rows().enumerate() {
        print!("{:2}: ", i);
        for (_j, cell) in row.into_iter().enumerate() {
            let ch = match cell {
                GameCell::Floor => '.',
                GameCell::Wall => '#',
                GameCell::Space => ' ',
                GameCell::Door(_) => '+',
                GameCell::Window(_) => '%',
                GameCell::Stairs => '>',
                GameCell::Spawn => '@',
            };
            print!("{}", ch);
        }
        println!("");
    }
}

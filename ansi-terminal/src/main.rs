use boat_journey_app::{app, AppArgs, InitialRngSeed};
use boat_journey_native::NativeCommon;
use chargrid_ansi_terminal::{col_encode, Context};
use rand::Rng;

enum ColEncodeChoice {
    TrueColour,
    Rgb,
    Greyscale,
    Ansi,
}

impl ColEncodeChoice {
    fn parser() -> impl meap::Parser<Item = Self> {
        use ColEncodeChoice::*;
        meap::choose_at_most_one!(
            flag("true-colour").some_if(TrueColour),
            flag("rgb").some_if(Rgb),
            flag("greyscale").some_if(Greyscale),
            flag("ansi").some_if(Ansi),
        )
        .with_default_general(TrueColour)
    }
}

struct Args {
    native_common: NativeCommon,
    col_encode_choice: ColEncodeChoice,
}

impl Args {
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                native_common = NativeCommon::parser();
                col_encode_choice = ColEncodeChoice::parser();
            } in {
                Self { native_common, col_encode_choice }
            }
        }
    }
}

fn main() {
    use meap::Parser;
    let Args {
        native_common:
            NativeCommon {
                storage,
                initial_rng_seed,
                omniscient,
                new_game,
            },
        col_encode_choice,
    } = Args::parser().with_help_default().parse_env_or_exit();
    if let ColEncodeChoice::TrueColour = col_encode_choice {
        println!("Running in true-colour mode.\nIf colours look wrong, run with `--rgb` or try a different terminal emulator.");
    }
    let initial_rng_seed = match initial_rng_seed {
        InitialRngSeed::U64(seed) => seed,
        InitialRngSeed::Random => rand::thread_rng().gen(),
    };
    println!("Initial RNG Seed: {}", initial_rng_seed);
    let context = Context::new().unwrap();
    let app = app(AppArgs {
        storage,
        initial_rng_seed: InitialRngSeed::U64(initial_rng_seed),
        omniscient,
        new_game,
    });
    use ColEncodeChoice as C;
    match col_encode_choice {
        C::TrueColour => context.run(app, col_encode::XtermTrueColour),
        C::Rgb => context.run(app, col_encode::FromTermInfoRgb),
        C::Greyscale => context.run(app, col_encode::FromTermInfoGreyscale),
        C::Ansi => context.run(app, col_encode::FromTermInfoAnsi16Colour),
    }
}

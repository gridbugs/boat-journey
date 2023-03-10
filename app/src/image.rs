use boat_journey_game::{MenuImage, Npc};
use gridbugs::{chargrid::prelude::*, grid_2d::Grid};

pub struct Image {
    pub grid: Grid<RenderCell>,
}

impl Image {
    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        for (coord, &cell) in self.grid.enumerate() {
            fb.set_cell_relative_to_ctx(ctx, coord, 0, cell);
        }
    }
}

#[derive(Clone, Copy)]
enum ImageName {
    Townsfolk1,
    Grave,
    Ocean,
    Boat,
    Soldier,
    Physicist,
}

impl ImageName {
    const fn data(self) -> &'static [u8] {
        match self {
            Self::Townsfolk1 => include_bytes!("images/townsfolk1.bin"),
            Self::Grave => include_bytes!("images/grave.bin"),
            Self::Ocean => include_bytes!("images/ocean.bin"),
            Self::Boat => include_bytes!("images/boat.bin"),
            Self::Soldier => include_bytes!("images/soldier.bin"),
            Self::Physicist => include_bytes!("images/physicist.bin"),
        }
    }

    fn load_grid(self) -> Image {
        let grid = bincode::deserialize::<Grid<RenderCell>>(self.data()).unwrap();
        Image { grid }
    }
}

pub struct Images {
    pub townsfolk1: Image,
    pub grave: Image,
    pub ocean: Image,
    pub boat: Image,
    pub soldier: Image,
    pub physicist: Image,
}

impl Images {
    pub fn new() -> Self {
        Self {
            townsfolk1: ImageName::Townsfolk1.load_grid(),
            grave: ImageName::Grave.load_grid(),
            ocean: ImageName::Ocean.load_grid(),
            boat: ImageName::Boat.load_grid(),
            soldier: ImageName::Soldier.load_grid(),
            physicist: ImageName::Physicist.load_grid(),
        }
    }

    pub fn image_from_menu_image(&self, menu_image: MenuImage) -> &Image {
        match menu_image {
            MenuImage::Townsperson => &self.townsfolk1,
            MenuImage::Grave => &self.grave,
            MenuImage::Npc(Npc::Soldier) => &self.soldier,
            MenuImage::Npc(Npc::Physicist) => &self.physicist,
        }
    }
}

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
}

impl ImageName {
    const fn data(self) -> &'static [u8] {
        match self {
            Self::Townsfolk1 => include_bytes!("images/townsfolk1.bin"),
        }
    }

    fn load_grid(self) -> Image {
        let grid = bincode::deserialize::<Grid<RenderCell>>(self.data()).unwrap();
        Image { grid }
    }
}

pub struct Images {
    pub townsfolk1: Image,
}

impl Images {
    pub fn new() -> Self {
        Self {
            townsfolk1: ImageName::Townsfolk1.load_grid(),
        }
    }
}

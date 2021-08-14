use crate::tile_3x3;
use chargrid::core::rgb_int::{rgb24, Rgb24};
use chargrid::prelude::*;
use orbital_decay_game::{CellVisibility, Game, WarningLight};

#[derive(Clone, Copy)]
struct Remembered;
impl Tint for Remembered {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        let mean = rgba32
            .to_rgb24()
            .weighted_mean_u16(rgb24::WeightsU16::new(1, 1, 1));
        Rgb24::new_grey(mean)
            .saturating_scalar_mul_div(1, 2)
            .to_rgba32(255)
    }
}

#[derive(Clone, Copy)]
struct LightBlend {
    light_colour: Rgb24,
}

impl Tint for LightBlend {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        rgba32
            .to_rgb24()
            .normalised_mul(self.light_colour)
            .saturating_add(self.light_colour.saturating_scalar_mul_div(1, 10))
            .to_rgba32(255)
    }
}

pub fn render_game(game: &Game, ctx: Ctx, fb: &mut FrameBuffer) {
    let vis_count = game.visibility_grid().count();
    for (coord, visibility_cell) in game.visibility_grid().enumerate() {
        match visibility_cell.visibility(vis_count) {
            CellVisibility::CurrentlyVisibleWithLightColour(Some(light_colour)) => {
                let light_colour = match game.warning_light(coord) {
                    Some(WarningLight::NoAir) => {
                        Rgb24::new(127, 127, 255).normalised_mul(light_colour)
                    }
                    Some(WarningLight::Decompression) => {
                        Rgb24::new(255, 127, 127).normalised_mul(light_colour)
                    }
                    None => light_colour,
                };
                let tint = |c| ctx.tint.tint(LightBlend { light_colour }.tint(c));
                let ctx = Ctx { tint: &tint, ..ctx };
                tile_3x3::render_3x3_from_visibility(coord, visibility_cell, game, ctx, fb);
            }
            CellVisibility::PreviouslyVisible => {
                let tint = |c| ctx.tint.tint(Remembered.tint(c));
                let ctx = Ctx { tint: &tint, ..ctx };
                tile_3x3::render_3x3_from_visibility_remembered(
                    coord,
                    visibility_cell,
                    game,
                    ctx,
                    fb,
                );
            }
            CellVisibility::NeverVisible
            | CellVisibility::CurrentlyVisibleWithLightColour(None) => (),
        }
    }
}

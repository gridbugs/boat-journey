use crate::colours;
use gridbugs::chargrid::prelude::*;
use template2023_game::{CellVisibility, VisibilityGrid};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Stars {
    rng: XorShiftRng,
}

impl Stars {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self {
            rng: XorShiftRng::seed_from_u64(rng.gen()),
        }
    }

    pub fn render(&self, ctx: Ctx, fb: &mut FrameBuffer) {
        render_stars(&mut self.rng.clone(), ctx, fb);
    }

    pub fn render_with_visibility(
        &self,
        visibility_grid: &VisibilityGrid,
        ctx: Ctx,
        fb: &mut FrameBuffer,
    ) {
        render_stars_with_visibility(visibility_grid, &mut self.rng.clone(), ctx, fb);
    }
}

fn render_stars<R: Rng>(star_rng: &mut R, ctx: Ctx, fb: &mut FrameBuffer) {
    enum Star {
        None,
        Dim,
        Bright,
    }
    for coord in ctx.bounding_box.size().coord_iter_row_major() {
        let star = if star_rng.gen::<u32>() % 60 == 0 {
            Star::Bright
        } else if star_rng.gen::<u32>() % 60 == 0 {
            Star::Dim
        } else {
            Star::None
        };
        let bg = colours::SPACE_BACKGROUND.saturating_scalar_mul_div(30 + coord.y as u32, 90);
        let (ch, style) = match star {
            Star::None => (' ', Style::new().with_background(bg)),
            Star::Dim => (
                '.',
                Style::new()
                    .with_bold(false)
                    .with_foreground(colours::SPACE_FOREGROUND_DIM)
                    .with_background(bg),
            ),
            Star::Bright => (
                '.',
                Style::new()
                    .with_bold(true)
                    .with_foreground(colours::SPACE_FOREGROUND)
                    .with_background(bg),
            ),
        };
        fb.set_cell_relative_to_ctx(
            ctx,
            coord,
            0,
            RenderCell::default().with_character(ch).with_style(style),
        );
    }
}

fn render_stars_with_visibility<R: Rng>(
    visibility_grid: &VisibilityGrid,
    star_rng: &mut R,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    enum Star {
        None,
        Dim,
        Bright,
    }
    for coord in ctx.bounding_box.size().coord_iter_row_major() {
        let visibility = visibility_grid.cell_visibility(coord / 3);
        let star = if star_rng.gen::<u32>() % 60 == 0 {
            Star::Bright
        } else if star_rng.gen::<u32>() % 60 == 0 {
            Star::Dim
        } else {
            Star::None
        };
        let bg = colours::SPACE_BACKGROUND.saturating_scalar_mul_div(30 + coord.y as u32, 90);
        match visibility {
            CellVisibility::NeverVisible => {
                fb.set_cell_relative_to_ctx(
                    ctx,
                    coord,
                    0,
                    RenderCell::default()
                        .with_character(' ')
                        .with_background(Rgba32::new_grey(0)),
                );
            }
            CellVisibility::PreviouslyVisible => {
                let num = 1;
                let denom = 4;
                let (ch, style) = match star {
                    Star::None => (
                        ' ',
                        Style::new()
                            .with_foreground(
                                colours::SPACE_FOREGROUND_DIM.saturating_scalar_mul_div(num, denom),
                            )
                            .with_background(
                                colours::SPACE_BACKGROUND.saturating_scalar_mul_div(num, denom),
                            ),
                    ),
                    Star::Dim => (
                        '.',
                        Style::new()
                            .with_foreground(
                                colours::SPACE_FOREGROUND_DIM.saturating_scalar_mul_div(num, denom),
                            )
                            .with_background(
                                colours::SPACE_BACKGROUND.saturating_scalar_mul_div(num, denom),
                            ),
                    ),
                    Star::Bright => (
                        '.',
                        Style::new()
                            .with_bold(true)
                            .with_foreground(
                                colours::SPACE_FOREGROUND_DIM.saturating_scalar_mul_div(num, denom),
                            )
                            .with_background(
                                colours::SPACE_BACKGROUND.saturating_scalar_mul_div(num, denom),
                            ),
                    ),
                };
                fb.set_cell_relative_to_ctx(
                    ctx,
                    coord,
                    0,
                    RenderCell::default().with_character(ch).with_style(style),
                );
            }
            CellVisibility::CurrentlyVisibleWithLightColour(_) => {
                let (ch, style) = match star {
                    Star::None => (' ', Style::new().with_background(bg)),
                    Star::Dim => (
                        '.',
                        Style::new()
                            .with_bold(false)
                            .with_foreground(colours::SPACE_FOREGROUND_DIM)
                            .with_background(bg),
                    ),
                    Star::Bright => (
                        '.',
                        Style::new()
                            .with_bold(true)
                            .with_foreground(colours::SPACE_FOREGROUND)
                            .with_background(bg),
                    ),
                };
                fb.set_cell_relative_to_ctx(
                    ctx,
                    coord,
                    0,
                    RenderCell::default().with_character(ch).with_style(style),
                );
            }
        }
    }
}

use chargrid::render::{ColModify, Coord, Frame, Rgb24, Style, View, ViewContext};
use chargrid::text::StringViewSingleLine;
use orbital_decay_game::player::Player;

pub struct Ui<'a> {
    pub player: &'a Player,
}

pub struct UiView;

impl UiView {
    pub fn view<F: Frame, C: ColModify>(&mut self, ui: Ui, context: ViewContext<C>, frame: &mut F) {
    }
}

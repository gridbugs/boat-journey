use gridbugs::{
    chargrid::input::{Input, KeyboardInput},
    direction::CardinalDirection,
};
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppInput {
    Direction(CardinalDirection),
    Wait,
    DriveToggle,
    Ability(u8),
}

#[derive(Serialize, Deserialize)]
pub struct Controls {
    keys: BTreeMap<KeyboardInput, AppInput>,
}

impl Default for Controls {
    fn default() -> Self {
        let keys = btreemap![
            KeyboardInput::Left => AppInput::Direction(CardinalDirection::West),
            KeyboardInput::Right => AppInput::Direction(CardinalDirection::East),
            KeyboardInput::Up => AppInput::Direction(CardinalDirection::North),
            KeyboardInput::Down => AppInput::Direction(CardinalDirection::South),
            KeyboardInput::Char(' ') => AppInput::Wait,
            KeyboardInput::Char('e') => AppInput::DriveToggle,
            KeyboardInput::Char('1') => AppInput::Ability(1),
            KeyboardInput::Char('2') => AppInput::Ability(2),
            KeyboardInput::Char('3') => AppInput::Ability(3),
            KeyboardInput::Char('4') => AppInput::Ability(4),
            KeyboardInput::Char('5') => AppInput::Ability(5),
            KeyboardInput::Char('6') => AppInput::Ability(6),
            KeyboardInput::Char('7') => AppInput::Ability(7),
            KeyboardInput::Char('8') => AppInput::Ability(8),
            KeyboardInput::Char('9') => AppInput::Ability(9),
        ];
        Self { keys }
    }
}
impl Controls {
    pub fn get(&self, input: Input) -> Option<AppInput> {
        match input {
            Input::Keyboard(keyboard_input) => self.keys.get(&keyboard_input).cloned(),
            Input::Gamepad(_) | Input::Mouse(_) => None,
        }
    }
}

use gridbugs::{
    chargrid::input::{GamepadButton, Input, KeyboardInput},
    direction::CardinalDirection,
};
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppInput {
    Direction(CardinalDirection),
    Wait,
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

    pub fn get_direction(&self, input: Input) -> Option<CardinalDirection> {
        self.get(input).and_then(|app_input| match app_input {
            AppInput::Direction(direction) => Some(direction),
            _ => None,
        })
    }
}

use crate::visibility::Light;
pub use crate::world::{
    explosion_spec,
    player::Player,
    spatial::{Layer, Location},
};
use direction::CardinalDirection;
use entity_table::declare_entity_module;
use grid_2d::coord_2d::Axis;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};

declare_entity_module! {
    components {
        tile: Tile,
        opacity: u8,
        solid: (),
        realtime: (),
        blocks_gameplay: (),
        light: Light,
        on_collision: OnCollision,
        colour_hint: Rgb24,
        npc: Npc,
        character: (),
        collides_with: CollidesWith,
        projectile_damage: ProjectileDamage,
        hit_points: HitPoints,
        oxygen: Oxygen,
        armour: Armour,
        blood: (),
        player: Player,
        ignore_lighting: (),
        door_state: DoorState,
        door_close_countdown: u32,
        stairs: (),
        next_action: NpcAction,
        to_remove: (),
        move_half_speed: MoveHalfSpeed,
        item: Item,
        damage: u32,
        particle: (),
    }
}
pub use components::Components;
pub use components::EntityData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    DoorClosed(Axis),
    DoorOpen(Axis),
    Stairs,
    Window(Axis),
    Zombie,
    WallText0,
    WallText1,
    WallText2,
    WallText3,
    FuelText0,
    FuelText1,
    FuelHatch,
    Bullet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Item {
    Attack { special: bool },
    Defend { special: bool },
    Tech { special: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,
    Afraid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Npc {
    pub disposition: Disposition,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OnCollision {
    Explode(explosion_spec::Explosion),
    Remove,
    RemoveRealtime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CollidesWith {
    pub solid: bool,
    pub character: bool,
}

impl Default for CollidesWith {
    fn default() -> Self {
        Self {
            solid: true,
            character: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectileDamage {
    pub hit_points: u32,
    pub push_back: bool,
    pub pen: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HitPoints {
    pub current: u32,
    pub max: u32,
}

impl HitPoints {
    pub fn new_full(max: u32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Oxygen {
    pub current: u32,
    pub max: u32,
}

impl Oxygen {
    pub fn new_full(max: u32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Armour {
    pub value: u32,
}

impl Armour {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcAction {
    Walk(CardinalDirection),
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MoveHalfSpeed {
    pub skip_next_move: bool,
}

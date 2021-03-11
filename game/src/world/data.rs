use crate::visibility::Light;
pub use crate::world::{
    explosion_spec,
    player::{self, Player},
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
        destructible: (),
        upgrade: (),
        weapon: player::Weapon,
        push_back: (),
        expoodes_on_death: (),
        skeleton: (),
        skeleton_respawn: u32,
        enemy: Enemy,
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
    Tank,
    Boomer,
    Skeleton,
    SkeletonRespawn,
    WallText0,
    WallText1,
    WallText2,
    WallText3,
    FuelText0,
    FuelText1,
    FuelHatch,
    Bullet,
    Credit1,
    Credit2,
    Upgrade,
    Chainsaw,
    Shotgun,
    Railgun,
    Rifle,
    GausCannon,
    Oxidiser,
    LifeStealer,
    Medkit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enemy {
    Zombie,
    Skeleton,
    Boomer,
    Tank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangedWeapon {
    Shotgun,
    Railgun,
    Rifle,
    GausCannon,
    Oxidiser,
    LifeStealer,
}

impl RangedWeapon {
    pub fn tile(self) -> Tile {
        use RangedWeapon::*;
        match self {
            Shotgun => Tile::Shotgun,
            Railgun => Tile::Railgun,
            Rifle => Tile::Rifle,
            GausCannon => Tile::GausCannon,
            Oxidiser => Tile::Oxidiser,
            LifeStealer => Tile::LifeStealer,
        }
    }

    pub fn new_weapon(self) -> player::Weapon {
        use player::Weapon;
        use RangedWeapon::*;
        match self {
            Shotgun => Weapon::new_shotgun(),
            Railgun => Weapon::new_railgun(),
            Rifle => Weapon::new_rifle(),
            GausCannon => Weapon::new_gaus_cannon(),
            Oxidiser => Weapon::new_oxidiser(),
            LifeStealer => Weapon::new_life_stealer(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeleeWeapon {
    Chainsaw,
}

impl MeleeWeapon {
    pub fn tile(self) -> Tile {
        use MeleeWeapon::*;
        match self {
            Chainsaw => Tile::Chainsaw,
        }
    }

    pub fn new_weapon(self) -> player::Weapon {
        use player::Weapon;
        use MeleeWeapon::*;
        match self {
            Chainsaw => Weapon::new_chainsaw(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Item {
    Credit(u32),
    RangedWeapon(RangedWeapon),
    MeleeWeapon(MeleeWeapon),
    Medkit,
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
    pub hull_pen_percent: u32,
    pub life_steal: bool,
    pub oxidise: bool,
    pub weapon_name: Option<player::WeaponName>,
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

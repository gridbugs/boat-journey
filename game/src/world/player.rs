use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub melee_weapon: Weapon,
}

impl Player {
    pub fn melee_dmg(&self) -> u32 {
        self.melee_weapon.dmg
    }

    pub fn melee_pen(&self) -> u32 {
        self.melee_weapon.pen
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponName {
    BareHands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponAbility {
    KnockBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub name: WeaponName,
    pub pen: u32,
    pub dmg: u32,
    pub hull_pen_percent: u32,
    pub abilities: Vec<WeaponAbility>,
}

impl Weapon {
    pub fn new_bare_hands() -> Self {
        Self {
            name: WeaponName::BareHands,
            pen: 2,
            dmg: 2,
            hull_pen_percent: 0,
            abilities: vec![WeaponAbility::KnockBack],
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RangedWeaponSlot {
    Slot1,
    Slot2,
}

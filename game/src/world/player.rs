use crate::world::{MeleeWeapon, RangedWeapon};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub melee_weapon: Weapon,
    pub credit: u32,
    pub ranged_weapons: Vec<Option<Weapon>>,
    pub upgrade_table: UpgradeTable,
    pub traits: PlayerTraits,
}

impl Player {
    pub fn new() -> Self {
        Self {
            melee_weapon: Weapon::new_bare_hands(),
            credit: 0,
            ranged_weapons: vec![None, None],
            upgrade_table: UpgradeTable {
                toughness: None,
                accuracy: None,
                endurance: None,
            },
            traits: Default::default(),
        }
    }

    pub fn melee_dmg(&self) -> u32 {
        self.melee_weapon.dmg
    }

    pub fn melee_pen(&self) -> u32 {
        self.melee_weapon.pen
    }

    pub fn available_upgrades(&self) -> Vec<Upgrade> {
        let mut out = Vec::new();
        match self.upgrade_table.toughness {
            None => out.push(Upgrade {
                typ: UpgradeType::Toughness,
                level: UpgradeLevel::Level1,
            }),
            Some(UpgradeLevel::Level1) => out.push(Upgrade {
                typ: UpgradeType::Toughness,
                level: UpgradeLevel::Level2,
            }),
            Some(UpgradeLevel::Level2) => (),
        }
        match self.upgrade_table.accuracy {
            None => out.push(Upgrade {
                typ: UpgradeType::Accuracy,
                level: UpgradeLevel::Level1,
            }),
            Some(UpgradeLevel::Level1) => out.push(Upgrade {
                typ: UpgradeType::Accuracy,
                level: UpgradeLevel::Level2,
            }),
            Some(UpgradeLevel::Level2) => (),
        }
        match self.upgrade_table.endurance {
            None => out.push(Upgrade {
                typ: UpgradeType::Endurance,
                level: UpgradeLevel::Level1,
            }),
            Some(UpgradeLevel::Level1) => out.push(Upgrade {
                typ: UpgradeType::Endurance,
                level: UpgradeLevel::Level2,
            }),
            Some(UpgradeLevel::Level2) => (),
        }
        out
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTraits {
    pub reduce_hull_pen: bool,
    pub double_damage: bool,
    pub half_vacuum_pull: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponName {
    BareHands,
    RangedWeapon(RangedWeapon),
    MeleeWeapon(MeleeWeapon),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponAbility {
    KnockBack,
    LifeSteal,
    Oxidise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ammo {
    pub current: u32,
    pub max: u32,
}

impl Ammo {
    pub fn new_full(max: u32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub name: WeaponName,
    pub ammo: Option<Ammo>,
    pub pen: u32,
    pub dmg: u32,
    pub hull_pen_percent: u32,
    pub abilities: Vec<WeaponAbility>,
}

impl Weapon {
    pub fn new_bare_hands() -> Self {
        Self {
            name: WeaponName::BareHands,
            ammo: None,
            pen: 2,
            dmg: 2,
            hull_pen_percent: 0,
            abilities: vec![WeaponAbility::KnockBack],
        }
    }
    pub fn new_chainsaw() -> Self {
        Self {
            name: WeaponName::MeleeWeapon(MeleeWeapon::Chainsaw),
            ammo: Some(Ammo::new_full(6)),
            pen: 10,
            dmg: 5,
            hull_pen_percent: 0,
            abilities: vec![],
        }
    }
    pub fn new_shotgun() -> Self {
        Self {
            name: WeaponName::RangedWeapon(RangedWeapon::Shotgun),
            ammo: Some(Ammo::new_full(8)),
            pen: 3,
            dmg: 5,
            hull_pen_percent: 30,
            abilities: vec![WeaponAbility::KnockBack],
        }
    }
    pub fn new_railgun() -> Self {
        Self {
            name: WeaponName::RangedWeapon(RangedWeapon::Railgun),
            ammo: Some(Ammo::new_full(8)),
            pen: 100,
            dmg: 5,
            hull_pen_percent: 50,
            abilities: vec![],
        }
    }
    pub fn new_rifle() -> Self {
        Self {
            name: WeaponName::RangedWeapon(RangedWeapon::Rifle),
            ammo: Some(Ammo::new_full(8)),
            pen: 8,
            dmg: 2,
            hull_pen_percent: 20,
            abilities: vec![],
        }
    }
    pub fn new_gaus_cannon() -> Self {
        Self {
            name: WeaponName::RangedWeapon(RangedWeapon::GausCannon),
            ammo: Some(Ammo::new_full(1)),
            pen: 20,
            dmg: 10,
            hull_pen_percent: 0,
            abilities: vec![],
        }
    }
    pub fn new_oxidiser() -> Self {
        Self {
            name: WeaponName::RangedWeapon(RangedWeapon::Oxidiser),
            ammo: Some(Ammo::new_full(4)),
            pen: 3,
            dmg: 1,
            hull_pen_percent: 0,
            abilities: vec![WeaponAbility::Oxidise],
        }
    }
    pub fn new_life_stealer() -> Self {
        Self {
            name: WeaponName::RangedWeapon(RangedWeapon::LifeStealer),
            ammo: Some(Ammo::new_full(4)),
            pen: 3,
            dmg: 1,
            hull_pen_percent: 0,
            abilities: vec![WeaponAbility::LifeSteal],
        }
    }
    pub fn is_ranged(&self) -> bool {
        match self.name {
            WeaponName::RangedWeapon(_) => true,
            _ => false,
        }
    }
    pub fn is_melee(&self) -> bool {
        match self.name {
            WeaponName::MeleeWeapon(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RangedWeaponSlot {
    Slot1,
    Slot2,
    Slot3,
}

impl RangedWeaponSlot {
    pub fn index(self) -> usize {
        match self {
            Self::Slot1 => 0,
            Self::Slot2 => 1,
            Self::Slot3 => 2,
        }
    }
}

// Toughness:
//   1. Extra weapon slot
//   2. Double HP
//
// Accuracy:
//   1. Reduce hull pen chance to half
//   2. Deal double damage
//
// Endurance:
//   1. Half effect of vacumm pull
//   2. Double oxygen
//

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeTable {
    pub toughness: Option<UpgradeLevel>,
    pub accuracy: Option<UpgradeLevel>,
    pub endurance: Option<UpgradeLevel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeLevel {
    Level1,
    Level2,
}

impl UpgradeLevel {
    pub fn cost(self) -> u32 {
        match self {
            Self::Level1 => 5,
            Self::Level2 => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeType {
    Toughness,
    Accuracy,
    Endurance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Upgrade {
    pub typ: UpgradeType,
    pub level: UpgradeLevel,
}

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
    Slot3,
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

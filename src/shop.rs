// Shop item catalog and generation.
//
// A shop is generated after every successful Challenge or Elite room.
// Items are drawn from the relic pool (filtered to exclude owned relics),
// plus a random special die and an HP potion (omitted if HP is full).
//
// Prices are computed at generation time and reflect any shop_price_multiplier
// bonuses the player has from relics (e.g. Cursed Chalice: 0.8x).

use rand::{Rng, seq::SliceRandom};

use crate::{
    game::{GameState, UpgradeKind},
    relics::{self, Relic},
};

// Base prices before relic multipliers.
const RELIC_BASE_PRICE: u32 = 75;
const SPECIAL_DIE_BASE_PRICE: u32 = 50;
const HP_POTION_BASE_PRICE: u32 = 40;
const HP_POTION_HEAL: u32 = 15;
const DIE_UPGRADE_BASE_PRICE: u32 = 60;

// ─── SpecialDieKind ───────────────────────────────────────────────────────────

pub enum SpecialDieKind {
    Wild,
    Cursed,
    Bones,
}

impl SpecialDieKind {
    pub fn name(&self) -> &str {
        match self {
            SpecialDieKind::Wild => "Wild Die",
            SpecialDieKind::Cursed => "Cursed Die",
            SpecialDieKind::Bones => "Dragon Bones",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            SpecialDieKind::Wild => "One face is WILD: scores as any value",
            SpecialDieKind::Cursed => "Three faces show 1; powerful if held carefully",
            SpecialDieKind::Bones => "D8 die: faces 1-8, values >6 cap at 6 for combos",
        }
    }

    pub fn create_die(&self) -> crate::dice::Die {
        match self {
            SpecialDieKind::Wild => crate::dice::Die::wild(),
            SpecialDieKind::Cursed => crate::dice::Die::cursed(),
            SpecialDieKind::Bones => crate::dice::Die::bones(),
        }
    }
}

// ─── ShopItem ─────────────────────────────────────────────────────────────────

pub struct ShopItem {
    pub kind: ShopItemKind,
    pub price: u32,
}

pub enum ShopItemKind {
    Relic(Box<dyn Relic>),
    SpecialDie(SpecialDieKind),
    HpPotion(u32), // heal amount
    DieUpgrade(UpgradeKind),
}

impl ShopItem {
    pub fn name(&self) -> &str {
        match &self.kind {
            ShopItemKind::Relic(r) => r.name(),
            ShopItemKind::SpecialDie(k) => k.name(),
            ShopItemKind::HpPotion(_) => "HP Potion",
            ShopItemKind::DieUpgrade(UpgradeKind::Augment) => "Augment",
            ShopItemKind::DieUpgrade(UpgradeKind::Enchant) => "Enchant",
        }
    }

    pub fn description(&self) -> String {
        match &self.kind {
            ShopItemKind::Relic(r) => r.description().to_string(),
            ShopItemKind::SpecialDie(k) => k.description().to_string(),
            ShopItemKind::HpPotion(amt) => format!("Restore {} HP", amt),
            ShopItemKind::DieUpgrade(UpgradeKind::Augment) => {
                "Pick a die face: raise its value by 1".to_string()
            }
            ShopItemKind::DieUpgrade(UpgradeKind::Enchant) => {
                "Pick a die face: add +5 bonus score on roll".to_string()
            }
        }
    }
}

// ─── Generation ───────────────────────────────────────────────────────────────

pub fn generate_shop_items(state: &GameState, rng: &mut impl Rng) -> Vec<ShopItem> {
    let multiplier: f32 = state
        .relics
        .iter()
        .map(|r| r.shop_price_multiplier())
        .fold(1.0, |acc, m| acc * m);

    let price = |base: u32| -> u32 { ((base as f32) * multiplier).round() as u32 };

    let owned_names: Vec<&str> = state.relics.iter().map(|r| r.name()).collect();
    let mut candidates: Vec<Box<dyn Relic>> = relics::all_relics()
        .into_iter()
        .filter(|r| !owned_names.contains(&r.name()))
        .collect();
    candidates.shuffle(rng);

    let mut items: Vec<ShopItem> = candidates
        .into_iter()
        .take(2)
        .map(|r| ShopItem { kind: ShopItemKind::Relic(r), price: price(RELIC_BASE_PRICE) })
        .collect();

    // Offer a random special die only if a standard die exists to replace.
    if state.dice_pool.has_standard_die() {
        let kind = match rng.random_range(0..3u8) {
            0 => SpecialDieKind::Wild,
            1 => SpecialDieKind::Cursed,
            _ => SpecialDieKind::Bones,
        };
        items.push(ShopItem { kind: ShopItemKind::SpecialDie(kind), price: price(SPECIAL_DIE_BASE_PRICE) });
    }

    // HP potion only when player is missing HP.
    if state.hp < state.max_hp {
        items.push(ShopItem { kind: ShopItemKind::HpPotion(HP_POTION_HEAL), price: price(HP_POTION_BASE_PRICE) });
    }

    // One die upgrade: randomly Augment or Enchant.
    let upgrade_kind = if rng.random_range(0..2u8) == 0 {
        UpgradeKind::Augment
    } else {
        UpgradeKind::Enchant
    };
    items.push(ShopItem { kind: ShopItemKind::DieUpgrade(upgrade_kind), price: price(DIE_UPGRADE_BASE_PRICE) });

    items
}

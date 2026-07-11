// Shop item catalog and generation.
//
// A shop is generated after every successful Challenge or Elite room.
// Items are drawn from the relic pool (filtered to exclude owned relics),
// plus a random special die and an HP potion (omitted if HP is full).
//
// Prices are computed at generation time and reflect any shop_price_multiplier
// bonuses the player has from relics (e.g. Cursed Chalice: 0.8x).

use rand::Rng;

use crate::{
    dice::Die,
    game::GameState,
    relics::{self, Relic},
};

// Base prices before relic multipliers.
const RELIC_BASE_PRICE: u32 = 75;
const SPECIAL_DIE_BASE_PRICE: u32 = 50;
const HP_POTION_BASE_PRICE: u32 = 40;
const HP_POTION_HEAL: u32 = 15;

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

    pub fn create_die(&self) -> Die {
        match self {
            SpecialDieKind::Wild => Die::wild(),
            SpecialDieKind::Cursed => Die::cursed(),
            SpecialDieKind::Bones => Die::bones(),
        }
    }
}

// ─── ShopItem ─────────────────────────────────────────────────────────────────

pub enum ShopItem {
    Relic(Box<dyn Relic>, u32),
    SpecialDie(SpecialDieKind, u32),
    HpPotion(u32, u32), // (heal_amount, price)
}

impl ShopItem {
    pub fn name(&self) -> &str {
        match self {
            ShopItem::Relic(r, _) => r.name(),
            ShopItem::SpecialDie(k, _) => k.name(),
            ShopItem::HpPotion(_, _) => "HP Potion",
        }
    }

    pub fn description(&self) -> String {
        match self {
            ShopItem::Relic(r, _) => r.description().to_string(),
            ShopItem::SpecialDie(k, _) => k.description().to_string(),
            ShopItem::HpPotion(amt, _) => format!("Restore {} HP", amt),
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            ShopItem::Relic(_, p) => *p,
            ShopItem::SpecialDie(_, p) => *p,
            ShopItem::HpPotion(_, p) => *p,
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

    let mut items: Vec<ShopItem> = Vec::new();

    // Pick up to 2 random relics.
    for _ in 0..2 {
        if candidates.is_empty() {
            break;
        }
        let idx = rng.random_range(0..candidates.len());
        let relic = candidates.remove(idx);
        items.push(ShopItem::Relic(relic, price(RELIC_BASE_PRICE)));
    }

    // Offer a random special die only if a standard die exists to replace.
    let has_standard_die = state
        .dice_pool
        .dice
        .iter()
        .any(|d| d.label() == "D6");
    if has_standard_die {
        let kind = match rng.random_range(0..3u8) {
            0 => SpecialDieKind::Wild,
            1 => SpecialDieKind::Cursed,
            _ => SpecialDieKind::Bones,
        };
        items.push(ShopItem::SpecialDie(kind, price(SPECIAL_DIE_BASE_PRICE)));
    }

    // HP potion only when player is missing HP.
    if state.hp < state.max_hp {
        items.push(ShopItem::HpPotion(HP_POTION_HEAL, price(HP_POTION_BASE_PRICE)));
    }

    items
}

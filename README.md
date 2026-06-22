# DUNGEON DICE

*A Yahtzee-based roguelite for the terminal*

Roll dice. Forge your set. Descend the dungeon.

**Dungeon Dice** is a roguelite inspired by Balatro and Slay the Spire, played entirely in the terminal. Instead of poker hands or playing cards, your weapon is a set of Yahtzee dice. Each floor of the dungeon is a series of score-based challenges: roll five dice, hold what you want, and re-roll up to twice more. Hit the target score to earn gold and press on; fail and you lose HP. Die and the run ends.

The twist: you start knowing only the **Chance** scoring category (sum all dice, no bonus). Defeating floor bosses unlocks new scoring categories (Full House, Straights, Yahtzee), dramatically expanding what combos are worth. Collect relics that bend the rules, and upgrade your actual dice (swap faces, add enchantments, replace standard d6s with special dice) to craft a set that builds toward devastating combos. How far into the dungeon can you descend?

---

## Core Loop

```
Enter Room → Roll Dice (up to 3 times, hold any dice) → Score with a category
→ Hit target: earn gold → Miss target: lose 10 HP → HP = 0: Run over
```

Each run starts with:
- **5 standard d6**, **3 rolls** per room
- **30 HP**
- **0 gold**
- Only the **Chance** category unlocked

---

## Floor Structure

Each floor has **3 rooms** followed by a **boss room**:

| Room Type       | Description                                                              |
|-----------------|--------------------------------------------------------------------------|
| Score Challenge | Roll to beat a target score. Earn 25 gold on success, lose 10 HP on fail |
| Elite           | Harder target, optional. Better gold reward + chance of rare relic       |
| Shop            | Spend gold on relics (40–80g), special dice (50–100g), die upgrades (30g)|
| Rest / Campfire | Choose: restore 15 HP **or** upgrade one die in your pool                |
| Boss            | Multi-round fight with unique weakness + debuff. Unlock a new category   |

---

## Scoring Categories

Unlocked by defeating floor bosses. You start knowing only Chance.

| Category        | Unlocked At    | Score Rule                                                |
|-----------------|----------------|-----------------------------------------------------------|
| Chance          | Start          | Sum of all dice                                           |
| Upper Section   | Floor 1 Boss   | Sum of all dice showing the chosen face (e.g., all 5s)   |
| Pair            | Floor 2 Boss   | Sum of the matching pair                                  |
| Three of a Kind | Floor 2 Boss   | Sum of all dice if 3+ match                               |
| Full House      | Floor 3 Boss   | 25 pts (three of one + two of another)                    |
| Four of a Kind  | Floor 3 Boss   | Sum of all dice if 4+ match                               |
| Small Straight  | Floor 4 Boss   | 30 pts (4 sequential dice)                                |
| Large Straight  | Floor 4 Boss   | 40 pts (5 sequential dice)                                |
| Yahtzee         | Floor 5 Boss   | 50 pts (all five dice the same) + escalating bonus        |

Categories are spent per room; you cannot reuse the same category twice in one room.

---

## Boss Encounters

Each boss applies a **debuff** at the start of the fight and has a **weakness category** that deals 1.5× progress when scored.

| Floor | Boss         | Weakness        | Debuff                                          |
|-------|--------------|-----------------|-------------------------------------------------|
| 1     | Rat King     | Pair            | One die always shows 1 at the start of each roll |
| 2     | Stone Golem  | Upper Section   | Each 1 rolled costs 2 extra HP                  |
| 3     | Goblin King  | Full House      | Only 2 rolls per attempt instead of 3           |
| 4     | Dark Wizard  | Straight        | One randomly selected die is locked each roll   |
| 5     | The Dragon   | Yahtzee         | Target is doubled; scoring Yahtzee heals 15 HP  |

---

## Dice System

Your dice pool is the heart of every run. You start with 5 standard d6 dice, but they can be replaced, upgraded, and enchanted as you progress.

### Special Dice (buy in shops, earn as rewards)

| Die            | Faces           | Effect                                                          |
|----------------|-----------------|-----------------------------------------------------------------|
| Standard d6    | 1, 2, 3, 4, 5, 6| Default starting die                                            |
| Lucky d6       | 2, 3, 4, 5, 6, 6| One face rerolled to 6 (slightly better average)                |
| Weighted d6    | 3, 4, 5, 6, 6, 6| Three high faces; strong for upper section & Yahtzee combos     |
| Wild Die       | 1, 2, 3, 4, 5, W| W face counts as any value when scoring a combo                 |
| Mirrored d6    | 3, 4, 4, 5, 5, 6| No low values; reliable, high floor                             |
| Cursed d6      | 1, 1, 1, 2, 5, 6| High variance; dangerous but big swings possible                |
| Exploding d6   | 1, 2, 3, 4, 5, 6| When it shows 6, reroll and add the result to your scoring total |
| Glass d6       | 1, 2, 4, 5, 6, 6| Extra 6 face, but rolling 1 breaks it until repaired at campfire|
| Odd d6         | 1, 3, 5, 1, 3, 5| Only odd values; great for straights, narrow combo window        |
| Dragon Bone d8 | 1–8             | Eight faces; counts as a 6 or lower for combo matching          |

### Die Upgrades (at campfire)

At a campfire, choose one die in your pool and one upgrade to apply:

| Upgrade    | Effect                                                              |
|------------|---------------------------------------------------------------------|
| Reface     | Replace the lowest face value with the highest face value           |
| Enchant    | One face (chosen) now triggers +10 bonus score when shown           |
| Stabilize  | One face (chosen) always shows when you hold this die               |
| Polish     | Remove a curse or broken status from the die                        |
| Augment    | Add +1 to one face (e.g., a 3-face becomes a 4-face)               |

---

## Relics

Passive items that persist for the entire run. Found in shops and elite rooms.

| Relic                  | Effect                                                                |
|------------------------|-----------------------------------------------------------------------|
| Loaded Dice            | Your first roll each turn rerolls any die showing 1 once for free    |
| Extra Die Slot         | Add a 6th die slot; fill it with any die you own                    |
| One More Chance        | Gain +1 roll per room (4 rolls total)                                 |
| Lucky Horseshoe        | Failing a target costs only 5 HP instead of 10                       |
| Goblin's Hoard         | Earn +15 bonus gold when you beat a target by 150%+                  |
| Cursed Chalice         | -10 max HP, but all shop prices are 20% cheaper                      |
| Enchanted Quill        | Once per floor, you may re-use a category you've already scored       |
| Shield of the Ancients | The first time you'd lose HP each floor, negate the damage            |
| Wizard's Grimoire      | Once per floor, preview what your next roll will be before committing |
| Dice Hoarder           | Start each floor with one extra die drawn from a pool of your spares  |

---

## Meta-Progression

Between runs, spend **Dungeon Coins** (earned based on floors cleared) to upgrade your table with persistent bonuses that apply to every future run.

| Upgrade          | Effect                                      | Cost |
|------------------|---------------------------------------------|------|
| Reinforced Chair | Start with 35 HP instead of 30             | 10 DC|
| Lucky Tablecloth | Start with 15 gold                          | 8 DC |
| Carved Grooves   | One free die upgrade per run at start       | 15 DC|
| Worn Felt        | Shops restock +1 extra item                 | 12 DC|
| Dice Vault       | Start each run with 1 special die of choice | 20 DC|
| Character Unlock | New starting character available            | 25 DC|

### Characters

| Character      | Starting Stats   | Perk                                 |
|----------------|------------------|--------------------------------------|
| The Adventurer | 30 HP, 0g        | Default, unlocked from the start     |
| The Gambler    | 25 HP, 50g       | Shop prices 20% cheaper              |
| The Warrior    | 40 HP, 0g        | Failing a target costs 5 less HP     |
| The Artificer  | 30 HP, 0g        | Start with Lucky d6 + Wild Die; 7-die pool |

---

## Controls

| Key    | Action                              |
|--------|-------------------------------------|
| 1–5    | Hold / unhold that die              |
| R      | Roll all unheld dice                |
| 1–9    | Select a scoring category           |
| Q      | Quit to menu                        |

---

## Building & Running

```bash
cargo build --release
cargo run
```

Requires Rust 2024 edition. Install via [rustup.rs](https://rustup.rs).

---

## Implementation Status

- [ ] Phase 1: Core dice engine (`dice.rs`, `scoring.rs`)
- [ ] Phase 2: Dice upgrade system & relics (`relics.rs`)
- [ ] Phase 3: Dungeon system (`dungeon/`)
- [ ] Phase 4: TUI & game loop (`ui/`, `game.rs`)
- [ ] Phase 5: Floor progression & polish

# 🎮 Dystrail

*What if Oregon Trail took a wrong turn and ended up in DC?*
**Dystrail** is a SNES-lite parody survival game where you march toward Capitol Hill, dodging tariffs, raw milk stands, brain worms, and the dreaded **National Pants Emergency**.

![Dystrail Social Card](static/img/social-card.png)

## 🕹️ Gameplay Loop
1. **Splash Screen** → loading bar, *Press Any Key to Begin*.
2. **Share Code Bar** → prefilled seed (e.g., `CL-PANTS42`), paste a friend/streamer’s code to replay their run.
3. **Mode Select** → **Classic** or **The Deep End** (edgier encounters).
4. **Travel** → burn Supplies/Sanity per leg; log updates.
5. **Encounters** → multi-choice cards with stat effects (Raw Milk, Brain Worms, 5G, Tariffs, National Guard, etc.).
6. **Executive Orders** → rotating global debuffs (Shutdown, Travel Ban Lite, Gas-Stove Police, Book Panic, Deportation Sweep, Tariff Tsunami, DoE Eliminated, Department of War Reorg).
7. **Filibuster Boss** → 3 phases (Cloture, Points of Order, Amendment Flood).
8. **Result Screen** → bold ending + stats + **shareable seed**.

## ✨ Features
- **SNES-lite 16-bit style** palette and sprites.
- **Modes**: `CL` (Classic) and `DP` (The Deep End).
- **Share Codes**: `CL-WORD42` / `DP-GATOR97` — short, speakable, replayable.
- **Pants Meter**: reach 💩 100% → **National Pants Emergency** fail state.
- **Data-driven content**: extend encounters via `assets/data/game.json`.
- **Accessibility**: keyboard navigation, high-contrast & reduced-motion toggles.
- **Meta tags**: clean unfurls on Discord, Slack, X/Twitter, FB.

## 📦 Assets
- `assets/gfx/palette.png` — locked SNES-lite palette
- `assets/gfx/spritesheet.png` — sample tiles: pants frames, gator, milk, tariff, receipt
- `assets/gfx/logo.png` — DYSTRAIL wordmark
- `assets/gfx/social-card.png` — 1200×630 Open Graph/Twitter card
- `favicon.ico` — pants sprite
- `assets/data/game.json` — stub for encounters

## 🛠 Dev Setup
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

## ➕ Contributing Encounters
Edit `assets/data/game.json`:
```json
{
  "id": "tariff_whiplash",
  "name": "Tariff Whiplash",
  "desc": "A surprise tariff now applies to... your stuff.",
  "weight": 5,
  "regions": ["RustBelt","Beltway"],
  "modes": ["classic","deep_end"],
  "choices": [
    { "label": "Pay the tariff", "effects": { "supplies": -2, "credibility": 1, "pants": 5, "log": "You pay the tariff. It stings." } }
  ]
}
```

## 📜 License
MIT

## 👥 Credits
Team Dystrail — design/dev, palette, prototypes. Community satire contributions welcome.

## 🚀 Roadmap
- SFX (encounter chimes, fail stings, filibuster fanfare)
- Full seed encoder/decoder (512-word curated list incl. ORANGE/CHEETO/MANGO)
- Background sets (travel map, boss arena, result screens)
- Result Screen (export as image + seed)

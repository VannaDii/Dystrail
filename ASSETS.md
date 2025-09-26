# Missing Graphical Assets Inventory for Dystrail

Based on comprehensive analysis of the Dystrail codebase, this document catalogs all missing graphical assets needed to complete the visual design of the game. All assets should maintain the retro, SNES-inspired pixel art aesthetic matching the Oregon Trail parody theme, using the established color palette (browns, tans, muted colors) with high contrast accessibility options.

## Character Portraits (6 assets)

### 1. Journalist Portrait

**Usage:** Persona selection screen **Description:** Professional, credible appearance with notepad/recorder. Represents the "receipts-focused" character type who finds more evidence but has moderate starting stats.

### 2. Organizer Portrait

**Usage:** Persona selection screen **Description:** Charismatic leader with community-focused appearance. Represents the morale/ally-building character type with strong social connections.

### 3. Whistleblower Portrait

**Usage:** Persona selection screen **Description:** Nervous but determined figure, possibly shadowed/anonymous. High-risk/high-reward character that draws heat but finds more receipts.

### 4. Lobbyist Portrait

**Usage:** Persona selection screen **Description:** Well-dressed, connected corporate appearance. Represents the character with cost reduction perks and insider connections.

### 5. Staffer Portrait

**Usage:** Persona selection screen **Description:** Generic bureaucrat with average appearance. Balanced character with no special perks or penalties.

### 6. Satirist Portrait

**Usage:** Persona selection screen **Description:** Creative, witty type with humor indicators. Character who copes with humor and has enhanced sanity restoration abilities.

## Weather Visual Indicators (5 assets)

### 1. Clear Weather Icon

**Usage:** Travel panel, stats display **Description:** Pleasant sunny conditions representing neutral gameplay effects with no penalties.

### 2. Storm Icon

**Usage:** Travel panel, stats display **Description:** Rain, lightning, severe weather. Increases supply consumption and encounter chances, mitigated by ponchos.

### 3. HeatWave Icon

**Usage:** Travel panel, stats display **Description:** Intense sun with heat waves. Causes supply drain and sanity loss due to extreme temperatures.

### 4. ColdSnap Icon

**Usage:** Travel panel, stats display **Description:** Snow, freezing conditions. Reduces sanity, mitigated by warm coats from the store.

### 5. Smoke Icon

**Usage:** Travel panel, stats display **Description:** Pollution/smog representation. Poor air quality causing sanity and supply issues, mitigated by masks.

## Store Item Icons (11 assets)

### Fuel/Food Category

#### 1. Rations Pack Icon

**Usage:** Outfitting store interface **Description:** Non-perishable survival food that grants 3 supplies when purchased.

#### 2. Water Jugs Icon

**Usage:** Outfitting store interface **Description:** Hydration supplies that grant 2 supplies when purchased.

### Vehicle Spares Category

#### 3. Spare Tire Icon

**Usage:** Outfitting store interface, vehicle status panel **Description:** Emergency tire for blowout repairs during vehicle breakdowns.

#### 4. Battery Icon

**Usage:** Outfitting store interface, vehicle status panel **Description:** Car battery replacement for cold start problems and electrical failures.

#### 5. Alternator Icon

**Usage:** Outfitting store interface, vehicle status panel **Description:** Charging system component for electrical system failures.

#### 6. Fuel Pump Icon

**Usage:** Outfitting store interface, vehicle status panel **Description:** Engine component for fuel system breakdowns and delivery issues.

### PPE & Clothing Category

#### 7. Masks Icon

**Usage:** Outfitting store interface **Description:** Protective equipment with "plague_resist" tag for smoke/illness protection.

#### 8. Warm Coats Icon

**Usage:** Outfitting store interface **Description:** Cold weather gear with "cold_resist" tag to mitigate ColdSnap weather effects.

#### 9. Ponchos Icon

**Usage:** Outfitting store interface **Description:** Rain protection gear with "rain_resist" tag to mitigate Storm weather effects.

### Documents & Permits Category

#### 10. Press Pass Icon

**Usage:** Outfitting store interface **Description:** Journalist credentials that act as crossing permits for checkpoint encounters.

#### 11. Legal Fund Icon

**Usage:** Outfitting store interface **Description:** Money/legal documents that boost credibility stat when purchased.

## Executive Order Icons (8 assets)

### 1. Shutdown Icon

**Usage:** Travel panel, active effects display **Description:** Government building closed, represents government shutdowns that increase sanity costs.

### 2. Travel Ban Lite Icon

**Usage:** Travel panel, active effects display **Description:** Restricted movement/checkpoints that increase daily supply costs.

### 3. Gas Stove Police Icon

**Usage:** Travel panel, active effects display **Description:** Regulatory enforcement representing domestic policy overreach.

### 4. Book Panic Icon

**Usage:** Travel panel, active effects display **Description:** Censorship/book banning representation affecting sanity.

### 5. Deportation Sweep Icon

**Usage:** Travel panel, active effects display **Description:** Immigration enforcement operations that increase sanity costs.

### 6. Tariff Tsunami Icon

**Usage:** Travel panel, active effects display **Description:** Trade barriers/economic disruption that increase supply costs.

### 7. DoE Eliminated Icon

**Usage:** Travel panel, active effects display **Description:** Department of Education closure representing institutional dismantling.

### 8. War Dept Reorg Icon

**Usage:** Travel panel, active effects display **Description:** Military reorganization/restructuring affecting encounter rates.

## Encounter Illustrations (2+ assets)

### 1. Raw Milk Stand Scene

**Usage:** Encounter card display **Description:** Dubious roadside stand with milk vendors. Scene for health/credibility choice encounter where players choose between drinking questionable milk or declining politely.

### 2. Tariff Whiplash Scene

**Usage:** Encounter card display **Description:** Border/customs checkpoint with officials. Economic policy encounter where players choose between paying tariffs or smuggling goods.

_Note: Additional encounter illustrations may be needed as more encounters are added to game.json_

## Vehicle & Breakdown Illustrations (5 assets)

### 1. Tire Blowout Illustration

**Usage:** Vehicle status panel, breakdown events **Description:** Flat tire scenario showing damaged wheel, used when tire breakdowns occur.

### 2. Battery Failure Illustration

**Usage:** Vehicle status panel, breakdown events **Description:** Dead battery/electrical problem scenario with car electrical components.

### 3. Alternator Failure Illustration

**Usage:** Vehicle status panel, breakdown events **Description:** Charging system breakdown scenario showing engine electrical components.

### 4. Fuel Pump Failure Illustration

**Usage:** Vehicle status panel, breakdown events **Description:** Engine/fuel system problem scenario showing fuel delivery issues.

### 5. General Vehicle Status

**Usage:** Vehicle status panel **Description:** Healthy vehicle representation for status panel when no breakdowns are active.

## Pace & Diet Setting Icons (6 assets)

### Pace Settings

#### 1. Steady Pace Icon

**Usage:** Pace/Diet panel **Description:** Normal speed setting with balanced risk/reward, no special effects.

#### 2. Heated Pace Icon

**Usage:** Pace/Diet panel **Description:** Faster travel with increased encounter chances and moderate pants accumulation.

#### 3. Blitz Pace Icon

**Usage:** Pace/Diet panel **Description:** Maximum speed setting with high encounter risk and significant pants accumulation.

### Diet Settings

#### 4. Quiet Diet Icon

**Usage:** Pace/Diet panel **Description:** Low-key social media presence, reduces receipt finding but helps sanity and reduces pants.

#### 5. Mixed Diet Icon

**Usage:** Pace/Diet panel **Description:** Balanced social media consumption with neutral effects across all stats.

#### 6. Doomscroll Diet Icon

**Usage:** Pace/Diet panel **Description:** Heavy social media use that finds more receipts but damages sanity and increases pants.

## Camp Action Illustrations (4 assets)

### 1. Rest Illustration

**Usage:** Camp panel **Description:** Character resting peacefully, represents the action that restores sanity and HP at the cost of supplies and time.

### 2. Therapy Illustration

**Usage:** Camp panel **Description:** Mental health session or reflection, burns collected receipts for significant sanity restoration.

### 3. Forage Illustration

**Usage:** Camp panel **Description:** Character searching/scavenging environment for supplies or receipts with random success rates.

### 4. Repair Illustration

**Usage:** Camp panel **Description:** Vehicle maintenance and repair work for fixing active breakdown scenarios.

## Stats Bar Icons (7 assets)

### 1. Supplies Icon

**Usage:** Stats bar, persistent UI element **Description:** Food/fuel resources representation (0-20 range), essential for survival.

### 2. HP Icon

**Usage:** Stats bar, persistent UI element **Description:** Health points representation (0-10 range), affects game over conditions.

### 3. Sanity Icon

**Usage:** Stats bar, persistent UI element **Description:** Mental health representation (0-10 range), critical for game completion.

### 4. Credibility Icon

**Usage:** Stats bar, persistent UI element **Description:** Professional reputation representation (0-20 range), affects encounter outcomes.

### 5. Morale Icon

**Usage:** Stats bar, persistent UI element **Description:** Team spirit representation (0-10 range), affects overall performance.

### 6. Allies Icon

**Usage:** Stats bar, persistent UI element **Description:** Network connections representation (0-50 range), provides various benefits.

### 7. Pants Meter

**Usage:** Stats bar, persistent UI element **Description:** Anxiety/panic level visual (0-100 range), possibly a thermometer-style gauge. Critical failure condition at high levels.

## Regional Background Elements (3 assets)

### 1. Heartland Background

**Usage:** Travel panel, regional identification **Description:** Rural/agricultural scenery representing middle America with farmland and small towns.

### 2. RustBelt Background

**Usage:** Travel panel, regional identification **Description:** Industrial/manufacturing region imagery with factories and urban decay.

### 3. Beltway Background

**Usage:** Travel panel, regional identification **Description:** Political/governmental Washington DC area with monuments and government buildings.

## Crossing Scenarios (2 assets)

### 1. Checkpoint Illustration

**Usage:** Crossing card display **Description:** Official border/security checkpoint with guards, representing bureaucratic obstacles that can be bypassed with permits, bribes, or detours.

### 2. Bridge Out Illustration

**Usage:** Crossing card display **Description:** Damaged infrastructure requiring detour or repair, representing infrastructure challenges with higher time/supply costs.

## Result Screen Illustrations (5 assets)

### 1. Victory Illustration

**Usage:** Result screen **Description:** Successful completion of the journey, player character celebrating reaching destination.

### 2. Boss Loss Illustration

**Usage:** Result screen **Description:** Defeat by final challenge/authority figure, representing failure at the ultimate confrontation.

### 3. Pants Threshold Illustration

**Usage:** Result screen **Description:** Anxiety-driven failure representation when pants meter reaches maximum (70+ threshold).

### 4. Sanity Loss Illustration

**Usage:** Result screen **Description:** Mental breakdown ending when sanity reaches zero, showing psychological collapse.

### 5. Collapse Illustration

**Usage:** Result screen **Description:** General failure/resource depletion ending when supplies or HP reach zero.

## UI Enhancement Assets (Optional)

### 1. Loading/Boot Screen Graphics

**Usage:** App initialization **Description:** Currently uses logo only, could add atmospheric elements or loading indicators.

### 2. Menu Navigation Icons

**Usage:** Various menu interfaces **Description:** Currently text-only menus could benefit from visual indicators for better usability.

### 3. Achievement/Score Display Graphics

**Usage:** Result screen, scoring system **Description:** Visual elements for result presentation and score multiplier displays.

---

## Summary

**Total Missing Assets: ~70 individual graphics**

### Asset Priorities

1. **High Priority:** Character portraits, stats icons, weather indicators (core gameplay elements)
2. **Medium Priority:** Store items, encounter illustrations, vehicle states (feature completeness)
3. **Lower Priority:** Regional backgrounds, UI enhancements (polish and atmosphere)

### Technical Requirements

- **Format:** PNG with transparency support
- **Style:** Retro, SNES-inspired pixel art
- **Palette:** Browns, tans, muted colors matching existing game aesthetic
- **Accessibility:** High contrast variants for accessibility mode
- **Size:** Consistent with existing spritesheet dimensions and UI layout constraints

### Integration Notes

- Assets should be organized in spritesheet format for efficient loading
- Icon sizes should be consistent within categories (e.g., all stats icons same size)
- Consider animation possibilities for key elements (weather effects, pants meter)
- Maintain visual hierarchy and readability at target display sizes

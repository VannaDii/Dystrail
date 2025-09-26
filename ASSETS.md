# Missing Graphical Assets Inventory for Dystrail

Based on comprehensive analysis of the Dystrail codebase, this document catalogs all missing graphical assets needed to complete the visual design of the game. All assets should maintain the retro, SNES-inspired pixel art aesthetic matching the Oregon Trail parody theme, using the established color palette and technical specifications below.

## Technical Specifications

### Color Palette

**Standard Theme:**

- Background: `#000000` (Pure Black)
- Panel Background: `#2D1B00` (Dark Brown)
- Panel Border: `#D4A574` (Light Brown/Tan)
- Primary Text: `#D4A574` (Light Brown/Tan)
- Dimmed Text: `#B8956A` (Medium Brown)
- Bright Text: `#F4E4C1` (Cream/Off-white)
- Accent Primary: `#F4E4C1` (Cream)
- Accent Secondary: `#D4A574` (Light Brown)
- Button Background: `#4A3728` (Medium Brown)
- Button Border: `#D4A574` (Light Brown)
- Shadow: `#1A1000` (Very Dark Brown)

**High Contrast Theme (Accessibility):**

- Background: `#0b0e12` (Dark Blue-Grey)
- Panel Background: `#141922` (Medium Blue-Grey)
- Panel Border: `#e9ecef` (Light Grey)
- Primary Text: `#e9ecef` (Light Grey)
- Dimmed Text: `#d0d6dd` (Medium Grey)
- Bright Text: `#ffffff` (Pure White)
- Accent Primary: `#00D9C0` (Bright Teal)
- Accent Secondary: `#00A896` (Dark Teal)

### File Format Requirements

- **Format:** PNG with alpha transparency
- **Bit Depth:** 32-bit RGBA or 8-bit indexed with transparency
- **Compression:** PNG-8 preferred for file size optimization
- **Naming Convention:** `snake_case` with category prefixes

### Spritesheet Organization

- **Master Spritesheet:** Single file containing all game assets
- **Grid System:** 16x16px base unit with larger assets as multiples
- **Padding:** 1px transparent border between sprites to prevent bleeding
- **Layout:** Organized by category in rows for efficient access

## Character Portraits (6 assets)

**Technical Specifications:**

- **Dimensions:** 64x64px (4x4 grid units)
- **Style:** Front-facing portrait, bust-only
- **Colors:** Use standard theme palette with skin tones in #D4A574 to #F4E4C1 range
- **Background:** Transparent or solid panel background (#2D1B00)
- **Details:** High contrast features, recognizable accessories/clothing
- **File Names:** `portrait_journalist.png`, `portrait_organizer.png`, etc.

### 1. Journalist Portrait

- **Usage:** Persona selection screen
- **Visual Elements:** Professional attire, notepad or recorder, glasses optional, determined expression
- **Key Colors:** #4A3728 (suit/jacket), #F4E4C1 (notepad), #D4A574 (skin tone)
- **Description:** Professional, credible appearance with notepad/recorder. Represents the "receipts-focused" character type who finds more evidence but has moderate starting stats.

### 2. Organizer Portrait

- **Usage:** Persona selection screen
- **Visual Elements:** Casual but organized appearance, campaign button or badge, friendly expression
- **Key Colors:** #D4A574 (shirt), #F4E4C1 (badge/button), warm skin tones
- **Description:** Charismatic leader with community-focused appearance. Represents the morale/ally-building character type with strong social connections.

### 3. Whistleblower Portrait

- **Usage:** Persona selection screen
- **Visual Elements:** Shadowed face, hood or hat, USB drive or documents, nervous expression
- **Key Colors:** #1A1000 (shadows), #2D1B00 (clothing), minimal highlights
- **Description:** Nervous but determined figure, possibly shadowed/anonymous. High-risk/high-reward character that draws heat but finds more receipts.

### 4. Lobbyist Portrait

- **Usage:** Persona selection screen
- **Visual Elements:** Expensive suit, tie, briefcase or phone, confident expression
- **Key Colors:** #4A3728 (suit), #D4A574 (tie), #F4E4C1 (shirt collar)
- **Description:** Well-dressed, connected corporate appearance. Represents the character with cost reduction perks and insider connections.

### 5. Staffer Portrait

- **Usage:** Persona selection screen
- **Visual Elements:** Generic office attire, ID badge, neutral expression, average features
- **Key Colors:** Standard office colors in muted palette range
- **Description:** Generic bureaucrat with average appearance. Balanced character with no special perks or penalties.

### 6. Satirist Portrait

- **Usage:** Persona selection screen
- **Visual Elements:** Creative attire, humor indicators (smile, wit in eyes), artistic accessories
- **Key Colors:** More varied palette within theme constraints, creative flair
- **Description:** Creative, witty type with humor indicators. Character who copes with humor and has enhanced sanity restoration abilities.

## Weather Visual Indicators (5 assets)

**Technical Specifications:**

- **Dimensions:** 24x24px (1.5x1.5 grid units)
- **Style:** Simple iconographic representation
- **Colors:** Use weather-appropriate variants within palette
- **Background:** Transparent
- **Animation:** Static icons (animation optional for polish)
- **File Names:** `weather_clear.png`, `weather_storm.png`, etc.

### 1. Clear Weather Icon

- **Usage:** Travel panel, stats display
- **Visual Elements:** Simple sun or clear sky indicator
- **Key Colors:** #F4E4C1 (sun), #D4A574 (rays)
- **Description:** Pleasant sunny conditions representing neutral gameplay effects with no penalties.

### 2. Storm Icon

- **Usage:** Travel panel, stats display
- **Visual Elements:** Rain drops, lightning bolt, or storm cloud
- **Key Colors:** #B8956A (cloud), #F4E4C1 (lightning), #D4A574 (rain)
- **Description:** Rain, lightning, severe weather. Increases supply consumption and encounter chances, mitigated by ponchos.

### 3. HeatWave Icon

- **Usage:** Travel panel, stats display
- **Visual Elements:** Intense sun with heat wave lines
- **Key Colors:** #F4E4C1 (sun), #D4A574 (heat waves)
- **Description:** Intense sun with heat waves. Causes supply drain and sanity loss due to extreme temperatures.

### 4. ColdSnap Icon

- **Usage:** Travel panel, stats display
- **Visual Elements:** Snowflake or icicle
- **Key Colors:** #F4E4C1 (snow), #B8956A (ice)
- **Description:** Snow, freezing conditions. Reduces sanity, mitigated by warm coats from the store.

### 5. Smoke Icon

- **Usage:** Travel panel, stats display
- **Visual Elements:** Smoke/smog cloud or pollution indicator
- **Key Colors:** #1A1000 (smoke), #2D1B00 (pollution), #B8956A (haze)
- **Description:** Pollution/smog representation. Poor air quality causing sanity and supply issues, mitigated by masks.

## Store Item Icons (11 assets)

**Technical Specifications:**

- **Dimensions:** 32x32px (2x2 grid units)
- **Style:** Isometric or side-view item representation
- **Colors:** Use appropriate material colors within theme palette
- **Background:** Transparent
- **Details:** Clear, recognizable silhouettes even at small size
- **File Names:** `item_rations.png`, `item_water.png`, etc.

### Fuel/Food Category

#### 1. Rations Pack Icon

- **Usage:** Outfitting store interface
- **Visual Elements:** Military-style food package or canned goods
- **Key Colors:** #4A3728 (package), #D4A574 (label), #B8956A (contents)
- **Description:** Non-perishable survival food that grants 3 supplies when purchased.

#### 2. Water Jugs Icon

- **Usage:** Outfitting store interface
- **Visual Elements:** Water containers or bottles
- **Key Colors:** #F4E4C1 (water/highlights), #D4A574 (container), #4A3728 (caps)
- **Description:** Hydration supplies that grant 2 supplies when purchased.

### Vehicle Spares Category

#### 3. Spare Tire Icon

- **Usage:** Outfitting store interface, vehicle status panel
- **Visual Elements:** Car tire with tread pattern visible
- **Key Colors:** #1A1000 (tire), #D4A574 (rim), #4A3728 (tread)
- **Description:** Emergency tire for blowout repairs during vehicle breakdowns.

#### 4. Battery Icon

- **Usage:** Outfitting store interface, vehicle status panel
- **Visual Elements:** Car battery with terminals and handle
- **Key Colors:** #2D1B00 (battery case), #D4A574 (terminals), #B8956A (labels)
- **Description:** Car battery replacement for cold start problems and electrical failures.

#### 5. Alternator Icon

- **Usage:** Outfitting store interface, vehicle status panel
- **Visual Elements:** Cylindrical automotive alternator with pulley
- **Key Colors:** #4A3728 (housing), #D4A574 (pulley), #B8956A (wiring)
- **Description:** Charging system component for electrical system failures.

#### 6. Fuel Pump Icon

- **Usage:** Outfitting store interface, vehicle status panel
- **Visual Elements:** Mechanical pump with fuel lines
- **Key Colors:** #4A3728 (pump body), #D4A574 (fittings), #B8956A (lines)
- **Description:** Engine component for fuel system breakdowns and delivery issues.

### PPE & Clothing Category

#### 7. Masks Icon

- **Usage:** Outfitting store interface
- **Visual Elements:** Medical or N95-style protective mask
- **Key Colors:** #F4E4C1 (mask material), #D4A574 (straps), #4A3728 (filter)
- **Description:** Protective equipment with "plague_resist" tag for smoke/illness protection.

#### 8. Warm Coats Icon

- **Usage:** Outfitting store interface
- **Visual Elements:** Heavy winter coat with hood or collar
- **Key Colors:** #4A3728 (coat), #D4A574 (zipper/buttons), #B8956A (lining)
- **Description:** Cold weather gear with "cold_resist" tag to mitigate ColdSnap weather effects.

#### 9. Ponchos Icon

- **Usage:** Outfitting store interface
- **Visual Elements:** Rain poncho or waterproof covering
- **Key Colors:** #2D1B00 (poncho), #D4A574 (hood), #B8956A (water droplets)
- **Description:** Rain protection gear with "rain_resist" tag to mitigate Storm weather effects.

### Documents & Permits Category

#### 10. Press Pass Icon

- **Usage:** Outfitting store interface
- **Visual Elements:** Official press badge or ID card
- **Key Colors:** #F4E4C1 (badge), #D4A574 (text), #4A3728 (lanyard)
- **Description:** Journalist credentials that act as crossing permits for checkpoint encounters.

#### 11. Legal Fund Icon

- **Usage:** Outfitting store interface
- **Visual Elements:** Stack of money or legal documents
- **Key Colors:** #F4E4C1 (money/papers), #D4A574 (briefcase), #4A3728 (binding)
- **Description:** Money/legal documents that boost credibility stat when purchased.

## Executive Order Icons (8 assets)

**Technical Specifications:**

- **Dimensions:** 20x20px (1.25x1.25 grid units)
- **Style:** Simple symbolic representation
- **Colors:** Monochromatic using #D4A574 with #1A1000 shadows
- **Background:** Transparent
- **Design:** Government/official iconography style
- **File Names:** `eo_shutdown.png`, `eo_travel_ban.png`, etc.

### 1. Shutdown Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** Government building with "CLOSED" or barred doors
- **Key Colors:** #D4A574 (building), #1A1000 (shadows/bars)
- **Description:** Government building closed, represents government shutdowns that increase sanity costs.

### 2. Travel Ban Lite Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** Road barrier or checkpoint symbol
- **Key Colors:** #D4A574 (barrier), #1A1000 (warning stripes)
- **Description:** Restricted movement/checkpoints that increase daily supply costs.

### 3. Gas Stove Police Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** Stove with prohibition symbol or badge
- **Key Colors:** #D4A574 (stove/badge), #1A1000 (prohibition)
- **Description:** Regulatory enforcement representing domestic policy overreach.

### 4. Book Panic Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** Book with prohibition symbol or flames
- **Key Colors:** #D4A574 (book), #1A1000 (flames/prohibition)
- **Description:** Censorship/book banning representation affecting sanity.

### 5. Deportation Sweep Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** Vehicle or enforcement symbol
- **Key Colors:** #D4A574 (vehicle), #1A1000 (official markings)
- **Description:** Immigration enforcement operations that increase sanity costs.

### 6. Tariff Tsunami Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** Wave or barrier with dollar sign
- **Key Colors:** #D4A574 (wave/barrier), #1A1000 (dollar symbol)
- **Description:** Trade barriers/economic disruption that increase supply costs.

### 7. DoE Eliminated Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** School building with X or demolition
- **Key Colors:** #D4A574 (building), #1A1000 (X/demolition)
- **Description:** Department of Education closure representing institutional dismantling.

### 8. War Dept Reorg Icon

- **Usage:** Travel panel, active effects display
- **Visual Elements:** Military insignia with reorganization arrows
- **Key Colors:** #D4A574 (insignia), #1A1000 (arrows)
- **Description:** Military reorganization/restructuring affecting encounter rates.

## Encounter Illustrations (2+ assets)

**Technical Specifications:**

- **Dimensions:** 240x160px (15x10 grid units)
- **Style:** Scene illustration, SNES-style environmental art
- **Colors:** Full theme palette with atmospheric lighting
- **Background:** Integrated environmental background
- **Perspective:** Slight isometric or side-view
- **File Names:** `encounter_raw_milk.png`, `encounter_tariff.png`, etc.

### 1. Raw Milk Stand Scene

- **Usage:** Encounter card display
- **Visual Elements:** Roadside stand, milk vendor, questionable signage, rural setting
- **Key Colors:** #2D1B00 (stand), #F4E4C1 (milk), #D4A574 (signage), #4A3728 (vendor)
- **Atmosphere:** Slightly ominous, questionable safety
- **Description:** Dubious roadside stand with milk vendors. Scene for health/credibility choice encounter where players choose between drinking questionable milk or declining politely.

### 2. Tariff Whiplash Scene

- **Usage:** Encounter card display
- **Visual Elements:** Border checkpoint, customs officials, vehicles, bureaucratic setting
- **Key Colors:** #4A3728 (uniforms), #D4A574 (checkpoint), #B8956A (vehicles)
- **Atmosphere:** Official, bureaucratic tension
- **Description:** Border/customs checkpoint with officials. Economic policy encounter where players choose between paying tariffs or smuggling goods.

_Note: Additional encounter illustrations may be needed as more encounters are added to game.json_

## Vehicle & Breakdown Illustrations (5 assets)

**Technical Specifications:**

- **Dimensions:** 96x64px (6x4 grid units)
- **Style:** Side-view or 3/4 view vehicle illustration
- **Colors:** Vehicle colors in #4A3728 to #D4A574 range with problem-specific highlights
- **Background:** Transparent or minimal ground context
- **Detail Level:** Clear problem visualization without overwhelming detail
- **File Names:** `vehicle_tire_blowout.png`, `vehicle_battery_dead.png`, etc.

### 1. Tire Blowout Illustration

- **Usage:** Vehicle status panel, breakdown events
- **Visual Elements:** Vehicle with flat/damaged tire, tire debris, jack or tools nearby
- **Key Colors:** #4A3728 (vehicle body), #1A1000 (flat tire), #D4A574 (rim/tools)
- **Problem Indicators:** Deflated tire, possible smoke or debris
- **Description:** Flat tire scenario showing damaged wheel, used when tire breakdowns occur.

### 2. Battery Failure Illustration

- **Usage:** Vehicle status panel, breakdown events
- **Visual Elements:** Vehicle with hood up, dead battery visible, jumper cables or warning signs
- **Key Colors:** #2D1B00 (battery), #D4A574 (cables), #B8956A (warning indicators)
- **Problem Indicators:** Dim/no headlights, battery corrosion, electrical failure symbols
- **Description:** Dead battery/electrical problem scenario with car electrical components.

### 3. Alternator Failure Illustration

- **Usage:** Vehicle status panel, breakdown events
- **Visual Elements:** Vehicle with electrical system failure, alternator visible under hood
- **Key Colors:** #4A3728 (engine bay), #D4A574 (alternator), #1A1000 (failure indicators)
- **Problem Indicators:** Electrical warning lights, charging system failure
- **Description:** Charging system breakdown scenario showing engine electrical components.

### 4. Fuel Pump Failure Illustration

- **Usage:** Vehicle status panel, breakdown events
- **Visual Elements:** Vehicle stalled, fuel system components, empty tank indicators
- **Key Colors:** #4A3728 (vehicle), #D4A574 (fuel components), #B8956A (fuel lines)
- **Problem Indicators:** Fuel gauge on empty, fuel pump components highlighted
- **Description:** Engine/fuel system problem scenario showing fuel delivery issues.

### 5. General Vehicle Status

- **Usage:** Vehicle status panel
- **Visual Elements:** Healthy vehicle in good condition, all systems functioning
- **Key Colors:** #4A3728 (vehicle body), #D4A574 (trim/details), #F4E4C1 (highlights)
- **Status Indicators:** Clean appearance, proper tire inflation, functioning lights
- **Description:** Healthy vehicle representation for status panel when no breakdowns are active.

## Pace & Diet Setting Icons (6 assets)

**Technical Specifications:**

- **Dimensions:** 24x24px (1.5x1.5 grid units)
- **Style:** Activity-based iconography
- **Colors:** Use #D4A574 primary with #F4E4C1 highlights
- **Background:** Transparent
- **Design:** Motion/activity indicators where appropriate
- **File Names:** `pace_steady.png`, `diet_quiet.png`, etc.

### Pace Settings

#### 1. Steady Pace Icon

- **Usage:** Pace/Diet panel
- **Visual Elements:** Normal walking figure or moderate speedometer
- **Key Colors:** #D4A574 (figure), #F4E4C1 (highlights)
- **Description:** Normal speed setting with balanced risk/reward, no special effects.

#### 2. Heated Pace Icon

- **Usage:** Pace/Diet panel
- **Visual Elements:** Running figure or elevated speedometer with motion lines
- **Key Colors:** #D4A574 (figure), #F4E4C1 (motion), #B8956A (stress)
- **Description:** Faster travel with increased encounter chances and moderate pants accumulation.

#### 3. Blitz Pace Icon

- **Usage:** Pace/Diet panel
- **Visual Elements:** Sprinting figure or maxed speedometer with intense motion
- **Key Colors:** #D4A574 (figure), #F4E4C1 (speed lines), #1A1000 (intensity)
- **Description:** Maximum speed setting with high encounter risk and significant pants accumulation.

### Diet Settings

#### 4. Quiet Diet Icon

- **Usage:** Pace/Diet panel
- **Visual Elements:** Peaceful or meditation symbol, minimal social media indicators
- **Key Colors:** #F4E4C1 (calm), #D4A574 (symbol)
- **Description:** Low-key social media presence, reduces receipt finding but helps sanity and reduces pants.

#### 5. Mixed Diet Icon

- **Usage:** Pace/Diet panel
- **Visual Elements:** Balanced scale or moderate activity indicator
- **Key Colors:** #D4A574 (balance), #F4E4C1 (neutrality)
- **Description:** Balanced social media consumption with neutral effects across all stats.

#### 6. Doomscroll Diet Icon

- **Usage:** Pace/Diet panel
- **Visual Elements:** Phone/screen with chaotic scroll indicators or stressed figure
- **Key Colors:** #B8956A (stress), #D4A574 (device), #1A1000 (chaos)
- **Description:** Heavy social media use that finds more receipts but damages sanity and increases pants.

## Camp Action Illustrations (4 assets)

**Technical Specifications:**

- **Dimensions:** 48x48px (3x3 grid units)
- **Style:** Character-focused activity scenes
- **Colors:** Full palette with environmental context
- **Background:** Simple environmental context included
- **Design:** Clear action representation
- **File Names:** `camp_rest.png`, `camp_therapy.png`, etc.

### 1. Rest Illustration

- **Usage:** Camp panel
- **Visual Elements:** Character sleeping or resting peacefully, campfire or shelter
- **Key Colors:** #2D1B00 (ground), #F4E4C1 (character), #D4A574 (fire/shelter)
- **Description:** Character resting peacefully, represents the action that restores sanity and HP at the cost of supplies and time.

### 2. Therapy Illustration

- **Usage:** Camp panel
- **Visual Elements:** Character in contemplation, papers burning, or journal writing
- **Key Colors:** #F4E4C1 (character), #D4A574 (papers), #B8956A (smoke/flames)
- **Description:** Mental health session or reflection, burns collected receipts for significant sanity restoration.

### 3. Forage Illustration

- **Usage:** Camp panel
- **Visual Elements:** Character searching through environment, finding supplies
- **Key Colors:** #D4A574 (character), #4A3728 (environment), #F4E4C1 (found items)
- **Description:** Character searching/scavenging environment for supplies or receipts with random success rates.

### 4. Repair Illustration

- **Usage:** Camp panel
- **Visual Elements:** Character working on vehicle with tools
- **Key Colors:** #4A3728 (vehicle), #D4A574 (character), #F4E4C1 (tools)
- **Description:** Vehicle maintenance and repair work for fixing active breakdown scenarios.

## Stats Bar Icons (7 assets)

**Technical Specifications:**

- **Dimensions:** 16x16px (1x1 grid units)
- **Style:** Clear iconographic symbols
- **Colors:** High contrast using #F4E4C1 with #1A1000 outlines
- **Background:** Transparent
- **Design:** Instantly recognizable at small size
- **File Names:** `stat_supplies.png`, `stat_hp.png`, etc.

### 1. Supplies Icon

- **Usage:** Stats bar, persistent UI element
- **Visual Elements:** Food container, fuel can, or supply box
- **Key Colors:** #F4E4C1 (container), #1A1000 (outline), #D4A574 (contents)
- **Description:** Food/fuel resources representation (0-20 range), essential for survival.

### 2. HP Icon

- **Usage:** Stats bar, persistent UI element
- **Visual Elements:** Heart, health cross, or life indicator
- **Key Colors:** #F4E4C1 (symbol), #1A1000 (outline), #D4A574 (fill)
- **Description:** Health points representation (0-10 range), affects game over conditions.

### 3. Sanity Icon

- **Usage:** Stats bar, persistent UI element
- **Visual Elements:** Brain, head silhouette, or mental health symbol
- **Key Colors:** #F4E4C1 (symbol), #1A1000 (outline), #B8956A (stress indicators)
- **Description:** Mental health representation (0-10 range), critical for game completion.

### 4. Credibility Icon

- **Usage:** Stats bar, persistent UI element
- **Visual Elements:** Badge, certificate, or reputation symbol
- **Key Colors:** #F4E4C1 (badge), #1A1000 (outline), #D4A574 (official markings)
- **Description:** Professional reputation representation (0-20 range), affects encounter outcomes.

### 5. Morale Icon

- **Usage:** Stats bar, persistent UI element
- **Visual Elements:** Thumbs up, smile, or team spirit indicator
- **Key Colors:** #F4E4C1 (symbol), #1A1000 (outline), #D4A574 (positive indicators)
- **Description:** Team spirit representation (0-10 range), affects overall performance.

### 6. Allies Icon

- **Usage:** Stats bar, persistent UI element
- **Visual Elements:** People silhouettes, handshake, or network symbol
- **Key Colors:** #F4E4C1 (figures), #1A1000 (outline), #D4A574 (connections)
- **Description:** Network connections representation (0-50 range), provides various benefits.

### 7. Pants Meter

- **Usage:** Stats bar, persistent UI element
- **Visual Elements:** Thermometer-style gauge, anxiety indicator, or pants symbol
- **Key Colors:** #F4E4C1 (gauge), #1A1000 (outline), #B8956A to #D4A574 (gradient fill)
- **Special Design:** Vertical or horizontal meter with clear level indicators
- **Description:** Anxiety/panic level visual (0-100 range), possibly a thermometer-style gauge. Critical failure condition at high levels.

## Regional Background Elements (3 assets)

**Technical Specifications:**

- **Dimensions:** 320x120px (20x7.5 grid units)
- **Style:** Wide panoramic background, low-detail environmental art
- **Colors:** Muted palette emphasizing regional characteristics
- **Background:** Integrated horizon line and sky
- **Perspective:** Wide landscape view
- **File Names:** `region_heartland.png`, `region_rustbelt.png`, `region_beltway.png`

### 1. Heartland Background

- **Usage:** Travel panel, regional identification
- **Visual Elements:** Rolling farmland, grain silos, rural roads, small farm buildings
- **Key Colors:** #4A3728 (earth), #D4A574 (grain fields), #F4E4C1 (sky), #2D1B00 (buildings)
- **Atmosphere:** Pastoral, agricultural, open spaces
- **Description:** Rural/agricultural scenery representing middle America with farmland and small towns.

### 2. RustBelt Background

- **Usage:** Travel panel, regional identification
- **Visual Elements:** Factory smokestacks, industrial buildings, urban infrastructure, weathered structures
- **Key Colors:** #2D1B00 (factories), #1A1000 (smoke), #B8956A (weathered metal), #4A3728 (infrastructure)
- **Atmosphere:** Industrial decline, manufacturing heritage, urban grit
- **Description:** Industrial/manufacturing region imagery with factories and urban decay.

### 3. Beltway Background

- **Usage:** Travel panel, regional identification
- **Visual Elements:** Government buildings, monuments, political infrastructure, official architecture
- **Key Colors:** #F4E4C1 (marble/stone), #D4A574 (buildings), #4A3728 (structures), #2D1B00 (shadows)
- **Atmosphere:** Official, bureaucratic, political power center
- **Description:** Political/governmental Washington DC area with monuments and government buildings.

## Crossing Scenarios (2 assets)

**Technical Specifications:**

- **Dimensions:** 160x120px (10x7.5 grid units)
- **Style:** Environmental scene with obstacle focus
- **Colors:** Situational colors emphasizing the barrier/challenge
- **Background:** Context-appropriate environment
- **Perspective:** Clear view of the obstacle and potential solutions
- **File Names:** `crossing_checkpoint.png`, `crossing_bridge_out.png`

### 1. Checkpoint Illustration

- **Usage:** Crossing card display
- **Visual Elements:** Security booth, barriers, guards or officials, official signage
- **Key Colors:** #4A3728 (booth), #D4A574 (barriers), #F4E4C1 (signage), #2D1B00 (uniforms)
- **Atmosphere:** Official checkpoint, bureaucratic control
- **Description:** Official border/security checkpoint with guards, representing bureaucratic obstacles that can be bypassed with permits, bribes, or detours.

### 2. Bridge Out Illustration

- **Usage:** Crossing card display
- **Visual Elements:** Broken or damaged bridge, water/gap below, warning signs, detour indicators
- **Key Colors:** #4A3728 (bridge structure), #B8956A (damage), #D4A574 (warning signs), #2D1B00 (water/gap)
- **Atmosphere:** Infrastructure failure, repair needed, alternate route required
- **Description:** Damaged infrastructure requiring detour or repair, representing infrastructure challenges with higher time/supply costs.

## Result Screen Illustrations (5 assets)

**Technical Specifications:**

- **Dimensions:** 200x150px (12.5x9.5 grid units)
- **Style:** Dramatic conclusion scenes with character focus
- **Colors:** Emotional palette appropriate to ending type
- **Background:** Full environmental context
- **Mood:** Clear emotional tone matching the ending
- **File Names:** `result_victory.png`, `result_boss_loss.png`, etc.

### 1. Victory Illustration

- **Usage:** Result screen
- **Visual Elements:** Character celebrating at destination, triumphant pose, journey completion indicators
- **Key Colors:** #F4E4C1 (celebration), #D4A574 (character), #4A3728 (destination), bright tones
- **Mood:** Triumphant, successful, accomplished
- **Description:** Successful completion of the journey, player character celebrating reaching destination.

### 2. Boss Loss Illustration

- **Usage:** Result screen
- **Visual Elements:** Character defeated by authority figure, imposing opponent, failure indicators
- **Key Colors:** #1A1000 (defeat shadows), #2D1B00 (authority), #B8956A (character), darker tones
- **Mood:** Defeated, overwhelmed, authoritarian victory
- **Description:** Defeat by final challenge/authority figure, representing failure at the ultimate confrontation.

### 3. Pants Threshold Illustration

- **Usage:** Result screen
- **Visual Elements:** Character overwhelmed by anxiety, pants meter maxed out, stress indicators
- **Key Colors:** #B8956A (stress), #1A1000 (anxiety shadows), #D4A574 (character), high contrast
- **Mood:** Panic, overwhelmed, anxiety-driven failure
- **Description:** Anxiety-driven failure representation when pants meter reaches maximum (70+ threshold).

### 4. Sanity Loss Illustration

- **Usage:** Result screen
- **Visual Elements:** Character in mental breakdown, scattered thoughts, psychological collapse indicators
- **Key Colors:** #1A1000 (darkness), #B8956A (confusion), #2D1B00 (shadows), fragmented visuals
- **Mood:** Mental breakdown, psychological collapse, losing grip on reality
- **Description:** Mental breakdown ending when sanity reaches zero, showing psychological collapse.

### 5. Collapse Illustration

- **Usage:** Result screen
- **Visual Elements:** Character exhausted/depleted, empty supplies, vehicle stopped, general failure
- **Key Colors:** #2D1B00 (depletion), #B8956A (exhaustion), #1A1000 (empty), muted palette
- **Mood:** Exhaustion, resource depletion, general system failure **Description:** General failure/resource depletion ending when supplies or HP reach zero.

## UI Enhancement Assets (Optional)

**Technical Specifications:**

- **Dimensions:** Variable based on usage context
- **Style:** Consistent with overall game aesthetic
- **Colors:** Standard theme palette
- **Purpose:** Polish and usability improvements
- **File Names:** `ui_loading.png`, `ui_nav_icon.png`, etc.

### 1. Loading/Boot Screen Graphics

- **Usage:** App initialization
- **Visual Elements:** Atmospheric elements, loading bars, game world preview
- **Key Colors:** #2D1B00 (background), #D4A574 (loading elements), #F4E4C1 (highlights)
- **Dimensions:** 320x240px (20x15 grid units)
- **Description:** Currently uses logo only, could add atmospheric elements or loading indicators.

### 2. Menu Navigation Icons

- **Usage:** Various menu interfaces
- **Visual Elements:** Arrow indicators, selection highlights, navigation aids
- **Key Colors:** #F4E4C1 (active), #D4A574 (inactive), #1A1000 (outlines)
- **Dimensions:** 16x16px to 24x24px (1x1 to 1.5x1.5 grid units)
- **Description:** Currently text-only menus could benefit from visual indicators for better usability.

### 3. Achievement/Score Display Graphics

- **Usage:** Result screen, scoring system
- **Visual Elements:** Score multipliers, achievement badges, performance indicators
- **Key Colors:** #F4E4C1 (achievements), #D4A574 (scores), #4A3728 (frames)
- **Dimensions:** 32x32px (2x2 grid units) for badges, variable for score displays
- **Description:** Visual elements for result presentation and score multiplier displays.

---

## Summary

**Total Missing Assets: ~70 individual graphics**

### Asset Priorities

1. **High Priority:** Character portraits, stats icons, weather indicators (core gameplay elements)
2. **Medium Priority:** Store items, encounter illustrations, vehicle states (feature completeness)
3. **Lower Priority:** Regional backgrounds, UI enhancements (polish and atmosphere)

### Technical Requirements

**File Specifications:**

- **Format:** PNG-8 with alpha transparency preferred for file size
- **Compression:** Optimize with tools like TinyPNG or similar
- **Naming:** snake*case with category prefixes (e.g., `portrait*`, `weather*`, `item*`)
- **Organization:** Separate files initially, then combined into spritesheet

**Visual Style Guidelines:**

- **Pixel Art:** Clean pixel boundaries, no anti-aliasing on edges
- **Scaling:** Design at 1x size, avoid upscaling artifacts
- **Contrast:** Minimum 3:1 contrast ratio for accessibility
- **Readability:** Clear silhouettes recognizable at target display size

**Color Usage:**

- **Primary Palette:** Use established theme colors as base
- **Color Count:** Limit to 16 colors per sprite for authentic retro feel
- **Dithering:** Use ordered dithering for gradients if needed
- **Consistency:** Maintain lighting direction (top-left) across all assets

**Accessibility Requirements:**

- **High Contrast Mode:** All assets must work with high contrast theme colors
- **Color Blindness:** Icons should be distinguishable by shape, not just color
- **Size Requirements:** Minimum 16x16px for readable details
- **Alternative Text:** Consider iconography that supports screen reader descriptions

### Integration Notes

**Spritesheet Layout:**

- **Grid System:** 16x16px base units with consistent spacing
- **Atlas Generation:** Use tools like TexturePacker or similar
- **JSON Mapping:** Generate coordinate mapping for Rust integration
- **Loading Strategy:** Single spritesheet load with cached sub-images

**Implementation Details:**

- **CSS Integration:** Assets referenced via background-image positioning
- **Rust Integration:** Sprite coordinates defined in game data structures
- **Animation Support:** Reserve space for multi-frame animations
- **Performance:** Target <2MB total spritesheet size for web delivery

**Quality Assurance:**

- **Testing:** Verify assets at multiple zoom levels (100%, 125%, 150%)
- **Consistency:** Ensure visual cohesion across all asset categories
- **Validation:** Test with both standard and high contrast themes
- **Fallbacks:** Provide text alternatives for critical UI elements

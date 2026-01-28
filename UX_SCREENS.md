Dystrail UX Design Spec — Screens 1 & 2 (DaisyUI)

This document defines pixel-faithful, Oregon Trail–aligned UX for the first two screens of Dystrail using DaisyUI components. It is written as a handoff-ready design + implementation reference for engineers.

Scope:
	•	Screen 1: Boot / Title / Loading
	•	Screen 2: Main Menu

Non-goals:
	•	No gameplay logic
	•	No routing beyond forward navigation
	•	No animations beyond subtle retro affordances

⸻

Global Constraints (Apply to Both Screens)

Visual Language
	•	SNES‑lite, panel‑based UI
	•	No floating cards
	•	No translucency / glassmorphism
	•	No modern gradients

Layout Rules
	•	Centered primary panel
	•	Fixed pixel rhythm (multiples of 8px)
	•	Single dominant panel per screen

DaisyUI Theme Usage
	•	Base theme: custom dystrail-dark
	•	Buttons: btn + btn-outline
	•	Panels: card with square corners (override radius)
	•	Typography: default DaisyUI sans, no decorative fonts

Accessibility
	•	Keyboard-only usable
	•	Visible focus ring
	•	No hover-only affordances
	•	Screen-reader readable hierarchy

⸻

SCREEN 1 — BOOT / TITLE / LOADING

Purpose

Establish tone, nostalgia, and deliberateness. This screen must feel ceremonial, not instant.

The user should feel they are entering something.

⸻

DaisyUI Component Breakdown

Root Container
	•	div.min-h-screen
	•	flex items-center justify-center
	•	Background: solid dark (no texture yet)

⸻

Primary Panel

Component: card
	•	Classes:
	•	card
	•	border border-base-content
	•	bg-base-200
	•	w-[420px] max-w-full
	•	Overrides:
	•	rounded-none
	•	shadow-none

This panel represents the entire boot UI.

⸻

Logo Block

Component: plain div
	•	Centered
	•	Fixed vertical spacing
	•	Image or text-based logo allowed

Typography guidance:
	•	Title size: text-3xl font-bold
	•	Subtitle: text-sm opacity-80

⸻

Loading Indicator

Component: progress
	•	DaisyUI: progress progress-primary
	•	Height increased slightly for retro feel
	•	Determinate if possible, indeterminate allowed

Label below:
	•	text-xs opacity-70
	•	Example: Loading encounters…

⸻

“Press Any Key” Prompt

Component: kbd + text
	•	Text: Press any key to begin
	•	Class:
	•	text-sm
	•	animate-pulse (slow, subtle)

This prompt only appears after loading completes.

⸻

Footer / Build Info

Component: div
	•	text-[10px] opacity-50
	•	Content examples:
	•	version
	•	seed engine ready

Purpose: subconscious trust signaling.

⸻

Interaction Rules
	•	No buttons on this screen
	•	Any key or click advances
	•	No backwards navigation
	•	Screen reader announces:
	•	“Dystrail loading”
	•	“Ready. Press any key to begin”

⸻

SCREEN 2 — MAIN MENU

Purpose

Present clear, sober choices. This is not playful. It is calm and ominous.

The menu should feel like standing at the trailhead.

⸻

DaisyUI Component Breakdown

Root Container

Same as Screen 1 for continuity.

⸻

Menu Panel

Component: card
	•	Same size and styling as Boot panel
	•	Ensures visual continuity

⸻

Title Header

Component: div
	•	text-2xl font-bold
	•	Centered
	•	No animation

Optional subtitle:
	•	text-xs opacity-60

⸻

Menu Options

Component: vertical button stack

Each option is:
	•	button.btn.btn-ghost
	•	Full width
	•	Left-aligned text
	•	Square corners

Spacing:
	•	space-y-1

Hover/focus:
	•	Background highlight only
	•	No scaling or motion

⸻

Primary Action — Start Journey

Special treatment:
	•	Still btn-ghost
	•	Slightly brighter text
	•	Optional ▸ indicator using text, not icon

This preserves OT’s understated emphasis style.

⸻

Footer Flavor Text

Component: div
	•	text-xs opacity-50
	•	Center aligned
	•	Example copy:
	•	“A parody survival game”

⸻

Navigation Rules
	•	Up/down arrow keys move focus
	•	Enter activates
	•	Escape does nothing here

Tab order:
	1.	Start Journey
	2.	About the Trail
	3.	Accessibility & Settings
	4.	Quit

⸻

Accessibility Notes
	•	Each menu item is a real <button>
	•	aria-current applied on focus
	•	Screen reader announces item + position
	•	“Start Journey, 1 of 4”

⸻

Intentional Omissions

These are deliberately excluded:
	•	Icons
	•	Images
	•	Sound
	•	Animations

Those come later. This screen must breathe.

⸻

Summary

These two screens set:
	•	Pace
	•	Trust
	•	Nostalgia
	•	Emotional gravity

If these feel right, the rest of the game inherits that confidence.

⸻

If you want, next logical docs are:
	•	Screen 3–5 (Persona, Mode, Store)
	•	Router + state transition spec
	•	DaisyUI theme token definitions

⸻

GLOBAL SCREEN INVENTORY & NAVIGATION HIERARCHY

This section defines all screens in Dystrail, their purpose, and how they connect. This is intended to drive:
	•	Router design (Yew Router)
	•	Navigation guards
	•	Allowed transitions (no free-roaming UI)
	•	Clear separation between meta, setup, journey, and terminal states

This is deliberately exhaustive but shallow — interaction details live in per-screen specs.

⸻

SCREEN GROUPS (TOP-LEVEL)

Dystrail screens fall into four navigation domains. Transitions are one-way only between domains.
	•	Meta / Shell — outside any run
	•	Run Setup — before the journey begins
	•	Journey Loop — repeated core gameplay
	•	Terminal — irreversible end states

⸻

1️⃣ META / SHELL SCREENS

These screens exist outside any run. No seed or game state is active.
	•	S1 — Boot / Title / Loading
Entry: App start
Exit: Main Menu
	•	S2 — Main Menu
Entry: Boot
Exit: Persona Select, About, Settings, Quit
	•	S2a — About the Trail
Entry: Main Menu
Exit: Main Menu
	•	S2b — Accessibility & Settings
Entry: Main Menu
Exit: Main Menu

Rules:
	•	No gameplay state exists here
	•	Settings persist across runs
	•	Escape always returns to Main Menu

⸻

2️⃣ RUN SETUP SCREENS

These screens define a single deterministic run. Once passed, the run is considered live.
	•	S3 — Persona Selection
Entry: Main Menu
Exit: Mode Select
	•	S4 — Mode Select
Entry: Persona Selection
Exit: Outfitting Store
	•	S5 — Outfitting Store
Entry: Mode Select
Exit: Journey Start (Travel)

Rules:
	•	Back navigation is allowed within setup only
	•	Seed is generated at Mode Select
	•	Leaving Setup always enters the Journey Loop

⸻

3️⃣ JOURNEY LOOP SCREENS (CORE GAME)

These screens repeat in a controlled loop until a terminal condition occurs.

Primary Loop Spine
	•	S6 — Travel / Status (Hub Screen)
Entry: From Setup or any Journey sub-screen
Exit: Encounter, Crossing, Camp, Map, Inventory, Pace/Diet

This is the hub screen of the entire game. All non-terminal gameplay flows through here.

⸻

Secondary Journey Screens

These screens temporarily replace Travel and then always return to it.
	•	S7 — Encounter
Triggered from: Travel
Returns to: Travel
	•	S8 — Crossing / Obstacle
Triggered from: Travel
Returns to: Travel
	•	S9 — Camp
Triggered from: Travel
Returns to: Travel
	•	S10 — Inventory / Supplies
Triggered from: Travel
Returns to: Travel
	•	S11 — Pace & Diet
Triggered from: Travel
Returns to: Travel
	•	S12 — Map / Progress
Triggered from: Travel
Returns to: Travel

Rules:
	•	No screen here can start a new run
	•	No screen here can exit directly to Main Menu
	•	Escape always returns to Travel (never to Menu)

⸻

Boss Sub-Loop (Late Game)
	•	S13 — Filibuster Boss
Entry: Travel (final region reached)
Exit: Result Screen

Rules:
	•	No escape back to Travel once boss begins
	•	Boss phases are internal state, not separate routes

⸻

4️⃣ TERMINAL SCREENS

These screens end the run. Gameplay cannot resume.
	•	S14 — Victory
Trigger: Boss defeated
Exit: Main Menu
	•	S15 — Failure: Pants Emergency
Trigger: Pants ≥ threshold
Exit: Main Menu
	•	S16 — Failure: Sanity Collapse
Trigger: Sanity = 0
Exit: Main Menu
	•	S17 — Failure: Resource Collapse
Trigger: Supplies or HP = 0
Exit: Main Menu
	•	S18 — Failure: Boss Loss
Trigger: Defeated during boss fight
Exit: Main Menu

Rules:
	•	Run state becomes read-only
	•	Share Code is visible and copyable
	•	Only forward action is return to Main Menu

⸻

NAVIGATION GRAPH (SIMPLIFIED)

```text
Boot
  ↓
Main Menu
  ↓
Persona Select
  ↓
Mode Select
  ↓
Outfitting Store
  ↓
Travel ────┬── Encounter ──┐
           ├── Crossing ───┤
           ├── Camp ───────┤
           ├── Inventory ──┤
           ├── Pace/Diet ──┤
           └── Map ────────┘
             ↓
        Filibuster Boss
             ↓
        Result Screen
             ↓
          Main Menu
```

⸻

ROUTER IMPLICATIONS (CODE-LEVEL)
	•	Exactly one active route at all times (no stacked modals)
	•	Travel is a hub screen, not a layout wrapper
	•	Setup screens use a setup-only state machine
	•	Journey screens use a run-locked state
	•	Terminal screens forbid state mutation

This hierarchy prevents:
	•	Accidental state corruption
	•	UI escape hatches
	•	Ambiguous back navigation

⸻

WHY THIS MATTERS

Oregon Trail works because:
	•	The player always knows where they are
	•	Navigation is restricted but predictable
	•	Screens feel like places, not UI fragments

This inventory preserves that mental model while remaining web-native.

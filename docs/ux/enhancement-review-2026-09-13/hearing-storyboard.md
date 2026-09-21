**Dystrail: arrival-to-verdict storyboard**

Implementation update, September 13: the approved hearing is now built locally. See the [implemented review and accepted campaign balance](implementation/hearing/README.md). The material below records the design review that preceded implementation.

Intended experience for the accepted staged-hearing goal. This is a design storyboard, with illustrative rolls and draft dialogue; it is not a capture of implemented gameplay. It uses the working model of 50% continuation checks, 50%–150% round influence, and 2 sanity per actual round. See the [hearing model](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/hearing-model.md) for the calculation and balance checks.

The experience should move through **arrival → preparation → hope → uncertainty → strain → relief → verdict**. The journey establishes the starting odds. The hearing gives those odds a visible, suspenseful resolution. The crew remains the emotional center of the scene.

**The main storyboard: one possible three-round hearing**

This example begins with 55% starting odds and 7 sanity. Its round influence values are 120%, 90%, and 150%; both continuation checks call another round. The average is 120%, giving a 66% final vote chance and leaving 1 sanity. An illustrative final draw of 42 produces a win. These example values demonstrate the flow, not an outcome promised to every player.

![Eight-frame hearing storyboard](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/storyboard/hearing-main-journey.png)

| Beat | What the player sees and hears | What the player does | What they understand and feel |
|---|---|---|---|
| **01 · Arrival** | The familiar van parks outside the D.C. hearing building. Surviving travelers gather their folders. Road noise gives way to the building's subdued atmosphere. “You made it. Now make them listen.” Light matches the actual game time. | Watch a brief arrival shot, or skip it. The preparation screen follows automatically. | Reaching the destination is an accomplishment, and the final challenge is ahead. Relief turns into anticipation. |
| **02 · Prepare** | In the anteroom, the crew sorts its evidence and takes a breath. The interface shows **Starting vote odds: 55%**, **Sanity: 7**, and “One to three rounds. Each round costs 2 sanity.” A short readiness line says this crew can withstand all three. | Choose **Rest before the hearing**, using the existing rest flow, or **Begin hearing**. Review fuller explanations on demand. | Preparation still matters. This is the final opportunity to improve condition before committing. Starting odds are not presented as the overall chance after exhaustion and hearing influence. |
| **03 · Opening argument** | Cut to a wide chamber view, then the speaking traveler and paper evidence. A short satirical exchange resolves into a favorable reaction. **Round influence: 120%** appears; sanity visibly changes **7 → 5**. “If the hearing ended now: 66%.” | Read the result, then **Continue**. The performance is revealed automatically during the scene, with no timing challenge. | The first round helps, but has a real cost. Hope rises. Only this round's cost and result are visible. |
| **04 · Another round?** | The chair consults the committee. A short pause ends with: “The committee calls a follow-up.” The next round indicator becomes active. | No separate click for the continuation check. The next round follows from the previous Continue action. | The committee determines whether the hearing continues. The check itself costs no sanity and changes no influence. Uncertainty returns. |
| **05 · Follow-up** | A skeptical question interrupts the crew's momentum. A traveler searches a folder while a companion steadies them. **Round influence: 90%**; sanity **5 → 3**. The running average is 105%, so provisional odds fall to **57.75%**, displayed as approximately 58%. | Read, then **Continue**. The second continuation check occurs: in this example the chair calls the final round. | An unfavorable round weakens the case. The crew can survive one more round, but will be nearly spent. Tension comes from both odds and condition. |
| **06 · Final challenge** | An exhausted traveler produces the crucial receipt. The clerk finally enters it into the record. **Round influence: 150%**; sanity **3 → 1**. The three-round average becomes 120%. | Watch and read the result. There is no fourth-round check. | The case recovers, at a visible personal cost. The extra round was neither automatically beneficial nor free. |
| **07 · Hearing closed** | “No further questions.” The chair sets down the gavel and the crew exhales. The interface settles on **Starting odds 55% × hearing influence 1.20 = final odds 66%**. Sanity remains 1. | Choose **Reveal verdict**. This begins the final presentation; it does not change the stored odds or draw. | The hearing is over. The final chance is now fixed, and the remaining uncertainty is the vote. Relief gives way to one last moment of suspense. |
| **08 · Verdict** | The clerk reads the decision: **Motion passes**. Let the crew react in the chamber before showing a statistics screen. A brief, specific closing line acknowledges their evidence and exhaustion. | Choose **View scorecard** after the outcome lands. | The player gets emotional closure first, then an explanation of how the journey, rounds, and vote produced it. |

Draft round dialogue, for tone rather than final localization:

- Opening: “Your receipts survive the first attempt to misfile them.”
- Follow-up: “The chair objects: those facts were not in the briefing.”
- Final challenge: “A receipt is entered into the record. Accidentally.”

Keep the satire aimed at the institution. The crew's exhaustion should be treated as a cost of the ordeal, not the punchline.

**The branches that make this a variable hearing**

![Four closing states](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/storyboard/hearing-endings.png)

The main strip illustrates three rounds; it must not make three rounds seem mandatory. Label progress **“Round 1 · Up to 3”**, with later rounds marked as possible, rather than “Round 1 of 3.”

| Branch | Scene and transition | Information the player needs |
|---|---|---|
| **The committee closes after round 1** | Beat 04 resolves as “The committee has heard enough.” Go directly to Hearing closed, then the appropriate ending. | With the example's first result, average influence is 120%, final odds are 66%, and sanity is 5. There are no missing-round penalties. |
| **The committee closes after round 2** | After the second result, the chair closes questioning. Skip the final challenge and proceed to Hearing closed. | Average only 120% and 90%: 105% influence, 57.75% nominal final odds, sanity 3. The interface should not imply that an unplayed round was lost. |
| **A · Final vote passes** | The clerk reads the favorable decision; the crew has a brief relieved celebration. | Show the fixed final odds and the passed result. Do not invent individual committee vote counts that the game never calculated. |
| **B · Final vote fails** | The room falls quiet; the crew gathers its folders. “The committee thanks you for participating in its decision not to act.” Then offer the scorecard. | A strengthened case can still lose. In the main example, a final draw of 78 fails against 66% odds. Distinguish this from exhaustion or a missed score target. |
| **C · Automatic victory** | After questioning closes, show **“Victory secured”** and let the clerk announce the favorable decision directly. No suspenseful final draw is performed. | Separate example: starting odds 80%, round values 140/130/120%, average 130%, adjusted odds 104%, sanity 7 → 1. Cap the actual probability at 100%; the explanation may show how it exceeded that value. |
| **D · Exhaustion** | A round's sanity cost reaches zero. The speaker cannot continue; companions support them and the hearing ends. Use a distinct exhausted ending, not a “vote lost” announcement. | Example: entering with 5 sanity, a third round causes 5 → 3 → 1 → 0. Stop immediately. There is no continuation check, final vote, or automatic victory after exhaustion. |

Automatic victory is checked only after the hearing closes. A provisional 100%+ result during an earlier round does not protect the player from a later unfavorable round or exhaustion. An existing policy guarantee remains conditional on surviving the hearing and uses the same automatic-victory presentation, with an accurate explanation of its source.

**Controls and pacing**

Default proposal: hold after meaningful round results so the player can read; **Continue** releases the committee check and the next round. Do not require additional clicks for a gavel, each number reveal, and each continuation roll. After the final actual round, replace Continue with **Reveal verdict**; when victory is already secured, show the secured result instead of offering a fake random vote.

Rest is a real preparation decision. After Begin hearing, there are no unapproved tactical choices, spend-to-reroll actions, or timing bonuses. Reading faster, clicking at a particular moment, or skipping does not improve the outcome. Player-controlled reveals provide pacing engagement; approaches or resource trades between rounds would be a separate mechanic.

Suggested presentation budget, excluding player reading and rest:

- Arrival: approximately 2–3 seconds, skippable.
- Each round: approximately 2–3 seconds of question/reaction, then a readable result hold.
- Each continuation announcement: approximately 1–2 seconds, with no separate input.
- Closed hearing and verdict: approximately 3–5 seconds combined, with the deliberate verdict reveal when a final vote exists.
- Aim for roughly 15–25 seconds of animation in a full three-round hearing, plus player-controlled reading time. Fast mode shortens transitions. Skip presents the complete, ordered result summary without changing anything.

These are pacing targets for playtesting, not measured timings. Avoid a long unskippable finale on repeated runs. Sound cues can reinforce arrival, a round result, committee continuation, closure, and verdict; all information must remain understandable with sound off.

**What should remain visible**

During the hearing, focus the status area on current sanity, actual round number, the recorded influences, and the current average. Clearly distinguish **starting odds**, **provisional odds if questioning ended now**, and **final odds**. A 120% round influence is a multiplier contribution, not a 120% chance of winning. Each round counts equally in the average; a favorable result below an earlier exceptional result can lower the running average.

Do not expose future rounds or deduct all possible sanity in the visible HUD at the start. A restored hearing resumes the presented beat with its matching stats. Menu use, browser refresh, Fast, Skip, and reduced motion read the same saved outcome. The full result can be resolved and stored in advance, but the scene reveals only what has happened in the player's presentation.

Keep the active roster and identities consistent across arrival, benches, microphone shots, reactions, and endings. Do not resurrect an absent companion for a crowd shot. The figures in the concept sheets illustrate the intended feminine/androgynous and racially diverse cast direction; they are not approved production character sheets.

Lighting follows the actual game time. The example uses daytime arrival and coherent interior light. A later arrival or rest can change the exterior/ambient treatment; simply waiting on a result does not turn day into night. This storyboard introduces no additional game-time cost.

**Accessibility and implementation handoff**

Use focused headings and a polite announcement of each newly revealed result. Move keyboard focus to the next meaningful action; do not repeatedly jump focus during animation. Provide text equivalents for committee reactions, color changes, and sounds. Reduced motion uses brief cuts and the same ordered results. Keep essential text readable on phones and allow translation expansion; avoid forcing dialogue into small speech bubbles.

Needed production pieces: an arrival/anteroom composition, a chamber master view, speaker and committee reaction views, three round contexts with favorable/neutral/unfavorable variants, two continuation-or-close announcements, four closing states, readable state controls, and audio cues if sound is included. Reuse the existing civic/ending settings and the approved cast wherever possible. The concept sheets explore the staging; they are not atlas-ready game assets.

Playtest the sequence as a game experience: can a first-time player explain why another round happened, what sanity cost them, how a 120% influence differs from vote odds, and why the hearing ended? Can a repeat player skip without losing the explanation? Check one, two, and three rounds; low-sanity entry and exhaustion; rising/falling averages; automatic victory; vote win/loss; and reload at every reveal.

Success is a finale the player can follow emotionally and mechanically: their arrival is acknowledged, preparation has meaning, the committee's demands create tension, each round has a legible consequence, and every ending has its own moment before the scorecard.

Production references: [generation prompts](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/storyboard/prompts.md), [illustrative state examples](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/storyboard/state-examples.json), and [art provenance](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/storyboard/assets.json). The two concept sheets were created with the built-in image-generation tool and visually inspected; the main sheet received a targeted character-art correction.

# Scene and shared-surface image decisions

Direct inspection of six retained atlases against the current workshop. These are proposed image decisions, not accepted scenes. Runtime is unchanged. All changes still require a search of other retained art before a new image is declared necessary. No generation is authorized.

**Exact geometry:** ally sheets are 1254 × 1254, 2 columns × 3 rows, cells 627 × 418. Care sheets are 1086 × 1448, 2 columns × 4 rows, cells 543 × 362. Cell order is row-major by numbered family, with separate A/B/C sheets. Preserve native aspect ratios; the historical requested sizes are not the actual PNG sizes.

**Composition rules:** reuse the cell camera and uniformly scale the assembled scene. Changed cells need one eye-level medium-wide view, believable person/furniture/vehicle relationships, established pixel-art treatment, and blank editable lettering surfaces. Keep every required story prop visible and the caption over the lower-left. Same source on desktop/mobile; no separate mobile generation. External allies never become crew. Care scenes use the saved affected traveler and actual surviving party.

**Existing standalone alternatives inspected:** CARE-03-A diner and CARE-08-A memoir originals are both 1536 × 1024. They preserve their core story props but still require correct crew compositing and, for the diner, source-appropriate night presentation. These alternatives do not resolve the other mismatched cells.

| Record | Decision | Inspection finding |
|---|---|---|
| ALLY-01-A | reuse with layers | Red hats and ring light match. Replace the inset hat/heart graphic with the localized goodbye-message presentation. |
| ALLY-01-B | change | Straw-hat stall replaces plain hat cartons, customs bill and crew phone request. |
| ALLY-01-C | change | Route map and lunch replace enormous donor microphone and disconnected small microphone. |
| ALLY-02-A | reuse | Contact carries belongings past an empty service counter; keep external to traveling party. |
| ALLY-02-B | change | Opposing arrow sheets replace disconnected headset and packed belongings. |
| ALLY-02-C | reuse | Packed belongings and a second occupied desk match broad staging; source copy establishes the relationship. |
| ALLY-03-A | change | Bank/mailbox lobby replaces required garage invoice, apron and overnight-shift alarm. |
| ALLY-03-B | change | Cash-register scene lacks the medical bill under a second-job timecard. |
| ALLY-03-C | change | Apron is present but groceries look abundant, contradicting sparse food budget. Preserve identity; revise bag/second-job context. |
| ALLY-04-A | change | Gaming classroom replaces rival-color merchandise tables and paired card readers. |
| ALLY-04-B | reuse | Identical coastal souvenirs with reversible blank tags match. |
| ALLY-04-C | change | Miniature objects replace laptop, monthly calendar and theatrical countdown. |
| ALLY-05-A | reuse | Phone and useless physical padlock match; localize any visible message. |
| ALLY-05-B | reuse with layers | Milk bottles and ignored phone match; cover baked notification symbols with the actual localized message. |
| ALLY-05-C | change | Football trophy and award replace discarded receipt and blocked phone message. |
| ALLY-06-A | change | Industrial shoulder injury replaces injured ankle, closed ramp and privileged entry. |
| ALLY-06-B | change | Machine-shop consultation replaces cleaning equipment and empty service desk. |
| ALLY-06-C | change | Sore back and ice pack replace sore shoulder, speaker handcart and distant donor stagehands. |
| CARE-01-A | reuse with layers | Vendor, bottle and portable toilet match; add actual affected traveler and small nonviolent knife-shaped bottle shadow. |
| CARE-01-B | reuse with layers | Unplugged fridge and wash station match; preserve clearly visible cable and add affected traveler. |
| CARE-01-C | change | Milk stall partly matches, but second sealed bottle is not distinctly separated and baked vehicle does not establish the actual ill crew member beside their van. |
| CARE-02-A | reuse with layers | Shaded boss, blank board, crates and heat match; add actual dehydrated traveler at compatible scale. |
| CARE-02-B | reuse with layers | Shaded cooler, supervisor glass and crates match; add affected traveler and preserve denied access staging. |
| CARE-02-C | reuse with layers | Rally equipment and private ice cooler match; add actual heat-exhausted traveler in shade. |
| CARE-03-A | reuse with layers | Diner/forms/glove match. Prefer standalone diner original; actual traveler behind table and night lighting limited to the window/exterior as source requires. |
| CARE-03-B | reuse with layers | Former worker with blank paper and laptop match; layer affected traveler at the same table, not over the worker. |
| CARE-03-C | reuse with layers | Application desk and child in executive-chair portrait match; add affected adult traveler across the desk. |
| CARE-04-A | change | Giant drink and fruit replace tonic bottles, under-counter bucket and loyalty card. |
| CARE-04-B | change | Cafe and pointing vendor replace wellness display, powder sachet and magnifying glass. |
| CARE-04-C | change | Snack table with pamphlet replaces supplement blame gesture and flipped guarantee card. |
| CARE-05-A | reuse with layers | Matching route maps and loose labels match; add actual exhausted traveler and editable label text. |
| CARE-05-B | change | Wallet/cards at bus shelter replace paper and digital maps with matching mountain silhouette. |
| CARE-05-C | change | Electronics recycling scene replaces phone route loop and repeated gas-station statue. |
| CARE-06-A | change | Laptop/router in picnic shelter replaces garage cashier, card reader, napkin and parts box. |
| CARE-06-B | change | Hand-print forms replace invoice, empty wallet and returning arrows. |
| CARE-06-C | change | Long office forms replace two vibrating handsets and parts invoice inside crew van. |
| CARE-07-A | reuse with layers | Laptop and all five required work objects match; add affected traveler without covering the objects. |
| CARE-07-B | reuse with layers | Delivery worker, pizza and phone match; needs actual crew-in-van composition and matching editable notifications. Baked delivery vehicle must not impersonate crew van. |
| CARE-07-C | change | Van/checklist scene replaces oversized phone padlock and piled notification tiles. |
| CARE-08-A | reuse with layers | Memoir crates, donor guest and podium match. Prefer standalone original; treat baked cargo van as vendor vehicle and place actual traveling crew/van only if coherent scale fits. Otherwise return composition to review. |
| CARE-08-B | reuse with layers | Open-bottomed box, photographer and groceries match; place affected traveler visibly next to heavy groceries. |
| CARE-08-C | change | Museum exhibit and active hand truck replace sacks, unused hand truck and volunteer inspecting ordinary dry ceiling. |

The JSON companion at `review/recovery/scene-cell-review.json` records each exact original path, hash, source direction, crop rectangle, camera/proportion constraint and remaining gate. The later retained B/C ally corrections were inspected; older similarly named outputs are not interchangeable.

**Result:** 6 ally cells are reuse candidates (2 need overlay work); 12 need changed artwork or another suitable retained source. Thirteen care cells are reuse-with-composition candidates; 11 need changed artwork or another suitable retained source. These counts are inspection decisions, not production coverage.

## Policy announcements and repair components

Policy originals are **1536 × 1024**, with 2 columns and 3 rows. Native row bounds are 0, 341, 682, 1024; each cell is 768 pixels wide. Preserve these wide cells without stretching them into the 3:2 care/ally format. Repair component originals are **1536 × 1024**, 2 columns × 4 rows, each **768 × 256**: failed left, repaired right. These are detail insets, not scene-sized physical objects.

The policy candidate set uses A `exec-a197569b-2e91-4720-81d0-47072ff12f03.png`, B `exec-8b1f64a8-a928-4926-af4b-6382ee788b26.png`, C `exec-f37982a1-f430-41ce-b7d4-85a1c55e3d5d.png`. B is the inspected coin-return correction, regardless of its misleading historical filename. Repair details use `exec-93f38f07-11f3-402c-bdf7-b9bee6608428.png`.

The retained inspection-pose sheet is **not accepted** merely because historical metadata says it has alpha. Its smooth shading/glow needs visual review against the required pixel-art standard. Reuse the established cast poses where appropriate before proposing a replacement. No generation authorized.

| Record | Decision | Inspection finding |
|---|---|---|
| ORDER-DEREGULATE-A | reuse with layers | Renaming ceremony on roadside display matches; actual stopped van/open hood still needs composition at coherent scale. |
| ORDER-DEREGULATE-B | reuse with layers | Pristine delivered sign and neglected machinery match; add actual traveler and van only if full composition fits. |
| ORDER-DEREGULATE-C | reuse with layers | Military-name unveiling on display matches; add actual crew beside their current van, not baked placeholder occupants. |
| ORDER-GAG-A | reuse | Corrected gray-haired librarian, locked cart and reading sheet match. Use latest retained correction, not earlier head-covering version. |
| ORDER-GAG-B | reuse with layers | Later retained correction moves the extra cardboard column from the snack pickup opening to the small coin-return area. Add only actual present travelers, with no purchase/retrieval action. |
| ORDER-GAG-C | reuse | Leaking hallway, two buckets and dry oversized binder match. |
| ORDER-MILITARIZE-A | reuse with layers | Barrier, officer and checklist match; actual van must wait behind the barrier with compatible road perspective. Assembled camera remains unverified. |
| ORDER-MILITARIZE-B | change | Retained cell is an indoor service counter, not the specified traveler/documents/van checkpoint. Existing A/C checkpoint cells are alternatives only if exact composition fits. |
| ORDER-MILITARIZE-C | reuse with layers | Domestic road, booth and barrier match; add actual traveler pointing across the same landscape without changing scale or stage. |
| ORDER-SHUTDOWN-A | reuse | Closed counters, worker lunch bag and lit fundraiser match. Worker is an external NPC, not a substitute crew member. |
| ORDER-SHUTDOWN-B | reuse with layers | Dark monitors and one bright donation screen match; replace baked heart/progress graphic with editable localized presentation. |
| ORDER-SHUTDOWN-C | reuse | Empty plate and oversized chairs match an editorial cutaway. Do not imply actual crew attendance. Inspect small flag finials during final symbol review. |
| ORDER-TARIFFS-A | reuse with layers | Grocery bag, empty wallet and invoice match; make any identifying customs copy an editable overlay. |
| ORDER-TARIFFS-B | reuse | Cashier, long receipt and modest loaf match; keep receipt wording editable. |
| ORDER-TARIFFS-C | reuse with layers | Groceries and tall blank price panel match; add actual traveler reaching, with the panel visibly intervening. |
| ORDER-TAXCUTS-A | reuse with layers | Empty education room, small lectern and illuminated blank exit panel match; localize any sign text. |
| ORDER-TAXCUTS-B | reuse with layers | Closed education office, unstaffed counter and blank jar match; localize fee label. |
| ORDER-TAXCUTS-C | reuse with layers | Teacher, short pencil, empty board and blank notice match; keep notice editable. |
| REPAIR-ALTERNATOR-A | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-ALTERNATOR-B | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-ALTERNATOR-C | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-BATTERY-A | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-BATTERY-B | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-BATTERY-C | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-FUELPUMP-A | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-FUELPUMP-B | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-FUELPUMP-C | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-TIRE-A | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-TIRE-B | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |
| REPAIR-TIRE-C | reuse component composition incomplete | Failed and repaired component close-ups are available. They do not by themselves fulfill the source-specific traveler, van and prop staging. Previously rejected opaque/checkerboard prop and vehicle sheets remain excluded. |

Repair guard checked against surviving code: `choose_repair` must return true before the app publishes aftermath. The historical renderer tests only establish ID/choice mapping and cannot independently prove a successful repair. Restore the caller guard with the renderer; preserve current simulation behavior.

This adds 18 policy decisions and 12 component/recipe decisions. It does not certify the composed scenes, authorize image production, or complete the full inventory.

## Functional references: preserve existing surfaces

This **approval proposal** accounts for all 77 functional records without commissioning 77 illustrations. It preserves the user's approved interface and avoids adding a new illustrated town-profile design. The source vignette suggestions remain preserved in the ledger; they are not represented as completed art.

| Records | Count | Proposed image decision |
|---|---:|---|
| PROFILE-01 through PROFILE-51 | 51 | Existing town-profile text surface; no new landmark painting or illustration card. |
| SHOP-* | 11 | Existing `items/*-v1.png` cutouts, each exactly 1254 × 1254; same icon/card layout on desktop and mobile. |
| PERSONA-* | 6 | Previously inspected 1774 × 887 cast sheets in existing selector. |
| CONTEXT-* | 7 | Existing origin/arrival surface, actual regional background and actual party; no extra landmark painting. |
| MODE-C / MODE-D | 2 | Existing selector with shared road/van and mode presentation; exclude old baked-crew images. |

PROFILE-17 proposes Frederick church spires and PROFILE-38 proposes San Antonio mission architecture. Those visual suggestions conflict with the user's explicit no-religious-symbols rule and are excluded; the factual text remains independently reviewable.

Additional road originals inspected for reuse: `open-california-hills.png`, `open-southwest-desert.png`, and `open-beltway-parkway.png`, each 1536 × 1024. Preserve their geography: a desert mesa landscape is not an interchangeable coastal-city establishing shot. No religious symbols were observed in these three inspected images. Final scene composition and localization still require review.

## Activities and crossings

36 further source-to-image proposals. Earlier barter sheets preserve all three item families; the newer refresh is tire-only. Crossing originals have improved wide framing but remain on style, geography and composition review hold. No new image is authorized.

Native canvases are 1536 × 1024. Barter uses two columns with 760px content widths and recorded nonuniform row bounds; gathering/work candidates use 768 × 256 cells, subject to final cell-edge inspection. Rest props use 768 × 341/342 cells. Crossings use the whole 3:2 source. Exact coordinates, hashes and source directions are in the JSON ledger. Preserve actual crew, bottom-left captions and approved stats.

| Record | Decision | Inspection finding |
|---|---|---|
| ACT-BARTERTIRE-A | reuse with layers | Intact tire, grocery bundle and radio match the essential trade. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERBATTERY-A | reuse with layers | Battery, groceries and radio match; resident is behind the table rather than holding the battery. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERSUPPLIES-A | reuse with layers | Groceries opposite intact spare, radio nearby. Before-cell goods remain unexchanged. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERTIRE-B | reuse with layers | Intact tire and groceries match. Add actual negotiating traveler, not a fixed crew portrait. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERBATTERY-B | reuse with layers | Battery and groceries match; an unopened envelope can represent the shop quote without generated lettering. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERSUPPLIES-B | reuse with layers | Tire and groceries match. Two current travelers must be composed only if they fit the native camera. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERTIRE-C | reuse with layers | Flat spare tire conveys the practical conference-table joke. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERBATTERY-C | reuse with layers | Battery, radio and blank leaflets match the essential trade. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-BARTERSUPPLIES-C | reuse with layers | Intact tire, groceries and radio match the essential trade. Use the right cell only after a successful completed trade, not merely after a click. |
| ACT-CASHWORK-A | candidate missing gag | Dirty dishes match the job; empty tip jar and owner offer are absent. Do not show clean completed dishes before accepting work. |
| ACT-CASHWORK-B | candidate context mismatch | Memoir donor/podium scene may supply context, but stage crates and untouched lifting job are not verified. Avoid substituting book selling for paid setup work. |
| ACT-CASHWORK-C | retained candidate unresolved | No verified retained delivery truck, damaged access road and untouched crates composition. |
| ACT-FOODWORK-A | reuse with layers | Pantry coordinator, unsorted donations and sorted outgoing groceries match broad staging. Verify suspicious plus-like carton-top marks against the no-symbol rule at final size; do not assume approval. |
| ACT-FOODWORK-B | candidate missing gag | Kitchen and takeaway container match broadly; source specifies untouched catering trays, while candidate emphasizes dishwashing. |
| ACT-FOODWORK-C | candidate context mismatch | Dirty-pan job does not depict an empty cash drawer, sorting bench or grocery invoices. |
| ACT-FORAGE-A | reuse with layers | Guide, berries and empty basket fit the activity. Required blank phone-form prop is absent; actual travelers remain runtime layers. |
| ACT-FORAGE-B | candidate missing gag | Botanist context fits, but two clearly different plants are not established by the retained cell. |
| ACT-FORAGE-C | candidate missing gag | Foraging setting lacks the distant fenced catered reception central to this contrast. |
| ACT-GLEAN-A | reuse with layers | Farmer, tractor and unharvested carrots match. Machinery invoice still needs a coherent prop placement. |
| ACT-GLEAN-B | candidate context mismatch | Labeled crate does not establish the required radio and benefits notice. |
| ACT-GLEAN-C | candidate context mismatch | Corn and baseball substitute a different joke for the conspicuously bent carrot. |
| ACT-REST-A | reuse props composition incomplete | Keys and radio can supplement actual crew at the parked van; source calls for keys in a traveler hand, not merely on a table. Check alpha and colored edge halo on the real backdrop; no automatic image edits. |
| ACT-REST-B | reuse props composition incomplete | Radio and blanket can supplement actual crew at a modest rest stop. Localized donor broadcast stays separate editable content. Check alpha and colored edge halo on the real backdrop; no automatic image edits. |
| ACT-REST-C | reuse props composition incomplete | Use the face-down phone cell (right), add wrapped food and actual resting crew. Do not display a face-up phone as the final state. Check alpha and colored edge halo on the real backdrop; no automatic image edits. |
| CROSS-01-A | retained candidate unresolved | No verified officer/open hood/ordinary alternator scene. The crouching inspection image is a different vehicle-plate story. |
| CROSS-01-B | reuse candidate review hold | Folded blank narrow map fits; use retained unfolded map as an existing alternate only if state warrants it. |
| CROSS-01-C | reuse candidate composition incomplete | Crouching officer and towering forms fit; actual van plate must meet the pointing/inspection position without inconsistent scale. |
| CROSS-02C-A | reuse candidate composition incomplete | Workers physically hold raised beam with a large blank roof sign. Beam is parallel to the road behind its shoulder, not a functioning cross-lane barrier. Do not animate the van underneath it as rendered. |
| CROSS-02C-B | reuse candidate review hold | Official with inspection sheet points to a concrete crack. Keep waiting van safely separate at plausible foreground scale. |
| CROSS-02C-C | retained candidate unresolved | No verified worker/steel quote/solitary cone/unfinished repair composition. |
| CROSS-02D-A | reuse candidate review hold | Separate woman asks the guard pointing toward a second camera. Both remain external to crew. |
| CROSS-02D-B | reuse candidate review hold | Separate traveler presents two matching portrait cards to monitor-facing guard. No readable private data or crew substitution. |
| CROSS-02D-C | reuse candidate review hold | Separate adult, unused chair and clipboard guard match; no treatment or distress depicted. |
| CROSS-03-A | candidate context mismatch | Two payment/document slots do not depict an official peering into the actual van or the required cash tray and case folder. |
| CROSS-03-B | reuse candidate review hold | Thick folder held beside thin booklet matches; crew van must wait without hiding the documents. |
| CROSS-03-C | reuse candidate composition incomplete | Papers on bucket in desk-less booth match. Waiting vehicle and any actual road barrier need coherent placement. |

## Road families C01–C08 and town 41

27 source-to-image decisions following direct inspection. All listed originals remain unchanged. Source revisions can invalidate historical filenames: town 41A is a concrete mismatch. Cross-like jewelry in the coast-chart atlas and a separate ally correction is excluded for user review, without an automatic correction attempt.

Exact native canvases, crops, hashes and outcome constraints are in the JSON ledger. C01 and C02 use nonuniform three-row atlases. C03–C08 use four-panel atlases; C08C needs its unequal row boundary corrected in layout. Every reuse decision still requires the assembled view to pass pixel style, surviving-crew, proportions, caption and localized-text review.

| Record | Decision | Inspection finding |
|---|---|---|
| ENC-C01-A | reuse with outcome bindings | Unused gloves and full bins match the offer; the right frame depicts cleaned bins, packed bags and food. |
| ENC-C01-B | reuse with outcome bindings | Switch, socks and salvage bin match. Localized machine rejection belongs on the blank display, not baked lettering. |
| ENC-C01-C | reuse with outcome bindings | Two identical blank sheets, opposing bins and shared shredder match; bin typography stays localized and editable. |
| ENC-C02-A | candidate gag review hold | The large political leaflet is at floor level but does not clearly read as folded beneath the short table leg. Preserve for user review rather than claiming the core physical gag is resolved. |
| ENC-C02-B | reuse with outcome bindings | Employment paperwork occupies a chair in the offer. The meal frame still leaves the chair occupied by papers, so its source-described cleared-chair result is not fully represented. |
| ENC-C02-C | reuse with outcome bindings | Ornate blank donor cards, large leftover meat portions and ordinary plates convey the source contrast. |
| ENC-C03-A | reuse with outcome bindings | Heavy desk, two parents at opposite ends, food and quiet corner match. Outcome cells distinguish donated groceries, recording the assembly diagram, and quiet reading. |
| ENC-C03-B | reuse with outcome bindings | Wet tent, elaborate ribbon, modest chair and collecting pots match broadly. In the offer the chair stands outside the roped dry area; do not describe that pose as already occupying the privileged dry corner. |
| ENC-C03-C | reuse with outcome bindings | Phone between two depleted casseroles, fresh casserole behind and open quiet-room door match. Driver and hosts are locals; never substitute them for crew. |
| ENC-C04-A | candidate gag review hold | Tilted lander and drill are present, but the drill rests separately rather than touching the table beneath the lander. Fine textured shading also needs pixel-standard review. |
| ENC-C04-B | reuse with outcome bindings | Lavish blank packaging surrounds disassembled ordinary stand parts. Four states distinguish untouched offer, hauling, assembly after donation, and recording with claim card turned down. |
| ENC-C04-C | reuse with outcome bindings | Canopy monopolized by sponsor, workers outside shade and cooler match. Corrected green roadside geography avoids the older desert mismatch. Outcome shade/donation/recording changes are distinct. |
| ENC-C05-A | reuse with outcome bindings | Held empty envelope and abundant correspondence match. Outcome cut-through envelope, record/food offer and shared reading must follow their actual action. |
| ENC-C05-B | reuse with outcome bindings | Muffin, oversized paperwork and paper airplane match. Keep drawn muffin-on-wheels outcome separate from the initial blank form. |
| ENC-C05-C | reuse with outcome bindings | Elaborate invitation clipped to bill and glove of coins match. Baked locals remain locals; payment envelope or food does not authorize a mechanical reward by itself. |
| ENC-C06-A | reuse with outcome bindings | Missing rung, loose rung and separate sturdy step stool match. Repaired-ladder and stool-demonstration cells represent different choices. |
| ENC-C06-B | reuse with outcome bindings | Small talking toy behind oversized desk, job board and folding interview stool match. No real traveler occupies the stool before the chosen scene calls for it. |
| ENC-C06-C | reuse with outcome bindings | Leak aligned with bucket, ambitious blank name card and mop match. Outcome tipped bucket is deliberately a puddle, not a completed roof repair. |
| ENC-C07-A | reuse with outcome bindings | Unplaced trophy, loose shelf and brackets between sports shelves match. Assembly, installed shelf/paid work and recorded conversation are separate outcomes. |
| ENC-C07-B | reuse with outcome bindings | Desks, absent dolly hook and blank job board match. Retained later opaque-correction candidates must be compared before selecting final bytes; source reference alone does not prove this atlas alpha is sound. |
| ENC-C07-C | candidate gag review hold | Freezer, demonstration ice, tray and puddle match the general job. Ice is on a separate stool beside the freezer, not underneath it as the source describes. Later opaque correction remains to be matched. |
| ENC-C08-A | exclude symbol review | Matching coast charts convey the unchanged-shoreline gag. A small pale pendant is cross-like and conflicts with the no-religious-symbol constraint; keep this candidate excluded pending user review, no automated retry. |
| ENC-C08-B | reuse with outcome bindings | Empty outdoor folding chair facing elaborate indoor place setting matches. Only outcomes move/unlock chair or collect evidence. |
| ENC-C08-C | reuse with outcome bindings | Toy paper crown, ordinary toll bill and two matching route maps match. Horizontal divider is near y=492 rather than 512; recovered equal-half renderer risks mixing neighboring cells. |
| TOWN-41-A | candidate context mismatch | Historical alias is stale: bus-stop worker with boot catalogue does not depict current print worker, postcard, envelope or supplier invoice. Do not restore this binding. |
| TOWN-41-B | reuse with crew composition | Cleaner searching old coat pocket, diner booth, meal plates and unlettered football TV match. Insert only actual surviving travelers behind the table edge; empty places may remain empty. |
| TOWN-41-C | reuse with crew composition | Groundskeeper, young vegetation, older industrial backdrop and closed sediment jar match. Keep actual travelers at the table; the worker remains a local. |

## Shared departures, conditions, Hearing and endings — 96 records

These records propose reusable context; their source-specific props and staging are still incomplete. They do not authorize production or imply 96 finished scenes. All source directions remain in the JSON ledger.

- **Departure (18):** Reuse the origin-appropriate road, masked van, retained cast and ordinary luggage. Journalist/organizer start in the Pacific Northwest; lobbyist/satirist/staffer/whistleblower use appropriate California geography. Current Setup defaults to Heartland and needs corrected binding. Six travelers at departure; props such as maps, folders, money, nameplate, breakfast and visible brake pedal still need a composition decision. No funds, food or mechanical condition are created by the artwork.
- **Condition (27):** Reuse the actual location plus clock/weather layers and affected active crew. Indoors receive no outdoor rain. Do not conjure shelter, water, coats, blankets or a leaking roof as a gameplay fact merely because a source direction suggests it. Distinguish atmospheric effects from optional editorial gags; literal source scenes with unavailable locations or props require review.
- **Hearing (6):** Reuse the retained empty courtroom candidate and compatible official/cast assets. Preserve the saved HearingReport and all existing phases, continuation rolls, sanity costs, probability and outcome. Six source vignettes are presentation variants, not six extra rounds or new mechanics. Menu, lunch, brochure, nurse, clerk, map and appeal-form staging remain unresolved. Official portraits are not full-body world sprites.
- **Ending (45):** Choose the recorded stopping place, cause, weather, time and present roster. Failure is not automatically a rest area, and success is not automatically a sunset. Show only the recorded damage; never invent crashes, fire, deaths, paid repairs or new rewards. Victory/vote-failure consume the saved Hearing outcome. Fallback uses a neutral incomplete-log scene and makes no claim about fate; incomplete runs remain nonterminal. Source editorial props/insets require review.

**Proposed common layout:** 1536×1024 master, 3:2 scene on desktop and mobile. Scale the whole scene uniformly; do not independently enlarge people or crop away the essential gag. Reserve bottom-left x=24..1000,y=850..1000 for the caption with a contrast scrim. At 390px viewport, wrap localized caption text without shrinking below the approved UI text size; increase the scene area if needed. No supplementary bust or redesigned stats.

**Perspective and state:** Only actual present travelers, stable persona identities and shared pose scale. Distinguish an editorial inset from world space; it must not assert an event, location, inventory item or outcome absent from state. No religious symbols. Source-specific missing props remain unresolved, not delivered.

The existing Hearing implementation consumes a committed report during animation. Art integration must not reroll or modify that report. The empty courtroom in retained original `exec-d40d6a47-2274-4dc8-9e74-0fcc5a357e42.png` is a context candidate (1536×1024 atlas, bottom-right 768×512 cell). Official atlas `exec-e0427b07-0f9a-422b-b0a3-0bc8cbd31ae9.png` is 1536×1024 with six 512×512 portrait cells; style and edge treatment need rendered review, and these are not full-body poses.

## Recovered alternatives and inactive dependencies

- **ENC-C07-B — retained original 100:** Classroom desks/dolly atlas, not shelf scene. Four 768×512 cells in 1536×1024; decoded PNG is RGB, hence fully opaque. Matching family correction recovered. Two baked local adults must remain external NPCs, not substituted for actual crew.
- **ENC-C07-A — retained original 159:** Shelf/trophy atlas, not classroom scene. Four 768×512 cells in 1536×1024; decoded PNG is RGB, hence fully opaque. Matching family correction recovered. Use completed shelf only after actual successful work; the single shopkeeper is an external NPC.
- **CARE-07-A — retained original 21:** 1536×1024 roadside laptop/table with keys, glove, brush, map and radio. No baked traveler; left actor area is empty. Reuse candidate. Table left edge about x660, top about y545; fit the affected actual traveler to its depth and contact surface. Laptop and props must remain visible; composition is not yet verified.
- **CARE-05-A — retained original 153:** 1536×1024 map table with central actor space; blank loose labels, thermos and duffel. Reuse candidate subject to comparison of both printed map routes: the source requires identical geography, which broad visual similarity alone does not prove. Table top spans roughly y510..820; mask the traveler behind its foreground, never float a portrait above it.
- **Shared context — retained original 141:** 1536×1024 care-room candidate with a feminine Black medic on the right and empty patient staging on the left. Plain outfit and room have no observed religious symbols. Medic visible extent approximately x895..1300,y110..900; actual traveler should share the floor plane near y900 with comparable adult scale. Do not auto-bind this room to outdoor or source-specific care incidents.
- **Shared context — retained original 170:** 1536×1024, twelve 512×256 cells (3 columns × 4 rows). Media workshop, community garden, bridge, service workshop, civic chamber, checkpoint, clinic, radio room, civic street, convoy/rest stop, dairy barn, night picnic. Previously targeted crosses/steeples are absent on visual inspection. Candidate contextual atlas only; it does not illustrate the remaining satire gags. Cell 11 has baked night and must not appear during daytime; bridge perspective differs from side-on travel van.

All 36 proposed-mechanics records now have an explicit **inactive / no production** decision. Their dimensions and placement are intentionally unset because no runtime scene is authorized. The remaining 300 active road/town records need a staging-scope decision and candidate reconciliation; source coverage is not finished artwork.

## Remaining source coverage — unresolved, not production-ready

All remaining 171 active road and 129 town records now explicitly retain their scene directions and mark staging unresolved. No matching inspected binding is asserted and no production request is created. Shared-context versus per-variant essential-gag coverage is awaiting the user’s staging decision. This completes source accounting, not the necessary-image inventory, artwork, or feature.

## Approved middle-ground policy applied to all remaining town records

The 129 town records outside the three already inspected TOWN-41 scenes are now classified: **120 dialogue-led shared-context scenes** and **9 essential visual compositions**. These are composition requirements, not nine automatic image-generation requests. Candidate selection, exact source dimensions/placement and production approval remain outstanding.

- **TOWN-14-B:** The owner dressing a refrigerator in a necktie is the central visible action. Keep refrigerator, tie and owner hand contact readable in one repair-shop composition; recover compatible props/poses before specifying generation.
- **TOWN-34-C:** Inspection of the wallet is the central visual contrast. Show the mechanic aiming a small inspection mirror into an open wallet, with the vehicle untouched. This requires a compatible hand/prop composition, not an oversized wallet or miniature van.
- **TOWN-01-A:** A small license-plate-shaped scrap held over ordinary shoes is the visual premise. Keep both at plausible hand/foot scale; no actual plate installation, visible identifier, or invented road hazard.
- **TOWN-03-C:** A person feeding a parking meter before eating is the visible contrast. Show coin approaching the slot and lunch bag in the other hand; ordinary meter height, no human face or mouth added to the pole.
- **TOWN-14-A:** The uncertain fare-card approach to a free streetcar is the core gesture. Card at normal hand scale, open level doorway and clerk welcoming the visitor; do not depict a literal horse or charge a fare.
- **TOWN-23-C:** The ID nearly entering the wash is the visible action. Keep a small blank portrait card in the cook’s hand above the open washer; no readable personal data or false loss of voting eligibility.
- **TOWN-29-C:** Shoes fitting into a bus-stop pole’s narrow shade make the waiting-for-trees joke visible. One coherent sunlight direction; do not use a lush existing canopy that contradicts the scene.
- **TOWN-35-B:** The long receipt occupying dinner space is the central visual personification. Keep it physically supported at the grocery counter beside a small food bag; no giant person or fictional tax payment animation.
- **TOWN-44-A:** One sandwich cut into two equal halves makes the accounting joke tangible. Same total food before and after; plain clean plate, no newly awarded food or duplicated sandwich.

Minor props and source directions remain retained for optional reuse; omitting an optional prop does not remove the dialogue. Religious references remain text-only, with no corresponding religious symbols or devotional art. The next classification batch is the 171 remaining active road records.

## Remaining road staging classified

All 171 remaining compatible road records are classified under the approved approach: 126 shared-context records and 45 essential visual compositions. These are not generation counts. Shared settings must still communicate the actual offer/task; first-choice artwork must never depict completed payment, work or rewards.

- **ENC-C09-C / secured_comment_box:** Disproportionate safe around an ordinary public-comment box; phone stays ordinary and separate.
- **ENC-C10-B / shortened_bed:** Oversized pillow on a shortened empty demonstration mattress; no patient or implied injury.
- **ENC-C10-C / paper_aircraft:** Cardboard aircraft made from forms with a modest tip jar; visibly a stationary display, not a real aircraft.
- **ENC-C11-A / overhead_freezer:** Unplugged demonstration freezer and melting ice contrasted with a powered fan; visible safe cable relationships.
- **ENC-C11-C / competing_socket:** Phone and pie warmer compete for one socket; keep plugs, hands and appliances coherent.
- **ENC-C12-C / hidden_fee_panel:** Large honesty sign partly hides a smaller charge panel; both labels are localized live text, not baked pixels.
- **ENC-C14-A / unequal_shade:** Cardboard politician gets a dedicated umbrella while real volunteers share a canopy; all stay on one ground plane.
- **ENC-C16-B / paint_over_rust:** Bright paint around an unrepaired rust hole, with daylight visibly passing through it.
- **ENC-C16-C / padded_phones_books:** Luxurious small phone pouches contrast with heavy books in a battered wheelbarrow; no phones or books granted to crew.
- **ENC-C18-B / biscuit_lift:** Envelope with a biscuit-shaped bulge beside a reserved lift and working stairs; stage the offer, not an already completed passage.
- **ENC-D01-C / phone_place_setting:** Phone gets a full menu/place setting while the nurse’s breakfast waits; do not replace the nurse with a traveler.
- **ENC-D04-C / sandwich_application:** Whole sandwich positioned over the blank photo field of a catering form; no invented approval or food reward.
- **ENC-D06-B / two_sided_filter:** One filter has a clean sales-facing side and dirty reverse, both legible in a single view.
- **ENC-D09-A / parking_without_service:** Decorated empty new-mother parking space faces the closed former maternity entrance; no patient or denial of treatment depicted.
- **ENC-D12-C / scattered_data:** Fan disperses demonstration glitter beyond a small dustpan at a repair kiosk; use fictional blank data cues.
- **ENC-D13-A / smaller_map_dots:** Promoter shrinks dots on a fabricated abstract map while privacy remains visibly unresolved; never map real clinic visitors.
- **ENC-S03-B / unsupported_cushion:** Prestigious cushion held above safe disassembled chair parts, with intact seats elsewhere; nobody sits on the broken display.
- **ENC-S05-A / layered_price_labels:** Built-up blank price stickers visibly thicken the shelf edge; readable localized pricing remains a UI overlay.
- **ENC-S07-C / freezer_leaflet:** Political leaflet offered as a shim for a worn freezer wheel. Compare retained freezer art first; do not claim a wheel repaired before the choice.
- **ENC-S09-A / unequal_cooling:** Cooled trailer exhaust reaches the worker outside; show separate air paths and a plausible shared scale.
- **ENC-S09-C / donor_fan:** One large fan serves the premium area rather than equal-access seating; preserve usable unoccupied rest space.
- **ENC-S10-A / flag_instead_of_tool:** Owner offers a small flag while water pools over a clogged drain; distinguish the prop from an actual repair tool.
- **ENC-S11-A / raccoon_cleanup:** Raccoon tangled in a picnic cloth among overflowing waste; reuse retained park context if suitable, not a new full landscape by default.
- **ENC-S13-A / flag_card_reader:** Cashier presents a tiny Chinese flag to a card reader while the traveler still holds their wallet; payment has not happened.
- **ENC-S14-A / cheque_doorway:** Ceremonial cheque on stands is too wide for the ordinary doorway; people and doorway retain normal proportions.
- **ENC-S16-B / voucher_cash_drawer:** A flimsy voucher held over real cash compartments; no refund or cash transfer is implied before resolution.
- **ENC-S16-C / balloon_doorway:** An oversized donor balloon obstructs the loading door above normal balloons; other scenery stays normal scale.
- **ENC-S17-A / flooded_food_display:** Wrapped sandwiches drift in a spraying demonstration tray; retain separation from the remaining food and no completed cleanup.
- **ENC-S17-B / premium_fan_work:** Heavy unassembled fan bases are offered as work while the contractor has iced drinks; do not depict travelers already working.
- **ENC-S18-A / flyer_supports_books:** Folded department-closure leaflet props the shelf holding educational books; label remains live localized text.
- **ENC-S19-B / overhead_leak:** Bucket catches a fictional leak threatening a funding display; reuse inspected room/leak components when consistent.
- **ENC-S20-B / leaflet_not_food:** Policy leaflet held over an empty bowl fails to supply food; avoid magical rewards or a completed meal.
- **ENC-S21-B / one_mug_two_names:** One mug and an unattached second-name sleeve; never duplicate the mug to illustrate two institutions.
- **ENC-S22-B / two_pins_one_building:** Two destination pins overlap on one fictional building outline; both destinations remain readable live text.
- **ENC-S23-A / straw_inspection:** Ordinary drink/straw dwarfed by inspection equipment; props, not human proportions, carry the exaggeration.
- **ENC-S24-A / wig_in_shower:** Loose wig drifts away from a spraying display/mannequin; no actual traveler loses hair or is injured.
- **ENC-S24-B / puddle_border:** Small flag marks the edge of a spreading lobby fountain puddle; no invented flooding outside this demonstration.
- **ENC-S24-C / melting_gateway:** Decorative gateway melts toward unused cleaning tools while guests approach its dry side; no completed cleanup.
- **ENC-S26-B / phone_free_photo:** Organizer uses a phone to photograph the empty phone-storage tray; no crew phone confiscation is implied.
- **ENC-S27-B / towels_as_room:** Folded towels fill the cupboard advertised as a luxury room; keep the cupboard visibly distinct from an occupied hotel room.
- **ENC-S30-C / anti_wind_banner:** Wind moves an anti-wind banner while boxes remain unopened; wind direction agrees with other scene effects.
- **ENC-S32-B / solar_powers_criticism:** Visible cable connects a solar panel to the presentation criticizing it; preserve a continuous plausible connection.
- **ENC-S33-A / water_tank_display:** A spraying display drains toward a low tank while its owner studies a freedom banner; no false water refill for travelers.
- **ENC-S33-B / straw_blocks_water:** Giant display straw physically blocks access to drinking water; ordinary people and fixtures retain consistent scale.
- **ENC-S34-B / folding_instructions:** Owner caught inside an ordinary folding object after removing instructions; clearly comic demonstration, no injury or invented crew entrapment.

The 300 previously unresolved road/town records are now staging-classified. Next consolidate reusable components and exact asset selections across all groups into the finite production specification; the older 308 proposals must also follow the approved selective-staging rule. No production authorization follows from this classification.

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

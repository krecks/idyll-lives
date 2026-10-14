# Primitives to Blender: the migration plan

Every object in the game gets a model built in Blender (`tools/art/blender`), exported as a `.glb`, in place
of its primitive placeholder parts. Each new model goes to the **Validate** buy category first, and gets the
**animations** it can show.

**Where it stands (2026-10-14):** 540 objects. **16 are done**:
- the seven sinks: `sink`, `sinkPedestal`, `vanityDouble`, `sinkVessel`, `sinkConsole`, `farmhouseSink`, `counterSinkDouble`;
- the `fridge`, the `wallBed` and the `sofaBed` (in Validate › Sofas);
- six garden decor models: `garden.gnome`, `birdbath`, `fountain`, `lantern`, `wheelbarrow`, `boulder`.

**524 are to do.** All of them still draw primitive parts:
- `tools/art/catalog.py`: 127;
- `catalog2.py`: 192;
- the base manifest: 77;
- `community.py`: 36;
- the personality packs: 92.

## The plan

| Category | Items | Tasks |
|---|---|---|
| [Engine and tools first](00-engine-and-tools.md) | — | **Done** (2026-10-14): T0 clips bench and gamekit helpers, E1–E8 engine work, C1 content. Its status table says how to use each. |
| [Kitchen](01-kitchen.md) | 68 | K1 Fridges (5), K2 Cookers (6), K3 Small appliances and dishwashers (4), K4 Base counters (8), K5 Tall and wall storage (11), K6 Islands and bars (5), K7 Dining tables (13), K8 Chairs and stools (12), K9 Dining storage (4) |
| [Bathroom](02-bathroom.md) | 21 | B1 Toilets (5), B2 Showers (4), B3 Baths (3), B4 Laundry (6), B5 Bathroom storage and mirrors (3) |
| [Bedroom](03-bedroom.md) | 27 | BR1 Beds (9), BR2 Nightstands and small storage (8), BR3 Wardrobes and drawers (6), BR4 Dressing and mirrors (4) |
| [Kids](04-kids.md) | 20 | KD1 Baby and kids' beds (4), KD2 Toys (10), KD3 Kids' study and art (6) |
| [Living room](05-living.md) | 49 | L1 Sofas and sectionals (11), L2 Armchairs and small seats (11), L3 Coffee and side tables (9), L4 Media (8), L5 Shelves (9), L6 Stairs (1) |
| [Office](06-office.md) | 14 | O1 Desks (8), O2 Desk chairs (3), O3 Storage and boards (3) |
| [Fitness](07-fitness.md) | 5 | F1 Home gym (5) |
| [Hobbies](08-hobbies.md) | 19 | H1 Music (6), H2 Games (6), H3 Crafts and science (7) |
| [Decor](09-decor.md) | 22 | D1 Wall art and clocks (6), D2 Ornaments (10), D3 Textiles (6) |
| [Tabletop](10-tabletop.md) | 39 | TT1 Kitchenware (11), TT2 Tableware (7), TT3 Books and small plants (7), TT4 Small ornaments (6), TT5 Bath, desk and toys (8) |
| [Lighting](11-lighting.md) | 21 | LT1 Ceiling lights (7), LT2 Wall lights (3), LT3 Floor lamps (5), LT4 Table lamps (6) |
| [Outdoor](12-outdoor.md) | 37 | OU1 Patio seating and tables (11), OU2 Loungers, swings and the hot tub (5), OU3 Grills and fire (3), OU4 Play and yard (9), OU5 Pond, small garden things and lights (9) |
| [Garden plants and trees](13-garden.md) | 53 | G1 Houseplants (10), G2 Flowers (15), G3 Shrubs (11), G4 Edibles (5), G5 Trees (12) |
| [Community and farm](14-community.md) | 36 | CM1 Playground (8), CM2 Street furniture (10), CM3 Food stalls and vending (5), CM4 Shops and café (5), CM5 Farm (8) |
| [Personality packs](15-personality-packs.md) | 94 | P1–P16: one task per pack (bookworm … slob, and the example pack) |
| [Sign-off](16-sign-off.md) | — | Back from Validate, per task and per category; done means |

Each task table lists, per item:
- the id and name;
- its size and layer;
- its uses (interaction id and animation), which are what trigger the clips;
- **Back to**: where it lives now, so it can return after Validate;
- the parts the player may colour (`paint`, labelled);
- the clip's kind, and what the clip shows.

## Doing a task

Follow `tools/art/blender/README.md` ("Working as an agent: the loop") and `docs/design/objects.md`. Per task:

1. **Read** the placeholder builder named in the task, and the content entries. The size, the top height
   (`tools/art/catalog2.py` `TOPS`), the seat height (`SEATS`), the mattress height (`BEDS`, `BLANKET_BEDS` in
   `web/src/render/babylon/characters.ts`) and the light position (`LIGHTS`) are facts the model must keep.
2. **Build** the family named in the task (copy `families/_template.py`, or add to an existing family). Do this:
   - one material per part, named for what it is;
   - `paint` with labelled parts, the main part first, e.g. `[('body', 'Cabinet'), ('stone', 'Worktop')]`;
   - the moving parts joined one per part (`G.join`), and the clips from `G.swing` / `G.slide` / `G.grow` (plus T0's
     `spin`, `rock`, `squash`).
3. **Preview** (`build.py <family> --preview`) and look at every model at rest and with every clip.
4. **Export** (`build.py <family>`). `check_glb.py` must say `0 failed`; read every warning. The Khronos validator
   must report 0 errors and 0 warnings.
5. **Into Validate.**
   - Add the task's group to the `validate` category in `web/public/content/base.json`.
   - Change each item's `category` to `validate` and its `group` to the task's group. Change nothing else.
   - The table's "Back to" column is where it returns.
6. **Test.** Run `cargo test -p sim-core --test shipped_content`, and the whole `sim-core` suite if content changed
   beyond the category.
7. **Look in the game.** Start your own dev server and run headless Chrome on the real GPU:
   - `/gallery.html?keys=…`, next to the placeholder (with the `.glb` requests answered 404);
   - the clips bench (T0) for every clip;
   - in play: the item in Validate, bought, used by a resident (the clip plays, and shuts after `hold`),
     turned to all four facings, coloured in the colour studio;
   - on WebGPU and `&renderer=webgl`, by day and at night.
8. **Finish.**
   - Add a line to `web/public/assets/CREDITS.md`.
   - Commit only the `.glb` files the task meant to change: re-exports aren't byte-for-byte the same, so
     `git checkout` the rest.
   - Never commit `previews/`.

## Validate

- **Per task:** each task opens its own group in the `validate` category (the sinks' `sinks` group is the first).
  The group is named in the task.
- **Not too many at once:** Validate should hold about three groups at a time. Sign a task off (16-sign-off.md)
  before opening many more, so the catalog stays usable.
- **Signing off** moves the items back to their "Back to" category and group, and removes the empty group.
  Once nothing is left in it, the category itself goes (and `icon.category.validate`).
- **Community things:** they are placed on town lots. They go to Validate too, so they can be bought at home
  for a look, and they are also checked on a town lot.

## Animations: the kinds in the tables

All of these work (the engine tasks are done); the manifest field and the gamekit helper for each are in
[00-engine-and-tools.md](00-engine-and-tools.md)'s status table.

| Kind | Plays | Manifest (`motions`) | Example |
|---|---|---|---|
| **Use** | While a resident uses the object (by interaction id, else by animation tag). Played back at the end of the use, or after its `hold` (per clip: E5). | `uses` | Fridge door, sink water, toilet lid |
| **Held open** | While the sim's `open` flag is set (an interaction's `when` / `leaves`) | `open` (+ content, C1) | Wall bed, futon, sofa bed |
| **Loop (E1)** | Repeats for as long as the use lasts | `loops` | Rocking chair, treadmill belt, drum spin |
| **Ambient (E2)** | Always, on its own, near the camera (not on Low) | `idle` | Ceiling fan, clock hands, fountain water |
| **Night (E3)** | Held while the lamps are lit | `night` | Curtains drawn; shades glow with a material's `glow` (E8) |
| **Growth (E4)** | Follows the crop's growth stage from the sim (content `grows`) | `growth` | Vegetables ripening |
| **Static** | No clip | — | Tables, rugs, ornaments |

Residents sitting or lying on a moving seat ride it with `ride` (a rocking chair, a swing).

- **No use yet:** "(+use)" marks a clip that needs an interaction first. These are ideas for when content gives
  the object a use. Until then the item is built with the part separate, so the clip can be added without
  remodelling.
- **Interim clips:** some tables also give a plain use clip "until E1" or "until then": no longer needed, build
  the loop (or the ambient, night or growth clip) itself.
- **Rules every clip keeps (`check_glb.py`):**
  - node transforms only (no skins or morph targets), starting at rest;
  - about 1 s long;
  - 1–3 draw calls (4 at most);
  - water, flames and contents that appear are `G.grow`n from 0.001 (gloss, never emissive).

## Order

The engine and tools tasks (T0, E1–E8, C1) are done, so every kind of clip can be built now.

1. The most-seen categories first:
   - Kitchen K1–K3;
   - Bathroom B1–B4;
   - Bedroom BR1, BR3;
   - Living L1 (the sofa bed is done), L2, L4, L5;
   - then the rest of those categories.
2. Office, Kids, Fitness, Hobbies, Decor, Lighting, Outdoor.
3. The personality packs (P1–P16), one pack per task.
4. Garden plants and trees, and the street and community props, each with a far version (`G.Model(far=…)`, E7).
   Plants also need the nature shader's wind and tint before they come from Blender. Tabletop at any point
   (small and quick).
5. The ambient and night clips on items already built (the sinks, the fridge, the garden decor), where they fit.

## Done means

- Every one of the 525 items is a Blender `.glb` with its clips. Each passed `check_glb` and the validator, and
  was looked at in play.
- Every item is back in its own category (Validate is gone).
- The placeholders stay as the fallback, and keys are unchanged, so saves and content need no migration.

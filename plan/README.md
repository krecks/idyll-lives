# Primitives to Blender: the migration plan

Every object in the game gets a model built in Blender (`tools/art/blender`), exported as a `.glb`, in place
of its primitive placeholder parts. Each new model goes to the **Validate** buy category first, and gets the
**animations** it can show.

**Where it stands (2026-10-16):** 541 objects. **All 541 are built** (each in Validate until it's signed off):
- the seven sinks: `sink`, `sinkPedestal`, `vanityDouble`, `sinkVessel`, `sinkConsole`, `farmhouseSink`, `counterSinkDouble` (signed off 2026-10-16: back in bathroom / sinks and kitchen / counters);
- the seven fridges: `fridge`, and the K1 six `fridgeMini`, `fridgeBuiltIn`, `fridgeFrenchDoor`, `fridgeRetro`, `fridgeMiniRetro`, `wineFridge` (signed off 2026-10-16: back in kitchen / appliances);
- the six K2 cookers: `stoveElectric`, `stoveGas`, `wallOven`, `rangeCooker`, `cooktopCounter`, `microwave` (signed off 2026-10-16: back in kitchen / appliances);
- the four K3 small appliances and dishwashers: `coffeeMachine`, `breakfastCounter`, `dishwasher`, `dishwasherCompact` (signed off 2026-10-16: back in kitchen / appliances);
- the eight K4 base counters: `counter`, `counterCorner`, `counterEnd`, `counterCornerOuter`, `counterDrawers`, `counterOpenShelf`, `butcherBlock`, `counterMarble` (signed off 2026-10-16: back in kitchen / counters);
- the eleven K5 tall and wall storage: `pantry`, `pantryTall`, `wallCabinet`, `wallCabinetGlass`, `wallCabinetCorner`, `rangeHood`, `spiceRack`, `potRail`, `kitchenShelfWall`, `trashCan`, `recyclingBins` (signed off 2026-10-16: back in kitchen / counters);
- the five K6 islands and bars: `kitchenIsland`, `islandSmall`, `islandLong`, `barSegment`, `barEnd` (signed off 2026-10-16: back in kitchen / counters and kitchen / islands);
- the thirteen K7 dining tables: `diningTable`, `bistroSet`, `diningGlass`, `diningOak`, `tableSquare2`, `tableBistroRound`, `tableDining4`, `tableDining6`, `tableBanquet8`, `tableRound4`, `tableSquare8`, `tableFarmhouse6`, `tableBarHigh` (signed off 2026-10-16: back in kitchen / dining);
- the twelve K8 chairs and stools: `barStool`, `stoolCounter`, `stoolBarHigh`, `diningChair`, `chairWood`, `chairUpholstered`, `chairModern`, `chairWindsor`, `chairBentwood`, `chairFolding`, `chairArm`, `benchDining` (signed off 2026-10-16: back in kitchen / dining);
- the four K9 dining storage things: `sideboard`, `buffet`, `chinaCabinet`, `barCart` (signed off 2026-10-16: back in kitchen / cabinets);
- the five B1 toilets: `toilet`, `toiletCompact`, `toiletSmart`, `toiletClassic`, `toiletWallHung` (signed off 2026-10-17);
- the four B2 showers: `shower`, `showerStall`, `showerWalkIn`, `showerRain` (in Validate › Showers);
- the three B3 baths: `bathtub`, `bathtubClawfoot`, `bathtubFreestanding` (in Validate › Baths);
- the six B4 laundry models: `laundryWasher`, `laundryDryer`, `towelRail`, `towelLadder`, `laundryBasket`, `bathMat` (signed off 2026-10-17);
- the three B5 storage and mirrors: `mirrorCabinet`, `mirrorRound`, `bathCabinetTall` (signed off 2026-10-17);
- the 21 lamps: LT1 `ceilingPendant`, `ceilingLight`, `chandelier`, `pendantIndustrial`, `ceilingFan`, `trackLights`, `paperLantern`; LT2 `wallSconce`, `wallLampArm`, `wallLightOutdoor`; LT3 `lamp`, `floorLampSimple`, `lampFloorArc`, `lampFloorTripod`, `lampTorchiere`; LT4 `tableLamp`, `lampTableRound`, `lampTableSquare`, `lampBanker`, `lampMushroom`, `lampLantern` (signed off 2026-10-16: back in lighting / ceiling, wall, floor and table);
- the five F1 home gym models: `treadmill`, `weightBench`, `exerciseBike`, `yogaMat`, `dumbbells` (in Validate › Fitness);
- the nine BR1 beds: `bed`, `bedSingle`, `bedBunk`, `bedFuton`, `bedKing`, `bedDouble`, `bedQueenUpholstered`, `bedCanopy`, `bedDaybed` (signed off 2026-10-17);
- the eight BR2 nightstands and small storage: `nightstand`, `nightstandModern`, `nightstandRound`, `nightstandDrawers`, `nightstandLamp`, `cedarChest`, `laundryHamper`, `benchBedEnd` (signed off 2026-10-17), and the six BR3 wardrobes and drawers: `dresser`, `wardrobe`, `chestOfDrawers`, `dresserLow`, `wardrobeSingle`, `armoire` (signed off 2026-10-17);
- the four BR4 mirrors and the dressing table: `mirror`, `dressingTable`, `mirrorWall`, `mirrorFloor` (signed off 2026-10-17);
- the twenty kids' things: KD1 `crib`, `kidsBed`, `kidsBedCar`, `highChair` (signed off 2026-10-17); KD2 `toyBox`, `toyChest`, `dollHouse`, `rockingHorse`, `playTent`, `ballPit`, `trainTable`, `playMat`, `teddyBear`, `beanBagKids` (signed off 2026-10-17); KD3 `kidsDesk`, `kidsTable`, `kidsChair`, `kidsBookshelf`, `kidsChalkboard`, `kidsEasel` (signed off 2026-10-17);
- the `wallBed`, and the eleven L1 sofas and sectionals: `sofa`, `loveseat`, `sofaThree`, `sofaDesign`, `sofaThreeSeat`, `sofaBed`, `sectionalMiddle`, `sectionalArmLeft`, `sectionalArmRight`, `sectionalCorner`, `chaise` (in Validate › Sofas);
- the eleven L2 armchairs and small seats: `armchair`, `armchairLounge`, `armchairLeather`, `armchairDesign`, `armchairWingback`, `recliner`, `rockingChair`, `pouf`, `ottoman`, `floorCushion`, `beanBag` (signed off 2026-10-17);
- the nine L3 coffee and side tables: `coffeeTable`, `coffeeTableGlass`, `coffeeTableMarble`, `sideTable`, `coffeeTableRound`, `coffeeTableLong`, `nestingTables`, `sideTableRound`, `consoleTable` (signed off 2026-10-17);
- the eight L4 media things: `tv`, `tvWall`, `tvRetro`, `homeCinema`, `stereo`, `boombox`, `tvStandLow`, `fireplaceElectric` (signed off 2026-10-17); the nine L5 shelves: `bookshelf`, `bookcaseLow`, `bookcaseWide`, `shelvingIndustrial`, `bookcaseTall`, `bookcaseCorner`, `cubeShelf`, `ladderShelf`, `displayCabinet` (signed off 2026-10-17); the L6 `stairs` (signed off 2026-10-17);
- the fourteen office models: O1 `deskSimple`, `deskCorner`, `deskStanding`, `deskExecutive`, `computerDesk`, `deskWriting`, `deskLaptop`, `computerGaming` (signed off 2026-10-17); O2 `chairExecutive`, `chairTask`, `officeChair` (signed off 2026-10-17); O3 `filingCabinet`, `whiteboard`, `corkboard` (signed off 2026-10-17);
- the 22 decor models: D1 `printFramed`, `photoPrint`, `paintingClassic`, `canvasAbstract`, `wallClock`, `wallShelf` (signed off 2026-10-16),
  D2 `grandfatherClock`, `vaseTall`, `urnPorcelain`, `vasePedestal`, `bustMarble`, `statueCat`, `statueHorse`, `coatRack`, `hangingPlant`, `fireplace` (signed off 2026-10-16),
  D3 `curtains`, `curtainsLong`, `rugRound`, `rugLarge`, `rugRunner`, `rugPattern` (signed off 2026-10-16);
- the nineteen hobby models: H1 `piano`, `ukulele`, `guitarElectric`, `keyboardDigital`, `drumKit`, `guitarAcoustic` (signed off 2026-10-16); H2 `chessTable`, `poolTable`, `foosball`, `dartboard`, `arcadeCabinet`, `puzzleTable` (signed off 2026-10-16); H3 `easel`, `workbench`, `telescope`, `sewingTable`, `chemistrySet`, `potteryWheel`, `craftTable` (signed off 2026-10-16);
- six garden decor models: `garden.gnome`, `birdbath`, `fountain`, `lantern`, `wheelbarrow`, `boulder` (in Validate › Garden Decor);
- the eleven OU1 patio models: `bench`, `patioSet`, `picnicTable`, `gardenChair`, `patioTableRound`, `patioTable4`, `patioChair`, `adirondackChair`, `benchGarden3`, `outdoorBar`, `parasol` (signed off 2026-10-17);
- the five OU2 loungers and swings: `sunLounger`, `hammock`, `porchSwing`, `swingSet`, `hotTubCedar` (signed off 2026-10-17);
- the three OU3 grills and fire: `grill`, `grillStation`, `firePit` (signed off 2026-10-17); the nine OU4 play and yard things: `sandbox`, `bikeRack`, `raisedBed`, `shedSmall`, `greenhouse`, `compostBin`, `rainBarrel`, `clothesline`, `firewoodStack` (signed off 2026-10-17); the nine OU5 pond, small garden things and lights: `garden.pond`, `lampPost`, `stringLights`, `planterBox`, `birdhouse`, `birdFeederPole`, `dogHouse`, `sundial`, `steppingStones` (signed off 2026-10-17);
- the 36 community and farm things, CM1–CM5 (in Validate › Playground, Street, Stalls, Shops and Farm);
- the ten G1 houseplants: `plant`, `garden.monstera`, `fiddleFig`, `snakePlant`, `cacti`, `succulents`, `bostonFern`, `palm`, `orchid`, `bonsai` (in Validate › Houseplants);
- the fifteen G2 flowers: `flowerbed`, `garden.tulips`, `daffodils`, `daisies`, `sunflowers`, `poppies`, `marigolds`, `pansies`, `lilies`, `lupins`, `forgetMeNots`, `cosmos`, `meadow`, `geraniums`, `flowerBox` (in Validate › Flowers);
- the eleven G3 shrubs: `bush`, `garden.boxwood`, `topiary`, `hedge`, `hydrangea`, `roses`, `azalea`, `lavender`, `fern`, `grass`, `hosta` (in Validate › Shrubs);
- the five G4 edibles: `garden.vegPatch`, `tomatoes`, `strawberries`, `herbs`, `pumpkins` (in Validate › Edibles);
- the twelve G5 trees: `tree`, `pine`, `garden.maple`, `birch`, `cherry`, `apple`, `japaneseMaple`, `magnolia`, `willow`, `spruce`, `cypress`, `lemonTree` (in Validate › Trees).
- the 39 tabletop things: TT1 `kettle`, `toaster`, `blender`, `standMixer`, `espressoMachineSmall`, `knifeBlock`, `utensilCrock`, `cuttingBoard`, `breadBin`, `spiceJars`, `cookieJar` (signed off 2026-10-16); TT2 `fruitBowl`, `plateSet`, `cakeStand`, `teapotSet`, `mugs`, `wineGlasses`, `candlesticks` (signed off 2026-10-16); TT3 `bookStack`, `booksUpright`, `vaseFlowers`, `vaseSmall`, `potSucculent`, `potCactus`, `potSmall` (Validate › Books And Pots); TT4 `photoFrame`, `tableClock`, `globe`, `snowGlobe`, `trophy`, `sculptureSmall` (signed off 2026-10-16); TT5 `soapSet`, `towelStack`, `perfumeTray`, `deskOrganiser`, `laptopClosed`, `monitor`, `stuffedToy`, `toyBlocks` (signed off 2026-10-16).
- the 94 personality pack models, P1–P16: bookworm, cheerful, couch potato, energetic, the example pack (`chefStove`, `whirlpoolTub`), foodie, gloomy, hot-headed, kind, lazy, loner, nature lover, neat, outgoing, romantic and slob (`families/pack_<trait>.py`, each in Validate › its trait);

**None are left to build.** Next is sign-off (`16-sign-off.md`): the sinks, K1–K9, LT1–LT4, D1–D3, H1–H3, TT1, TT2, TT4, TT5, O1–O3, B1, B4, B5, KD1–KD3, BR1–BR4, OU1–OU5, L2–L6 are back home (2026-10-16); Validate holds the other 220 items in 32 groups.

**No primitive models are left either** (2026-10-16): the 36 non-object models that were still drawn from
placeholder parts are Blender models too: the 20 things residents carry and cook with (`families/held.py`), the 9
street props of the town (`families/street.py`) and the lawn's grass tufts and wildflowers, the woods' rocks and the
selection marker (`families/ground.py`). The three unused base entries (`model.stove`, `model.plantSmall`,
`model.ceilingLamp`) are gone. Every model key in the manifests now has a `.glb`; the placeholders stay as the fallback.

## The plan

| Category | Items | Tasks |
|---|---|---|
| [Engine and tools first](00-engine-and-tools.md) | — | **Done** (2026-10-14): T0 clips bench and gamekit helpers, E1–E8 engine work, C1 content. Its status table says how to use each. |
| [Kitchen](01-kitchen.md) | 69 | K1 Fridges (6, **done**), K2 Cookers (6, **done**), K3 Small appliances and dishwashers (4, **done**), K4 Base counters (8, **done**), K5 Tall and wall storage (11, **done**), K6 Islands and bars (5, **done**), K7 Dining tables (13, **done**), K8 Chairs and stools (12, **done**), K9 Dining storage (4, **done**) |
| [Bathroom](02-bathroom.md) | 21 | B1 Toilets (5, **done**), B2 Showers (4, **done**), B3 Baths (3, **done**), B4 Laundry (6, **done**), B5 Bathroom storage and mirrors (3, **done**) |
| [Bedroom](03-bedroom.md) | 27 | BR1 Beds (9, **done**), BR2 Nightstands and small storage (8, **done**), BR3 Wardrobes and drawers (6, **done**), BR4 Dressing and mirrors (4, **done**) |
| [Kids](04-kids.md) | 20 | KD1 Baby and kids' beds (4, **done**), KD2 Toys (10, **done**), KD3 Kids' study and art (6, **done**) |
| [Living room](05-living.md) | 49 | L1 Sofas and sectionals (11, **done**), L2 Armchairs and small seats (11, **done**), L3 Coffee and side tables (9, **done**), L4 Media (8, **done**), L5 Shelves (9, **done**), L6 Stairs (1, **done**) |
| [Office](06-office.md) | 14 | O1 Desks (8, **done**), O2 Desk chairs (3, **done**), O3 Storage and boards (3, **done**) |
| [Fitness](07-fitness.md) | 5 | F1 Home gym (5, **done**) |
| [Hobbies](08-hobbies.md) | 19 | H1 Music (6, **done**), H2 Games (6, **done**), H3 Crafts and science (7, **done**) |
| [Decor](09-decor.md) | 22 | D1 Wall art and clocks (6, **done**), D2 Ornaments (10, **done**), D3 Textiles (6, **done**) |
| [Tabletop](10-tabletop.md) | 39 | TT1 Kitchenware (11, **done**), TT2 Tableware (7, **done**), TT3 Books and small plants (7, **done**), TT4 Small ornaments (6, **done**), TT5 Bath, desk and toys (8, **done**) |
| [Lighting](11-lighting.md) | 21 | LT1 Ceiling lights (7, **done**), LT2 Wall lights (3, **done**), LT3 Floor lamps (5, **done**), LT4 Table lamps (6, **done**) |
| [Outdoor](12-outdoor.md) | 37 | OU1 Patio seating and tables (11, **done**), OU2 Loungers, swings and the hot tub (5, **done**), OU3 Grills and fire (3, **done**), OU4 Play and yard (9, **done**), OU5 Pond, small garden things and lights (9, **done**) |
| [Garden plants and trees](13-garden.md) | 53 | G1 Houseplants (10, **done**), G2 Flowers (15, **done**), G3 Shrubs (11, **done**), G4 Edibles (5, **done**), G5 Trees (12, **done**) |
| [Community and farm](14-community.md) | 36 | CM1 Playground (8, **done**), CM2 Street furniture (10, **done**), CM3 Food stalls and vending (5, **done**), CM4 Shops and café (5, **done**), CM5 Farm (8, **done**) |
| [Personality packs](15-personality-packs.md) | 94 | P1–P16 (**done**): one task per pack (bookworm … slob, and the example pack) |
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
   - Kitchen K1–K5;
   - Bathroom B1–B4;
   - Bedroom BR1, BR3;
   - Living L1 (the sofa bed is done), L2, L4, L5;
   - then the rest of those categories.
2. Office, Kids, Fitness, Hobbies, Decor, Lighting, Outdoor.
3. The personality packs (P1–P16), one pack per task.
4. Garden plants and trees, and the street and community props, each with a far version (`G.Model(far=…)`, E7).
   A plant's parts on `foliage` and `bark` take the nature shader's wind and tint as the placeholders' do (G1). Tabletop at any point
   (small and quick).
5. The ambient and night clips on items already built (the sinks, the fridge, the garden decor), where they fit.

## Done means

- Every one of the 541 items is a Blender `.glb` with its clips. Each passed `check_glb` and the validator, and
  was looked at in play.
- Every item is back in its own category (Validate is gone).
- The placeholders stay as the fallback, and keys are unchanged, so saves and content need no migration.

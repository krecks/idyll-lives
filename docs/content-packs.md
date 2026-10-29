# Content packs

A content pack adds objects, feelings (and other content) to the game without touching
`base.json`. Packs are plain JSON, merged into the base content when the game loads; the
simulation (Rust) and the web UI (TypeScript) merge them the same way.

```
cargo run -q -p sim-core --example check_pack -- web/public/content/packs/<name>.json
```

Run the checker after every change (see [Checking a pack](#checking-a-pack)).
`web/public/content/packs/example.json` uses every feature described here.

## File location and loading

- Put the pack in `web/public/content/packs/<name>.json`.
- Add it to the `"include"` list at the end of `web/public/content/base.json`, after
  `careers.json`: `"include": ["careers.json", "packs/example.json", "packs/<name>.json"]`.
- Files are merged in that order: base, careers, then packs.
- Keep ids unique across *all* files. Prefix ids with your pack's theme when in doubt
  (`zenTeaSet`, not `teaSet`), because another pack may want the obvious name.
- **In a built site** (`bun run build`, the published game) the content and the asset manifests are
  also packed into one file, `bundle.json`, and the game reads them from there: editing a file
  in `dist/content/` changes nothing until you rebuild, or delete `dist/bundle.json` (the game
  then fetches the files one by one). The dev server has no bundle, and an art pack loaded with
  `?pack=` is fetched as before.

## Merge rules

Applied file by file (`sim_core::pack::merge_content` in `crates/sim-core/src/pack.rs`,
mirrored by `mergeContent` in `web/src/content/content.ts`):

| Top-level value | Rule |
|---|---|
| array (`objects`, `feelings`, `emotions`, `buyCategories`, `socials`, `skills`, `tags`, `animations`, ...) | appended. An `id` that appears twice in the same merged array is an error naming both files. |
| object (`economy`, `skillRules`, `bondPresets`, ...) | merged shallowly: your keys are added or replace existing keys, other keys stay. |
| anything else | replaced. |
| `traitPatches` | merged *into* existing traits (below). Never copied into the result. |
| `include`, `$comment` | dropped. Use `$comment` for notes. |
| `voice` | spoken lines (see [Voice lines](#voice-lines)): merged at every depth by the web side, so a pack adds or replaces single groups of lines. The simulation drops it. |
| `talk` | how the language model is asked for residents' lines, per language, laid out like `web/public/content/talk/<lang>.json` (speech-style hints per trait and perk, event wording, blocklist): merged at every depth by the web side. The simulation drops it. |

A key whose type differs from earlier files (array in base, object in your pack) is an error.
Packs should normally only add `objects`, `feelings`, `tags`, `buyCategories` and
`traitPatches`; don't replace global rules (`economy`, `skillRules`, ...) from a trait pack.

**Older packs:** feelings had a different name in early versions of the game. Packs that
still use the old key names (for the list, the interaction fields and the social and rule
fields) load unchanged: both mergers rename the keys first (`LEGACY_KEYS` in `pack.rs` and
`content.ts`). Use the `feeling` names in new packs.

### `traitPatches`

```json
"traitPatches": {
  "lazy": {
    "effects": { "tagPreference": { "spa": 1.6 }, "needGain": { "comfort": 1.1 }, "mood": 0.02, "walkSpeed": 0.95 },
    "startingSkills": { "cooking": 1 }
  }
}
```

- The trait must exist (else: error).
- `effects.needDecay`, `needGain`, `tagPreference`, `tagAcceptance`, `tagSuccess`,
  `skillGain`, and `startingSkills`: entries are set per key (yours win; the trait's other
  entries stay). A later pack patching the same key wins.
- `effects.mood` is **added** to the trait's mood; `effects.walkSpeed` is **multiplied**.
- Other keys are errors. Patch only your own trait, so packs don't fight.
- A trait's `startingSkills` are the skills new residents with it start with. (Before 0.2.0 the
  game dropped them while loading, so neither traits nor patches gave any; old saves keep the
  skills their residents have.) An unknown skill id there is an error.

## Objects

```json
{
  "id": "chefStove",
  "name": "Chef's stove",
  "model": "model.chefStove",
  "icon": "icon.object.chefStove",
  "footprint": [2, 1],
  "price": 1500,
  "category": "kitchen",
  "slots": 1,
  "description": "Six burners and a proper oven.",
  "interactions": [ ... ]
}
```

| Field | Meaning |
|---|---|
| `id` | Unique object id (camelCase). |
| `name` | Shown in the catalog. |
| `model` | Asset key of the 3D model (`model.<id>`). Art is registered separately in the asset manifest; a missing key (or a file that fails to load) draws a plain box in its place: among the placed objects in view, in the street's view of public places on other lots, in the town overview and as buy mode's copy in hand; food, dishes and carried things get a small block, and a catalog picture of a file that fails shows the box but isn't kept. A file that fails is tried again in the background (after 5 s, 30 s and 2 minutes, and again when the browser comes back online), and what was drawn without it is drawn again with it as soon as it comes, without a reload (the landscape's own models and the selection marker too: they're left out, not boxed, until then). A missing key also logs a warning. |
| `icon` | Optional catalog icon key. Without one (or if the key is missing) the catalog shows a picture of the 3D model. |
| `footprint` | `[width, depth]` in tiles, default `[1, 1]`. The resident uses it from the tile in front of the footprint's centre (local +z), which must be free (unless nobody uses it: no interactions; see `approach` for chairs). |
| `price` | Buy-mode price in dollars. Without a price the object isn't sold (scenery). |
| `category` | Buy-mode category id (see [Buy categories](#buy-categories)). |
| `group` | Optional group within the category, for categories that have `groups` (the garden: `trees`, `shrubs`, `flowers`, `edibles`, `houseplants`). The catalog shows the groups as chips. |
| `outdoors` | `true` for things that can only stand outdoors (trees, flower beds, ponds). Buying or moving one indoors is refused ("Goes outdoors"). Default `false`. |
| `private` | `true` for the household's own things (a wardrobe, a dresser, the bath cabinet, a vanity): someone visiting uses none of its uses, chosen or told ("Ada is a guest there, and the wardrobe is the household's own"), as with beds and showers (`busyTags`). Code: `crates/sim-core/src/privacy.rs`. Default `false`. |
| `layer` | Where it goes: `floor` (default: furniture, which takes its tiles), `wall` (hangs with its back on a plain wall: pictures, shelves, wall lamps; on a window too with `overWindow`), `ceiling` (hangs in a room: pendant lamps, fans) or `rug` (lies on the floor). Off the floor layer it takes no floor space: residents walk under and over it, furniture stands in front of, under or on it, and only things on the same layer can't overlap. A wall with something on it can't be removed or get a door. Model things on a wall with their back on the wall face (`z = -0.43` in a 1-deep footprint) at their real height, ceiling things hanging from 2.8 m. `surface`: small things standing on another object's top (vases, books, plates, table lamps); see `surface` and `radius` below. They must be 1×1 with one slot, always turn freely, and stand anywhere on a top: the disc they take (`radius`) lies wholly on one top, clear of its edges and of anything else standing there. Moving or turning the top carries them along, selling it sells them too. Model them standing on y = 0; the game lifts them to the top's height. Things nobody uses (no interactions), on any layer, needn't be reachable: a table can have chairs all round it. Their interactions (the kettle's tea, the laptop's browsing) are used from the floor beside their top: in front of the counter, desk or table they stand on, else at its side (the top's front, then its left, right and back of the tile they stand over, then anywhere along its edge, nearest first), standing and facing them. With `"pose": "sit"` the resident sits on a free seat pulled up to that top (its `seatAt`), else on one within `seatNear`, else stands; with no side of the top free (chairs all round a table) they sit down at one of its chairs to use them. They can't be lain on, stood in (`inside`), taken `whole`, be a baby's or a carer's, need a table or start a `recipe` (an `eat` is fine: the toaster's toast). Building never has to leave a way to them: they are reached when their top is. |
| `overWindow` | Things on a wall that belong over a window (curtains, blinds): `true` lets its back be on a window as well as on a plain wall. Everything else on a wall needs plain wall behind it ("that can't hang over a window"), and a window can't go in behind it. Default `false`. |
| `fillsWall` | Furniture as tall as the wall (a wardrobe, a tall bookcase, a tall fridge, a shower, a fireplace with its chimney breast): `true` keeps the wall and the ceiling over its tiles free. Nothing on the `wall` or `ceiling` layer goes on its tiles ("that would hang into the wardrobe"), and it isn't put under one ("that is too tall for the picture hanging there"). Rugs and things on its top are fine. Default `false`. |
| `overhang` | Tiles beside its own that a 1×1 thing on the floor reaches over at head height, as `[right, front]` tiles from it (up to 2): `[[0, 1]]` for an arc lamp whose shade hangs over the sofa in front of it. As under a lamp at head height, furniture may stand there and nobody walks there; nothing as tall as the wall, hanging low or at head height (nor another shade) goes there, no wall may cross the arm, and it can't hang off the lot or over the stairs. Default none. Build mode checks it as the sim does (`overhangTiles` in `game/surfaces.ts`); the house generator hangs the shade over a seat beside the thing if it can, else over free floor, which it takes from walking. Not with `freeRotation` (the tiles turn in quarter turns). The arc lamp has `[[0, 1]]`. |
| `hangsLow` | Things from the ceiling that hang low enough to walk into (a hanging plant): `true` makes it take the floor under it as furniture does. Nobody walks under it, and nothing on the floor layer stands under it (either way round). Default `false`. |
| `headHeight` | Things from the ceiling that hang at head height to hang over a table (a chandelier, a low pendant): `true` keeps people from walking under it, but furniture may stand under it (not furniture as tall as the wall). On the `ceiling` layer only; not with `hangsLow`. Default `false`. |
| `againstWall` | Furniture built to stand with its back on a wall (a wall-hung toilet's cistern box, a wall bed, a ladder shelf): `true` needs plain wall all along its back, and that wall can't then be removed, nor turned into a window or a door, while it stands there. On the `floor` layer only. Default `false`. |
| `surface` | A top small things stand on: `{"height": 0.75, "kind": "table", "size": [1.6, 0.8], "at": [0, 0.1], "inset": 0.04}`: its height (m), its kind (`table`, `bar`, `counter`, `desk`, `low` for coffee and side tables and nightstands, `shelf`), its width and depth (m; default the footprint) and where its middle is off the footprint's middle (m, to the object's right and front; default `[0, 0]`), and the margin kept clear along its edges (m, default 0.04). Measure `size` and `at` from the model: a counter's worktop is at the back of its tile, a desk's top over its front. Only furniture on the floor has one. |
| `radius` | Things on the `surface` layer: the radius (m) of the disc they take on a top, 0.02 to 0.45. Default 0.12. |
| `approach` | Where it's used from: `front` (default: the tile in front of it, which must be free; on a seat for several each sitter comes from the tile in front of their own slot), `around` (chairs pushed under a table: the first free tile in front, behind, left or right of it, with no wall between), `side` (beds, loungers, bikes, a chaise: sitters and sleepers come from beside their slot, the slot's own side first, in the row beside the slot's middle or the one just towards its foot; a bunk's lower sleeper from its ladder side first, the upper one from the front, up the ladder) or `behind` (a seat behind the work: a desk's chair, a drum kit's throne; from behind the slot, else beside it; building keeps one of those free, or the front if it can be stood on, so a desk may stand with its screen to a wall). With a `side` or `behind` tile that can't be stood on they come from the front. Used standing, `side` and `behind` things are used from the front as usual. An `around` seat needs one free side. |
| `seatAt` | Seats: the kinds of top they tuck under (`["table"]`, `["bar", "counter"]`). Facing one (it covers the tile in front of the seat, with no wall between), the seat is a seat at that table: residents eat there first, and `needsTable` interactions are offered. In Buy mode such a seat turns to face a table it's put beside. |
| `seat` | Seats (looks only): `{"height": 0.46, "forward": 0}`: the height of the seat (m) and how far forward of the object's middle the resident's hips are (m). Default: a few known objects' own values, else 0.45 and 0. Optional: `"facing": "back"` turns the sitter round to face the object's back (a pianist at an upright that stands with its back to the wall, a writer at a bureau; they still come from the tile in front); `"legs": "out"` stretches the legs out along the seat (a chaise); `"recline": 24` leans the sitter back that many degrees from sitting up, against a slanted back, while they sit, relax, nap or watch (an Adirondack chair; default: upright sitting, about 8 relaxing), so set `forward` with it, where the hips sit against the back; `"pedals": {"height": 0.3, "forward": 0.04, "crank": 0.16}` puts the feet on pedals turning about that axle (m, the object's frame), in time with the model's loop (an exercise bike). |
| `held` | What the resident holds while using it (looks only): a moving part of its model, by its node's name, and how it's held: `{"part": "guitar", "grip": "guitar"}`. Grips: `bar` (across both hands, a barbell pressed lying down), `pair` (two parts, `["left", "right"]`, one in each hand: dumbbells curled), `guitar` (the neck in the left hand, the body against the belly, strummed) and `pole` (its top in the hands, its foot on the floor ahead: a mop pushed). The model hides its own part while it's used (`motions.uses`, a `G.squash` to 0.001), so there are never two; without a model file nothing is held. |
| `stand` | Used standing up on it (looks only): `{"height": 0.156}`, the height (m) of what the resident stands on (a treadmill's belt, a trampoline's mat); they step up onto its middle from the tile in front. `"motion": "bounce"` bounces them on it, `"buzz"` shakes them a little (a vibration plate). |
| `bunk` | Bunk beds (2 `slots`): both sleepers lie in the middle of the bed's width, the second on the upper bunk (heights per object in `render/shared/characters/furniture.json`, `bunks`). Its model can play a clip per bunk (`motions.uses` with `slot`: docs/design/objects.md, "Moving parts"). |
| `stages` | Who may use it at their age: life stages (`life.stages` ids), `["child"]` for the kids' furniture and the toddler rides, `["youngAdult", "adult", "elder"]` for wine. For every interaction without `stages` of its own (or a recipe's, below). The player's order fails ("Dee is too young to pour a glass of wine", "Ada is too big for the rocking horse"), residents never choose it, and the pie menu greys it out; a seat (a `sit` interaction) out of their stages is no seat for them to eat, phone or watch TV on. Within a rank, a diner takes a seat made for their age first (a seat whose stages leave someone out: a child has a snack at the kids' table). An unknown id or an empty list is an error. Default: every stage but a baby's (and for a `baby` interaction, a baby's). |
| `upperStages` | Bunk beds: the life stages that may take the upper bunk (base game: not an elder). With the lower bunk taken by someone else, anyone else is refused ("Gran is too old for the top bunk") and doesn't choose it. Only with `bunk`. Default: everyone who may use the bed. |
| `slots` | How many residents use it at once, 1 to 4 (double bed, sofa, a long sofa for 3, a merry-go-round for 4), spread across its width, facing its front (unless `places` puts them). Used standing, each stands in front of their own slot along its front, facing straight in (two at a double sink, one at each basin; a carer at a thing with a baby's slot, the crib, in front of the baby; a use that opens or shuts it, or takes the whole of it, in front of its middle), as does the one user of a thing an even number of tiles wide. Default 1. |
| `places` | Where each slot's user sits, lies or stands and which way they face, one a slot: `[{"at": [-0.3, 0.7], "facing": 180}, {"at": [-0.3, -0.7]}]` (a picnic table's two, across it from each other). `at`: metres off the footprint's middle, to the object's right and front, before turning (a resized object moves them out with its size); within 2 m of the footprint. `facing`: degrees from the object's front towards its right (90 faces its right, 180 its back, -90 its left; default 0). A resident takes the free slot whose place is nearest them. Sitting or lying, they walk to the first tile off the footprint behind their place (round to a picnic table's far bench) and sit or lie at it, the seat's `forward` then measured along their facing (give the depth in `at` and keep `forward` 0); standing, they walk to a place off the footprint and stand on it, facing the way it says (the two ends of a ping-pong table, the pool and foosball players on its two long sides, the oche two tiles out from the dartboard). A tile that can't be stood on (a wall, furniture) sends them to the object's own tile in front, as without places. Floor and wall things, not a `bunk`. Default: spread across its width, facing its front. |
| `light` | Lamps: `{"range": 4.5, "intensity": 1, "height": 1.55, "tiles": 10}` makes it a light at night (reach in metres, relative brightness, bulb height; `at`: `[x, z]`, where the bulb is off the middle of its footprint, metres in its own frame before turning, for a lamp whose shade isn't over its middle (an arc lamp, a lamp at one end of its table), default `[0, 0]`; `tiles`: floor it lights fully as its room's light is scored, default `roomRules.lampTiles`: a chandelier ~25, a ceiling light ~16, a floor lamp ~10, a sconce ~5, a table lamp ~4–6; `switch`: `false` for a light nobody switches: it is lit every night; `fire`: `true` for a fire (a fire pit, a fireplace), switched as a lamp is but lit by someone who sits down by it in the dark, its light, glow and flames together, out by the morning; `aim`: where its shade sends the light, default all round: `down` (a pendant's dome, a desk lamp), `up` (a sconce's open cup: a wash up the wall), `out` (a wall light: away from its wall and down, none behind it) or `ceiling` (an uplighter: up at the ceiling, which lights the room). Other lamps can be switched off, one by one by the player, and a room's all together by residents coming and going after dark; one that's off gives no light and its shade no glow). |
| `sounds` | What's done at it sounds like (looks only, the renderer's world sounds), by animation tag: `{"stir": "none"}` (an oven: a meal's Cook step doesn't fry), `{"wash": "crockery"}` (a dishwasher: dishes go in without a running tap), `{"bake": "fire"}` (a pizza oven), `{"fetch": "fridgeDoor"}` (a fridge: a snack or a meal's ingredients taken out). For its uses without a `sound` and the cooking steps done at it as a station. Keys, played while it lasts (`web/src/audio/world.ts` `LOOPS`): `sizzle`, `chop`, `shower`, `tap`, `washUp`, `bathFill`, `bathSoak`, `washer`, `dryer`, `crockery`, `fountain`, `tv`, `music`, `drums`, and from `web/src/audio/things.ts`: `treadmill`, `bike`, `tools`, `bikeTools`, `pool`, `foosball`, `pingPong`, `darts`, `arcade`, `sewing`, `potteryWheel`, `bubbling`, `espresso`, `blender`, `mixer`, `microwave`, `toaster`, `ovenFan`, `hood`, `fire`, `basketball`, `singing`, `cello`, `punchBag`, `smash`; played once as it starts (`ONCE_THINGS`): `fridgeDoor`, `binLid`, `vendSnack`, `vendCan`, `phoneBox`, `pour`; or `none` for silence. Default: by the animation (`ACTION_SOUNDS`: a `bake` in an oven is its fan). |
| `ambient` | A world sound it makes by itself, used or not, while its lot is looked at (a key as for `sounds`): `"fountain"`. A broken one is silent. |
| `ambientWhile` | With `ambient`: it sounds only while a use on its tiles sounds like this (a key): the cooker hood's `"hood"` while the hob under it is in use (`"sizzle"`). |
| `runs` | A machine that runs on its own: `{"minutes": 60, "sound": "washer"}`. An interaction that `loads` it (below) is offered only while it's empty; once that use ends it runs `minutes` game minutes with nobody at it (its model's `motions.running` clip goes round, and `sound`, a key as for `sounds`, is heard while its lot is looked at), and whoever loaded it is free. Then it's done, the load still in it, until an interaction that `unloads` it (offered only then) empties it. Nothing else waits on it: a done machine waits, as a full washer does, until someone takes the washing out (residents choose it as any chore). Saved with the object (`objects[].run`). The washer (60), the dryer (45), the washing line (180: the washing drying in the sun), the neat pack's laundry centre (90) and the dishwashers (90, the slimline one 75; see `rack`). Code: `crates/sim-core/src/machines.rs`. |
| `rack` | A machine that runs on its own, filled a dirty thing at a time (it needs `runs` and the `dishes` station): how many fill it. What's washed up at it goes in and waits (`objects[].rack`, saved) instead of being washed there and then and put away; it runs once full, or once the washing up is done (nothing else of the household's waits to be cleared) with it a third full. While it runs, or waits done, nothing more goes in (the washing up goes to a sink, or waits) and its `workOn: "dishes"` use is refused; an interaction that `unloads` it empties it, the clean things away. The dishwasher 12, the slimline one 8. |
| `stations` | Cooking stations it is (see [Cooking](#cooking)): `["ingredients"]` for a fridge or pantry, `["prep"]` for a counter or island, `["stove", "oven"]` for a cooker, `["grill"]`, `["microwave"]`, `["dishes"]` for a sink or dishwasher, `["cupboard"]` for a wall cabinet, sideboard or china cabinet (clean things are put away there). Recipe steps happen at stations. A station with no interactions (a wall cabinet) is used only while the tile in front of it is free. |
| `reach` | How high a hand goes to use it as a station, in metres: a wall cabinet's bottom shelf (`1.47`). Someone whose hand doesn't reach that high (a child: a life stage's `scale` × 2.1 m) goes to another station: the clean plates go on the counter, not up in the wall cupboard. Default `0`, within anyone's reach. |
| `footstool` | A footstool (a pouf, an ottoman): the feet go up on it from a seat in front, and someone sitting down alone on a sofa takes the seat with one 0.3–1.2 m in front of it. Default `false`. |
| `provides` | Things it provides while it works, for interactions elsewhere on the same lot that list them in `requires`: `["coffee"]` for a coffee machine. A broken object provides nothing. |
| `staff` | Someone serves here on a public lot (a vendor, a shopkeeper, the baker, a barista): `{"at": [0, -0.85], "hours": [10, 20]}`. `at`: where they stand, metres off the footprint's middle to its right and front before turning, as `places` (behind it: a negative front; within 1.5 m of the footprint); `hours`: when it's open, `[from, to]` (0 to 24, past midnight when `from` is later). Its uses with `selfServe` are then sold: the customer comes to the front and waits for the server, who serves one at a time, first come first served (`service.waitMinutes` at most, then they give up); while they're served the use runs, drawn as ordering, being handed it and paying (`buy`, the server's `sell`), and outside the hours it's closed (last orders: someone who set off while it was open and gets there within half an hour of closing is still served; otherwise residents don't choose it, the order fails with "the ice cream cart is closed: it opens at 10:00", the pie menu greys it out and says when). Posts on one lot within `service.share` metres of each other, with a straight walk between, share a server who steps over (the café's counter and coffee machine). Nobody serves where there's no room for them (`at` on a wall, another thing or off the lot) nor on a household's own lot: there the uses are self-serve, as `selfServe` says. The staff aren't residents: they need no slot, have no needs and aren't saved (a town has at most 32). Code: `crates/sim-core/src/staff.rs`. Default: nobody. |
| `stores` | Fridges: keeps leftovers. Food served where no surface is free goes in, and keeps longer there (`cooking.storedSpoilHours`). Default `false`. |
| `grows` | A crop (or a fruit tree's fruit: the apple and lemon trees, `"fallow": 0`, picked bare and fruiting again over `days`): `{"days": 2, "fallow": 0.5}`. Bought, it is just planted and ripens over `days`; one in a lot's or a house's file comes ripe. An interaction with `"leaves": "picked"` (the harvest) picks it as the use starts, and one with `"when": "ripe"` is offered only while it's ripe (the order fails with "the … isn't ripe yet"; residents don't choose it). Picked, it lies fallow for `fallow` game days (default 0.5), is planted again by itself and ripens over `days` (sooner for uses with `tends`), showing as just planted for the first third, then growing. Saved. Its model's `motions.growth` clip shows the stage (docs/design/objects.md, "Moving parts"). |
| `freeRotation` | `true` lets the player turn it to any angle in 15° steps, only for looks. (Anything but stairs stands anywhere on its tiles, in quarter tiles or to the centimetre with Shift, as long as no wall, straight or diagonal, runs through its body: see `offset` in a town's `objects`.) Default: 1×1 objects not on a wall whose category is listed in `objectRules.freeRotation` (base game: `decor`, `garden`, `outdoor`, `lighting`, `kids`, `community`). |
| `decor` | Decor points it gives the room it stands in (see [Rooms](#rooms-wear-and-accidents)). Default: by category, `roomRules.decorByCategory`. |
| `wearPerUse` | Wear added by each completed use; at 1 it breaks. Default: by category, `objectRules.wear` (nothing wears out that nobody uses). |
| `description` | Catalog text: say what it's for and what makes it special. |
| `interactions` | What residents can do with it (below). Can be empty for decor. |

### Interactions

```json
{
  "id": "cook",
  "label": "Cook a hearty meal",
  "anim": "cook",
  "minutes": 45,
  "effects": { "hunger": 0.8, "fun": 0.15 },
  "tags": ["food", "cooking"],
  "skills": { "cooking": 0.2 },
  "pose": "stand",
  "autonomous": true,
  "cost": 6,
  "skill": "cooking",
  "feeling": "deliciousMeal",
  "feelingMinSkill": 4
}
```

| Field | Meaning |
|---|---|
| `id`, `label` | Unique within the object; the label is the menu text ("Soak in the tub"). |
| `anim` | What the resident is shown doing while using it: one tag from the content's `animations` list (see [Animations](#animations)), e.g. `eat` for a snack, `cook` for a meal, `wash` at a sink, `type` at a computer. An unknown tag is an error. Optional, but give every interaction one; without it the game guesses from `tags`, `pose` and `effects`. |
| `minutes` | Game minutes for the whole interaction (> 0). Residents stop early when every need it raises is full. |
| `effects` | Need id → total change over the full duration (0..1 scale; negative = a cost, e.g. a workout makes you sweaty). Gains are spread evenly over the minutes. |
| `broadcast` | Share (0..1) of this interaction's positive need gains that other residents nearby get too, as if they heard or saw it (watching TV from the sofa while someone eats at the table). Same room or within `broadcastRange`, not asleep, and not the resident using it. Default 0 (nobody else). |
| `broadcastRange` | How far (m) `broadcast` carries outside the same room. Default 6. |
| `tags` | Tag ids (see [Tags](#tags)): drive trait/emotion/schedule preferences, "busy" (can't be interrupted by a chat) and what counts as sleep. |
| `skills` | Skill id → levels gained per hour of use (before the per-level slowdown). |
| `pose` | `stand` (default), `sit` or `lie`. |
| `seatNear` | Sit on a free seat within this many metres of the object before using it, if there is one (watching TV from the sofa). The resident walks to the seat, sits facing the way it faces, and uses the object from there; the seat is reserved while they do. Seats are sitting interactions with a `cooking.seatTags` tag and no `seatAvoid` tag. Default 0 (use the object standing in front of it). |
| `seatFacing` | With `seatNear`: only a seat that looks at the object, within 60° of straight ahead (the place they would take on it, see `places`): `front` from in front of it too, within 60° of the way it faces (a screen: no sofa with its back to the TV, none behind it); `around` from any side (a fire pit). With no seat like that near, it is used as `pose` says (standing). Default: any seat near. |
| `inside` | Used standing on the object rather than in front of it: the resident steps in onto its centre (or its slot), facing the way it faces, and steps back out where they came from when done (the same as sitting down and getting up). Every shower has it. Default `false`. |
| `autonomous` | `false` = only when the player asks. Default `true`. |
| `stages` | Who may take it at their age (life-stage ids), as the object's `stages` (above), which it overrides: the party bar's *Mix drinks for everyone* is for grown-ups, its soda for anyone. Default: its recipe's `stages`, else its object's. A `baby` interaction can't name another stage. Two rules hold whatever the stages: `homework` is done only by pupils (an order fails with "… has no homework to do"), and `care` only by grown-ups ("… is too young to look after a baby"), with a baby there ("there's no baby in the crib"). |
| `needsTable` | Only offered while the seat faces a table it tucks under (its `seatAt`): dining at a table. A chair alone offers only its other interactions; the player's order fails with "needs a table". Needs a `seatAt`. Default `false`. |
| `requires` | Things a working object on the same lot must provide (`ObjectDef.provides`): `["coffee"]` on *Have a coffee*, so a table or chair offers it only with a coffee machine (or espresso bar) there. Broken providers don't count. Residents don't choose it, and the player's order fails, while one is missing. Default `[]`. |
| `fetch` | With `requires`: what it requires is fetched first, with the interaction of this id on the nearest working object on the lot that provides it, then the use goes on here (the resident comes back to it). *Have a coffee* has `"fetch": "pour"`: the coffee is poured at the machine (*Pour a coffee to take to a table*, 2 minutes, not chosen on its own) and had at the table; the café chair's *Have a slice of cake* takes it from the bakery case (`takeCake`). With no way to the source, they do without the fetching. The interaction it names must be a plain use (no `recipe`, no `eat`) on something that provides everything this one requires. Default: nothing fetched. |
| `whole` | Takes every slot of the object (see `slots`): residents choose it, and it starts, only with nobody else on the object, and while it lasts the object is full for everyone else (their orders for it are dropped when they come up, as for any full object). The player's order fails with "someone is using the …" while someone else is on it. For a nap along a 2-seat bench: the window seat's *Doze under the throw*, the hearth settee's *Nap under the quilt* (the renderer lays the napper along it, `BEDS` `across`). Default `false`. |
| `recipe` | Starts a recipe (`recipes` id; see [Cooking](#cooking)) instead of being used here alone: the resident goes round the recipe's stations (fridge, counter, stove), from whichever one they clicked, and serves a dish. `minutes` isn't needed (the steps take theirs); `effects` are then only what cooking does for the cook (`{"fun": 0.05}`), the meal's own are the recipe's. Keep the `tags` (`food`, `cooking`): kitchens, plans and traits go by them. `cost` comes from the recipe. |
| `eat` | Food to sit down with: `{"item": "snack", "minutes": 8}`. `minutes` is then the fetching or making (a snack from the fridge: 2, toast: 4); once that's done the resident holds the `item` (an `items` id, a plate or a bowl), sits down with it like with a served dish (a chair at a table, then a table with seats, then any seat, else standing; see [Cooking](#cooking)) and eats it for the `eat` minutes. The positive `effects` come from eating (times the object's quality), so autonomy weighs the interaction just as before; a negative one is still spread over the fetching. Give it a fetching `anim` (`fetch`, `cook`). Not with `recipe`. Drinks stay plain interactions, gated with `requires` when they need an appliance (coffee needs a coffee machine; `fetch` has it poured there first). Food bought in town is the same: the stall's fruit, the cart's ice cream and hot dog, the snack machine's snack, the bakery's pastry and the grocer's sandwich are fetched (and paid for) at their source, then eaten on a bench or at a café table there (their items have no `emptyAs`: nothing is left to clear; a plate on a table off the household's lot stays while it's eaten). |
| `yields` | A harvest's produce: `{"item": "gardenProduce", "servings": 2, "minutes": 8}`. When the use finishes (not when it's cancelled or given up at once) the resident holds the `item` (an `items` id) with that many servings and carries it to the home's nearest working fridge (`stores`), puts it away there (a moment, the `cooking.putAwayAnim`) and it's food in the fridge like leftovers: anyone at home eats a serving later, at a table, for `minutes`, and gets the interaction's positive `effects` then (they're moved off the harvest, as with `eat`; residents weigh it all the same). A harvester hungry for it has a serving there and then. With no fridge at home they sit down and eat a serving at once (as `eat`); called away on the way, what they carry is put down (a surface, else the floor). Not with `eat` or `recipe`; `servings` at least 1. The base game's vegetable patch, raised bed, tomatoes, strawberries, pumpkins and apple tree's *Pick the apples* (G-68). |
| `tends` | Brings the object's crop on (0..1; the object needs `grows`): a use that finishes grows it this share of its way from planted to ripe at once (a use cut short, its share of that), while it grows (not ripe, not fallow). Weeding and watering (0.15 for a 30-minute *Weed and water*, 0.1 for watering the lemon tree): a crop tended once a day ripens a few hours sooner. Default 0. |
| `when` | `"open"` or `"closed"`: only offered while the object is so (a wall bed's *Fold up* while it's down). `"ripe"`: only while its crop is (see `grows`). Objects start closed. Residents choose it, and the player's order works, only then; the order fails with "the … is closed" / "is already open". Default: always. |
| `leaves` | `"picked"`: picks its crop (see `grows`) when the use ends: the hands pick it through the harvest, and such a use goes on to its end even once the needs it fills are full. `"open"` or `"closed"`: opens or closes the object as the use starts (as the resident gets to it), or with `gradual` when it ends, and leaves it so; saved with the object. Nothing closes an object someone else is using or walking to ("someone is using the …"; when someone else gets there first after the order was given, the order is dropped and the story says so, event `notClosed`). After a night's sleep or a nap (`dayRhythm.sleepTags`, `napTags`) in an object that is open, the last one up uses its interaction that `leaves` it `"closed"` (a wall bed's *Sleep* has `"leaves": "open"`; its *Fold up* `"when": "open", "leaves": "closed"`), and so does the last one done after a use with `closeAfter` (below). Nothing else is closed for you (the washing stays on the line until someone takes it in). Its model holds the entry's `motions.open` clip open while it's open (docs/design/objects.md, "Moving parts"). Default: leaves it as it is. |
| `gradual` | With `"leaves": "open"` or `"closed"`: what it leaves comes about through the use, as the hands get there, so the object is so once the use is done (and it goes on to its end, as a harvest does): the washing is hung out piece by piece, taken in again. Its model's clip plays along with the use (`motions.uses` with `over`, docs/design/objects.md); a use cut short leaves the object as it was. Default `false`: as the use starts (a wall bed folded down to lie in). |
| `hours` | `[from, to]`, hours of the day (0 to 24, two different ones): only started then; past midnight when `from` is later than `to` (the telescope's *Stargaze*: `[20, 5]`, after dark). Residents don't choose it outside them, and the player's order fails with "Ada can only stargaze from 20:00 to 5:00"; one ordered in time but reached too late doesn't start. A use already going runs on. Default: any hour. |
| `sky` | `true`: only under open sky, the object standing outside on every tile (the telescope). The order fails indoors with "… can only stargaze under open sky: the telescope has to stand outside"; residents don't choose it there. Default `false`. |
| `workOn` | Something it works on, which must be there for it to start, taken up as it starts: `"dishes"` (dirty dishes or food gone off of the household's to clear: *Wash the dishes*, the dishwasher, sorting the recycling), `"laundry"` (a load of the household's dirty washing: loading the washer), `"washed"` (a washed load, wet: loading the dryer, hanging out the washing) or `"dried"` (a dry load: folding it at the laundry basket). Washing piles up as content `laundry` says (below). Residents don't choose it with nothing there, and the order fails with "Wash the dishes: there are no dishes to wash" / "there's no washing to do" / "there's no washed laundry to dry" / "there's no dry laundry to fold". An unknown value is an error, and so is a load of washing without content `laundry`. Default: nothing needed. |
| `makes` | A load of washing it leaves the household when it ends (not when cancelled or given up at once): `"washed"` (taking the washing out of the washer), `"dried"` (out of the dryer, in off the line) or `"laundry"`. Such a use goes on to its end. Then the resident carries the load on to the nearest working thing on the same lot with a use that works on it (`workOn`): from the washer to the dryer or the line, from the dryer to the basket; with none, it waits for one. Needs content `laundry`. Default: none. |
| `loads` | Puts a load in a machine that `runs` (above): only while it's empty ("the … is still running", "the … is done: take the washing out first"), and it starts its run once the use ends. Goes on to its end. Default `false`. |
| `unloads` | Takes the load out of a machine that `runs`, once its run is done ("there's nothing in the …", "the … is still running"): it's empty again as the use ends. Goes on to its end. Not with `loads`. Default `false`. |
| `brings` | An item (an `items` id) carried in the arms on the way to it, only to be seen: the basket of washing to the washer, the dryer and the line, an armful of dry washing to the basket. Nothing is put down or picked up. Default: none. |
| `holds` | An item held in the hands while it's done, only to be seen, as `brings`: a towel being folded at the basket, a slice of cake at the café (eaten with a fork: it is also what they eat, the snapshot's `plate`), the bottle the baby is fed from at the crib (drawn as the prop in the hand, not carried: `PROP_ITEMS` in `actions.ts`). Default: none. |
| `partner` | `true`: a game for two (the seesaw, ping-pong, park chess, pool, foosball), on an object with `"slots": 2` and its two `places` (an error otherwise, and with `whole`, `inside`, `seatNear`, `recipe`, `eat`, `yields`, `carried`, `needsTable` or a baby's place). Whoever starts it (both places free) fetches someone to play: on the same lot, allowed to use it there (a guest as guests may), its age for them (`stages`), free (doing nothing or something they picked that isn't busy, nothing the player asked for next, not pressed, not about to leave for work or school) and not on bad terms with them; one the player told to play the same game first, else the nearest. They drop what they're doing and go to the other place. Each waits at their place for the other (drawn waiting, not playing: the seesaw doesn't tip with one); then it runs for both, each getting all its `effects` (so `social` is earned with someone), goes on to its end even once the needs it fills are full, and is over for both as soon as either stops (done, cancelled, called away). Residents choose it only with someone free there; the player's order fails with "nobody is free to ride the seesaw with Dee" (or "someone is using the …" with another use going on at it), unless it has `alone`. With nobody to play after all, it's given up, and the player hears (story `noPartner`). Code: `crates/sim-core/src/partners.rs`. Default `false`. |
| `private` | `true`: done behind a shut door. While it lasts the room it's done in is kept shut: others wait outside its door or go elsewhere (see `roomKinds` `private`). Default `false`. |
| `conceive` | `true`: where partners try for a baby (the double beds' *Try for a baby*). Needs `partner`, `"pose": "lie"` and `"autonomous": false` (an error otherwise), and is `private` too. It is started only by the social whose success has `"effect": "conceive"`: when that goes well the two go to the nearest such bed at home that nobody else is on, in a room indoors with a door and nobody else in it (a baby may be), and lie there under the covers for its `minutes` with the door shut (someone else in the room as they get there: they don't); a baby may come of it once it has run its time (`life.pregnancy`). Not in the object's menu; an order for it is refused ("… is asked of a partner"). Code: `crates/sim-core/src/privacy.rs`. Default `false`. |
| `alone` | With `partner`: what the player's order to play does with nobody free, another use of the same object by id, for one (no `partner`) and in the same pose: the pool table's *Shoot pool* has `"alone": "trickShots"`, the foosball table's *Play foosball* `"alone": "practise"`. The story says so (`playedAlone`: "Ada had nobody free to play with at the pool table, so practised alone"). An unknown id, or one that's itself a game or in another pose, is an error. Default: none (the order is refused). |
| `closeAfter` | With `"leaves": "open"`: opening it is for this use only. When the use finishes, the last one done uses the object's interaction that leaves it closed, as after a night's sleep (the hot tub's *Soak* puts its cover back on). Not when the use is cancelled. Default `false`. |
| `changesOutfit` | `true`: changing clothes (a wardrobe, a chest of drawers). Halfway through, the resident is in their next outfit: the one made in the creator, then two more the game makes for them, in turn; they keep it until they change again (saved with them). The use lasts its `minutes` even with the needs it fills full. |
| `spare` | A spare bed's night, on a use with a `dayRhythm.sleepTags` tag (an error on any other): the sofa bed pulled out for a guest. Its tags don't count towards what the room is for, so a sofa bed leaves the living room a living room (a `sleep` tag elsewhere makes the room a bedroom, `roomKinds`); residents sleep in it only with no other bed free for them (one in the bedroom takes the bed, the other the sofa bed), and it counts as a bed place for a household moving in. A household buying a bed for a child out of the crib buys a real one. Default `false`. |
| `carried` | A baby's care (`care`) on an object with a place for a baby (`baby`: the high chair's *Sit in the high chair*), done with the baby carried there: by day (not at night, when the feed stays at the crib, in the arms), a grown-up doing the crib's care of the same `id` (the crib's *Feed the baby*) goes over to the crib, lifts the baby out, carries them in the arms to the nearest free such object on the lot, puts them in it and does this care from beside it (a high chair at a table has the table in front and its back to the room); done, they carry the baby back to a crib. Carried, the baby is in no place and goes where the carer goes; put down anywhere else (the order cancelled, the carer called away) or left there when the baby place's `minutes` are up, the baby is back in their crib at once. The carry isn't saved (after a load the baby is back in the crib, or still in the high chair). An error without `care` or with no `baby` interaction on the object. Default `false`. |
| `atSlot` | Done standing in front of the place the resident takes on the object (`slots`, spread across its width), not its middle, and facing it: the crib's *Change the baby* at its changing table, the right half of the 2 × 1 crib (slot 1, the carers'; the baby's cot is slot 0). Falls back to the object's front when that tile can't be stood on. Default `false`. |
| `after` | A tag: what the resident does straight afterwards. When the interaction finishes (not when it's cancelled), they go on to the nearest working object on the same plot with an interaction of that tag, before anything else they'd choose (an order from the player still comes first), and it runs its full `minutes` even with the needs it fills full. With nothing like it there, nothing follows. Toilets have `"after": "handwash"` and sinks' *Wash hands* the `handwash` tag. |
| `cost` | Dollars charged from the resident's household each time a resident **starts** using it (on arrival). Default 0. Only for real consumables bought from outside (takeout, deliveries, a rented movie, donations); owning and running an item costs money through the weekly bills instead. Residents never choose something their household can't afford, and the player's order fails with "not enough money". Guests pay from their own household. |
| `skill` | Skill id that makes this interaction better: need gains and skill gains are multiplied by `1 + skillRules.effectPerLevel × level` (0.05 per whole level: level 4 = ×1.2, level 8 = ×1.4). Autonomy knows this, so good cooks prefer cooking. |
| `feeling` | Feeling id granted when the interaction finishes after at least half its `minutes` (not when cancelled early). |
| `feelingMinSkill` | Only grant `feeling` if the resident's whole level in `skill` is at least this (needs `skill` and `feeling`). Default 0. |
| `dirt` | Mess each use leaves on the tiles around the object (0..1). Default: by tag, `roomRules.dirt` (cooking 0.12, food 0.06, bathroom 0.05, hygiene 0.04). |
| `sound` | What it sounds like while it lasts (looks only): a world sound key (as for the object's `sounds`), `"chop"` for a cheese board, `"toaster"`, or `"none"` (silence). A key played once plays as it starts: `"vendCan"` (a can from the machine, then drunk in silence). Default: the object's `sounds` for its `anim`, else by `anim`. Not for a `recipe`: its steps sound by the station they're done at (and a step's own `sound`). |
| `endSound` | A sound played once as it ends: `"flush"` (toilets), `"ping"` (a microwave), `"toastPop"`, `"hangUp"` (the phone box). |
| `selfServe` | Sold by the object's server (`staff`): `{"label": "Take an ice cream from the freezer", "anim": "fetch", "minutes": 2}`, the label, animation and time where nobody serves (a household's own cart). Where someone serves, the interaction's own `label` and `anim` (`buy`) are used, it waits for the server, is closed outside the post's hours and takes the interaction's `minutes` (the order, the hand-over and the paying: 5, the `buy` and `sell` clips' five seconds at speed 1). Where nobody serves it takes `selfServe.minutes` instead, the helping oneself (2: no order, nobody to pay), else the interaction's own `minutes`; with `eat`, the food is what's handed over. An error on an object without `staff`, or with `minutes` not above 0. Default: not sold by anyone. |

### How the multipliers stack

Per need, each tick: `gain/min × needGain (traits × perks × active feeling buffs) × quality × skill`

- quality: `1 + objectRules.qualityBonus × stars` = +25% per upgrade star (0–3 stars, bought in Build › Furnish);
- skill: `1 + 0.05 × level` for interactions with `skill`.
- Negative effects (costs) are not multiplied.

Skill gains: `skills[s] × quality × skill factor × trait skillGain / (1 + level × 0.25)`.
So a 3-star object used by a level-8 resident gives `1.75 × 1.4 = 2.45×` the listed need gains.
Balance the listed numbers for a **0-star object and a level-0 resident**.

## Feelings

```json
{
  "id": "relaxed",
  "label": "Relaxed",
  "emotion": "relaxed",
  "mood": 0.08,
  "hours": 8,
  "effects": { "needGain": { "energy": 1.25, "comfort": 1.2 } }
}
```

| Field | Meaning |
|---|---|
| `id`, `label` | Unique id; label shown in the resident panel. |
| `emotion` | Optional emotion id. The resident's emotion is that of its active feeling with the largest `|mood|`; the emotion biases what the resident wants to do. |
| `mood` | Added to mood while active (range used so far −0.25..+0.2). |
| `hours` | How long it lasts (> 0). Getting it again refreshes the timer. |
| `effects` | Optional buff/debuff while active, same shape as trait effects: `needDecay`, `needGain` (need → multiplier), `tagPreference` (tag → autonomy multiplier), `tagAcceptance` / `tagSuccess` (tag → added success chance of socials), `walkSpeed` (multiplier), `skillGain` (skill → learning multiplier). Do **not** put `mood` here; use the feeling's `mood`. |

Buffs combine with traits and perks multiplicatively (acceptance/success add) and vanish
when the feeling expires. Typical sizes: multipliers 1.1–1.5 (0.7–0.9 for debuffs).

## Tags

Tags in use (64 maximum for the whole game, base plus all packs). Names in `provides` and `requires`
share this namespace and count towards the limit (base uses `coffee`):

| Tag | Used by |
|---|---|
| `food` | fridge snack/meal/gourmet, coffee at a table with a coffee machine |
| `cooking` | fridge meal, gourmet |
| `training` | anything practised for skill (gourmet, study, treadmill, weights, chess, computer, easel, mirror, workbench, telescope, piano) |
| `creative` | write (computer), paint (easel), play (piano) |
| `fitness` | treadmill, weight bench |
| `reading` | bookshelf |
| `screen` | TV, computer |
| `entertainment` | TV, read, chess, browse, piano |
| `rest` | sleep, nap, sofa |
| `sleep` | bed sleep (counts as a night's sleep: `dayRhythm.sleepTags`) |
| `nap` | bed nap, sofa doze (`dayRhythm.napTags`: short, and not again for a while) |
| `lounge` | sofa, armchair, bench, coffee |
| `spa` | whirlpool tub (example pack), baths and hot tubs; declared in base.json `tags` |
| `toys` | kids' things (toy box, play mat, doll house…); grown-up life stages like them far less (`tagPreference` 0.15, teens 0.4); declared in furniture.json `tags` |
| `hygiene` | sink, shower |
| `handwash` | sink *Wash hands* (what toilets' `after` goes on to) |
| `bathroom` | toilet |
| `chores` | water plant, tend flowers |
| `nature` | plant, flowerbed |
| `outdoors` | bench, flowerbed, telescope |
| `social`, `friendly`, `fun`, `touch`, `romantic`, `breakup`, `mean`, `violent`, `makeup` | socials (conversations) |
| `visit` | built in: visiting another household |
| `phone` | the phone's actions (`phone.actions`): texting, calling, inviting, scrolling |

Special meanings: `sleep`, `bathroom`, `hygiene`, `nap` and `intimate` (trying for a baby) make a
resident busy (no chats, guests can't use them, except those in `socialRules.guestTags`:
`bathroom`, so a guest may use the toilet); `romantic` socials need attraction; `sleep` lasts
through the night. An object marked `private` is the household's own: guests use none of it.

**Group conversations:** a social with `"joinable": true` (base game: chat, joke, deep talk) can
be joined by up to two others nearby, who listen and fill their needs by its `targetNeeds`.
When it goes well, those listening get its `targetFeeling`, and everyone there gains
`socialRules.joinFriendship` friendship with everyone else (default 2; the two talking get
their social's outcome with each other instead).

**Adding a tag:** reuse an existing tag whenever it fits. Add one only if a trait, emotion,
schedule or buff must tell your objects apart. A tag exists once any interaction uses it;
to refer to a tag before anything uses it (e.g. in a `traitPatches` preference), declare it
in a top-level `"tags": ["myTag"]` array. Unknown tags in effects are errors. With 15 packs
sharing 64 slots (28 in use), **add at most two new tags per pack**.

## Animations

The renderer can't tell eating from cooking by where a resident stands, so each interaction
names an animation tag in `anim`. The tags are declared once, in the top-level
`"animations"` list in `base.json`; that list (in that order) is what the render snapshot
sends as `layout.actions`, and each resident's `sim.action` float is an index into it (-1 =
nothing). The snapshot also sends `sim.object`, the id of the object the resident is using or
walking to (-1 = none); `sim.action` is set only once the resident is actually using it, and
is `talk` during conversations. `sim.objectDef` and `sim.interaction` say what is being used, as
indices into the content's `objects` and that object's `interactions` (-1 = none); resident
voices pick their lines by them.

| Tag | Use it for |
|---|---|
| `eat` | eating food: fridge snack, takeout, a meal at the table |
| `cook` | preparing food at a counter, stove or oven (fridge meal, gourmet, baking) |
| `drink` | coffee, tea, sodas, smoothies, mocktails |
| `wash` | sink, quick grooming (spritz, vanity) |
| `shower` | standing showers |
| `bath` | tubs, hot tubs, cold plunges |
| `toilet` | toilet |
| `sleep` | a night's sleep in bed |
| `nap` | naps and dozing (bed, sofa, hammock, recliner) |
| `sit` | just sitting (armchair, bench) |
| `relax` | lounging: sofa, recliner, hammock swing, sauna, warming up |
| `type` | keyboards and touch panels: computer desk, programming a device |
| `write` | pen and paper: letters, journals, poems, ledgers |
| `read` | books, magazines, recipes |
| `watch` | TV and movie screens |
| `look` | looking at something: stargazing, birdwatching, gazing out of a window, photos |
| `admire` | looking up at a tree's crown or blossom |
| `smell` | bending down to low flowers, watching fish in a pond |
| `listen` | listening to records or a story |
| `play` | games: chess, puzzles, consoles, arcade |
| `music` | playing an instrument: piano, cello, drums |
| `sing` | karaoke |
| `dance` | dancing |
| `paint` | easel |
| `tinker` | handiwork: workbench, radio repair, knitting |
| `exercise` | general workouts: climbing, cycling, stretching, punching bag |
| `run` | treadmill |
| `lift` | weights |
| `water` | watering plants and flowers, misting, feeding |
| `tend` | kneeling work on beds: weeding, tending vegetables (also patting the gnome) |
| `harvest` | picking: vegetables, fruit, herbs, flowers |
| `prune` | trimming hedges and shrubs, shaping a bonsai, cutting lavender |
| `garden` | other garden work (bird feeder refills, feeding fish); prefer one of the four above |
| `clean` | mopping, laundry, tidying |
| `talk` | conversations (automatic); talking to a mirror |
| `phone` | calls and video calls, radio chats; everyone's smartphone (`phone.anim`) |
| `phoneScreen` | looking at a phone held in front: texting and scrolling (`phone.actions[].anim`) |
| `meditate` | breathing, counting to ten |
| `idle` | standing at an object with nothing better to show (donation jar) |
| `fetch` | taking ingredients out of the fridge or pantry (a recipe's first step) |
| `chop` | preparing food on a counter (a chopping board is drawn on its top) |
| `stir` | cooking on the stove (a pan is drawn on it) |
| `flip` | grilling (food is drawn on the grill) |
| `serve` | putting a dish down |
| `mix` | mixing batter or making pastry on a counter (a mixing bowl is drawn on its top) |
| `bake` | putting a dish in the oven or the microwave and watching it (nothing drawn: it's inside) |
| `buy` | being served at a cart, a stall or a counter (`selfServe`): ordering, taking it, paying |
| `sell` | the server behind it (`service.anim`, not for interactions): listening, reaching for it, handing it over, taking the money |

**Adding an animation:** reuse a tag whenever one is close enough; the characters need a
clip for every tag. If you must add one, append it to a top-level `"animations"` list in
your pack (lists from all files are combined; repeats are fine) and tell the art side.

Without an `anim`, the game picks the first that fits: `reading` tag → `read`; `cooking` →
`cook` (`eat` when seated); `sleep` → `sleep`; `nap` → `nap`; `bathroom` or mostly bladder
→ `toilet`; mostly hunger → `eat`; `social` or mostly social → `talk`; `fitness`/`sport` →
`exercise`; `gaming` → `play`; `music` → `music`; `garden` (or `chores` + `nature`) →
`garden`; `cleaning`/`chores` → `clean`; `spa`/`hygiene` (tag or main need) → `bath`
seated, else `wash` (≤ 10 minutes) or `shower`; `screen` → `type` (seated training) or
`watch`; other `food` → `drink`; `creative` → `write` seated, `paint` standing;
`rest`/`lounge` or lying → `relax`; seated → `sit`; else `idle`. A result missing from
`animations` means no animation (-1).

## Voice lines

What residents say out loud lives in `web/public/content/voice/<lang>.json` (English:
`en.json`; its `_comment` describes the layout). A pack adds lines for its own items, or
replaces lines, under a top-level `voice` key with the same layout, per language:

```json
"voice": {
  "en": {
    "object": {
      "chefStove": {
        "lines": ["A proper chef's stove."],
        "interactions": {
          "cook": ["Six burners, all mine!", "Let it sizzle!", "Tonight, I cook like a pro."],
          "practise": { "lines": ["Practice makes perfect.", "Let's try that recipe again."], "tone": "happy" }
        }
      }
    },
    "interaction": { "soak": ["A long soak, at last.", "Pure bliss."] },
    "action": { "cook": ["Smells good already."] }
  }
}
```

| Section | Keyed by | Said when |
|---|---|---|
| `object` | object id: `lines` for any of its interactions, `interactions` per interaction id | a resident starts using it |
| `interaction` | interaction id, on any object | the same, if the object has no lines of its own |
| `action` | animation tag (see [Animations](#animations)) | the same, if neither has lines; also repairs, accidents, knocking, tidying |
| `social` | social id: `start`, `good`, `bad` | starting a conversation; answering, by outcome |
| `emotion` | emotion id | the resident's emotion changes |
| `thought` | planner thought kind (`skipped`, `noPlace`, `kept`, `goal`, `roomLoved`, `roomDisliked`, `broken`, `accident`, `crying`) | a thought bubble appears |

- For something being used, the most specific lines win: object + interaction, then the object's
  `lines`, then the interaction id, then the animation tag.
- A group is a list of lines or `{"lines": [...], "tone": "..."}`; tones are `happy`, `sad`,
  `angry`, `flirty` and `question`. `{name}` is the other resident's first name, `{me}` the
  speaker's. `[word](/phonemes/)` gives a word's phonemes in Misaki's alphabet for the mouth
  shapes (the voice model reads the word itself).
- Packs merge in `include` order, key by key: a later pack's list replaces an earlier one for the
  same key, other keys stay. Prefer adding lines for your own objects to replacing the base game's.
- Write 3–6 lines per key (a resident doesn't repeat their last few), short (most under 40
  characters), and right for children, teens and grown-ups alike. Babies don't speak.
- Check them with `cargo test -p voice --release --test lines` (after `bun run voice` has fetched
  the dictionary): every line must phonemize from the dictionary, and every key must name content
  that exists.

## Wall and floor coverings, door and window styles

Build mode's looks are top-level arrays, appended like objects, so a pack can add its own:

```json
"wallCoverings": [{ "id": "mypack.tealPaint", "label": "Teal paint", "finish": "plaster", "color": "#5E9C96", "price": 0 }],
"floorCoverings": [{ "id": "mypack.cherry", "label": "Cherry boards", "finish": "wood", "color": "#B9785A", "price": 8 }],
"doorStyles": [{ "id": "mypack.barn", "label": "Barn door", "leaf": "oak", "price": 240 }],
"windowStyles": [{ "id": "mypack.slim", "label": "Slim", "panes": "bar", "sill": 0.6, "head": 2.3, "inset": 0.3, "shutters": false, "price": 110 }]
```

| Field | Meaning |
|---|---|
| `price` | Wall covering: per wall face covered. Floor covering: per floor tile. Door / window: per door or window (replacing one with another style costs the new style's price). |
| `finish` | Wall covering texture: `plaster`, `wallpaper`, `siding`, `brick`, `wood`, `stone` or `tile`; floor covering: `wood`, `tile`, `carpet` or `stone`. `color` tints it (wood is a neutral grain: give it its colour). |
| `leaf`, `color` | Door: `panel` (painted; without `color`, front doors take the house's accent colour), `oak`, `halfGlass` or `glass` (`color` paints the frame). |
| `panes`, `sill`, `head`, `inset`, `shutters` | Window: glazing bars (`cross`, `grid`, `bar`, `transom`, `none`), opening bottom and top (m, walls are 2.8 m), wall left either side of the glazing (m, of the 1 m edge), shutters outside (`true`, `false`, or `"house"`: if the house's style has them). |

Saves store looks by id; a look whose id no longer exists loads as the default (the house's own
look for walls, the room's own floor, the first style for doors and windows). Ids must be unique within each list.

## Text and languages

The game is English only for now; other languages will come (text and voices). Write labels
and templates as plain text in the content (they are the English default and will be
overridden by id from locale files later), keep placeholders named (`{a}`, `{skill}`, `{n}`)
rather than building sentences from parts, and never refer to content by its label, only by id.

## Planner: activities, goals, discipline

The planner (each resident's weekly routines and life goals) is data too, so packs can add to it.

**Activities** are what a routine block is for. A block never names an object, only an
activity, so plans keep working when the house changes. Interactions with any of the
activity's `tags` count; `skill: true` lets a block name a skill instead (interactions that
train it count); `social` and `visit` count conversations and visiting friends; `sleep`
blocks decide when the resident's night is.

```json
"activities": [{ "id": "mypack.yoga", "label": "Yoga", "icon": "icon.ui.skills", "tags": ["yoga", "fitness"] }]
```

While a block runs, choices that fit it are boosted and others damped (urgent needs are
exempt); `planner` sets how strongly. If nothing at home or in a park offers the activity, the
block counts as "nowhere to do it" and the resident wishes for a place (the player can open
the catalog filtered to it).

```json
"planner": {
  "boost": 6, "offBlock": 0.35, "urgentBelow": 0.15, "floor": 0.15, "skipChance": 0.1,
  "keptShare": 0.75, "maxMinutes": 240, "maxSleepMinutes": 720, "reviewHour": 7,
  "maxGoals": 10, "maxSuggestions": 2, "goalFalloff": 0.15, "suggestionDays": 2,
  "discipline": { "lazy": 0.6, "energetic": 1.3 },
  "keptFeeling": "keptPlan", "noPlaceFeeling": "nowhereToDoIt", "goalFeeling": "goalReached",
  "askAgainDays": 3, "wishFeeling": "gotWish", "deniedFeelings": ["deniedAgain", "fedUp"]
}
```

`goalFalloff`: goals higher on a resident's list steer more; the goal at place *n* (0 = the top)
steers with strength `1 / (1 + goalFalloff × n)`, and only the highest goal of each kind counts.
`suggestionDays`: a goal a resident suggests is taken on by itself if the player neither
accepts nor dismisses it for that many days. `discipline` multiplies how well residents with a trait stick to plans (with their mood);
traits whose `tagPreference` dislikes an activity's tags skip it more. A pack's own top-level
`planner` object replaces keys of the base one (objects merge shallowly), so repeat the whole
`discipline` map if you change it.

**Goals** pick from a fixed set of kinds: `hasJob` (label may use `{category}`), `jobLevel`,
`promoted`, `skill` (`{skill}`, `{n}`), `friends` (`{n}`), `partner`, `funds` (`{n}`),
`own` (`{object}`: something from the catalog they'd like to have). A career goal (`promoted`,
`jobLevel`) takes the skill levels its grades need: they count toward it, and residents train
them harder. For `own`, residents pick what suits them (their traits' liked tags, skills they're
after, somewhere for a planned activity, or a better one of something they have), save up, and
once they can afford it with a week's rent and bills left over they ask the player
(`askAgainDays` after a no; `wishFeeling` on getting it; `deniedFeelings` from the second no for
the same thing on). Households nobody plays buy it themselves.
`weight` plus per-trait `traits` weights decide how often residents suggest them; `feeling`
(optional) is granted on reaching it.

```json
"goals": [{ "id": "mypack.chef", "label": "Reach {skill} level {n}", "icon": "icon.skill.cooking", "kind": "skill", "weight": 0.5, "traits": { "foodie": 2 } }]
```

**Day rhythm:** `dayRhythm.wakeFor` lists the needs that wake a sleeper when they get urgent
(default: all; the base game wakes for `bladder` and `hunger` only), and
`dayRhythm.workDecay` multiplies need decay while at work. `dayRhythm.workBreaks` are what
they see to there (and at school): each `{ "need", "below", "gain", "cost" }` tops `need` up by
`gain` once it drops under `below`, paid from the household at work (school lunch is free; a
break they can't afford is skipped). The base game has lunch (hunger under 0.5: +0.6, $10), the
toilet (bladder under 0.35), a coffee (energy under 0.3: +0.15, $3) and a wash (hygiene under
0.3: +0.2). A sleep (`sleepTags`) lasts until
the need beds fill most (energy) is full, at most the planner's `maxSleepMinutes`; at night a
rested sleeper lies in until `lieInHours` (default 0.5) before they'd get up (and nobody goes
back to bed nearly rested outside the night). A bed's stars add
`sleepQualityBonus` each (default 0.06) to sleep, instead of `objectRules.qualityBonus`, so a
better bed shortens a night a little. Naps (`napTags`) can't follow each other within
`napCooldownHours` (default 4), aren't taken when woken by an urgent need, and below
`exhaustedBelow` energy (default 0.2) residents go to bed whatever the hour (the schedule's
daytime damping on sleep is lifted, naps count a quarter; and a nap is allowed again, on the
sofa when no bed is free). On a work or school day the alarm goes off: a sleep ends
`wakeBeforeWorkMinutes` before they leave, rested or not, and nobody goes back to bed then.
`dayRhythm.babyWakes` (without it, nobody gets up for a baby) has a baby in their crib at home,
crying for one of its `needs` (default: all but surroundings) under `below` (default 0.3) while
every grown-up of the household at home is asleep, wake one of them: family first, then someone
not worn out, then whoever was woken longest ago (partners take turns), then the most rested.
At night, a grown-up at home who's up comes at once instead, dropping what they picked
themselves (a meal is left where it is, to finish later; a cook with no more than
`finishMinutes` of a step left, default 15, finishes it first, and the recipe goes on after;
not the toilet or shower, washing up, a longer step, what the player told them to do, or when
they leave for work or school within `readyMinutes`); with nobody up who can, one asleep is
woken. They do the crib's care interactions for each of those needs under `tendBelow`
(default 0.6), and those woken go back to bed, with the `feeling` (optional); the same baby
calls nobody again for `cooldownMinutes` (default 90). The base game wakes for hunger, bladder
and hygiene.
**Jobs:** `careerRules.market` turns on residents finding, quitting and losing jobs on their own.

**Needs and autonomy:** a need's `comfortable` (0..1, default 0) is the level residents like to
keep it at: below it the need weighs `autonomy.comfortBoost` times more (eased in over the tenth
below it), so they see to it when it's convenient rather than once it's empty (the base game:
bladder 0.6, hunger and social 0.4, hygiene 0.35, fun and comfort 0.3; the toilet comes at about
half). The `autonomy` object:

| Key | Meaning |
|---|---|
| `urgent` | Needs that get urgent (the essentials; default none, base game hunger, energy, bladder and hygiene). |
| `urgentBelow` | Below this such a need is urgent, if home has something for it (default 0.08, base 0.1): whatever doesn't fill one is damped by `lookAhead`; what they chose themselves (not a routine, not a player's order, not something for one of them) stops if the need would otherwise drop under half of this before the end; and a need a chat doesn't fill keeps them out of conversations (and breaks one off). It's also when `wakeFor` needs wake a sleeper and when draining and planned activities stop. |
| `comfortBoost` | See `comfortable` above (default 0, base 2: three times the weight). |
| `lookAhead` | Multiplier (default 1, off; base 0.25) on something another need home has a thing for would run down to `urgentBelow` during, counting the walk there and from there to the nearest thing for that need (for a night's sleep only the `wakeFor` needs count, at `asleepDecay`), and on anything that wouldn't be over before they leave for work or school. A recipe isn't started at all then (hunger counts too: it's filled only at the end). |
| `quickMinutes` | The longest a habit's top-up may take (default 30). |
| `retryMinutes` | Something they chose but couldn't get to (a plate on a counter nobody can reach, a bed boxed in) isn't chosen again for this long (default 30). |

`dayRhythm.habits` are what residents see to at the turning points of their day, each
`{"at", "need", "below"}`: `wake` (after a night's sleep), `bed` (on the way to bed; they choose
bed again after), `leave` (in the `dayRhythm.readyMinutes`, default 30, before leaving for work or
school, dropping what they chose themselves), `home` (on getting home from work or school) and
`meal` (after eating a serving). Unless `need` is at `below` or better (default 1), they first go
to the nearest quick thing that fills it (no longer than `quickMinutes`, no recipe, nothing that
needs a table, on the plot they're on); with several due, the one lowest for its `below` first.
The base game: the toilet on getting up (below 0.95), before bed (0.9), before leaving (0.85), on
getting home (0.9) and after a meal (0.85); a shower before bed below 0.3 hygiene; a snack before
leaving below 0.7 hunger; `readyMinutes` 45.

## Life: ages, family, moving

Content `life` (an object, so a pack can change single keys; see `docs/design/life-cycle.md`):

| Key | Meaning |
|---|---|
| `daysPerYear` | Game days per year of age at the *Normal* lifespan (default 2; *Short* halves it, *Long* doubles it). |
| `startAge` | `[min, max]` ages for residents nothing gives one (default `[25, 50]`). |
| `stages` | Youngest first: `{"id", "label", "from", "story", "feeling", "effects", "adult", "baby", "school", "scale", "head", "stoop", "greyHair"}`. `from` is the age it starts at; `story` the journal's words ("is an elder now"); `feeling` granted on reaching it; `effects` like a trait's (combined with traits and perks; `tagPreference` 0 means never on their own: children don't cook). `adult` (default: unless `baby` or `school`) may work, fall in love, move and have children, and a household needs one; `baby` lives in a crib and is cared for; `school` goes to school. `scale` and `head` size the body and the head (renderer), `stoop` 0..1 bends the upper back, `greyHair` 0..1 greys the hair. |
| `school` | `{"start", "hours", "days", "skills", "grades"}`: school days (weekday numbers, 0 = Monday) for stages with `school`; `skills` (id → weight) practised a little there; `grades` `{"attend", "mood", "missed", "start", "good", "goodFeeling", "poor", "poorFeeling"}`: grade points (0..100) per day at school (plus up to `mood` by how they felt), off for a day missed, a new pupil's grade, and the feeling on leaving school with a grade of at least `good` or below `poor`. Interactions with `"homework": points` are done only by pupils and raise the grade. |
| `pregnancy` | `{"chance", "days", "maxAge"}`: when partners' *conceive* social goes well (a social whose `success` has `"effect": "conceive"`), they go to bed (where the content has a use with `conceive`: a double bed behind a shut door; without one at home the social isn't chosen, and the player's order is refused, "… have nowhere private to try for a baby"), and once they've lain there its time a baby is on the way with this chance, born `days` later, if they live together, are both grown-ups at most `maxAge`, and there's room. Content with no `conceive` use anywhere: the chance comes at the end of the social itself. Absent: nobody has babies. |
| `adoption` | `{"cost", "neighbours"}`: what adopting a baby or a child costs; `neighbours` the daily chance a neighbour household of grown-ups with no children (and three times the cost) adopts. Absent: no adoption. |
| `retireAt`, `pension`, `basicPension`, `retireFeeling` | At this age residents retire, with a weekly pension of `pension` × their last weekly pay and at least `basicPension` (a weekly amount, at least 0; default 0): a working resident on reaching it, anyone without a job at the next midnight (after losing or quitting one too). Residents made, placed or moving in at or past the age start retired. The job market doesn't hire anyone at or past the age; a job the player gives them is theirs to keep. |
| `death` | `{"from", "perYear", "growth"}`: from age `from`, a yearly chance of `perYear`, ×e^`growth` per year older. |
| `grief` | `{"feeling", "lightFeeling", "close", "friend"}`: family, partners and friends from `close` get `feeling` when someone dies; friends from `friend` get `lightFeeling`. |
| `moving` | `{"hour", "partners", "romance", "leaveHomeAge", "leaveHome", "leaveTownAge", "leaveTown"}`: daily chances of partners moving in together (romance at least `romance`), grown children moving out, and leaving town when no house is free. Absent: nobody moves on their own. |
| `newcomers` | `{"hour", "days"}`: a vacant house gets a household with a chance of 1 in `days` a day. Absent: nobody new comes. |

**Family** comes from bond presets with a `kin`, the family link of the bond's *second*
resident to the first: `{"parent": {"friendship": 50, "kin": "parent"}}` in a bond `a, b` makes
`b` the parent of `a` (`kin`: `parent`, `child` or `sibling`). Relatives don't flirt.

**Babies and cribs:** an interaction with `"baby": true` is the place a baby lies (only babies
use it); an interaction with `care` (need → gain over the whole interaction) fills the needs of
the baby lying in the same object, and only grown-ups at home do it. The base game's `crib` has
both (lie; feed, change, play); a baby who arrives without a free crib gets the first object
for sale with a `baby` interaction lain on (`"pose": "lie"`) delivered. A `baby` place sat in (the
high chair's, `"pose": "sit"`) is no crib: babies are carried to it for a `carried` care and back,
it ends after its `minutes`, and the baby sits in its middle (its other slots are the carers'). The
crib's *Change the baby* (`atSlot`) is done at its changing table, the baby laid on the pad for it
(drawn so; the sim keeps them in the cot's slot). The player's orders keep these rules as free will
does (an adult can't lie in the crib, a child can't feed the baby), and the age rules of
objects' and interactions' `stages` (see [Objects](#objects)).

**Socials and age:** a social's `stages` (who may start it) and `targetStages` (with whom),
life-stage ids, default everyone. The base game's *Start a fight* is teens and up on both
sides: nobody starts a fight with a child. Free will and the player's orders keep it alike, and
the social menu doesn't offer it.

**Romance between singles:** `socialRules.singleSpark` is added to the flirting preference of
two single grown-ups who are attracted to each other and have met (romance starts at 0, so
without it singles hardly ever begin; base game 1.0).

**Names** for newcomers and babies come from content `names`: `first`, `byGender` (first names per gender
id) and `last` (household names).

**Story texts** (`events`): `grewOlder` (`{stage}`: the stage's `story`), `retired` (`{job}`),
`died`, `movedInWith` (`{b}`: the partner), `movedOut`, `movedAway`, `expecting` (`{a}` and
`{b}`), `born` (`{b}` and `{and c}`: " and" the second parent, if any), `adopted` (`{b}` adopted
`{a}`), `graduated` (`{grade}`: "an A", "a B"...).

## Ids you can refer to

- **Needs:** `hunger`, `energy`, `bladder`, `hygiene`, `fun`, `comfort`, `social`.
- **Skills:** `intelligence`, `strength`, `endurance`, `charisma`, `creativity`,
  `technology`, `handiness`, `cooking`, `writing`, `perception`, `dexterity`, `business`,
  `gardening`.
- **Emotions:** `happy`, `flirty`, `angry`, `sad`, `embarrassed`, `focused` (likes
  `training`), `inspired` (likes `creative`), `energized` (likes `fitness`), `relaxed`
  (likes `rest`, `lounge`, `spa`). Icons are `icon.emotion.<id>`. An emotion's `walkSpeed` multiplies its
  residents' pace while it is theirs (sad 0.88, embarrassed and relaxed 0.95, inspired 1.03, happy 1.05, angry
  and energized 1.08); its body language is the renderer's, by id (`bodyLanguage.ts` `BEARINGS`).
- **Buy categories** (groups in brackets): `kitchen` (`appliances`, `counters`, `islands`,
  `dining`, `cabinets`), `bathroom` (`toilets`, `bathing`, `sinks`, `laundry`), `bedroom`
  (`beds`, `nightstands`, `storage`, `dressing`), `kids`, `living` (`seating`, `sectionals`,
  `tables`, `media`, `shelves`), `office` (`desks`, `chairs`), `fitness`, `hobbies` (`music`,
  `games`, `crafts`), `decor` (`wall`, `ornaments`, `textiles`), `tabletop` (things on the
  `surface` layer: `kitchenware`, `tableware`, `books`, `plants`, `ornaments`, `bath`, `desk`,
  `toys`), `lighting` (`ceiling`, `wall`, `floor`, `table`), `outdoor` (`seating`, `cooking`,
  `play`, `lights`, `decor`, `yard`), `garden` (groups `trees`, `shrubs`, `flowers`, `edibles`,
  `houseplants`), `community` (`play`, `street`, `food`, `shops`: the town's playground,
  street furniture, carts and vending machines, shop fittings; `content/community.json`),
  `wellness` (example pack). An object without a `group` shows under its
  category's *All*. Add a category with
  `"buyCategories": [{ "id": "...", "label": "...", "icon": "icon.category.<id>" }]` only if
  none fits (`icon` is optional: the catalog tab's icon, an `icon` asset key; `groups`
  optional: `[{ "id": "...", "label": "..." }]`, chips that objects pick with `group`;
  `"outside": true` on a category or group marks garden things, which wishes for a room don't
  offer: base game `outdoor` and `community` (except its `shops` group, `"outside": false`)
  and the garden's `trees`, `shrubs`, `flowers`, `edibles`).
- **Activities** (routine blocks): `sleep`, `eat`, `cook`, `wash`, `train` (names a skill),
  `workout`, `read`, `create`, `fun`, `relax`, `garden`, `chores`, `social`, `visit`.
- **Traits** (for `traitPatches`): `foodie`, `bookworm`, `couchPotato`, `neat`, `slob`,
  `energetic`, `lazy`, `cheerful`, `gloomy`, `natureLover`, `outgoing`, `loner`,
  `romantic`, `hotHeaded`, `kind`.

## Fence styles

`fenceStyles` lists the looks of Build mode's Fence and Gate tools; a fence's (or gate's) style
is an index into this list, saved by `id`.

```json
{"id": "picket", "label": "White picket", "kind": "picket", "color": "#F2F0EA", "height": 1.0, "price": 25}
```

`kind` is how it's drawn: `picket`, `rails`, `slats`, `iron` or `stone`; `price` is per metre
(default `build.fence`), and a gate costs a metre plus `build.gate`.

## Roofs

`roofStyles` and `roofColors` are what Build mode's Roof tool offers (saved by `id`):

```json
{"id": "steepGable", "label": "Steep gable", "shape": "gable", "pitch": 46}
{"id": "navy", "label": "Navy", "color": "#33465E"}
```

`shape` is `gable`, `hip` or `flat`; `pitch` is in degrees (default 34).

## Rooms, wear and accidents

How the house shows up in residents' lives (see `docs/design/house-matters.md`).

**Room kinds** (`roomKinds`): a room is of the first kind whose `tags` something in it offers.

```json
{"id": "bathroom", "label": "Bathroom", "icon": "icon.category.bathroom", "tags": ["bathroom"],
 "essentials": [["bathroom"], ["hygiene"]], "size": [3, 6], "exclusive": true, "private": true}
```

`essentials`: tag groups something in the room must offer for full function; `size`:
`[cramped, comfortable]` in tiles; `exclusive`: can't share a room with another exclusive kind
(a bed in the kitchen); `private`: kept shut while someone is in it. Nobody else goes in: someone
on their way in stops short of the door and waits (an hour at most, then they give up), free will
doesn't choose what's in it unless a need is about to run out, and kept out the accident comes
after its `graceMinutes` and the toilet counts as taken (`anotherRoom`). Not a room of two kinds
(`mixed`), one with a door to the outside, or one on a public place. The renderer keeps its
doors shut (snapshot `shuts`). Code: `crates/sim-core/src/privacy.rs`.

**Scoring** (`roomRules`, an object, so a pack can change single keys): `weights` of size, light,
decor, cleanliness and function; `windowTiles` and `lampTiles` (tiles one window or lamp lights
fully; a lamp's own `light.tiles` comes first); `decorPerTile` and `gardenDecorPerTile` (decor points for full decor); `decorByCategory`;
`love` and `dislike` (overall scores that make residents love or dislike a room) with
`loveFeeling` and `dislikeFeeling`; `drift` (how fast the `environment` need follows the room,
per hour) and `away` (its target at work); `preference` (how much the AI prefers nicer rooms);
`dirt` (mess by tag); `clean` (the *Tidy up* action: `label`, `minutes`, `threshold`, `amount`,
`interest`). A need with `"room": true` is the one that follows the room.

**Wear and repairs** (`objectRules.wear`: `perUse`, `byCategory`, `perQuality`;
`objectRules.repair`: `label`, `minutes`, `skill`, `skillGainPerHour`, `chance`,
`chancePerLevel`, `interest`, `cost` as a share of the price for the paid quick fix, `tags`).

**Laundry** (`laundry`: `{"perDay": 0.4, "most": 4}`): each resident in town makes `perDay` loads
of dirty washing a day for their household, up to `most` loads waiting (dirty, washed waiting to
dry, and dried waiting to be folded, each). An interaction with `"workOn"` takes a load of its kind
as it starts, and one that `makes` a load leaves one as it ends: the washer is loaded with a dirty
load and its washing taken out washed, the dryer is loaded with that and emptied dry, the washing
line the same, and the dry load is folded at the laundry basket. A household starts with a load
dirty and one washed. Saved. Without `laundry`, nothing piles up and no interaction may work on
washing.

**Accidents** (`accidents`): what happens when a need has been empty for `graceMinutes` and the
home has nothing working for it.

```json
{"id": "takeout", "need": "hunger", "story": "ordered takeout", "cost": 25, "cooldownHours": 2,
 "graceMinutes": 60, "rest": {"label": "Eat takeout", "minutes": 20, "pose": "stand", "anim": "eat",
 "gains": {"hunger": 0.7}, "tags": ["food"]}}
```

`effects` change needs at once, `dirt` leaves a mess where it happened, `feeling` is granted,
`cost` is charged, `rest` is something they do right away (asleep on the floor), `story` puts it
in the journal. `anotherRoom` (a room kind id): residents who badly need it and find what fills
it taken, on two different days within a week, wish for another room of that kind (the base
game's `wetSelf`: a second bathroom). `crowdedGraceMinutes` also lets it happen when the home has the essential but it's
taken (the base game's `fellAsleep`: after 120 minutes, when nobody can get to a free bed). At
most 8.

## Cooking

Cooking makes food. A recipe (top-level `recipes`) is a chain of steps, each at a kind of
**station**: the fridge for ingredients, a counter to prep on, the stove to cook. Clicking the
stove's "Cook dinner" sends the resident to the fridge first, then a counter, then back to that
stove; between steps they carry what the step leaves (`holds`: the ingredients, a prepped bowl).
The last step serves the dish (an item with servings) on a free surface near the cook, outdoors
for outdoor recipes, else in the fridge: one serving for each of the household at home, within
the recipe's `servings`. The ingredients are paid at the first step (`costPerServing` × servings).
Housemates are told dinner's ready (a thought bubble); whoever's hungry takes a serving, sits
down with it (a chair at a table, then a table with seats of its own, then any seat in
`cooking.seatTags`, else standing), eats, and leaves a dirty plate on the table (whoever takes the last serving from the fridge
puts the empty pot down next to it, to be washed up). Someone
clears away later (a chore, like tidying up): they pick up the dish and the dirty things next to
it, up to `carryDishes` at a time, scrape any food off into a `bin` station (the pedal bin; with
none, at the sink), wash up at a sink or dishwasher and put the clean things away in the nearest cupboard or
counter (`putAwayAt`; a wall cabinet over a counter first; with none, from the sink). Food goes off after `spoilHours` (`storedSpoilHours` in a fridge), and
dirty or spoilt things slowly make their room dirty. The player can order any of it: clicking a
dish offers **Eat** and **Clear away** (full, half-eaten or gone-off food and dirty dishes alike;
the home needs a sink or dishwasher).

```json
"recipes": [{
  "id": "meal", "label": "Make a meal", "item": "mealPot", "servings": [1, 4], "costPerServing": 3,
  "effects": { "hunger": 1.0 },
  "steps": [
    { "label": "Get ingredients", "at": ["ingredients"], "minutes": 3, "anim": "fetch", "holds": "ingredients" },
    { "label": "Prep", "at": ["prep"], "minutes": 10, "anim": "chop", "optional": true, "holds": "prepBowl" },
    { "label": "Cook", "at": ["stove", "oven", "microwave"], "minutes": 20, "anim": "stir",
      "animAt": { "oven": "bake", "microwave": "bake" }, "optional": true }
  ]
}]
```

| Recipe field | Meaning |
|---|---|
| `item` | The dish it serves (an `items` id with a `serving`, or a plate itself for `eatNow`). |
| `servings` | `[fewest, most]` servings it makes. Default `[1, 1]`. |
| `costPerServing` | Dollars per serving, paid when the cook takes the ingredients. |
| `effects` | What eating one serving gives (times the dish's quality: the cook's skill in the interaction's `skill`, the station's quality, and a little less per step done without its station). |
| `eatFeeling` | Feeling for eating a serving of good quality (1 or better). |
| `eatMinutes` | How long eating a serving takes (default `cooking.eatMinutes`). |
| `eatNow` | The cook eats it at once (a sandwich): no dish is served. |
| `outdoor` | A barbecue: outdoor stations first, served on an outdoor table. |
| `stages` | Who may cook it (life-stage ids), for every interaction that starts it without `stages` of its own (see the interactions' `stages`). Base game: a recipe with a hot step (stove, oven, grill) is for teens and up; the sandwich for anyone. |
| `steps` | In order: `at` (stations, the first preferred; the one clicked is used when it's one of them), `minutes`, `anim`, `animAt` (another `anim` at some of its stations, `{"oven": "bake"}`: the station object's first kind in `at` decides, so a cooker, a stove and an oven, stirs on its hob; the station's moving parts play by this tag, as they do by an interaction's id), `label`, `optional` (done without when the home has none: the dish is a little plainer, and its minutes are spent at the next step that has a station, or, with none after it, at the last one that has: no counter means chopping by the stove, a fridge-only kitchen makes the whole meal at the fridge) and `holds` (an item carried away from it), and `sound` (looks only: what it sounds like over the station's `sounds`, `"none"` for a bake, which a cooker's hob would otherwise sizzle). |

Residents choose a recipe only if their home has a station for every step that isn't
optional; a long recipe tempts them less, a hungry family more, and food already made hardly
at all (`leftoverPreference`: they eat that first). Many homes have only a fridge and a sink:
keep the prep and cooking steps of everyday recipes optional, and fetching the ingredients
the shortest step (3 minutes in the shipped ones). A station that's busy is waited
for (`stationWaitMinutes`), then done without (optional steps) or given up (others). Quick food
(a snack, toast, reheating) isn't a recipe but an interaction with `eat`: fetched or made at the
object, then eaten sitting down like a dish (it has a hidden recipe of its own, which the
game makes, so a plate in someone's hands survives a reload). Drinks (a coffee, a soda) stay
plain interactions.

**`items`**: `{ "id": "mealPot", "label": "Home-cooked meal", "model": "item.mealPot", "hold": "both", "serving": "mealPlate", "emptyAs": "potDirty", "radius": 0.16 }`.
`label` names it ("Eat Home-cooked meal"); `model` is its asset key; `hold` is `hand` or
`both`; taking a serving gives a `serving` (a plate) and an empty dish becomes `emptyAs`;
`dirty` things are washed up; `radius` is the disc it takes on a surface. Ingredients and bowls
are only carried (they're used up). Items aren't objects: they belong to a household, keep
their place on the surface they stand on (moving the table carries them, selling it throws
them away) and are saved with the game.

**`cooking`** rules (all optional): `stations` (the station kinds, at most 16; default
`ingredients`, `prep`, `stove`, `oven`, `grill`, `microwave`, `dishes`, `bin`; the base game adds
`cupboard`), `spoilHours` 8,
`storedSpoilHours` 72, `eatMinutes` 25, `dishesMinutes` 5 (per thing washed), `carryDishes` 3,
`leftoverPreference` 1.3, `feedFamily` 0.15 (extra pull per hungry housemate), `hungryBelow` 0.5,
`invitedBoost` 2.5 and `inviteMinutes` 60 (dinner's ready), `stationWaitMinutes` 10,
`dirtyDirtPerHour` 0.02, `skipPenalty` 0.05, `seatTags` (`lounge`, `food`) and `seatAvoid`
(`spa`, `hygiene`, `bathroom`), `defaultSurfaceHeight` 0.9 (where tools are drawn on a station
without a `surface`), `serveRange` 6 (m), `tableRange` 12 (m: the cook carries the dish to the nearest
dining table this near on their storey, one with a chair pulled up to it or seats of its own, an outdoor
one first for an outdoor recipe, and serves it there; with none, it's served within `serveRange`),
`serveMinutes` 1, `serveAnim` (`serve`) and `serveLabel` ("Serve it": putting it down on the table), `eatLabel`, `eatTags` (`food`), `eatAnim` (`eat`),
`dishesLabel`, `dishesTags` (`chores`, `cleaning`), `dishesAnim` (`wash`), `dishesInterest` 0.6,
`scrapeMinutes` 1 (per thing with food on it) and `scrapeAnim` (`clean`), `putAwayMinutes` 2 (per
armful), `putAwayAnim` (`fetch`) and `putAwayAt` (stations: the nearest, on a tie the first listed; `prep`; the
base game has `cupboard`, `prep`).
Packs merge `cooking` key by key and append to `items` and `recipes`.

## Phone

Everyone from `phone.fromStage` on carries a smartphone and uses it wherever they are: on a free
seat within `seatRange` metres if there is one, else standing, with the `anim` animation (the
renderer plays it standing and as an upper-body layer seated). With someone on the other end,
it's decided once the phone is out whether they answer: someone asleep never does; a call (not
a `message`) also doesn't get through to someone at work, at school, busy or on another call;
nobody phones people on the same lot (they talk face to face). Those who answer say yes or no
by `acceptance`, like the target of a social. While connected the caller's `needs` fill; on
hanging up the contact gets `contactNeeds`, and `success` or `failure` changes the
relationship both ways like a conversation (milestones included). Nobody answering changes
nothing. After a yes, `meet` sends someone: `"here"` (an invitation) brings the contact over
to the caller's home, `"there"` sends the caller to the contact's home; both are ordinary
visits (they knock and are let in, see `visits.door`), only within `visits.earliestHour` and
`latestHour`, between different households, and `here` only from home. Residents phone on
their own like they choose anything else: talking scored by `needs` × `autonomyWeight` ×
`prefer` (like a social), meeting up scored like a visit (`visits.needs`, friendship), without
`contact` like a pastime. The player orders any of it from the resident's **Phone** tab.

```json
"phone": {
  "fromStage": "teen", "anim": "phone", "seatTags": ["lounge"], "seatRange": 5, "noAnswerMinutes": 3,
  "actions": [
    { "id": "call", "label": "Call", "icon": "icon.phone.call", "minutes": 25, "contact": true,
      "tags": ["social", "friendly", "phone"], "prefer": "liked", "autonomyWeight": 0.5,
      "acceptance": { "base": 0.75, "friendship": 0.004, "mood": 0.4 },
      "needs": { "social": 0.28, "fun": 0.04 }, "contactNeeds": { "social": 0.2 },
      "success": { "friendship": 2, "targetFriendship": 2 }, "failure": { "friendship": -1, "feeling": "awkward" } },
    { "id": "invite", "label": "Invite over", "icon": "icon.phone.invite", "minutes": 5, "contact": true, "meet": "here" },
    { "id": "browse", "label": "Scroll the phone", "icon": "icon.phone.browse", "minutes": 20, "tags": ["fun", "phone"], "needs": { "fun": 0.15 } }
  ]
}
```

| `phone` key | Meaning |
|---|---|
| `fromStage` | The life stage (`life.stages` id) from which residents have a phone. Default: the first that isn't `baby`. |
| `anim` | Animation tag while on the phone (default `phone`). |
| `seatTags`, `seatAvoid`, `seatRange` | Seats to sit on: sitting interactions with any of `seatTags` (default `lounge`) and none of `seatAvoid` (default `spa`, `hygiene`, `bathroom`, `sleep`) within `seatRange` metres (default 5; 0: always standing). |
| `noAnswerMinutes` | How long they try when nobody answers (default 3). |
| `actions` | What there is to do on the phone, in the order the UI shows them (below). |

| Action field | Meaning |
|---|---|
| `id`, `label`, `icon` | Its id, its name ("Call", shown with who: "Call · Bo") and its icon (an asset key). |
| `minutes` | How long it takes (a no ends after 40 % of it). |
| `contact` | With someone (default false: on their own, like scrolling). |
| `message` | A message, not a call: reaches anyone awake. |
| `meet` | After a yes, `here` (they come over) or `there` (the caller goes over). |
| `anim` | Animation tag while at it (default the phone's `anim`): the base game's text and scrolling use `phoneScreen`, the phone held in front; the calls the phone at the ear. |
| `tags`, `autonomous`, `autonomyWeight`, `prefer`, `requires`, `acceptance` | As for socials. `requires` defaults to having met. |
| `needs`, `contactNeeds` | Need gains over the whole action for the caller (while connected) and, on hanging up, for the contact. |
| `success`, `failure` | Outcomes as for socials (`friendship`, `targetFriendship`, `romance`, `targetRomance`, `feeling`, `targetFeeling`, `needs`, `targetNeeds`), without an `effect`. |

The base game: text (10 min, a message), call (25 min), invite over and ask to come over (5
min; the player's only, `autonomous: false`) and scrolling (20 min, fun). A call fills somewhat
less than a chat face to face, so residents still meet. **Story texts** (`events`): `invited`
(`{a}` invited `{b}` over, and they're coming) and `declined` (`{b}` said no to meeting up with
`{a}`), told for the player's household. Packs replace `phone` key by key (a pack's `actions`
replaces the whole list). Content without `phone` has no phones. The market square's phone box
(`phoneBooth`) is no longer placed; it stays in the catalogue as street furniture, and nobody
walks to it on their own any more.

## Balance guidance

Money: households start with $2,500 (`economy.startingFunds`), plus $10,000 to build with when
they start on an empty lot in a Living game (`economy.emptyLotFunds`; a ready-made house is worth
about $7,500–12,000); rent is about $200 a week on a house plot; an
entry-level job pays roughly $70–150 a shift (grades run from $12/h to $60/h). A track's
`payFactor` sets its pay at the top of the field: entry grades all pay the field's rate, and pay
rises in a straight line from there to the last grade, which pays `payFactor` × that rate
(0.5–1.83 in the base game). A field's own `payFactor` scales all of its tracks alike (1 = the
grades' rates). Running costs
come as weekly **bills**, paid with the rent: `economy.rent.billsBase` ($25) plus `billsRate`
(1.5%) of what the household's objects are worth (price plus upgrades) plus `perResident`
($150 per resident). A $2,000 luxury item adds about $30 a week. A working household saves
roughly $100–2,000 a week; `crates/sim-core/tests/balance.rs` checks this (and how fast
careers go) for the base game, so run it after changing prices, pay or rent.

**Prices by category** (base game):

| Category | Range | Examples |
|---|---|---|
| decor | $40–150 | plant 40, lamp 120 |
| outdoor | $40–250 | bush 40, flowerbed 60, bench 200 |
| community | $60–2,600 | litter bin 60, café chair 160, park bench 380, slide 520, vending machine 800, town fountain 2,600 |
| bathroom | $180–650 | sink 180, toilet 300, shower 650 |
| kitchen | $350–600 | dining table 350, fridge 600 |
| bedroom | $180–750 | mirror 180, bed 750 |
| living | $250–500 | bookshelf 250, TV 400, sofa 500 |
| fitness | $480–650 | weight bench 480, treadmill 650 |
| hobbies | $300–1,400 | easel 300, chess 350, telescope 900, piano 1,400 |
| office | ~$1,100 | computer desk 1,100 |

The second catalogue (`content/catalog2/*.json`, separate pieces: a table and its chairs are
bought one by one) fills in below and between those:

| Category / group | Range | Examples |
|---|---|---|
| tabletop (all groups) | $10–80, decor 0.2–0.5 | toy blocks 12, mugs 15, vase of flowers 40, stand mixer 80 |
| kitchen counters, islands | $110–520 (long island 1,100) | counter end 140, butcher block 260, double sink 520 |
| kitchen appliances | $380–1,600 | dishwasher 380, oven tower 900, French-door fridge 1,500, range cooker 1,600 |
| kitchen dining: tables | $90–620 | bistro table 90, table for four 260, farmhouse table 520, banquet table 620 |
| kitchen dining: chairs, stools | $30–160 | folding chair 30, wooden chair 55, carver chair 160 |
| kitchen cabinets | $180–650 | bar cart 180, sideboard 380, china cabinet 650 |
| living sectionals | $280–420 a piece | middle, arm or corner seat 280, chaise 420 |
| living seating, tables, shelves | $35–980 | floor cushion 35, coffee table 180, recliner 650, three-seater 980 |
| bedroom nightstands, storage | $45–900 | nightstand 95, dresser 360, armoire 900 |
| bedroom beds | $480–2,200 | daybed 480, double 900, four-poster 2,200 |
| office desks, chairs | $160–1,200 | desk chair 160, desk 180, standing desk 650, executive desk 1,200 |
| lighting: table lamps (on tops) | $45–90 | lantern 45, banker's lamp 90 |
| outdoor yard | $70–2,400 | rain barrel 70, raised bed 160, shed 900, greenhouse 2,400 |

Something clearly better than the basic version (bigger gains, a buff, two slots) costs
2–3× the basic item; luxury items $1,500–3,000. Per-use `cost` ($4–15) only for
consumables bought from outside, such as takeout or deliveries; never for using something the household owns.

**Need decay per hour** (base game; a whole need is 1): hunger 0.07, energy 0.058, bladder
0.12, hygiene 0.045, fun 0.06, comfort 0.06, social 0.06. Asleep (`dayRhythm.asleepDecay`):
energy 0, hunger ×0.3, bladder ×0.15, hygiene ×0.5, comfort ×0.3, fun and social ×0.1. At work or
school (`workDecay`, a career's `workEffects` on top): hunger ×0.2, energy ×0.35, bladder ×0.1,
hygiene ×0.3, fun and comfort ×0.2, social ×0.1, with `workBreaks` on top. Awake at night energy goes ×1.8 (`nightDecay`).
Walking takes time too (1.5 tiles a game minute): `crates/sim-core/tests/routines.rs` prints
residents' days (toilet visits, how low needs go and what they were doing then, hours a day
walking, eating, asleep...). With these numbers residents go to the toilet about 4 times a day
at home and needs rarely run out; mostly for loners (social), long early shifts and homes where
a bed can't be reached.

**Need gains per hour** (at 0 stars; compare yours with these):

| Object | Per hour |
|---|---|
| fridge snack / meal / gourmet | hunger 1.8 (0.3 over 2 min fetching + 8 eating) / 1.28 / 0.76 |
| toilet | bladder 12 |
| shower / sink (hand washing) | hygiene 3.0 / 1.2 |
| bed sleep / nap | energy 0.125 (+ comfort 0.05): 8 h from empty; futon 0.11, bunk/kids/daybed 0.12, queen/canopy 0.13, king 0.14 / nap energy 0.24 (30 min) |
| sofa / lounger doze | energy 0.2 (30 min) / 0.16 (45 min) |
| sofa / armchair / bench | comfort 1.2 / 1.5 / 0.7 |
| TV / browse / read | fun 0.7 / 0.6 / 0.6 |
| coffee (with a coffee machine) | comfort 0.75, energy 0.24 |
| example: whirlpool soak | comfort 1.07, hygiene 0.33, fun 0.2 |

Keep new items within about ±30% of the closest base item, plus a modest extra (a second
need, a buff feeling, a skill bonus) that justifies the price. A need gain over a whole
interaction above 1.0 is wasted (needs cap at 1).

**Skill gains per hour:** dedicated training items give 0.20–0.25 in their main skill plus
0.04–0.08 in a secondary one; fun items that teach a little give 0.04–0.12; work gives
0.04 × weight. Training interactions should have little or no need gain (study: fun 0.1/h).

**Feelings:** `mood` +0.05..+0.12 for everyday treats, up to +0.2 for big events; `hours`
2–8 for everyday treats. Gate the nice ones behind `feelingMinSkill` so skill matters.

## Towns: houses and public places

`content/houses.json` holds the house layouts a town is built from and, under `parks`, its
public places. Everything is drawn for a medium lot (`plot`, 26×22 tiles) with the street
along z = 0; plots across the street are the same layout turned round 180°.

```json
{
  "id": "cafe", "name": "Corner Café", "kind": "shop", "description": "Coffee, cake and a terrace in the sun.",
  "walls": [[7, 6, 19, 6], [7, 13, 19, 13], [7, 6, 7, 13], [19, 6, 19, 13]],
  "doors": [{"x": 12, "z": 6, "axis": "x"}],
  "windows": [{"x": 8, "z": 6, "axis": "x"}],
  "paths": [{"x": 1, "z": 1, "w": 24, "d": 5}],
  "entry": [12.5, 4.5],
  "objects": [{"def": "cafeTable", "x": 9, "z": 8, "rot": 0, "turn": 30}, {"def": "cafeChair", "x": 8, "z": 8, "rot": 1}]
}
```

| Field | Meaning |
|---|---|
| `kind` | `park` (default: laid out in the open) or `shop` (a small shell with `walls`, `doors` and `windows`, placed like a house's). |
| `walls`, `doors`, `windows` | As in house layouts (optional). A shop's first door gets a garden path to the street. |
| `paths` | Paved ground in plot tiles (`x`, `z`, `w`, `d`, any size): walks, squares, terraces. Only drawn. |
| `entry` | Where visitors arrive (default: the middle of the street front, 1.5 m in). |
| `objects` | Placements: `def`, `x`, `z` (the footprint's corner), `rot` (0 = facing +z, away from the street; 2 = facing the street), for things that turn freely (`freeRotation`) `turn` (degrees past `rot`, 0..89), and for anything but stairs `offset` (`[x, z]` off the middle of its tiles in hundredths of a tile, up to ±50, things on a wall only along it; turned round with the plot it points the other way), `size` (`[across, deep]` in centimetres before turning, 10 to 600 each; it takes its tiles rounded up and stands in their middle; things on a wall keep their depth, things on a top their footprint) and `tall` (centimetres taller, −100 to 200). |

A thing someone serves at (`staff`) needs room behind it for its server: the café's and the bakery's
counters stand a tile in from the back wall (`every_post_in_town_has_someone_behind_it`).

**Service** (`service`: `{"share": 2.0, "waitMinutes": 20, "anim": "sell"}`): how the town's vendors and
shopkeepers serve (`staff` on objects, `selfServe` on what they sell). Posts whose servers stand within
`share` metres of each other on one lot, with a straight walk between, share a server; a customer
waits `waitMinutes` for the server at most, then gives up; `anim` (an `animations` tag) is what the
server is shown doing while serving. Without it: no sharing, 20 minutes, the server stands.

Every town has the first entry (the Town Park) in the middle of one row and gets more public
places round it by size (small 1, medium 2, large 3), picked at random from the rest. Everyone in
town may use what stands on a public place; residents walk there on their own when it's worth
the walk (food carts and machines when hungry, a bench, the playground). Town files are loaded
strictly: one thing that doesn't fit (overlaps, faces off the lot, a wall in the way, a turn on
something that doesn't turn) and the town doesn't load, so `cargo test -p sim-core --test
public_places` loads every place on both sides of the street and checks that everything in it
can be reached from the street.

## What content is refused

Since 0.2.0 the simulation refuses content that would break the game when it loads, with the
id and the rule in the message, for example `lazy: walkSpeed 0 must be above 0`. The checks run
on the merged content, so they don't name a file; only errors while merging the files do
(`crates/sim-core/src/content.rs`; one test case per
rule in `refuses_content_that_would_break_the_game`). Unknown keys are still ignored, because UI-only
keys share these objects. The rules:

| What | Must be | Why |
|---|---|---|
| `walkSpeed` (in the effects of traits, perks, emotions and feelings, life stages, the schedule) | a number above 0 and at most 10 | 0 left residents walking for ever, and gave infinite or NaN scores |
| A need's `decayPerHour`; need decay and gain, skill gain and tag preference multipliers; the day rhythm's decay multipliers | numbers, 0 or more | a negative decay pushed needs past full |
| `genders` | at most 32 | attraction is a 32-bit mask |
| A recipe's `steps` | 1 to 255 | a cook counts steps in a byte, and more looped for ever |
| `objectRules.resale` | 0 to 1 | above 1 buying and selling made money; below 0 selling cost money |
| `objectRules.upgradeCost` | 0 or more | a negative cost paid the player for upgrading |
| An object's `footprint` | at most 6 × 6 tiles | larger can't be bought, and broke lot loading |
| `overWindow`, `fillsWall`, `hangsLow` | only on the `wall`, `floor` and `ceiling` layer | elsewhere they'd mean nothing |
| An interaction's `closeAfter` | with `"leaves": "open"`, on an object with an interaction that leaves it closed | there'd be nothing to close it with |
| An interaction's `gradual` | with `"leaves": "open"` or `"closed"` | there'd be nothing to change |
| An interaction's `fetch` | with `requires`, naming a plain interaction of something that provides all it requires | there'd be nothing to fetch it with |
| An interaction's `hours` | two different hours of the day, 0 to 24 | the same hour twice is an empty window, and an hour past 24 never comes |
| An interaction's `workOn` | `dishes`, `laundry`, `washed` or `dried`; the last three only with content `laundry` | with no washing piling up it could never start |
| An interaction's `makes` | `laundry`, `washed` or `dried`, only with content `laundry` | the dishes aren't a load; with no washing piling up there'd be nowhere to put it |
| An interaction's `loads`, `unloads` | on an object that `runs` (`minutes` above 0), not both | there'd be no run to start or to end |
| An interaction's `brings`, `holds` | an `items` id | there'd be nothing to show |
| Interactions of things on the `surface` layer | not `lie`, `inside`, `whole`, `baby`, `care`, `needsTable` or a `recipe` | a thing on a top is used standing or sitting at its top |
| `laundry` | `perDay` above 0, `most` at least 1 | no washing would ever pile up to a load |
| An object's `staff` | `at` within 1.5 m of the footprint, `hours` two different hours of the day | a server off in the street; an empty window |
| An interaction's `selfServe` | on an object with `staff`, an `anim` the content has, `minutes` above 0 | nobody could sell it; a use that takes no time |
| `service` | `share` 0 or more, `waitMinutes` above 0, an `anim` the content has | |
| Ids of an object's interactions, and of needs, emotions and feelings | unique | two with one id are ambiguous: a save reloaded the first of two interactions |
| Friendship and romance in `bondPresets` | −100 to 100 | relationships are kept in that range |
| Tags, animations and skills named in the day rhythm (`sleepTags`, `napTags`, …), room kinds, room dirt, cleaning, repairs (`objectRules.repair` skill and tags), accidents (`anim`, `tags`), social rules (`busyTags`, `guestTags`), cooking tag lists and activities | ones the content has | a typo used to be dropped silently and turn the feature off (nobody counted as asleep; repairs trained nothing) |
| Times that become ticks: feelings' `hours`, interaction and social `minutes`, the commute, accident cooldowns and grace, the baby's quiet spell, cooking invitations and station waits | numbers (a value too large for a float, such as `1e39`, reads as infinity and is refused) | an infinite time overflowed the tick count |
| A grade's `payPerHour` | 0 to 10000 | pay becomes shift pay and promotion bonuses; a huge one overflowed the sum |
| `payFactor` (a career category's, a track's) | 0.25 to 3 | past that a track's pay no longer resembles its field |
| `careerRules.probationLevels` | 0 to 1 | a shortfall of more than a level is never close enough to hire on probation |

At run time the game also keeps needs between 0 and 1, and friendship and romance within ±100,
whatever the content says; a score that isn't a number is never chosen. The hours of the
once-a-day events are kept to 0–23.9 instead of being refused: rent (`economy.rent.hour`), the job
market (`careerRules.market.hour`), the planner's review (`planner.reviewHour`), moving and
newcomers. At 24 or later they would never come round.

## Checking a pack

```
cargo run -q -p sim-core --example check_pack -- web/public/content/packs/<name>.json
```

It merges base + careers + your pack (other packs are left out), reports JSON and content
errors with the file and id, then for every object in your pack:

- places it on a 14×14 test lot with one resident (plenty of money) and runs each interaction as a
  player order: one line per interaction with minutes used, its animation, money spent,
  change in the needs it affects, skills gained and feelings gained (twice, at skill 0 and
  8, if it has a `skill`);
- then lives three game days on the starter lot with your objects bought into the home,
  printing how long residents used each of them and what they spent, and fails if the household's
  average needs drop to 0.25 or below.

It exits non-zero on any failure. Also run `cargo test -p sim-core`: the shipped-content tests
load base.json with every file in its `include` list.

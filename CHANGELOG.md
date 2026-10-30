# Changelog

All notable changes to Idyll Lives. Versions follow [semantic versioning](https://semver.org); each release is a git tag `vX.Y.Z`.

## Unreleased

- **Cars:** most households have a car on their driveway, the one you play too: your home keeps the driveway and car the town showed. Only its owners drive it, and only from 18: the household's grown-ups when the town was made (a household left without one hands it down to its grown-ups). Those going by car leave early enough to be at work on time (the walk to the car, the drive out of town and on to work), and their shift starts at its hour, not as they get in. An owner walks to the driver's door, it opens, they step in and sit, and it shuts before they're gone from view; they wait a moment for the rest of the household leaving at the same time (children off to school too), each getting in by a door of their own, never one with no room to swing open (next to the house, say). The car backs or pulls out and drives off along the street out of town. Whoever finishes first waits at work for the others if they're done within the hour, and the car brings them home, nose in; its doors open and they step out. A rider whose driver works on longer walks home as before. A parked car takes up its spot: nobody walks through it and nothing can be put there. A car on the move stops for anyone in its way (who steps aside after a moment), and nobody walks into one driving by. Garages stand where the town showed them, your home's too, and can't be walked through or built on. When a household moves house, its car goes with it to the new driveway. The neighbours' cars come and go as well. A car whose spot was built over while it was out stays put, unused, and isn't drawn.
- **Bedrooms are their sleepers' own:** only those who belong together share a bedroom. While someone sleeps in one, its door is open to them, their partner, parents, children, brothers and sisters with a bed there, and whoever minds a crib in there, and keeps everyone else, guests included, out. The other half of a double bed is the partner's (or a child's for the night); a bunk bed is a bed for each. A lodger, even one sent to the bed, waits outside like anyone else; walking through a bedroom to a room beyond is fine.
- **A calmer interface:** the top bar is three small islands in one row, with the lot showing between them (the clock and speeds take a second row only in a small window or at a large interface scale), and Undo, Redo and the eyedropper sit in the Build dock beside its tabs. The Furnish catalog has a fixed height: the categories along its top with their names, a grid of items below, and a search; the sort, the filters and the group chips are gone (each category still lists its groups in order), as is the *Build catalog* size setting. A quiet interface while watching can't be clicked by accident.
- **Newcomers and bed places:** households moving into town are a single, a couple (perhaps with a grown child) or siblings, never strangers, and bed places count a double bed as one place unless a couple sleeps in it (who moves in, the no-bed notice, the beds the town buys for its children).
- **No talking through walls:** residents talk, and listen in, only where they can see each other, never through a wall, a window or the shut front door; someone going over to talk to a person the other side of a wall goes round by the door, and nobody calls out through the front door. Whoever answers the door steps out onto the path to say hello to the visitor, rather than talking to them through the shut door.
- **The neighbours' gardens:** the things in your neighbours' gardens (garden furniture, planters, loungers, swings) stand there while you play, as the town seen from the menus shows them, instead of bare patches in the grass where they should be.
- **Grass past the edge of town:** the lawn's grass no longer stops in a line a little way out of town. Its blades grow on over the whole landscape round the town, however far out the camera goes, thinning out with distance from the camera as they do in town, and the land beyond keeps the lawn's colour and its gusts of wind, fading gently into the far hills.
- **A basement you can see:** looking at the basement, your lot's garage, car, driveway and garden paths go with the rest of the ground instead of floating over the dug-out lot, the earth round the basement is earth-brown instead of a near-black slab, and rooms nobody is in (their lamps off) are softly lit enough to see what's in them, still well below a room with its lamps on. That soft light is as bright at night as by day, with no moonlight's blue down there. A new game's lamps are on only in rooms someone is in (they all came on before, and stayed on in a basement nobody went down to); residents switch them as they come and go, as before. The basement's walls have clean tops: no dark stone ledge along them and no dark notches at every post, which showed once it was lit.

### Also
- **Saves:** format 34 (where every car is, who owns it, every house's driveway and the garages). Older saves open with no cars in the simulation: their neighbours' cars stay parked, as before.

## 0.5.1 — 2026-10-29

**Play it in your browser:** https://krecks.github.io/idyll-lives/

Three fixes: the town seen from the menus, how smoothly it moves, and residents going next door.

- **Streets in the menus' town:** no more green patches on the roads. From the overview's height the graphics card couldn't tell the roads from the ground just beneath them; the camera now keeps its precision for where it is looking, near or far. The game's own wide view gains a cleaner edge to the side roads too.
- **Frame rate in the menus:** the town behind the menus draws at the frame rate cap from Settings (30, 60, 120 or none), as the game does. Before, it kept to 30 frames a second, and its camera juddered as it glided in or was dragged. Standing still it still costs nothing: the last picture is shown until something changes.
- **Next door by the street:** residents going into a neighbour's garden no longer walk through the hedge or fence between the gardens. They go out to the street and in at the front.

### Also
- **Saves:** unchanged (format 33): 0.5.0 and 0.5.1 open each other's saves.
- **Known:** choosing another world on the new-world screen still takes a tenth of a second while the world is made, as before.

## 0.5.0 — 2026-10-29

**Play it in your browser:** https://krecks.github.io/idyll-lives/

Everything left open after 0.4.2, done: residents hold what they use, carry themselves as they feel, keep the bathroom door shut, and buy from people in town; the washing runs on its own.

- **Things in hand:** residents hold what they use instead of miming it: a book, a cup, cutlery and the food on it, the phone, a pen, the whiteboard's marker, a card, tools, the pool cue, ping-pong bats, the cocktail shaker, a coin, a dart, bird seed (and the birds come down for it), the basketball while dribbling and in the air, the phone box's receiver. Food from town comes in its own wrapping.
- **Body language:** a resident's mood shows in how they stand and walk: upright with the arms swinging when happy, head down and slower when sad, stiff and quick when angry, shoulders drawn up when tense. They step round to turn on the spot instead of spinning, and children run at game speed 3 without their feet sliding.
- **Privacy:** a resident in the bathroom keeps its door shut, and others wait outside rather than walk in. *Try for a baby* happens in a double bed behind a shut door, with nobody else in the room.
- **Vendors and shopkeepers:** someone serves at the town's carts, stalls and counters, turns to their customer and hands over what's bought; a customer who gives up waiting gets their money back, and one on the way at closing time is still served. Self-serve uses take their own time.
- **Laundry and machines:** the washer, the dryer and the dishwasher run on their own: loaded, left to run (their drums turn and are heard), then emptied. The washing goes out on the line a piece at a time and comes in to the basket in an armful; a home with a washer and no dryer gets a washing line in the garden.
- **In the garden and at home:** the apple tree's low boughs can be picked, by children too; the home chess table seats two and is played with a partner; the bonsai, the arbour's roses, the lectern and the basketball hoop are used at the right part.
- **Moving parts:** the kettle's lid, the bread bin's roll-top and the laptop's lid open for their uses; the high chair's tray slides out; the hot tub's cover folds back in two halves; the arc lamp reaches over the next tile.
- **Sounds:** the uses that were silent now sound like their things, 34 new sounds made in the game: machines, games, tools, a fire, a cello, singing.
- **Graphics:** on Low every lamp keeps to its own room, as on Medium and High, and the woods are drawn in squares, so far fewer trees are drawn that aren't on screen. Fixed: things hanging over lowered walls, the stairs from the basement, the town map's backdrop. The camera keeps up with whoever it follows at game speeds 3 to 5.

### Also
- **Saves:** format 33. 0.4.2's saves open in 0.5.0; a home chess table, now two tiles long, moves over to fit or is sold back for what it was worth. 0.4.2 can't open 0.5.0's saves: it keeps them and lists only the ones it can open.
- **Updated:** PlayCanvas 2.23.3 and ONNX Runtime Web 1.31.1 (the voices sound the same; the one-time download is 218.7 MB).
- **Known:** on Low with WebGL2 the graphics processor is now within 5 % of Babylon's in the wide view (it was +16–18 %), but the close orbits are still 11–16 % over (`docs/design/performance.md`).

## 0.4.2 — 2026-10-25

**Play it in your browser:** https://krecks.github.io/idyll-lives/

The lawn is a lawn now: real blades of grass that sway in the wind and part where residents walk.

- **Grass:** the lawn's little star-shaped tufts are replaced by blades drawn by the graphics processor, thousands of them round the camera: curved and fine up close, dark at the root and light at the tip, taking the colour of the ground they grow on in every kind of world. Farther off the ground carries the lawn, so there's no edge where the blades end and no twinkling when you zoom out.
- **Wind:** gusts roll across the lawn in waves you can see, bending the blades as they pass, and carry on over the ground beyond them. Reduce motion stills it, as it does the trees.
- **Residents:** the grass bends away round anyone walking through it and springs back after them.
- **Where it grows:** not indoors, under furniture, on patios, paths, streets or the roads out of town, nor right against a wall; building a room or placing furniture on the lawn clears the grass under it.
- **Graphics settings:** Low draws half the blades, wider, nearer the camera; Medium and High the full lawn. On the game's benchmark the lawn costs about as much as the old tufts did (Medium, WebGL2: +0.45 ms a frame in the close view, less in the wide one).

### Also
- **Saves:** unchanged (format 32): 0.4.1 and 0.4.2 open each other's saves.

## 0.4.1 — 2026-10-24

**Play it in your browser:** https://krecks.github.io/idyll-lives/

Two fixes to the renderer and a tool.

- **Changing graphics settings between games:** the renderer that's replaced now lets its WebGL context go at once, and the new one draws on a fresh canvas. Before, each change left a context behind until the browser cleared it, and after enough of them the browser could drop the one in use. Replacing the renderer on WebGL also no longer throws an error for every model it had loaded.
- **`cargo envconv`:** the sky's HDRI download stops as soon as an answer is longer than the pinned file, instead of writing it all to disk before the checksum refuses it.

### Also
- **Saves:** unchanged (format 32): 0.4.0 and 0.4.1 open each other's saves.
- **Known:** on Low with WebGL2 the graphics processor still does more work than it did with Babylon, most in the wide view (+16–18 %). It was profiled for this release: not the FXAA, but nature's share, which needs switches of its own to go further (`docs/design/performance.md`, Low on WebGL2).

## 0.4.0 — 2026-10-24

**Play it in your browser:** https://krecks.github.io/idyll-lives/

The game draws with **PlayCanvas** instead of Babylon.js.

- **Renderer:** PlayCanvas 2.23 on WebGPU, with the WebGL2 fallback, replaces Babylon.js 9.30. On the game's own scene PlayCanvas took about a third of Babylon's main-thread time in Chrome and about a ninth in Safari's engine, where Babylon fell below 60 fps (`docs/design/performance.md`). The pictures were matched to Babylon's stage by stage: the same light, shadows, soft look, colour grading, residents, mirrors, build tools, previews and town overview; the camera moves as it did.
- **The menus' town:** standing still, it isn't drawn again until something in it changes, on WebGPU and WebGL2 alike (Babylon's snapshot rendering only spared the processor, on WebGPU).
- **Under the hood:** the engine-neutral parts of the renderer (`web/src/render/shared`) are tested by `bun run test`, and `bun run check:chunks` checks the PlayCanvas internals the renderer patches; both run in CI. Parts of the renderer are ported from Babylon.js (Apache-2.0) and say so; its licence and notice ship in `THIRD-PARTY-LICENSES.txt`. The decoders Babylon needed are gone; PlayCanvas's Basis transcoder decodes the textures.
- **No stalls at the start of play:** the game compiles its shaders behind the loading screen, so the first seconds of play, the first night, a mirror and Build mode no longer stutter while they compile (only a model bought for the first time compiles its own).

### Also
- **Saves:** unchanged (format 32): 0.3.2 and 0.4.0 open each other's saves.
- **Generated houses:** a house is laid out the same every time and for whoever moves in, so the home you pick is the one you get. Since 0.3.2 the same house could come out with other rooms from one time to the next, in Safari especially. New towns get new houses; the towns in saved games keep theirs.
- **Known:** on Low with WebGL2 the graphics processor does up to 15 % more work than it did with Babylon; everywhere else it does the same or less.
- **The sky's light** is made straight from the CC0 HDRI it came from (Poly Haven's Belfast Open Field, `cargo envconv`), instead of from Babylon's packed copy of it; the lighting is unchanged.

## 0.3.2 — 2026-10-22

**Play it in your browser:** https://krecks.github.io/idyll-lives/

Birds live in the town now.

- **Birds:** blue tits, chaffinches, warblers, blackbirds and collared doves live in the trees, bushes and hedges round the lot. They sit on the twigs and sway with them, glance about, flick their tails and preen, hop from twig to twig, and fly between the gardens: tits and finches bound, blackbirds fly low, doves flap and glide, always over the houses, never through them. Blackbirds, chaffinches and doves come down to the lawn to hop, walk and peck, and fly up when someone comes near. They are busiest at dawn and in the evening and roost in their trees at night. The ones you can see sing from where they sit, louder nearer, and the blackbird has a song of its own. Low has 6 birds, Medium 24, High 40; with Reduce motion they stay perched.

### Also
- **Loading and streaming:** the first loading screen also fetches every menu icon and the character set, so nothing reachable before Play is left to stream. Starting a house now has its own loading screen that fetches all the viewed house's furniture, garden, textures and residents before the world is built; the rest of the town and the catalog then stream in the background. A small bottom-centre indicator shows “Game Data is streamed and cached” with a percentage.
- **Saves:** unchanged (format 32): 0.3.1 and 0.3.2 open each other's saves.
- **Models:** every tree, bush and hedge knows where a bird can perch on it, and what kind of plant it is; `cargo check-glb` checks the birds' parts and sizes and every perch.
- **Build:** Rust 1.97.1 (1.97.0 miscompiled a review tool on x86-64); the web app and tools run on Bun 1.4 instead of Node and pnpm (`bun.lock`).
- **README:** opens with screenshots; the full guide to playing is in `docs/playing.md`.

## 0.3.1 — 2026-10-22

**Play it in your browser:** https://krecks.github.io/idyll-lives/

After 0.3.0 every model, animation and interaction in the game was reviewed against the real thing, and what the review found was fixed: 281 of its 342 findings, every one that broke belief among them, and nine more in part (details in `docs/authenticity-review.md`). Residents now use things the way people do: from the right place and side, at the right height, facing each other across a table, with someone to play against, and only what their age allows. Things are at their real sizes, light stays in its room, doors open for the one going through, and a use sounds like the thing.

### Residents and how they move
- **Each in their own place:** two at a double sink, a pool table or a shop counter stand apart, each at their own end or basin, facing it. A cook waiting for a busy counter steps aside. Darts are thrown from the oche, and a visitor knocks at the front door, facing it.
- **On and off from the right side:** beds are got into from the side and left that way, desks are sat at from behind, bikes mounted from the side. Getting up happens at the seat before the step back, and a step onto a seat, round a chair built into a table or up to a talk is walked, not jumped.
- **Seated at every age:** feet on the floor and knees clear of deep seats at every life stage; on a stool the feet rest on its foot ring; feet go up on a footstool in front of the armchair or sofa. Cross-legged on floor cushions and the hearth, against a tree in its shade, into the ball pit and the play tent.
- **New clips:** up the slide's ladder and down the chute sat up; up and down the bunk bed's ladder; rides sat astride with the hands on the grips; darts, pool, foosball, table tennis, bouldering, yoga and the smash bin; a hoop shot; reaching up to the wall cupboards, the clock's dial, the feeder and the whiteboard; the telescope's eyepiece; picking the apple and lemon; a proper shower; the guitar strummed; the bench press with the feet down; the hands warmed at the fire. At speed 3 a run takes over from the jog, so grown-ups' feet no longer slide (children's a little), and two children at the climbing frame's pole take turns.
- **Clothes:** pyjamas only for a night's sleep, a nap barefoot in day clothes; towels in the shower and the bath, swimwear in the hot tubs, kept until out of the shower; skirts and jackets lie on the lap when seated. "Pick an outfit" changes the clothes.

### Objects and models
- **Real sizes:** 134 models in 42 families rebuilt or new. Working heights are in the real range: the workbench, the digital piano (an 88-key upright against the wall), the vessel basins, the kettle grill, the picnic table, the chef's island's stools, the pack desks. Dishwashers are 60 and 45 cm wide, the ping-pong table is full size, hammocks are long enough to lie in, the corner desk is an L, the cot is oblong, and the trees, roses, meadow and boulders read as themselves.
- **Moving parts:** the shower's glass door slides open and the stall's curtain is drawn; the hot tub's cover folds back; the dishwasher stays open while it is loaded; the washer's drum turns after its door shuts; the grandfather clock's pendulum swings; the pool rack lifts off; the blender, the robot vacuum and the stereo's record move; the upper bunk's duvet turns down once its sleeper is up.
- **The fireplace** is sat at on its hearth, in front of the fire, not in it; the fire pit is warmed at from the seats round it.
- **Quilts** take each bed's own colours, and none is left at the world's origin. Food has its own models (ice cream, toast, a smoothie, noodles, a takeaway box), and the board or dish that went in comes out dirty.
- **Names that match the models:** the king-size bed, the counter-depth fridge, the lantern pendant, the recumbent bike, tables named by their seats.
- **Doors** open only for the one going through, in time at every speed, and garden gates swing.
- **Houses and the street:** panelled doors, a light oak, patterned wallpapers and limed floors, chimneys from the eaves, railings with balusters; a one-lane street without a centre line, cars at their real clearance, garages a car fits; curtains as wide as the window.

### Who may use what
- **Age rules:** what a resident may use depends on their life stage, for free will and your orders alike, and the menu greys out the rest. Children don't pour wine, cook at a hob, lift weights or start a fight; grown-ups don't ride the toddlers' things, climb the fort or lie in the cot. Coffee is for teens and up, a child eats at the kids' table, and nobody is sent to a cupboard they can't reach.
- **Uses with a time and a reason:** stargazing only after dark and under open sky, washing up only with dishes to wash, the sunrise lamp's stretches only in the morning. Laundry piles up and is washed, then dried or hung out.
- **Things on tops are used:** the kettle, the toaster, the fruit bowl, the laptop and the rest are used where they stand, from beside the counter or a chair pulled up to the table.
- **Where things may stand:** things made for a wall need one behind them, pendants hang high enough to walk under, and nothing is put where a lamp hangs at head height.

### Games and partners
- **Games for two need two:** pool, foosball, the seesaw, ping-pong and park chess fetch a second player, who comes to the other place; the first waits, and the seesaw doesn't rock for one. With nobody free, pool and foosball are practised alone and the others are refused.
- **Facing each other:** a dining set or tea table for two, the picnic table, park chess, ping-pong, the seesaw and the merry-go-round seat their players across from each other. Watching takes a seat that faces the screen.

### Food and cooking
- **Cooking looks like cooking:** ovens and microwaves bake instead of stirring a pan, mixing has a bowl, the pan sits on the burner that's lit, and the cook's hands are free at each station. The dish is carried to the table and eaten there.
- **Food in town** is self-serve and eaten sitting down: an ice cream from the cart, a hot dog, a piece of fruit from the stall, a pastry from the case. The café's coffee is poured at its machine first.
- **A word over a meal:** residents talk across the table while they eat, and a meal nobody needs is cooked only when there is nothing else to do.

### Babies and the garden
- **Babies** are lifted out of the crib, carried to the high chair and fed there by day, and changed at the changing table.
- **The garden:** tending brings crops on, and a harvest goes to the fridge. The apple and lemon trees ripen, are picked bare and fruit again. Crops are picked and the washing hung out as the hands go, not in the first second.
- **A guest's night** on the sofa bed leaves the living room a living room.

### Lamps and light
- **A lamp bought is off** until someone switches it on.
- **Light goes where the shade sends it:** pendants light down, sconces up the wall, the outdoor light outwards, the uplighter off the ceiling. On Medium and High a lamp lights only its own room.
- **Fires are lit by sitting by them**; garden and street lamps cast pools of light on the lawn; the fan turns only while it's on.
- **Glow:** coloured shades glow their colour, candles burn at night in a lit room, the packs' screens and mood lamps glow and light, and a glow no longer shows through the parts in front of it (the grill's coals through its shut lid).

### Sound and talk
- **Things sound like themselves:** the toilet flushes as you leave it, the dryer tumbles, the dishwasher clinks, the drum kit plays a beat, the fountains run on their own. What only sounded wrong (toast sizzling) is silent instead.
- **Talk:** residents talk from their seats and someone joining walks over to listen. Kisses and punches meet at any height, a refused hug or kiss and a flat joke show it, each social has its own gestures, and the phone is held in front to text or scroll.

### Checked afterwards
- **After a hidden tab** residents who sat down meanwhile are seated, and walkers walk, instead of replaying the sit-down or running on the spot.
- **In a crowded garden** a resident told to use something boxed in walks to whatever side is free, instead of dropping it.
- **The stairs** keep their top over a basement; **the siding** no longer borrows the pavement's shader; **reduce motion** stills the plants in the colour studio and the catalog pictures too.

### For developers
- **New content fields** (`docs/content-packs.md`): `stages` on objects, recipes and uses, `stages`/`targetStages` on socials and `upperStages` on bunks; `places` (a place and facing per slot) and `seatFacing`; `approach: side|behind`; `hours`, `sky` and `workOn`, with `laundry` in the rules; `partner` and `alone`; `spare`, `tends`, `yields`, `carried` and `atSlot`; `changesOutfit`; `footstool`; `sound`, `endSound`, a recipe step's `sound`, an object's `sounds` and `ambient`; `light.fire` and `light.aim`; `againstWall`, `headHeight` and `overhang`; `reach`; `gradual` and `fetch`; a recipe step's `animAt`.
- **Review tools:** `cargo review sizes` (every object's size and working heights against `docs/review/reference.json`) and `cargo review uses` (every use's pose, clip, minutes, effects, motions and sound, with flags); `cargo check-glb` measures seat, top, mattress and lamp heights against content and `characters/furniture.json`; `tools/bench/check.mjs sheet` (contact sheets), `gallery --each` and `clips --each` (a picture per model), and `play --with` (helpers placed first), `--use-existing` (the state's own things) and `--skills`; the `review-*` debug states (`review-stage`, `review-stage-night`, `review-garden`, `review-baby`, …); `poses.html?stage=` and `seats=`/`beds=`.
- **`bun run check:clips`** checks every resident clip on both bodies at every life stage, in CI, against a baseline that is now empty; CI also runs the tools' tests (the model gate over every shipped model).
- **Saves:** the save format is version 32 (30 adds the laundry, 31 the outfit each resident wears, 32 the day a pupil last went to school, so a load no longer takes 6 points off every pupil's grade as a missed day). 0.3.0's saves (version 29) load and play on, with one dirty and one washed load of laundry and everyone in the creator's outfit. Things standing where a grown footprint now reaches may overlap it in an old save.
- **The ledgers** of the review are in `docs/review/`, one per category, with every finding's status and its commits.

### Saves
- 0.3.1 writes save format 32. 0.3.0's saves load and play on; 0.3.0 can't open 0.3.1's (it keeps them and skips them).

### Known
- **Props in hand, body language and privacy** are still to come: books, cups, cutlery, the phone, the cue and the shaker are mimed; the body doesn't show feelings and residents turn on the spot without steps; a bathroom in use opens for anyone walking in, and "Try for a baby" is a long standing kiss.
- **Model work left:** the home chess table seats one (chess at home is practice), the apple tree's crown is out of reach but for one apple, and a child can't reach the picked fruit; the arc lamp's arc stops at its own tile; crisps, fruit and pastries from town come on a plate; the washing comes down onto the line all at once; the hot tub's cover folds as one piece.
- **Nobody serves in town**, the washer is stood at for its whole wash, and about thirty uses (the treadmill, the ball games, the coffee machines…) still have no sound.
- **On Low** a lamp's light still passes through walls, and at speeds 4 and 5 feet slide.

## 0.3.0 — 2026-10-21

**Play it in your browser:** https://krecks.github.io/idyll-lives/

In this release every object in the game is a model built in Blender, and many of them move as they are used: doors, drawers and lids open, beds fold out, grills lift their lids and clocks tell the time. Styles are gone; you give the parts of what you own a colour of your own instead. Light behaves as in a real house: the sun comes in only through windows and doorways, a room without windows needs its lamps, and residents switch lamps on and off. Generated houses are furnished for the household who lives there. Underneath, every object keeps an id of its own for life, Low draws far less, voices choose between the graphics card and the processor on their own, and the small things the 0.2.0 review left are fixed or explicitly deferred.

### New
- **Every object built in Blender:** the furniture, appliances, plants, the street's props, the things residents carry and cook with, the lawn's tufts and wildflowers and the woods' rocks are glTF models made with the game's Blender toolkit, each within a triangle and draw-call budget, with a far version where they are seen from afar. The code-built models stay as fallbacks, and a model file that fails to load is drawn as a box and tried again.
- **Things move as they're used:** fridge and freezer doors, cupboards, drawers, wardrobes and lids open for what a resident takes out or puts away; the futon, the sofa bed and a new fold-down wall bed open to sleep in and stay open until someone folds them up; the hot tub's cover comes off and goes back on after a soak. Grill lids and hoods lift, the fire pit's flames grow, TV screens glow, the cinema screen unrolls, the jukebox's record spins, the hoops' nets swish, crops grow from planted to ripe and picked, curtains close at night, and the wall, grandfather and alarm clocks show the game's time. A few things move on their own near the camera.
- **Residents hold what they use:** the barbell is pressed lying down, a dumbbell goes in each hand, the guitar and ukulele come off their stands and the steam mop is pushed over the floor. Riders pedal in time with the bike's cranks, recliners and the Adirondack chair tip their sitters back, and clean things go into the nearest cupboard or counter.
- **Colours instead of styles:** Modern, Cozy and Minimal are gone. Things marked "Any colour" take a colour on each of their parts, from the palette or any colour of your own, chosen in a colour studio that shows the item in use too (a fridge's inside). Recolouring what you own is free and can be undone.
- **Lamps switch on and off:** click a lamp to turn it on or off. Residents switch a room's lamps on as they come in after dark and off as the last one leaves. A lamp's light comes from its shade.
- **A softer look:** every lit surface is shaded with a wide, soft ramp in the light's own colour, a cool fill from the sky in the shade and a soft rim. Settings › Performance › Shading › Classic brings back the old shading, at once.
- **Sunlight through windows:** the sun comes in only through windows and doorways, whichever way the walls are shown. A room without windows is dark by day and lit by its lamps. Neighbours' windows go dark when they go to bed.
- **Doors swing:** front doors and bathroom doors stand shut and swing open as someone walks up to them. A door beside a diagonal wall opens only as far as the wall lets it, or slides into the wall.
- **Mirrors on every preset:** Low and Medium keep one reflection of the nearest mirror and draw it again when the camera, a resident in it or the room changes; High draws two every frame, as before.
- **Homes for who lives there:** generated houses are furnished for their household. Partners share a bed and everyone else has one of their own; cribs, toys and children's furniture are only in homes with children; spare bedrooms become a guest room, a study or a hobby room. The front door opens into the hall, the living room, the kitchen or the dining room, never a bedroom, nobody walks through a bathroom or a bedroom to get somewhere, and a garden door leads to the patio. Neighbours are picked to fit their house.
- **Too few beds:** moving a household into a home with fewer beds than residents asks first ("Not enough beds"). They can still move in, and you're told who has no bed of their own when the game starts.
- **Starting a game:** **Play** on a household's card in the neighbourhood starts the game in their home, and the card opens the household's details. Every household in a town has its own family name. A household is prepared while you choose, so the game starts without a long loading screen.

### Changes
- **Furniture can't be resized for now:** moving parts don't follow a resize yet. Something resized in an older save keeps its size, and "Reset size" takes it back. Doors and windows still resize.

### Fixes
- **A sofa bed no longer opens under someone sitting on it:** a resident relaxing, eating or watching TV on it keeps their seat, and an opened sofa bed is no longer offered as a seat.
- **An urgent need no longer cuts everything short:** a resident no longer starts, over and over, an activity their urgent need would stop at once.
- **Poses:** the bean bag's snoozer sits up, sleepers lie on the mattress with their heads on the pillow and their legs under the quilt, two sleepers' quilts meet in the middle of a double bed, a hug or a kiss steps in until the two meet, a chair is pulled in to its desk or table with the hands on the top, bathers sit along the tub, the climbing frame is climbed and the phone box's door shuts behind its caller.
- **The graphics API switch needs no reload:** Settings › Performance › Troubleshooting › Graphics API changes the drawing on the next start of the game, as its "Next load" badge says. A canvas that has held a WebGPU context can't give a WebGL2 one, so the new renderer gets a fresh canvas instead of failing to start.
- **Save and reload saves over an autosave:** the crash screen's save goes into the older of the game's two autosaves, not into a new manual save on every crash.
- **A stuck start says so:** a start that makes no progress for 30 seconds now reports "The game took too long to start." instead of waiting on "Moving in…" for ever. A slow download or a big town that keeps making progress isn't given up.
- **Walks no longer jump at the start and the end:** at speeds 4 and 5 a walk's first and last step snapped, because only steps that moved in both snapshots were smoothed. A step that starts or ends standing now glides too.
- **Retirees aren't told they're "between jobs"** or offered job suggestions: they have a pension and can still browse for work.
- **Progress bars no longer make the browser lay out the page every frame:** the skills, needs, career and goal bars animate a transform instead of their width.
- **Licence links show the licence:** the voice, dictionary and language-model licences end in `.txt`, so Credits' links open them instead of downloading them.
- **Cross-origin isolation is retried:** the one-time reload flag is cleared once the page is isolated, so a page whose service worker was later evicted tries again on the next load.
- **Interface:** right-clicking the game no longer opens the browser's menu (except in text fields); with an object's or a resident's menu open the camera still turns, pans and zooms; the top bar, the HUD and the menus fit small windows and large interface scales, and the new-game screens stack on narrow windows. A new game's first story event shows in the feed.
- **Building:** nothing stands off its tile through a wall, straight or diagonal; only curtains hang over a window; things stand with their backs on the wall's face, not in it; every kitchen worktop is 0.95 m high, so a run of counters lines up.

### Development
- **Every push and pull request is checked:** a new `ci.yml` runs clippy, the simulation's tests and the sample-house and sample-world tests. The release build and Pages deploy stay in `pages.yml`.
- **The tools are Rust:** `cargo art` builds objects in Blender, `cargo check-glb` gates every model file (size, triangles, draws, clips, far versions), and `cargo content`, `cargo characters` and `cargo tex` replace the Python scripts. Python is left only inside Blender. How to build an object is in `docs/design/objects.md` and `tools/art/blender/README.md`.
- **Checking in a browser:** `tools/bench/check.mjs` takes the model gallery, the clips bench (`/clips.html`) and buy-and-use pictures headless with its own server; the simulation's tests write named game states (`DUMP_STATES=1`, opened with `?state=<name>`); `?debug` adds `__sims()`, `__addFunds`, `__camera` and following.
- **The generated-house and generated-world tests can't pass vacuously:** they require `HOUSES`/`WORLDS`, and CI makes the sample files first.
- **The benchmark can measure the interface:** `tools/bench/render.mjs --hud 1` (the default is `0`, the renderer alone).
- **Crash stacks from the site can be read:** CI's wasm artifact is now the site's own optimisations with the name section kept (`wasm-opt --names`), not the pre-optimisation wasm with different function numbers. `wasm-opt`'s results are cached between CI runs.

### Simulation
- **Every object keeps an id of its own for life:** commands, the view, the renderer and saves name objects by a stable id instead of their place in the list, so selling, moving or undoing one thing (or moving a whole room) can't retarget another, and a menu left open on something sold does nothing.
- **Goals nobody can reach are refused at once:** the lot's walkable regions are labelled once per change to the house, and a resident asking for a way to another region is turned down before the search starts. No more flooding the whole town for a bed behind a wall.
- **People are remembered too:** a resident who couldn't get to someone they meant to talk to or join doesn't try again for half a second; things and places keep the content's retry time.
- **Walkers go round a change, or give up:** when a wall, door, stair or thing placed in their way changes the route they were walking, they work out a new one from where they stand. If there is none, they stop, and remember it if they chose it themselves.
- Looks, repairs and crops no longer rebuild what walkers can reach or the objects-by-plot index; only changes to walking do, and rooms are rebuilt only when the lot's layout changes.

### Objects
- **Ice cream from the freezer:** the fridge, built-in fridge and French-door fridge have "Get some ice cream", so their freezer doors and drawer open for a use instead of carrying a clip nobody played. The other modelled parts without a use stay decorative, documented in `docs/objects/open.md`.

### Settings
- **Settings are versioned and migrated in order.** Values from a hand-edited, truncated or older file are put back in range (numbers, switches and choices) instead of being used as they are.

### Voices
- **Voices choose where they run:** Settings › Voices › Runs on is Auto, Graphics card or Processor. Auto uses the graphics card when WebGPU is there, starts on the processor when it isn't, and steps down from a card that cannot start the voices or turns out too slow, with a non-blocking message. A manual choice is never overridden. The processor keeps up with speech at about 2.5× real time, so a laptop without a usable graphics card can still hear its residents.
- **Voices are about a quarter faster on the graphics card:** the voice model's working data stays on the card between passes, and six passes replace eight, sounding the same. On an M3 Pro a warm line takes about 185 ms instead of 233.

### Graphics
- **Low draws far less:** the village orbit by day falls from 447 draws and 5.1 M triangles a frame to 293 and 1.2 M, and from 12 long tasks to none (M3 Pro proxy, 1920×1080). The quilts and held things draw only their live rows instead of the whole town's capacity, the street uses its far models, the street and residents cast no sun shadows on Low, and one merged shadow caster replaces the lot's furniture templates.
- **Less drawn everywhere:** props nobody holds aren't drawn, a reflection draws only what its glass shows and only the residents in it, and the graphics-preset switch no longer keeps the old renderer in memory.

### Compatibility
- **Saves:** the save format is version 29 (0.2.0 wrote 23, 0.1.0 wrote 17); saves from 0.2.0 and 0.1.0 load and play on. Their objects get stable ids, their styles are dropped (every object shows its own model, in its colours as made) and lamps start switched on. A 0.2.0 build can't open a 0.3.0 save.
- **Runs on:** desktop Chrome, Edge, Brave or Safari with WebGPU; other browsers use the WebGL2 fallback. The language model needs WebGPU; voices run on the processor without it. Tested on Apple Silicon Macs in Chrome. There is no frame-rate target for particular machines: Low is for older laptops, best effort, and hasn't been measured on Intel graphics.

## 0.2.0 — 2026-10-13

**Play it in your browser:** https://krecks.github.io/idyll-lives/

In this release babies are cared for, meals are cooked from start to finish, everyone carries a phone and some houses have basements, in worlds of eight kinds. Residents can speak with new voices and, if you like, in their own words, written by a small language model on your computer. Before the release the whole game went through a review of animations, objects, bugs, performance and release readiness, and this release fixes what it found (details in `docs/release-0.2.0-review.md`).

### New
- **Babies:** a baby lies in the crib, curled up asleep or kicking awake, and cries when they need feeding, changing or a wash. Grown-ups pick them up for a cuddle, and at night a crying baby wakes one parent (partners take turns), or whoever is up comes. Nobody leaves a baby home alone: when everyone has to be at work, a babysitter comes for the day ($40), or a grandparent looks after them. A child who outgrows the crib gets a bed: neighbours buy one, and you're told when your own child has none.
- **Cooking:** meals are made from start to finish: fetch the ingredients, prepare, cook, serve, sit down at the table together, and wash up after. Nine recipes, from a sandwich to a tasting menu, a barbecue feast and pie. Food goes off, dirty plates make a room dirty, and "Clear away" scrapes, washes and puts away. Residents wash their hands after the toilet and eat snacks at the table.
- **A phone for everyone** from teens up: text, call, invite someone over, ask to come over, or scroll. A Phone tab shows contacts closest first and who is free, asleep or at work, and greys out what can't be done, with the reason. The phone box is gone from the market square.
- **Basements:** some generated houses on medium and large lots come with a basement down the stairs (a games room, a gym, a workshop, a laundry or a storeroom), lit by its own lamps. Build down there yourself too.
- **A new world first:** New game starts by generating a world of eight kinds (small, medium or dense towns, a farm, a seaside village, a mountain hamlet, a lakeside town, a forest clearing), each from a seed with its own name, streets and landscape. Towns get public places round the park (a playground, a market square, a café, a grocer, a bakery), and the streets are dressed.
- **Resident voices on the graphics card** (experimental, off by default): residents speak with Supertone's Supertonic 3 on WebGPU, about 28 times faster than real time on an M3 Pro. Ten voices, each resident with their own pitch, depth and speed, shaped in the household creator; children from the higher voices. About 217 MB, downloaded once. It replaces KittenTTS.
- **Residents in their own words** (optional): Settings › Voices › What residents say › Language model. A small language model, Liquid AI's LFM2.5 350M, runs on your graphics card and writes each line from who the resident is and what's going on, one to five sentences; lines show in the speech bubble too. About 259 MB, downloaded once.
- **Residents made in code:** new bodies for every life stage, faces with expressions and lip sync, hair, clothes and animations, all generated by the game's own tools. A build slider next to height.
- **Furniture:** 196 more things to buy, with tables and chairs apart, chairs that pull up to tables, and a Tabletop category of small things that stand on tops, plus 28 things for the town. Resize anything (whole tiles, or to the centimetre with Shift), drag furniture anywhere on its tiles, and make wide windows and double doors. Mirrors reflect the room (on High). Every model is built from primitives; no downloaded models are left.
- **Homes:** generated houses are furnished (dining sets, counter runs, kitchen islands, things on tops), have more and smaller rooms, and light every room at night. The lights go out in a room when everyone in it is asleep.
- **Everyday life:** residents look after their needs before they run out, sleep one proper night and rarely nap, take breaks at work, and want things of their own (they save up and ask before buying). Guests keep their hosts company, may use the toilet and say goodbye before they go home. A third and a fourth resident can join a conversation. Residents keep to the street instead of cutting through gardens.
- **Interface:** two modes, Live and Build (Furnish, Construct and Home in one dock), with Creation as a step of the new game. First-use tips instead of explanations always on screen.
- **Sound:** showers, taps, washing up, baths and the washing machine sound like real water.

### Stability and saves
- **A crash screen** instead of a silent freeze: if the game hits an error it says so ("Your last save is safe."), offers to reload, and can save a debug report. If only drawing or the interface broke, it offers Save and reload. After a graphics reset (a driver update, a GPU switch) the game offers Save and reload too, and if WebGPU won't start it draws with WebGL instead.
- **Damaged and too-new saves:** a save that can't be opened no longer blocks Continue. One the game finds damaged is removed with a short notice. One from a newer version, or one this version fails to load, is kept and marked ("This version couldn't open it"): Continue skips it, Load game can still export it, and a later version tries again.
- **Saves you can keep:** Load game can export any save as a file and import it again, and the game asks the browser to keep its saves.
- **Autosaves per game:** each game keeps its own two latest autosaves, so one game no longer overwrites another's. The game also autosaves when you leave the tab, and says so if an autosave fails. "Game saved" only shows once the save is really stored.
- **Long games:** from game day 1167 (about 7.8 hours at the fastest speed) births, ageing and deaths used to stop. They don't any more.
- **Furniture edits are safe:** selling or moving furniture no longer crashes the game or sends a resident to the wrong thing, a held key no longer turns the wrong object, and selling a fruit bowl or a rug leaves the dinner on the table.
- **After an update:** a tab left open across an update offers to reload instead of hanging, and a new release no longer mixes with files cached from the last one.

### Graphics
- **Graphics is Low, Medium or High,** and Medium, the game's intended look, is everyone's default. Low is for older laptops: simpler shadows, edges and lamps, half the lawn's tufts and flowers, and no shadows from the woods round the town. High adds sharper shadows and live mirrors. Ultra, Custom, the fine-tune switches and ambient occlusion are gone.
- **Resolution: Auto** draws the screen's own pixels up to about 1920×1080 and scales up beyond, so 4K and Retina screens run about as fast as a 1080p one; it follows browser zoom and a move to another screen. Custom draws 50–100 %.
- **The frame rate cap no longer judders:** frames are paced in whole display refreshes.
- The WebGL2 switch is under Settings › Performance › Troubleshooting.

### Performance and load time
Measured on an M3 Pro in Chrome, 1920×1080:
- **Faster start:** the menu appears about 1.4 s sooner for returning players (2.83 → 1.42 s with a save, warm cache). The game's data arrives in one download instead of about 60, the sky's lighting starts downloading with the page, and returning players load the game's scripts from their browser. A first visit at 10 Mbit/s reaches the menu in 3.14 s instead of 3.44.
- **Smaller download:** the simulation and voice WebAssembly are optimised and stripped (620 KB smaller, 44 KB gzipped).
- **Lighter drawing, the same picture:** lamps that are off by day cost nothing, grass, flowers and woods out of view aren't drawn, and the sun's shadows are drawn only from what stands near your lot and only when something in them moves. Medium by day on an M3 Pro (WebGPU, 1920×1080) now takes about 5.4 ms of graphics-card work a frame instead of 8.6 before these savings: about 154 fps instead of 96 without a cap. Medium also uses 141 MB of render targets a frame instead of 216.
- **Nothing drawn behind menus:** behind the pause menu, Settings, Load game, Credits and the planner the game stops drawing until you close them, and the menus' town is drawn at about 30 fps. High's mirrors take video memory only while they reflect.
- **Lighter interface:** progress and need bars no longer make the browser lay out the page on every frame, speech bubbles are placed without forcing a layout, goal ideas are worked out only when a plan changes, bird songs are made off the main thread, furniture for other lots is built in small steps, and the light changing at dusk no longer stalls a frame. With the processor slowed 3× to stand in for an older laptop, at speed 5 a frame now takes 7.2 ms of the main thread on average instead of 7.9 before the release review (9.4 instead of 12.4 for the slowest one in twenty), with no long stalls, and the town behind the menus about half the main thread's time it took. F3's "Busy (frames)" now counts the browser's layout and paint too.
- **Smoother motion:** residents glide instead of jumping at the fastest speeds, jog instead of sliding, face where they're going, and keep their animation when something in the house changes; the camera and clouds no longer jump after a hidden tab.

### Fixes
- **Residents:** a resident too tired to talk no longer pulls others into chats all night; nobody follows someone across town for a chat; practising a skill no longer stutters on and off; worn out with every bed taken, residents no longer give up the toilet at once; residents no longer use something that broke on the way, nor the neighbours' things when sent; two housemates no longer both walk over to the same repair; newborns are born at age 0; a family moving into town brings a grown-up child, not a baby left at the door; new residents start with the skills their traits give.
- **Money:** a recipe given up after buying the ingredients gives the money back. Residents at retirement age (65) without a job now retire with a basic pension of $500 a week, and every pension from now on is at least that; a job you give a retiree sticks.
- **Kitchens:** buying a range cooker no longer sells the fridge, and a stove without a fridge no longer counts as food at home, so hungry residents order takeout instead of going round in circles.
- **Saves and crashes:** Load game lists a save's own household, not everyone in town, and no "Autosave failed" notice shows over the crash screen. A game in which building left a ceiling lamp without a ceiling opens again, without that lamp.
- **Input, interface and sound:** after Alt+Tab, Build mode no longer picks up the floor on a plain click; a wall or carried object no longer sticks to the mouse; held keys no longer flicker pause or panels; the arrow keys turn the camera as fast at any frame rate; reduce motion also stills the scene's decorative movement. News in the feed can't be clicked once it has faded, and with reduced motion on it shows for 10 seconds instead of not at all. A master volume, sound stops when the game is in the background ("Mute in background", on), and an option to pause when the window loses focus.
- **Content packs:** content that would break the game (a walking speed of 0, a resale price above 1, a recipe of 300 steps, an unknown tag) is refused with a clear message (`docs/content-packs.md`).

### Compatibility
- **Saves:** the save format is version 23 (0.1.0 wrote 17); saves from 0.1.0 and from the builds after it load and play on. Saves now remember the seed the game started from.
- **Settings:** a Low graphics setting from 0.1 stays Low; everything else becomes Medium. Players who never changed it had High's settings, so they now get Medium, which has no live mirrors. Resolution starts on Auto.
- **Runs on:** desktop Chrome, Edge, Brave or Safari with WebGPU; other browsers use the WebGL2 fallback. Voices and the language model need WebGPU. Tested on Apple Silicon Macs in Chrome. On older Intel laptops (HD 620-class graphics) Medium's target is 30 fps, but it hasn't been measured on Intel hardware: scaled from Mac measurements, Medium is estimated at about 10–21 fps there by day (8–13 at night), so it probably falls short until someone measures it on a real laptop. Low there is best effort.

### Licence
Idyll Lives is free software under the GNU General Public License v3.0 (or later). Third-party parts keep their own licences: the art is CC0; the voice model, Supertone's Supertonic 3, is under the BigScience OpenRAIL-M licence, whose use restrictions bind everyone who plays; the dictionary (Misaki) is Apache-2.0; and the optional language model (Liquid AI's LFM2.5 350M) is under the LFM Open License v1.0. Every build lists the code and files it bundles with their licence texts in `THIRD-PARTY-LICENSES.txt` (the Zstandard texture decoder and TalkingHead's facial expressions included), and Credits names them.

## 0.1.0 — 2026-10-10

**Play it in your browser:** https://krecks.github.io/idyll-lives/

The first release: a home-building game with a living simulation, in the browser. You build and furnish a home; its residents live in it on their own while you watch. (Earlier development builds were called Littlelives, numbered 0.3–0.20, and MIT-licensed; their notes are in the git history. Saves, settings and downloads from them carry over.)

### In this release
- **A living town:** create a neighbourhood (town size, neighbour households), then play one of the households who already live there or make your own in the 3D household creator (gender, who they're attracted to, skin, hair, clothes, traits, perks, starting bonds, voice) and move into a vacant house. Neighbour houses are generated, with one or two storeys. Everyone in town lives, works, socialises, visits friends and falls in and out of love on their own; the rest of the town keeps simulating while only your lot is drawn.
- **Watching:** left alone, the camera follows what happens at home (conversations, fights, first kisses, guests, someone using what you just built). The household strip, resident panels, following (F), the whole house (H), the journal (J), the town map (M), and speeds up to time-lapse with quiet hours skipped.
- **Planner (P):** a week per resident: activities on a calendar and life goals. Plans are a strong nudge, not orders, and each block shows whether it was kept and why not.
- **Jobs, skills and money:** 1,000 jobs (25 categories × 4 tracks × 10 grades), probation, promotions and workweeks that shrink with skill; 13 skills trained at work and at home; rent and bills every Sunday, and debt when a household can't pay.
- **Buy mode (V):** about 210 things to buy across Kitchen, Bathroom, Bedroom, Kids, Living, Office, Fitness, Hobbies, Decor, Lighting, Outdoor and Garden, shown as 3D pictures, with styles (Modern, Cozy, Minimal), upgrades (★ quality), things on walls and ceilings, rugs, lamps that light their room at night, and over 50 plants residents tend and harvest.
- **Build mode (B):** walls (straight and diagonal, full or half height, with coverings), rooms, paint, doors and windows in styles, stairs to a second storey, and walls up, cut away or down (W).
- **Resident voices (experimental, off by default):** the resident you're looking at, and conversations you start, are spoken in English by KittenTTS nano running on your computer (about 32 MB, downloaded once): eight real voices, mixed so every resident sounds their own, children higher and smaller, about 2,350 lines per item, interaction, mood and thought. Voices pause for the session if this computer can't keep up.
- **Sound:** birds by species singing in bouts through the day, crickets at night, and the sounds of things in use at home.
- **Saves, settings and tools:** save and load, Settings (graphics quality, audio, controls), a performance overlay (F3) and debug reports (F8). Content packs can add objects, art and voice lines (see `docs/content-packs.md`).
- **Runs on:** desktop Chrome, Edge, Brave or Safari with WebGPU; other browsers use the WebGL2 fallback. The simulation is Rust compiled to WebAssembly in its own worker.
- **Licence:** Idyll Lives is free software, released under the GNU General Public License v3.0 (or later). You may use, study, share, and modify it; if you distribute Idyll Lives or a modified version, you must make the source available under the same licence. Third-party parts keep their own licences (art CC0, voice model and dictionary Apache-2.0).

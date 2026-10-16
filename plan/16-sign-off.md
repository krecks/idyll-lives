# Sign-off: back from Validate

A task is signed off when its items have been checked in play and look and move right. Then they leave Validate.

## Per task

1. **Check it in play.** For every item in the task's Validate group:
   - buy it and place it in all four facings;
   - colour every part in the colour studio;
   - let a resident use it, so every clip plays and shuts when it should;
   - check it by day and at night, on WebGPU and `&renderer=webgl`, and on the Medium and Low presets.
2. **Move it back.** Set each item's `category` and `group` back to the task table's "Back to" column. An item
   whose "Back to" group is "—" has no group (the personality packs). Validate turns nothing freely by
   itself: items whose home category turns them (`objectRules.freeRotation`) carry `"freeRotation": true`
   while they're in Validate. Back home, the flag can go (the category gives it again) or stay.
   `python3 -B tools/content/validate.py out --group <group>` does it by id, keeping the files' formatting:
   each item goes back to its "Back to" (or `--to <category>/<group>`), and the emptied Validate group is
   dropped. Try it with `--dry-run` first; `validate.py list` shows what's in Validate and where it goes.
3. **Tidy up.**
   - Remove the task's group from the `validate` category in `web/public/content/base.json` (the tool does).
   - Run `cargo test -p sim-core --test shipped_content`.
   - Commit: `feat: <task> back in <category> after validation`.

## The sinks first

The seven sinks (Validate › Sinks) were the first group, signed off on 2026-10-16: bought in all four facings,
every part coloured (through `buy` and in the colour studio), washed at (and washed up at, the farmhouse sink) with
`water_on` playing and stopping when the use ended, by day and at night, on WebGPU and WebGL2, on Medium and Low.
Before commit 1caf5ec they were, and are again:
- `sink`, `sinkPedestal`, `vanityDouble`, `sinkVessel`, `sinkConsole`: bathroom / sinks;
- `farmhouseSink`, `counterSinkDouble`: kitchen / counters.

## The fridges

The seven fridges (Validate › Fridges) were signed off on 2026-10-16, checked as the sinks were: every use
(snack, meal, gourmet, cold drink, wine) opens its door clip and shuts it after its `hold`; the retro fridge pulls
its lever first; the French-door fridge opens one door for a cold drink. Gourmet needs a stove and a counter
(its prep and cook steps), which the check house hasn't: checked with both bought. All back in kitchen / appliances.

## The cookers

The six K2 cookers (Validate › Cookers) were signed off on 2026-10-16, checked as the sinks were, each use run
as a recipe (fridge, then a counter, then the cooker): the gas range's and range cooker's flame rings and the
electric cooker's knob for cook (and gourmet), the oven doors for bake and roast, the hob counter's heat ring, the
microwave's door and turntable for reheat. Each shuts when the resident moves on. Seen in passing: at the
microwave the resident's hands reach for the cabinet below, not the door (the shared "cook" pose). All back in
kitchen / appliances.

## The small appliances and dishwashers

The four K3 things (Validate › Appliances) were signed off on 2026-10-16, checked as the sinks were, after two
fixes found in the check:
- **The dishwashers' doors** drop 0.44 m into the tile in front, where the loader stood on them. Residents
  now stand 15 cm further back while they use either one (`characters.ts` `STAND_BACK`).
- **The blender** only lifted its lid: the smoothie under it now swirls while it blends (`blades`, a loop,
  E1). The toaster's lever and toast became one part (the carriage, 5 cm down) to keep 4 draw calls.

The coffee machine's cup and stream, the toaster and both dishwashers' doors and racks play and shut as
they should. All back in kitchen / appliances.

## The base counters

The eight K4 counters (Validate › Counters) were signed off on 2026-10-16, checked as the sinks were: the
counter, corner counter, butcher block and marble counter open their drawer for a sandwich (fridge, then the
counter); the drawer counter opens its top drawer when clean dishes are put away there (a snack's plate
cleared: bin, sink, then `putAwayAt`); the end, outer corner and open-shelf counters serve as the prep station
of a meal. Two fixes from the check: the open-shelf counter's back is the cabinet's colour outside (it was
the shelves' dark back, showing when the counter stands free), and the butcher block's top surface has its
own colour (`Top`; colouring "Block top" only coloured the block's rim). All back in kitchen / counters.

## The tall and wall storage

The eleven K5 things (Validate › Kitchen Storage) were signed off on 2026-10-16, checked as the sinks were:
the pantry's and tall larder's doors for a snack; the three wall cabinets' doors when clean dishes are put
away in them (a snack's plate cleared, the cupboard over the counter first); the pedal bin's lid when a plate
is scraped (checked with the house's sinks sold, so the scraps go in the bin); the recycling bins' three lids
one after another for "Sort the recycling". The hood, spice rack, pot rail and wall shelves are still, looked
at on all four walls. One fix: the recycling bins' "Left bin" and "Right bin" coloured the bins the other way
round as seen from the front; swapped. All back in kitchen / counters.

## The islands and bars

The five K6 things (Validate › Islands & Bars) were signed off on 2026-10-16, checked as the sinks were, each
standing free in the room: the kitchen island's drawer for a sandwich (and none for breakfast, eaten at it,
with and without a stool behind); the small and long islands as the prep station of a meal; the bar counter
and bar end looked at from all four sides. Nothing needed fixing. The kitchen island back in kitchen /
counters, the others in kitchen / islands.

## The dining tables

The thirteen K7 tables (Validate › Tables) were signed off on 2026-10-16, checked as the sinks were, each
standing free and looked at in all four facings. The four with their own seats were used as they are:
coffee at the dining table; breakfast and coffee at the bistro set and the glass table; breakfast, coffee and
dinner at the farmhouse dining table. At the nine plain tables a snack from the fridge was eaten from a dining
chair (a bar stool at the high café table) pushed up to it: the plate on the top, the eater sat up to it.
Nothing needed fixing. The dining, glass and farmhouse sets' chairs take their table's "Legs" colour (they
are one model). All back in kitchen / dining.

## The chairs and stools

The twelve K8 seats (Validate › Chairs) were signed off on 2026-10-16, checked as the sinks were, each
standing free: sat on in all four facings; the three stools swivel (`swivel`, with the sitter riding it) for
sit and drink and shut after their `hold`; a snack eaten from each at a table, counter or high café table
pushed up to it; coffee (or a drink, on the counter and high stools) at a table, which those uses need
(`needsTable`: refused for a seat standing alone). Nothing needed fixing. All back in kitchen / dining.

## The dining storage

The four K9 things (Validate › Dining Storage) were signed off on 2026-10-16, checked as the sinks were: the
sideboard's, buffet's and china cabinet's door (the china cabinet's left glazed door) when clean dishes are put
away in them (a snack's plate cleared: bin, sink, then the cupboard), in all four facings (three against a wall,
one standing free); the bar cart's shaker lifts and tips towards the resident (`shake`) for "Mix a drink" and
shuts after its `hold`, standing free in all four facings. One fix: the china cabinet's hutch back was blue
outside as well as in, showing when it stands free or with the wall cut away; it is the body's colour outside
now. Seen in passing: at the china cabinet the resident reaches low (the shared "fetch" pose) while the glazed
door above opens. All back in kitchen / cabinets.

## The ceiling lights

The seven LT1 lights (Validate › Ceiling Lights) were signed off on 2026-10-16, checked as above, hung at 24,17 in
the living room (camera at 2.1 m): the pendant, ceiling light, ring chandelier, industrial pendant, ceiling fan,
track lights and paper lantern hang from the ceiling height in all four facings, nothing floating or cut. At
night each lights a pool on the floor under it and its shade (globes, panels, paper, the industrial pendant's and
track heads' liners and lenses) glows; by day none glows. The fan's `blades_turn` (2.5 s) and the lantern's `sway`
(2 s) go round all the time near the camera on WebGPU, day and night, and are off on Low, as designed (E2).
Nothing needed fixing. Seen in passing: the industrial pendant's green dome looks black from above at night (only
its white liner glows, inside), as the notes intend. All back in lighting / ceiling.

## The wall lights

The three LT2 lights (Validate › Wall Lights) were signed off on 2026-10-16, checked as above on four walls with
the camera at 1.6 m: the living room's north (23,14,0) and east (28,19,3) walls, and, for the other two facings,
the bedroom's west outer wall (15,19,1) and the living room's south outer wall (22,25,2), day and night. The
sconce, swing-arm lamp and bulkhead sit with their backs on the wall face at their height; at night each throws a
warm pool on the wall round it and its shade or lens glows. Nothing needed fixing. Seen in passing: on a wall the
cutaway cuts (an inner wall between two rooms, or an outer wall seen from outside: 21,19,1, 20,19,2, 29,19,1,
23,13,2 were tried first) the lamp hangs in the air over the cut wall, as anything on a cut wall does (renderer,
not the models). The bulkhead's "Shade" is its glass lens: coloured, it tints faintly, as see-through glass does.
All back in lighting / wall.

## The floor lamps

The five LT3 lamps (Validate › Floor Lamps) were signed off on 2026-10-16, checked as above standing free at 23,16
(camera at 1.1 m), and the arc lamp also with its base against the north and east walls (23,14,0, 28,19,3), as
its notes intend: its arc reaches out over the room. Floor lamp, simple floor lamp, arc lamp, tripod lamp and
uplighter stand on the floor in all four facings; at night each lights the floor and wall round it, the drum,
square and linen shades and the uplighter's bowl glow, the arc lamp's metal dome glows inside. The floor lamp's
third part "Fittings" colours its brass collars and harp, the arc lamp's and uplighter's "Base" their foot.
Nothing needed fixing. Seen in passing: the light is at the footprint's middle, so the arc lamp's light is 28 cm
from its shade (the pool still reads as the lamp's, against a wall it is on the wall beside the shade); the
uplighter's coloured glass bowl tints faintly. All back in lighting / floor.

## The table lamps

The six LT4 lamps (Validate › Table Lamps) were signed off on 2026-10-16, checked as above at 23,16 (camera at
0.7–0.8 m): the table lamp (a floor object with its own oak side table; "Table" colours the table) and the five
surface lamps (ginger-jar, square, banker's, mushroom, candle lantern) each stood on a side table bought first,
turned to all four facings on it. They stand on the top, carried at its height (lift 0.55 m), with their light at
the top plus `light.height`; at night the shades, the mushroom's cap, the banker's liner and the lantern's flame
glow and the table and floor round them are lit. Nothing needed fixing. Seen in passing: the table lamp's light
(0.95 m, content) is 14 cm above its bulb and 12 cm to the side of it (the lamp stands off-centre on its table),
just above the shade's top; it looks right. A top under a lit lamp is washed almost white at night (the light is
30–45 cm above it and the shade casts no shadow: renderer). The lantern's "Glass" tints faintly, as glass does.
All back in lighting / table.

## The wall decor

The six D1 things (Validate › Wall Decor) were signed off on 2026-10-16, checked as the sinks were, each hung on
all four walls of the living room (north 23,14; west 21,19; south 22,25; east 28,17; the 2-wide abstract canvas
on the bathroom's and bedroom's plain walls, 17,20 and 20,18, where two plain tiles side by side are free of
windows), with walls up and the camera in the room: the framed print, photo print and floating shelves looked at;
a resident admiring the oil painting and the abstract canvas (the "look" pose, facing it); the wall clock's hands
telling the game's time (ten past ten in the morning, ten to ten at night). By day and at night (lamps lit), on
WebGPU and WebGL2, on Medium and Low; every part coloured through `buy` and in the colour studio. One fix: the
wall clock's face sat behind the front of its case, so the clock read steel grey all over and "Face" coloured
nothing; the case's well now ends behind the face (white by default, colourable). All back in decor / wall.

## The ornaments

The ten D2 things (Validate › Ornaments) were signed off on 2026-10-16, checked as the sinks were: the eight
standing pieces (vases, urn, bust, cat, horse, coat stand) standing free in all four facings; the bust admired
from each side; the grandfather clock wound (the "tinker" pose at its case) against all four living-room walls,
its hands keeping the game's time; the hanging plant watered, its pot swaying out (`sway`) and settling after its
`hold`; the fireplace on the bathroom's, bedroom's and living room's walls, a resident warming up sitting on the
hearth bench with the flames (`flames`) growing behind her and dying down when she got up, the fire lighting
the room at night (checked with the room's ceiling lights sold). One fix, in the content: the horse statue was
named and described as a rearing porcelain horse on a wooden stand, from the downloaded file it once was; it has
been a bronze horse standing on an oak plinth since the primitive placeholders, and the Blender model is that
too, so it is now the "Bronze horse". The pedestal vase's description said white; it is blue (model and
placeholder). All back in decor / ornaments.

## The textiles

The six D3 things (Validate › Textiles) were signed off on 2026-10-16, checked as the sinks were: the curtains
and floor-length curtains over windows on all four sides (the living room's north, south and east windows, the
bedroom's west one), open by day and drawn closed at night (`close`, the night clip), on WebGPU and WebGL2 at
Low; the four rugs flat on the floor in all four facings. Every part coloured: fabric and rail; each rug's field
(the patterned rug checked in teal as well, its default being close to the test colour), its border and motifs
staying. Nothing needed fixing. Seen in passing: a 2-wide curtain over the house's 1-wide windows hangs with the
window in one half of its rail (that's the placement, not the model). All back in decor / textiles.

## The music

The six H1 instruments (Validate › Music) were signed off on 2026-10-16, checked as the sinks were: bought in
all four facings (the piano against three walls and standing free), every part coloured (through `buy` and in
the colour studio), played by day and at night, on WebGPU and WebGL2, on Medium and Low. The piano's fallboard
lifts and its music desk tilts for play and shut when the use ends. The ukulele and the two guitars leave their
stands (`off_stand`) for strum, jam and practise and are drawn in the player's hands (the neck in the left hand,
the body against the belly), back on the stand when the use ends; looked at from behind the stand as well. The
drum kit's cymbals rock in a loop (`cymbals`) for drum and practise and stop with the use. The digital piano is
still. Nothing needed fixing. Seen in passing: the acoustic guitar's and the ukulele's colour ("Guitar",
"Instrument") goes on the back and sides, not the soundboard, so from the front and in the hands a new colour
shows little (the electric's whole body takes it). All back in hobbies / music.

## The games

The six H2 things (Validate › Games) were signed off on 2026-10-16, checked as the sinks were, each standing
free (the dartboard on the north, west and east walls and on the north wall's outside face, the arcade cabinet
against three walls and standing free): the chess pawn slides for chess; the pool rack lifts for pool and trick
shots; the foosball rods kick in a loop and the arcade joystick waggles in a loop while played, both stopping
with the use; three darts grow in the board for darts; the jigsaw patch spreads for jigsaw. The arcade's screen
and marquee glow at night. One fix: the chess table's and jigsaw table's second colour was called "Trim" but
coloured the chess chair and the jigsaw stool; it is "Chair" and "Stool" now, and the chessboard's rim (which
took the chair's colour) is the table's wood. Seen in passing: the darts are thrown from the tile in front of
the board, half a metre from the wall. All back in hobbies / games.

## The crafts

The seven H3 things (Validate › Crafts) were signed off on 2026-10-16, checked as the sinks were, each standing
free (the workbench against three walls and standing free): the painting grows on the easel's canvas for paint;
the workbench's vice shuts for tinker; the telescope's tube tilts up for stargaze and comes back down; the sewing
table's presser foot lowers and its handwheel turns for sew and mend; the chemistry bench's flask fills for
experiment; the pot rises and the wheel head spins for throw; the scrapbook opens for scrapbook. Every clip
shuts when the resident moves on. Nothing needed fixing. Seen in passing: the sewing table's "Table" colours its
top and chair but not its legs and apron (their own, darker wood). All back in hobbies / crafts.

## The kitchenware

The eleven TT1 things (Validate › Kitchenware) were signed off on 2026-10-16. They have no uses, so each was bought
on a square table (`tableSquare2`, its top at 0.75 m) standing free in the living room and looked at close up, from
its front three-quarter and level from its side: in all four facings by day; with every paint slot a distinct colour
by day (facings 0 and 2) and at night (21:30, lamps lit); at night on Low with WebGL2. Each sits on the top (lift
0.75; every model's lowest point is at its origin), turns with its facing, and lights right at night. The colour
studio opened for each with its labels as the manifest has them. The stand mixer and cookie jar, red by default,
were also coloured purple to see the colour land. Two fixes:
- **The bread bin's wooden ends** showed a flickering V where the round end and its foot were one plane, and the
  round end's flat face shaded like a dome (a smooth-shaded cylinder). The foot is 2 mm thinner and the round end
  flat-shaded: flat D-shaped ends now.
- **The kettle's** second colour, "Base & lid", also colours the handle: now "Base, handle & lid".
Seen in passing: the knife block's tilted block rests on one edge of its foot, a small gap at the other (stylised,
left). All back in tabletop / kitchenware.

## The tableware

The seven TT2 things (Validate › Tableware) were signed off on 2026-10-16, checked as the kitchenware: on the
table in all four facings, every part coloured by day and at night, Low and WebGL2 at night, and the colour studio.
The place setting's Body, Rim and Napkin and the cake's Body and Icing land where named; the tea set's Body
colours the cups as well as the pot (a set). Nothing needed fixing. All back in tabletop / tableware.

## The small ornaments

The six TT4 things (Validate › Small Ornaments) were signed off on 2026-10-16, checked as the kitchenware. The
alarm clock's hands tell the game's time: ten o'clock at 10:01, 9:35 at night, and following it when time runs
(12:34, 15:04, run at top speed and paused). One fix: its second colour, "Bells", also colours its legs and feet:
now "Bells & legs" (and coloured purple and orange to tell it from its red default). All back in tabletop /
ornaments.

## The desk and bath bits

The eight TT5 things (Validate › Desk And Bath Bits) were signed off on 2026-10-16, checked as the kitchenware. The
toy blocks' Block 1–3 are the red, blue and yellow blocks (seen with a second palette, as the first matched their
defaults); the green blocks, pink block and roof keep their colours. The monitor's Body colours its bezel,
keyboard and mouse, not the stand. Nothing needed fixing. Back in tabletop / bath (soap set, towels, perfume tray),
tabletop / desk (organiser, laptop, monitor) and tabletop / toys (teddy, blocks).

## The desks

The eight O1 desks (Validate › Desks) were signed off on 2026-10-17, checked as the sinks were, each in all
four facings: against the north wall (23,14,0), the west wall beside the bookshelf (21,18,1), standing free
facing north (23,17,2) and against the east wall (28,17,3); the corner desk at 23,14,0, 21,18,1, 23,16,2 and
27,17,3. Each also with every part a distinct colour by day (WebGPU), on Low with WebGL2 at night, and coloured
at night (WebGPU), then in the colour studio, each part coloured in turn and the desks turned round to see the
parts at the back. The standing desk's keyboard shelf slides out for Browse; the computer desk's keyboard tray
for Practise coding; the writing desk's drawer for Write (it shuts after its `hold`); the laptop opens for
Browse; the gaming PC's fans go round while a game is played and finish their round when it ends. Each shuts when
the use ends. At the four desks with their own chair the resident sits facing the wall, the screen towards them.
The simple, corner and executive desks are still, looked at from all four sides. One fix: the executive desk's
plinths, under its pedestals, took the "Drawers" colour; they are the case's ("Frame") now. Seen in passing: the
desk chairs built into the computer, writing and laptop desks take the desk's "Frame" colour on their bases and
castors ("Chair" colours the seat and back). The simple, corner, standing and executive desks back in
office / desks; the computer, writing and laptop desks and the gaming PC in office (no group).

## The office chairs

The three O2 chairs (Validate › Office Chairs) were signed off on 2026-10-17, checked as the sinks were, each
standing free (23,16) and sat on in all four facings: the seat swivels (`swivel`) and shuts after its `hold`, by
day and at night, on WebGPU and WebGL2, on Medium and Low. Each was also pushed up to a simple desk (chair at
23,15 facing it): Write at the desk and Do homework at the desk (the executive and desk chairs, which need a desk,
`needsTable`) and Sit (the office chair), the sitter facing the desk, the swivel playing for each. Colour studio:
Frame (base, lift and arms), Seat (seat and back; the desk chair's mesh back is its own "Back"). Nothing needed
fixing. The executive and desk chairs back in office / chairs; the office chair in office (no group).

## The office storage

The three O3 things (Validate › Office Storage) were signed off on 2026-10-17, checked as the sinks were. The
filing cabinet stands against the wall (north, west, east) and free facing north; its top drawer slides out
(`drawer_open`) for File papers and shuts after its `hold` in every facing (the plan's table still calls it
static: the use came with the model). The whiteboard and corkboard hang on the north, west and east walls and
on the outside of the north wall (23,13,2); the whiteboard's plan grows across the board while the resident
plans at it, and is wiped as the clip plays back. The corkboard is still. Colour studio: the cabinet and its
handles; each board and its frame (the whiteboard's marker tray with the frame); "In use" opens the drawer and
fills the board. Nothing needed fixing. All back in office (no group).

## Done before their tasks had tables

Moved into Validate on 2026-10-15 (finished but never put in): the fridge into Validate › Fridges, the wall bed
into Validate › Sofas, the six garden decor models into Validate › Garden Decor. They go back to:
- `fridge`: kitchen / appliances;
- `wallBed`: living / seating;
- `garden.gnome`, `garden.birdbath`, `garden.fountain`, `garden.wheelbarrow`, `garden.boulder`: outdoor / decor;
- `garden.lantern`: outdoor / lights.

## Per category

A category is signed off when all its tasks are: its file's tables are all built, back, and committed. Mark each
task done in the category file (strike through its heading, with the commit).

- **Kitchen** (`01-kitchen.md`, K1–K9): signed off 2026-10-16.
- **Lighting** (`11-lighting.md`, LT1–LT4): signed off 2026-10-16.
- **Decor** (`09-decor.md`, D1–D3): signed off 2026-10-16.
- **Hobbies** (`08-hobbies.md`, H1–H3): signed off 2026-10-16.
- **Office** (`06-office.md`, O1–O3): signed off 2026-10-17.

## The end

- **Last group:** when the last group leaves Validate, remove the `validate` category and `icon.category.validate`
  (`web/public/assets/manifest.json`).
- **Docs:** update `docs/design/objects.md`. "From 0.3.0 a new object's model is built in Blender" becomes "every
  object's model is built in Blender", and the "Migrate or keep" section changes to match.
- **Placeholders:** stay as the fallback (keys unchanged; saves and content need no migration).

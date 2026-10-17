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
top and chair but not its legs and apron (their own, darker wood); on 2026-10-17 they got a colour of their own,
"Legs" (with the drawer), dark wood as made. All back in hobbies / crafts.

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

## The toilets

The five B1 toilets (Validate › Toilets) were signed off on 2026-10-16, checked as the sinks were: each bought in all
four facings (against the north, west and east walls, and standing free facing north), used ("Use toilet") with the
resident's bladder first let down to 0.65 so the use lasts (at 0.8 it ends in a blink, the use stopping once the need
is full), by day and at night, on WebGPU and WebGL2, on Medium and Low, every part coloured (Bowl, Seat & lid,
Cistern: on the wall-hung and smart toilets the Cistern is the boxing panel). `lid_up` plays in full (0.58 s; the
smart toilet's slower 0.79 s) as the resident sits and shuts as they get up; the resident sits on the seat, leaning
forward (the shared "toilet" pose). The colour studio's "In use" lifts the lid. Nothing needed fixing. All back in
bathroom / toilets.

## The laundry

The six B4 things (Validate › Laundry) were signed off on 2026-10-16, checked as the sinks were: the washing machine
("Do the laundry") and tumble dryer ("Fold the laundry") against the walls in all four facings, their round door
swinging open (`door_open`, held 2.5 s) and shutting, the drum turning (`drum_spin`) round and round for the use and
stopping after it; the towel rail on the walls in all four facings, the towel ladder leaning on them, the laundry
basket and bath mat standing free in all four; every part coloured by day and at night, WebGPU and WebGL2, Medium and
Low. One fix: the towel ladder's colours were "Towels" (the middle towel), "Second towel" (the top one) and "Third
towel" (the bottom one); they are Top towel, Middle towel and Bottom towel now, listed top to bottom (manifest labels
only; the file is unchanged). All back in bathroom / laundry.

## The bathroom storage

The three B5 things (Validate › Bathroom Storage) were signed off on 2026-10-16, checked as the sinks were, on the
walls in all four facings: the mirror cabinet's mirrored door swings open (`door_open`, held 1.5 s) for "Brush your
teeth" and shuts, the resident standing on the tile in front; the tall cabinet's upper door opens onto its towels for
"Take a fresh towel" and shuts; the round mirror hangs on its strap. Every part coloured, day and night, WebGPU and
WebGL2, Medium and Low; the studio's "In use" opens both doors. One fix: the tall cabinet's door colour was called
"Frame"; it colours the two doors, and is called Doors now (manifest label only). The mirror cabinet's "Frame" (its
door, a white frame round the mirror) stays. All back in bathroom / sinks.

## The baby and kids' beds

The four KD1 things (Validate › Baby And Beds) were signed off on 2026-10-16, checked as the sinks were, against
the four walls of the family house's living room: the crib's front rail slides down 25 cm (`side_down`) for
feed, change and cuddle, and back up when the use ends, with the baby lying in the crib half in every facing;
its mobile turns (`mobile`, a loop) while a grown-up plays with the baby and finishes its round after. The kids'
bed and race car bed turn their duvet down (`turn_down`) for sleep and nap, a child lying in them with the
game's quilt over them, and shut when the use ends (sleep cancelled after the picture; naps end by themselves).
The high chair is static, looked at standing free in all four facings; it has no use, and no sit interaction,
so nobody can eat from it yet (as the plan says: a "tray slides out" use comes with the sim). Colours land on
their parts (crib Frame, Bedding and Mobile; beds Frame / Car body and Bedding; high chair Frame and Bedding
on the cushions). Nothing needed fixing. Seen in passing: the carer at the crib stands or kneels in different
spots by facing, once a tile out in front of the changing table (slot positions and the shared "tend" pose);
the plan's line says the mobile turns "while the baby lies in it", the model turns it for "play" on purpose
(kids.md: the uses can't tell an awake baby from a sleeping one). All back in kids / —.

## The toys

The ten KD2 toys (Validate › Toys) were signed off on 2026-10-16, checked as the sinks were, a child (9) playing
with each: the toy box's and chest's lids open (`lid_open`) for play and shut after; the doll house's front
swings open (`front_open`) for dolls; the rocking horse rocks (`rock`, a loop, the rider riding it) and finishes
its round; the tent's flap rolls up (`flap_open`); the ball pit's balls bob (`bounce`, a loop); the train goes
round its ring (`train`, a loop; also seen at quarter turns in the clips bench); the play mat's tower of three
blocks grows (`blocks`); the teddy squishes (`squish`) for hug and the bean bag flattens (`squash`) under the
sitter. Free-standing toys in all four facings in the middle of the room, the doll house, ball pit and play mat
against all four walls. One fix: the toy box's lid was labelled "Trim" (the box has no trim; the part is the whole
lid): it is "Lid" now (labels only, b43a0e6). Seen in passing: at the teddy the child stands beside it (the shared
"relax" pose), not hugging it; the toy box's player stands at its front corner. All back in kids / —.

## The kids' study

The six KD3 things (Validate › Kids Study) were signed off on 2026-10-16, checked as the sinks were: the desk's
pencil drawer slides out (`drawer_open`, held 2 s) for homework and drawing, the child sitting at it, against
all four walls; the picture-book shelf's middle book tips out (`book_out`) for read, against all four walls; the
chalkboard's house draws itself (`drawing`) and the easel's painting paints itself (`painting`), each standing
free in all four facings; the kids' chair sat on in all four facings, and pushed up to the kids' table for
"Draw at the little table" and "Do homework at the little table" (`needsTable`); the kids' table looked at in all
four facings. Two fixes: the chalkboard's drawing and the easel's painting showed at rest as a hairline of their
colours down the board (a part grown along x only, squeezed to 0.001); they now rest as a speck and get their
height and depth back in the clip's first frames (fb19592; gate 0 failed, Khronos 0 errors 0 warnings, checked
in the studio and the clips bench and in play). The kids' chair's seat was labelled "Top": it is "Seat" now
(b43a0e6). Seen in passing: the drawing grows from the board's right as you face it (the family's "left" is the
object's own). All back in kids / —.

## The beds

The nine BR1 beds (Validate › Beds) were signed off on 2026-10-16, checked as the sinks were: each standing free and
napped in, in all four facings (`nap`: the 480-minute sleep plays the same clips), by day and at night, on WebGPU and
WebGL2, on Medium and Low, with every part coloured (through `buy` and in the colour studio). The duvet's top third
turns down (`turn_down`) as the sleeper gets in and back up as they get up, under the game's blanket; with two
sleepers (Ada and Ben in `family-check`) both quilts cover the turned-down duvet and nothing pokes through, on the
double, king, oak double, upholstered queen and four-poster. The bunk turns down the lower duvet for the one in
slot 0 and the upper (`turn_down_upper`) for the one in slot 1, each shutting when its sleeper gets up. The futon's
back folds flat (`unfold`) for a nap or a night, and the last one up folds it away (as the wall bed); Unfold leaves it
open and Fold away closes it. The daybed is still. Two fixes:
- **The four-poster's duvet foot** flickered: its end was in the plane of the mattress's end. It now stands 1.5 cm
  past it (529605e).
- **The daybed's back cushions** took in a sleeper's shoulder and hip. They are thinner, closer to the back and lean
  less (51bb85d).

Seen in passing: two residents getting up from the same bed stand on the same spot for a moment (sim); the oak double's
"Headboard" colours its teak slats, the posts and rails going with the frame; the daybed's back runs along a long
side, so standing head-end to the wall it doesn't read as a sofa (as built: head end on the wall). All back in
bedroom / beds.

## The nightstands

The eight BR2 things (Validate › Nightstands) were signed off on 2026-10-16, checked as the sinks were, in all four
facings standing free and against the wall: the three-drawer nightstand's top drawer slides out for "Pick an
outfit"; the blanket chest's lid lifts (`lid_open`) for "Take out a blanket" and the hamper's for "Sort the washing",
each shutting after its use; the bed-end bench is sat on. The nightstand with a lamp lights at night
(its shade glows, the top lit warm; coloured by day, the shade shows its glow's colour at night). The rest are still.
One fix: **the blanket chest's lid underside** flickered with the edge band below it (same plane), seen with the lid
up; the lid's underside is now 2 mm lower (a23e9c7). `nightstand`, `nightstandLamp`, `cedarChest`, `benchBedEnd`,
`laundryHamper` back in bedroom / storage; `nightstandModern`, `nightstandRound`, `nightstandDrawers` in bedroom /
nightstands.

## The wardrobes

The six BR3 things (Validate › Wardrobes) were signed off on 2026-10-16, checked as the sinks were, each standing
free in all four facings: the dresser's, tall chest's and low dresser's drawer slides out 30 cm for "Pick an outfit"
and the wardrobe's, armoire's (both doors, `doors_open`) and single wardrobe's (`door_open`) doors swing out onto the
rail of clothes, each shutting after its `hold`. One fix: **the wardrobe's "Body" coloured only its birch doors**, the
white carcass staying white. Body now colours the carcass and the doors are their own part, Doors (Body, Doors,
Handles; as the butcher block got its own Top in K4; the plan table updated) (0fa072f). All back in bedroom / storage.

## The dressing things

The four BR4 things (Validate › Dressing) were signed off on 2026-10-16, checked as the sinks were: the cheval mirror
(speech) and the standing mirror (Check the mirror) standing free in all four facings, the wall mirror on all four
walls (speech), the resident in front of the glass, which reflects the room; the dressing table's drawer slides out
(`drawer_open`) for "Get ready", the resident on the pouf facing the mirror, and shuts after its `hold`. Nothing needed
fixing. Seen in passing: a wall mirror on a wall cut down for the camera stays drawn at full height above it
(renderer). All back in bedroom / dressing.

## The patio

The eleven OU1 things (Validate › Patio) were signed off on 2026-10-17, checked as the sinks were, standing free in the garden: sat on (the park bench, the garden bench, the garden chair, the patio chair), relaxed in (the Adirondack chair), coffee and sitting in the sun at the bistro set, a picnic and a coffee at the picnic table, all in four facings. The patio chair's coffee (`needsTable`) was checked at the round patio table and at the teak table pushed up to it. The round and teak tables, the outdoor bar and the parasol are still, looked at in four facings; the parasol's `stir` is ambient (not on Low). Every colour part lands on the part its label names (the round table's Frame is its rim and pedestal, Top and Tiles the mosaic; the parasol's Edge is the valance). Nothing needed fixing. All back in outdoor / seating.

## The loungers

The five OU2 things (Validate › Loungers) were signed off on 2026-10-17, checked as the sinks were, each in four facings: the sun lounger's back lowers (`recline`) for sunbathe and doze and comes up after; the hammock's `sway` and the porch swing's and swing set's `swing` loop while used, ridden by the resident, and stop after; the hot tub's cover lifts (`cover_off`) and the water bubbles for a soak, and the resident puts the cover back on after (`closeAfter`: "Put the cover on" follows, the cover shuts). Lying was checked with shots 1, 4 and 8 s into the use (also on Low WebGL2 at night): they sit on first, then lie along the lounger and in the hammock's sling. Seen in passing: a swing or a sunbathe ends within seconds when the need it fills is full (fun 0.8 to 1 in about 8 s), and the next one is refused until it drops: that is the needs, not the model. All back in outdoor / seating, the swing set in outdoor / play.

## The grills

The three OU3 things (Validate › Grills) were signed off on 2026-10-17, checked as the sinks were, each in four facings: the charcoal grill's lid swings up (`lid_open`) for the whole of "Grill dinner" over the coals and food; the gas station's hood (`hood_up`) for cook and for the feast; the fire pit's flames grow for "Warm up by the fire", the resident on its front capstones, and the embers and flames glow at night (the content's light). Each shuts when the use ends. Nothing needed fixing. All back in outdoor / cooking.

## The yard

The nine OU4 things (Validate › Yard) were signed off on 2026-10-17, checked as the sinks were, each in four facings: the sand castle rises (`castle`) for a sandcastle; the bike tips onto its stand and its back wheel turns (`wheel_up`, `wheel_turn`) for a tune-up; the shed's door (`door_open`) for pottering, the dark inside showing; the greenhouse's door and roof vent for tend and harvest; the compost lid for 2 s; the raised bed's plants rustle every 2.4 s while tended, and a ripe bed (a state with the bed bought ripe: a new bed is planted, and harvest isn't offered) is harvested, the crop picked away and the bed left at picked; the washing comes down onto the line for "Hang out the washing" and stays out after (held open), and "Take the washing in" takes it in. The rain barrel and log store are still. Colour parts land where named (the log store's Shelter is the whole iron rack and roof, one material). Nothing needed fixing. The sandpit and bike rack back in outdoor / play, the rest in outdoor / yard.

## The garden things

The nine OU5 things (Validate › Garden Things) were signed off on 2026-10-17, checked as the sinks were, each in four facings: the koi comes up (`fish`) for "Feed the fish" and surfaces every 7 s while watching the fish, with the ambient ripples; the blue tit appears on the feeder (`bird`) for "Watch the birds"; the planter box is watered. The lamp post's panes and the string lights' bulbs glow at night, on WebGPU and on Low WebGL2; the birdhouse, dog house, sundial and stepping stones are still. Nothing needed fixing. Seen in passing: one stepping stone has a small dark nick on its edge (a dent in the boulder shape, a few pixels from the game camera). The pond, planter box, birdhouse, feeders, dog house, sundial and stepping stones back in outdoor / decor, the lamp post and string lights in outdoor / lights.

## The armchairs

The eleven L2 seats (Validate › Armchairs) were signed off on 2026-10-16, checked as the sinks were, each standing
free (23,16): sat on in all four facings, by day and at night, on WebGPU and WebGL2, Medium and Low, every paint slot
a colour of its own (buy colours, then the colour studio: labels, "In use"). `sink_in` plays and holds while sat on
(sit, and read on the leather and wingback chairs) and shuts when the resident gets up; the recliner's back and seat
tip back with her riding them and the footrest swings up for relax and doze; the rocking chair rocks (loop) with her
for relax and read and stops; the bean bag flattens for flop; the designer lounge chair is still (relax, doze). One
fix: the leather armchair's "Read a book" had no pose, so the reader stood beside the chair while its cushion sank;
it is `"pose": "sit"` now, as the wingback's (`furniture.json`, commit 6c23960; rechecked at two facings: she sits
and reads, `sink_in` plays and shuts). All back in living / seating.

## The coffee tables

The nine L3 tables (Validate › Coffee Tables) were signed off on 2026-10-16, checked as the sinks were, each standing
free (the console table against a wall, and free at the 180° facing), in all four facings: the coffee table's drawer
slides out with its magazine for "Flip through a magazine" and shuts after its `hold`; the glass and marble tables'
magazine use runs without a clip, as planned; the six plain tables looked at by day and night, WebGPU and Low WebGL,
coloured (the nesting tables' small table takes its own colour). Nothing needed fixing. All back in living / tables.

## The media

The eight L4 things (Validate › Media) were signed off on 2026-10-16, checked as the sinks were, against the north,
west and east walls and free at the 180° facing (the wall TV on the only window-free walls the house has: the living
room's west wall, and both sides of the bedroom/bathroom wall): watched, listened to, danced to and warmed up at.
The portable TV's ears turn and stretch, the home cinema's screen unrolls and rolls up, the low bench's TV rises and
sinks back, the stereo's tray slides out and back (its woofers pulse while listened and danced to), the boombox's
cassette door opens and shuts after its `hold`, the fireplace's flames grow for relax, with the resident on its
floor cushion facing the fire. At night every screen glows while on (the TV and wall TV while watched, the home
cinema's screen while down, the bench TV while up) and the fire lights the room. Nothing needed fixing. Seen in
passing: the low bench's "TV" colour lands on the set's thin bezel and the remote (the lift hood is the bench's), so
it shows little; the TV and home cinema's "Body" is the unit's carcase and "Trim" its top and doors, as the model
notes say. All back in living / media.

## The shelves

The nine L5 shelves (Validate › Shelves) were signed off on 2026-10-16, checked as the sinks were, against the
north, west and east walls and free at the 180° facing (the corner bookcase in the living room's north-west corner at
facings 0 and 1, its quarter-round end into the room): the five bookcases' red book tips out for read (and study)
and back at the end; the industrial shelving, cube shelf, ladder shelf and display cabinet looked at, coloured, by
day and night, WebGPU and Low WebGL. Nothing needed fixing. Seen in passing: standing free, the ladder shelf leans on
nothing (it is made to lean on a wall). All back in living / shelves.

## The stairs

The staircase (Validate › Stairs) was signed off on 2026-10-16. Bought in the check house in all four facings and
coloured (treads, railing; by day and night, WebGPU and Low WebGL). Climbed in new games' generated houses (the check
house has no floor above or below): residents walked up flights at facings 0 and 2 to the floor above and back down,
and down basement flights at facings 2 and 3 and back up; at facing 1 one started down (the goal tile was on the
flight, so she stopped on it). The stairwells seen from the floor above, with their railings. Nothing needed fixing.
Back in living (no group).

## The houseplants

The ten G1 houseplants (Validate › Houseplants) were signed off on 2026-10-17, checked as above in the living room:
watered in all four facings (the resident stands in front and pours), talked to (monstera), admired (orchid); the
bonsai pruned in all four facings, its stray twig shrinking away (`trim`, 0.79 s) and back after the use, and
watered. "Pot" colours the pot of each (the bonsai's tray, the fern's pot on its stand, the orchid's pot on its
pedestal). Nothing needed fixing. All back in garden / houseplants.

## The flowers

The fifteen G2 flowers (Validate › Flowers) were signed off on 2026-10-17, checked as above outdoors: watered
(tended: the raised bed, kneeling at its front) and smelled or admired; the meadow's white daisy shrinks away for
"Pick a wildflower" (`pick`) and is back after, in all four facings. "Bed" colours each mound, "Petals" the daffodils',
daisies', sunflowers' and poppies' petals, "Box" the raised bed's boards and the flower box. One fix: the potted
geraniums' "Flowers" coloured only two of their six clusters (the clusters cycled three reds and only the first
was the paint slot); all clusters are the painted material now (940c678). And the pansies' faces stood 9–15 cm above
their leaves with nothing under them (plain from the side and in the colour studio): each has a thin stem now
(3bfa080), same size and draw calls. Seen in passing: the marigolds' back poms sit a little above the leaves too,
but read as one mass of flowers; left as they are. All back in garden / flowers.

## The shrubs

The eleven G3 shrubs (Validate › Shrubs) were signed off on 2026-10-17, checked as above outdoors: trimmed (bush,
boxwood, topiary, hedge, azalea), pruned (roses), cut (lavender) in all four facings, each clip (`trim`, `cut`) playing
its 0.79 s and shutting after the use; watered or admired (hydrangea, azalea, fern, hosta), smelled (roses, lavender);
the ornamental grass looked at. Two fixes:
- **"Flowers" coloured a few flowers**: the hydrangea's three of eleven mopheads, the azalea's five of twenty, one
  lavender spike in three (shades cycled over the heads, only the first the paint slot). Every head is the painted
  material now, far versions too (940c678).
- **The topiary's trim showed nothing**: the sprigs it takes off stood inside the cone. They reach out of the cone's
  sides near the tip now, and go during "Clip the topiary" (2e6b1d3).
The rose bush's "Roses" colours every rose. Seen in passing: from the west (facing 3) the house hides most of the
grass at this spot (camera, not model). All back in garden / shrubs.

## The edibles

The five G4 edibles (Validate › Edibles) were signed off on 2026-10-17, checked as above outdoors: the four crops
tended (kneeling at the bed) in all four facings, bought new at the planted stage; then each grown at speed through
its stages, `grow` held at planted (1.5 s), growing (1.0) and ripe (0: tomatoes red, strawberries red, pumpkins
orange, the veg patch's lettuces and cabbages full), and harvested: the clip plays on to picked (0.5), the crop
shrinking away while the resident picks and the plants staying. The herb planter's basil shrinks away for "Pick fresh
herbs" (`harvest`) and is back after, in all four facings; watered. "Bed" colours each bed (the raised bed's and the
planter's boards). Nothing needed fixing. Seen in passing: for "Pick fresh herbs" the resident bends down to the
ground beside the waist-high planter (the shared `harvest` pose). Growing back from picked over the days was not
watched. All back in garden / edibles.

## The trees

The twelve G5 trees (Validate › Trees) were signed off on 2026-10-17, checked as above outdoors (the lemon tree also
close up): relaxed under in the shade (standing by the trunk), admired (cherry), smelled (magnolia), watered (lemon
tree); the apple and the lemon tree each lose their front fruit for "Pick" (`pick`), the Japanese maple its top sprigs
for "Shape the maple" (`trim`), in all four facings, shutting after the use. The lemon tree's "Pot" colours its pot.
Nothing needed fixing. Seen in passing: two tiles from the house the pine's and spruce's crowns (6 m and 5.6 m across
on a one-tile footprint) reach through the house's wall; the game lets a tree stand that close, and the placeholders
were as wide. All back in garden / trees.

## The bookworm pack

The reading nook's cushion settles for "Curl up", the library wall's ladder rolls for research and browsing, the
writer's bureau's flap drops (writer facing the wall, `seat.facing: back`), the tea trolley's teapot pours, the
fireside lounge's fire grows for both readings, the bedside stack's top book opens. One fix: **the fireside
lounge's hood hung on a flue that ended about 2 m up, in mid-air**; the flue now rises to the 2.8 m ceiling with a
collar there (`pack_bookworm.py`; 3 draws, 0 failed; validator 0/0; the placeholder stays 1.96 m tall, only the
fallback). Back in living, office, kitchen and bedroom (no group).

## The cheerful pack

The jukebox's record swings up and its label spins while dancing, the karaoke machine's microphone rises, the
mocktail cart's shaker tips, the sunrise lamp's sun rises for "Stretch", the party booth's curtain swings aside and
holds 1.5 s; the rainbow hammock sways (lain across, `BEDS` `across`) for swing and doze. Nothing needed fixing.
Back in living, hobbies, kitchen, outdoor and bedroom.

## The couch potato pack

The cinema wall's curtains part for both watches, the console's tray slides out, the recliner tips back with its
sitter riding it (recline and doze), the snack fridge's door opens for snack and takeout, the workout game's board
tilts, the lava lamp's wax rises for "Zone out" over its ambient `drift`. Seen in passing: the cinema wall and the
console put their screen at the footprint's front facing the seats, so from the room one sees the screen's back
(the placeholders' layout). Back in living, hobbies, kitchen, fitness and decor.

## The energetic pack

The bouldering wall is still (looked at in all facings, climbed); the spin bike's pedals turn while riding; the
trampoline's mat dips with the bouncer; the streetball hoop's net sways after each shot (1.4 s in, every 3.2 s); the
power blender's lid lifts and settles; the contrast shower's rain grows; the recovery pod's canopy lowers and lifts.
Nothing needed fixing. Back in fitness, outdoor, kitchen and bathroom.

## The example pack

The chef's stove opens its oven door and lights its burners for cook and practise (each run as a recipe from the
house's fridge), the whirlpool tub fills and bubbles for "Soak". Nothing needed fixing. Back in kitchen and wellness
(the pack's own category).

## The foodie pack

The herb garden grows a sprig, the deli pantry's doors open for nibble and cheese board (hold 2.5 s), the espresso
bar's stream grows and wand swings out, the cookbook lectern turns a page, the pizza oven's door lifts aside and
fire grows for "Bake a pizza" and the fire alone for "Warm up", the chef's island opens its drawer for the tasting
menu and is sat at for a bar bite. Nothing needed fixing. Back in decor, kitchen and outdoor.

## The gloomy pack

The record cabinet's lid lifts and its platter turns for listen and new album, the cello comes off its stand, the
journal desk's candle grows, the rain lamp's strands grow for "Watch the rain" over its ambient ripple, the window
seat's cushion sinks for brood and doze, the soaking tub fills. One fix: **the record cabinet's back was the Trim
(ink) colour outside**, navy from behind when it stands free; it is walnut outside now, ink inside the bays
(`pack_gloomy.py`; 4 draws as before, 0 failed; validator 0/0). Seen in passing: dozing on the window seat (2x1,
`lie`) lies front to back, head at the window, legs off the front edge: the renderer's `BEDS` table has no entry for
it (the hammocks lie `across`). Back in living, hobbies, office, decor and bathroom.

## The hot-headed pack

The heavy bag swings while pounded, the smash bin's lid flips up (hold 1 s), the drum kit's cymbals shake while
drumming, the cool-down cushion sinks for both meditations, the grudge desk opens its drawer, the cold plunge's
cover slides off. Nothing needed fixing. Back in fitness, hobbies, decor, office and bathroom.

## The kind pack

The baking counter's oven opens for pie and treats (recipes from the house's fridge), the giving jar takes a coin,
the knitting rocker rocks with its sitter for knit and rock, the tea table's pot pours, the songbird feeder's lid
opens for refill and birds come for "Watch the birds", the hearth settee's cushions sink. Seen in passing: the quilt
nap on the settee (2x1, `lie`) lies front to back with the legs off the front, as on the window seat. Back in
kitchen, decor, hobbies, living and outdoor.

## The loner pack

The quiet pod's hood swings half closed for retreat and read, the shortwave radio's dial turns for "Tune" and its
antenna rises then the dial for "Airwaves", the puzzle grows for both uses, the bonsai's twig shrinks for "Shape" and
the mist plays for "Tend", the sauna's door opens and shuts behind the sitter (hold 1.5 s), the delivery hatch
opens for order and treat. Nothing needed fixing. Back in living, office, hobbies, bathroom and kitchen.

## The nature lover pack

The veggie bed's plants sway every 2.4 s while tended (over the growth pose), the potting bench grows a seedling
for "Pot seedlings" and a bouquet for "Tie a bouquet", the bird feeder's bird comes, the garden hammock sways
(lain across), the cedar shower's stream grows, the moss terrarium's lid lifts, the glasshouse's door and vent open
for unwind and orchids. Nothing needed fixing. Not seen in a picture: "Pick something fresh" (`when: ripe`); the
crop takes two game days to ripen and the resident fell asleep each time it did; its `harvest` clip was seen
playing (sampled to 0.62 of 0.79 s at speed) and shutting when the ripe crop was picked in the first run. Back in
garden / edibles (veggie bed), hobbies, outdoor and decor.

## The neat pack

The steam mop leaves its dock into the resident's hands, the laundry centre's washer door swings for laundry and
fold, the rain shower's rain grows, the organizer desk's bin drawer slides out for sort and ledger, the robot vacuum
rolls out and back (hold 2 s), the entry console's key tray slides out. Seen in passing: at the organizer desk the
writer's hands come through the modesty panel to the room side (the shared `write` pose reaches low and far; the
candlelit journal desk shows the hands under its apron too); the entry console's leaning mirror and hook board (and
the slob spritz station's round mirror) stand on the wall face, so they hang in the air when the piece stands free.
Back in kitchen, bathroom, office, living and decor.

## The outgoing pack

The party bar's tap pulls for mix and soda (hold 2 s), the sectional's cushions sink, the karaoke stage's mic stand
rises for practise and perform, the video-call ring light swings up, the dance floor's tiles pulse while dancing,
a photo grows on the photo wall. Seen in passing: the party bar's "Back bar" colours the low back cabinet, not the
bottle shelves above it. Back in kitchen, living, hobbies, office and decor.

## The romantic pack

The candlelit table's flames grow for dinner and dessert, the heart tub fills with petals, the letter bureau's flap
drops, the vanity's drawer opens (hold 2 s), the gramophone's arm swings onto the record and the platter spins for
listen and waltz, the rose-arbour swing swings with its sitter and a bloom shrinks for "Tend the roses". Nothing
needed fixing. Back in kitchen, bathroom, bedroom, living, office and outdoor.

## The slob pack

The microwave cart's door opens and shuts for nuke and invent, the beanbag squashes for flop and snooze, the snack
nightstand's drawer opens (eaten sitting on its padded top), the spritz bottle tips, the arcade's joystick moves
while playing, the gaming recliner tips back for the marathon and its cup holder swings out for a soda. Nothing
needed fixing. Back in kitchen, living, bedroom, bathroom and hobbies.

## The playground

The eight CM1 things (Validate › Playground) were signed off on 2026-10-16, checked as the sinks were, each
standing in the west yard of the check house (10,18; outdoor things can't be bought indoors) and looked at
from one camera outside in all four facings: the slide gone down, the frame climbed, the ping-pong table
played at; the seesaw (`seesaw`), spring rider (`rock`) and merry-go-round (`spin`) loops go round with the
rider on them and finish their round when the use ends; the hoop's net swishes after each shot (`swish`, every
3.2 s); a pawn slides for chess (`move`). Every part coloured in buy (two colour sets) and the colour studio.
Nothing needed fixing. Seen in passing: the climbing frame is "climbed" with the standing `exercise` pose by
the net. All back in community / play.

## The street furniture

The ten CM2 things (Validate › Street) were signed off on 2026-10-16, checked as the sinks were, outdoors at
10,18 (the notice board indoors at 23,16): sat on (bench, shelter), pigeons fed, the timetable read, the
drinking fountain's water arc (`water_on`), the post box's flap and the phone box's folding door (`door_open`,
held 1.5 s) for their uses, the statue admired, a wish made at the town fountain and its rim sat on, with its
water running all the time (`water_run`, ambient: seen on WebGPU Medium, off on Low as designed). The
street lamp's lantern lights at night (WebGPU and Low WebGL). Also seen on the `town-things` lot. Nothing
needed fixing. Seen in passing: at the drinking fountain the resident drinks a step away with the shared
`drink` pose (a cup to the mouth), not bent over the spout; at the phone box the caller stands outside on the
front tile while the door opens and shuts (the plan says "shuts behind them"). All back in community / street.

## The food stalls and vending

The five CM3 things (Validate › Stalls) were signed off on 2026-10-16, checked as the sinks were (the carts
and market stall outdoors at 10,18, the machines indoors at 23,16): the ice cream cart's lid for an ice cream
(held 2 s), the hot dog stand's lid, a snack and a can dropping in the machines, the market stall browsed and
fruit bought. Each clip plays its full length and shuts in every run. Nothing needed fixing. All back in
community / food.

## The shops and café

The five CM4 things (Validate › Shops) were signed off on 2026-10-16, checked as the sinks were, indoors at
23,16: the shop counter's till drawer (held 1.5 s) and the bakery case's back door (held 2 s) for their uses,
the grocery shelves browsed and a sandwich taken, the café chair sat on in all four facings and coffee and
cake had at the café table (two placements; refused at a chair standing alone, `needsTable`). Nothing needed
fixing. Seen in passing: the shop counter's "Counter" colours its front panels and "Trim" the walnut top and
frame. All back in community / shops.

## The farm

The eight CM5 things (Validate › Farm) were signed off on 2026-10-16, checked as the sinks were, outdoors (the
barn at 8,16, the rest at 10,18), all still: looked at in all four facings by day, at night and on Low WebGL.
The scarecrow sways (`sway`, ambient: off on Low as designed); the coop's hatch slides shut at night
(`close`, held from dusk) and is open by day. Also seen together on the `town-things` lot, near and far.
Nothing needed fixing. All back in community / farm.

## The showers

The four B2 showers (Validate › Showers) were signed off on 2026-10-17. The bathroom agent checked them as the
sinks were: each bought in all four facings (23,14 north, 21,18 west, 22,23 south, 28,17 east), showered in ("Take
a shower"), by day and at night, on WebGPU and WebGL2, on Medium and Low, every part coloured. `water_on` (0.5 s) grows
the rain from the head as the use starts, holds it for the whole use and shuts it as it ends; the stall's rain falls
slanted from its wall head onto the resident. They were held back only because the colour studio drew nothing for
the shower stall, at rest or in use. One fix, in the studio (5104118): the stage framed a moving part by its rest
box's corners, posed. The stall's water is grown from nothing (flat at rest) and turned for the slant, so its box had
corners off the flat, which the pose threw tens of metres away; the camera ended up 80 m back, beyond its far plane.
The stage now frames a moving part by its posed vertices, so any turned part that grows from nothing frames right.
Rechecked in the studio: all four showers draw at rest and In use, with every part coloured (the stall: Tray, Rail,
Head & tap, Curtain, Tiles), the stall's rain slanted onto the tray; the fridge, dishwasher, rain lamp and lava lamp
still frame and open as before; the stall in play unchanged. The plan's colour-part cells now list what the studio
lists (Tiles, Walls, Wall, the stall's Rail and Curtain were missing). All back in bathroom / bathing.

## The home gym

The five F1 things (Validate › Fitness) were signed off on 2026-10-17. They were checked on 2026-10-16/17 as the
sinks were (the fitness agent's notes, `$S/fitness/signoff.md`): each bought on free floor in all four facings and
against the north wall, and used there: the treadmill's `belt_run` loops under the jogging runner, the bike's
`pedals_turn` with the rider's feet on the pedals, the yoga mat unrolls for yoga and rolls up again, the weight
bench's `bar_off` and the dumbbell rack's `pair_off`, by day and at night, WebGPU and `renderer=webgl` Low, every
paint slot a colour of its own, and the colour studio (labels, colour landing, "In use"). They were held for one
fault: with "Weights" coloured, the pair in the resident's hands kept the model's dark grey. Two fixes:
- **Held parts in the object's colours** (`characters.ts`, commit 10b8dc1). A held part's prop key now carries the
  object's player colours (`<model>#<part>@<colours>`) and the part is built painted, as the object is. Rechecked:
  the rack coloured (Rack terracotta, Weights blue), lifted in all four facings at 23,16 (both dumbbells blue in the
  hands, the rack's six others blue), at night on WebGPU and on Low WebGL2 at night; the ukulele, acoustic and
  electric guitar played with their instrument coloured green (each held green, the stand or amp teal); the rack
  and the ukulele uncoloured still hold dark grey dumbbells and a wooden ukulele.
- **The bench's bar no longer vanishes for the lie-down** (`fitness.py` `BAR_DELAY`, manifest only: the `.glb` is
  byte-identical; same commit). The game draws the bar in the hands once the lifter has sat down (the `sitDown`
  clip less its last 0.35 s, 0.98 s), so `bar_off` waits 0.8 s and ends as the bar shows in the hands. Rechecked in
  two facings with pictures 0.25–2 s into the lift: the bar is on the hooks while she sits down, shrinks away at
  0.8–1.0 s, and is in her hands from about 1.0 s on, never missing.
All back in fitness (no group).

## The baths

The three B3 baths (Validate › Baths) were signed off on 2026-10-17. The bathroom agent had checked them in full
(four facings, `tap_on` and `fill` playing, holding and draining, colours, night, Low), but held them because the
bather sat across the tub facing its long side, legs out through its side. One fix (`characters.ts`, commit
1a0ad51): `SEATS` gives the baths `along` (the seat faced a quarter turn from the object's front, `fwd` then along
it) and `legsOut`; the back at the end without the taps (the bathtub's and clawfoot's right end, the freestanding
bath's left), the hips 0.38–0.42 m from the middle, the legs out towards the taps; the arms rest on the long rims.
The freestanding bath's seat moved from its content into `SEATS` (`SEATS` overrode it anyway). Rechecked: each bath
used in all four facings (23,16 at 0, 2 and 3 facings; 21,18 at 1), seen from the front and from above: the legs in
the tub and under the water, the back at the end, nothing through the panel or the rim; by day, at night on WebGPU
and on Low WebGL2 at night; coloured by day and at night (Tub blue, Panel/Feet terracotta, Taps green, Towel
purple: each lands where it did); `tap_on` and `fill` play (fill 2.42 s, held for the bath) and drain as the bath
ends (logged every 150 ms: `fill` falls from 2.4 to nothing in about 2 s, then shut). The colour studio was not
re-run (nothing in it changed). All back in bathroom / bathing.

## The sofas

The twelve things of Validate › Sofas (the ten L1 sofas and sectionals, the sofa bed, the wall bed) were signed off on
2026-10-17. The living agent had checked them in full (`$S/living/signoff.md`, `$S/held.md`) and held them for one
fault: pulling the sofa bed out and folding it away, the resident stood inside the pulled-out mattress (her spot is
the middle of the tile in front; the mattress reaches 1.52 m from the sofa's middle). One fix (`characters.ts`,
commit 6c98e40): `STAND_BACK` 1 m for the sofa bed, as the dishwashers' 0.15 m, so she stands half a metre past the
mattress's foot and reaches to it. Rechecked in all four facings (23,16 at 0, 2, 3; 21,18 at 1), 0.25 s and 1.3 s
into "Pull out the bed" and into "Fold the bed away", from the side and from above: clear of the mattress
throughout, her hands at its foot when it is out. Doze still lies her on the bed; the wall bed's fold down and up
and a sofa's relax are as before (their code paths take no `STAND_BACK`). All back in living / seating (the sofas,
the sofa bed, the wall bed) and living / sectionals.

## The books and pots

The seven TT3 things (Validate › Books And Pots) were signed off on 2026-10-17: the stack of books, the vase of
flowers, the bud vase, the succulent, the cactus and the small potted plant were checked on 2026-10-16 as the
kitchenware were. They needed nothing. The books and bookends were held back for one label: "Books" coloured only the
two books that share cover `book_a`. They have eight books in five cover colours, and now list each colour as the
stack of books does: Bookends, then Book 1 to 5, left to right in the order each colour first shows. Book 1 to 3
colour two books each (`tabletop.py`; the change is only in the manifest, so the GLB is unchanged). This was
rechecked on the square table at 23,16 in all four facings by day, with every slot a colour of its own (r0 and r2:
all eight books and the bookends take their slot's colour), at night on WebGPU and on Low WebGL2, and in the colour
studio (labels Bookends, Book 1 to 5, each landing on its covers). The bookStack, booksUpright and their row go back
to tabletop / books, the plants to tabletop / plants.

## The lazy pack

The six P10 things (Validate › Lazy) were signed off on 2026-10-17. Four needed nothing (`$S/packs/signoff.md`).
One fix: **the quilt table's "Doze under the quilt" laid the resident on the table top, and the nap pod's "Take a
power nap" tipped her over in the egg chair with her legs out.** Both are now sitting naps (`"pose": "sit"`, anim
`nap`, set up like the sofas' and recliners' dozes) in `packs/lazy.json`. Both were rechecked in all four facings,
with pictures 1, 4 and 8 s into the use, then with colours by day, at night on WebGPU and on Low WebGL2. Each time she
sits and naps, and the quilt and hood clips play and shut. Close views from the side, back and front show the
following. At the quilt table she sits on the left pouf (slot 0) with its backrest behind her, arms on the top and
legs under the quilt, the same as "Tuck in under the quilt". In the pod she sits in the egg chair under the lowered
hood, feet on the floor. Seen in passing: at the quilt table her shoes poke out at the foot of the quilt's front, as
they did for tuckIn, which was accepted before. All go back to bedroom, living, kitchen, fitness and office (no group).

## The garden decor

The six garden decor models (Validate › Garden Decor) were signed off on 2026-10-17. They were checked in play
before (`$S/outdoor/signoff.md`) and were held back because none had colour parts. Each now has 1 to 3, every label
naming exactly what it colours (`garden_decor.py`; manifest only, GLBs unchanged):
- gnome: Hat, Tunic (with its sleeves), Belt. Its boots sit wholly inside the tunic bell, so the label names only the belt.
- bird bath: Bowl & column, Plinth.
- fountain: Fountain (basins, column, finial), Plinth.
- lantern: Lantern, Base. The lit fire box stays as made.
- wheelbarrow: Tray, Handles, Legs & tyre.
- boulders: Large boulder, Middle boulder, Small boulder (one stone each; the moss stays as made).
Rechecked in the north yard (19,9 / 20,10) in all four facings with their uses (pat, watch the birds, listen to
the water, water the planter; the lantern and boulders are still). Colours were checked by day in two facings, with
test colours far from the defaults, then at night on WebGPU, on Low WebGL2 at night (the lantern's fire box lit as
before) and in the colour studio. Every colour lands on the part its label names. The gnome, bird bath, fountain,
wheelbarrow and boulders go back to outdoor / decor, the lantern to outdoor / lights.

## The acoustic guitar and ukulele (hobbies, already signed off)

Their colour went on the back and sides, not the soundboard. Each now has a third slot, "Top", for the
soundboard. The body's slot is relabelled to say what it colours: the guitar's "Body" (back and sides), the
ukulele's "Body & neck" (`hobbies.py`; manifest only). Rechecked on the stand from the front and the back (Top,
body and stand each their own colour), by day, at night on WebGPU and on Low WebGL2, and played (strum and practise,
in all four facings): `off_stand` plays and shuts each time. In the hands the instrument shows its made colours, as
expected until held parts take the object's colours. In the colour studio, labels Body/Top/Stand and Body &
neck/Top/Stand each land where named, and "In use" shows the empty stand.

## After the sign-off: what it saw, fixed (2026-10-17)

The things the sign-off saw but didn't change were fixed after it, by four agents, each checked in play in four
facings, by day and on Low WebGL at night (logs and pictures in the session's scratchpad), then together on the
merged branch:
- **Lamp light** (`house.ts`, content `light.at`): the arc lamp's light was 28–32 cm from its shade, the table lamp's
  12 cm and the nightstand lamp's 26 cm; each now sits at the shade in every facing, against a wall and free. The
  other 26 lights were looked at and left (the wall lights' pool on the wall reads better as it is).
- **Cut walls** (`BabylonRenderer.ts`): a wall thing was hidden only when its own wall stood between it and the
  camera, so with the wall dropped from the other side it floated over the stub. Each wall thing now knows, from when
  it is placed, which views drop its wall, and hides with it. Checked on inner and outer walls from all four sides,
  walls up and down; floor things against a wall stay.
- **Screens** (`pack_couchPotato.py`, content seats `facing: "back"`): the cinema wall and the game console are built
  turned round, the screen on the wall face and the seats in front; two sitters face the picture, the curtains part,
  the tray slides out. Placeholders turned to match; 3 draws each.
- **Naps on the window seat and hearth settee** (`BEDS` `across`): a lone napper lies along the seat, the head at the
  pillow (seat) or the lantern end (settee); with a second user each keeps their own half, as before.
- **Teddy**: the hugger crouches in, arms round its neck, cheek on its head, while it squishes (`hugArms` shared with
  the social hug, unchanged there).
- **Herb planter**: "Pick fresh herbs" is a standing reach into the plants (`pick`), the basil shrinking under the
  hands; other crops still kneel.
- **Desks**: of the 13 seated writing desks, four with their chair built in 0.3 m from the top had the hands through
  their front; they use nearer hands (`sitHandsNear`) and seats of their own. The other nine look as before.
- **Drinking fountain**: the drinker steps up and bends over the bowl, hands on the rim, mouth at the arc
  (`fountainDrink<cm>` in `clips/street.ts`, one per height).
- **Phone box**: the door opens outwards, the caller steps inside onto the plinth, it shuts behind them, and it opens
  again to let them out (motions `out`: a use's clip opens again for N s at its end).
- **Climbing frame**: climbed up its rope net, hands and feet on the ropes, the hands on the deck's edge at the top
  (`climb<cm>`, an 8 s loop); a second climber takes the other half of the net.

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
- **Kids** (`04-kids.md`, KD1–KD3): signed off 2026-10-17.
- **Bedroom** (`03-bedroom.md`, BR1–BR4): signed off 2026-10-17.
- **Garden** (`13-garden.md`, G1–G5): signed off 2026-10-17.
- **Community** (`14-community.md`, CM1–CM5): signed off 2026-10-17.
- **Fitness** (`07-fitness.md`, F1): signed off 2026-10-17.
- **Bathroom** (`02-bathroom.md`, B1–B5): signed off 2026-10-17.
- **Living** (`05-living.md`, L1–L6): signed off 2026-10-17.
- **Tabletop** (`10-tabletop.md`, TT1–TT5): signed off 2026-10-17.
- **Outdoor** (`12-outdoor.md`, OU1–OU5 and the garden decor): signed off 2026-10-17.
- **Personality packs** (`15-personality-packs.md`, P1–P16): signed off 2026-10-17.

## Lessons for the tooling (2026-10-17)

The sign-off of 2026-10-16/17 ran 14 category agents at once, one worktree each. Their logs (steps, timings,
failures, ideas) are summed up here; the same faults cost nearly every agent time, so fix them before the next
large check.

**Setup (every agent hit these)**
- Worktree vite: the git-ignored wasm packages must be *copied* into the worktree (a symlink is refused with
  403 and the sim, font and environment light silently fail: "lights only" pictures). Vite needs a wrapper
  config that adds the main checkout's `node_modules` to `server.fs.allow` **and** its own `cacheDir`: the shared
  `node_modules/.vite` is re-optimised by every agent's server (ENOTEMPTY at start, reloads mid-run, lost
  environment maps). Better still, the repo's `vite.config.ts` could do both when `node_modules` is a symlink.
- Setup must fail loudly: fetch the wasm through the new server (expect 200) and load the check house once
  (`__sims()` answers, no "using lights only") before any run; the runner should record HTTP ≥ 400 and console
  errors per job.
- The agent sandbox refuses `bash script.sh`, `cd … &&` chains, `$VAR` in command or option position and loops
  calling bash: ship setup as plain commands, with absolute paths, and a node launcher that sets
  PORT/PLAYWRIGHT/CHROME itself.
- Debug states are shared through the link: give each worktree a copy, or a run can change the check house.
- One CPU core per agent (1 page, no browser and Blender at once), never more agents than cores.

**The runner (one shared version with what the agents each re-added)**
- Reuse one page for many jobs (sell, buy the next facing): a fresh page cost 10–75 s under load; reuse made
  jobs 5–15 s (outdoor: 85 s → 10 s; static shooters ~4× faster).
- Wait for every loading text ("Moving in", "Getting ready", "Drawing portraits", "Building the world",
  "Warming up") before buying and again before each shot; time out `goto` at 90 s; count the cap from page ready
  and from when the use is sent.
- Walk fast and slow down only within a few tiles of the object (never above 10×: 30× skips short uses); the
  old "heading" test never fired because `s.object` is empty while walking.
- Sample clips in the page every animation frame, not from node every 100 ms (short clips were missed and
  "max" read low under load).
- Ignore always-running clips (`idle`, `growth`, `night`, clock hands) for at-it / picture / shut; know held-open
  things (`motions.open`: beds, washing line, hot tub) and `then=<use>` to close them.
- Frame the object, not the resident: clear the follow (`__controls.follow(null)`), aim at the footprint's
  middle with a radius from its size (trees ~17 m, small things closer than the 5 m house limit), re-aim just
  before the shot (the intro tween and the game's auto-watch override it); a back view; walls kept up for wall
  things or an uncut wall chosen.
- Shoot sit and lie uses again ~4 s in (the full-clip moment catches the sit-down); print the resident's needs (a
  use ends when its need is full: toilets, swings, sunbathing), with `drain=<need>:<level>` and `cancel=<s>`.
- Record what each shot shows (object found, its facing, colours set) and flag mismatches: a late buy once put
  the wrong facing's object in a plausible picture.
- Options: `sim=` (any resident), `host=` (a table bought first for surface things), `pre=`, `sellAny=`, camera
  `radius=`/`alpha=`/`beta=`, `SKIP=1` resume; `sell=ceilingLight`/`chandelier` by day (they hang in front of the
  23,14 shots).
- Generate the jobs from content + manifest (main use = first with a clip, extra runs only for uses whose
  pose/anim/clip differ, outdoor things to the yard, radius from the footprint): 776 jobs for 94 items, no hand
  editing. A report from the JSONs per group (clips, max/duration, shut, errors) instead of greps.

**Colours and the studio**
- Pick test colours far from each part's default (read it from the GLB), or run a second rotated colour run as
  standard: a part whose default is already the first test colour proves nothing.
- One studio script that clicks a distinct swatch per part, shoots, turns the turntable and takes "In use":
  labels, colour landing and the open clip in one picture. Wait for the stage to draw (poll the canvas) instead
  of a fixed delay; loop-only models have no "In use".
- Contact sheets from a node script (some sandboxes refuse bash), 7 play shots plus the studio in one row per item,
  with a label tile per shot.

**States, spots and gates**
- A spot table per check house: wall spots with and without windows, plain 2-wide walls, the yard, the r2
  spots that work (several agents probed for them).
- Debug states for what play can't stage quickly: a ripe crop or a `__debugSet` for growth and needs, a two-storey
  house with a basement (stairs), a two-resident house (double beds, bunks), `family-night` (children at night).
- The clips bench at t=0 / t=99 finds clips that show nothing (the topiary) faster than play runs: run it first.
- `check_glb.py` could flag coplanar faces of different materials in one part (three z-fight fixes: duvet end,
  chest lid, bread bin) and parts scaled to a sliver at rest that are still visible (the kids' hairline).
- `gamekit` smooth-shades cylinders without a bevel, so flat caps can shade like domes (the bread bin).

## The end

Done 2026-10-17: the last groups left Validate; the `validate` category and `icon.category.validate` are
removed and `docs/design/objects.md` says every object's model is built in Blender. (`tools/content/validate.py`
needs the category back, with an empty `groups`, before it can put new items into Validate.)

- **Last group:** when the last group leaves Validate, remove the `validate` category and `icon.category.validate`
  (`web/public/assets/manifest.json`).
- **Docs:** update `docs/design/objects.md`. "From 0.3.0 a new object's model is built in Blender" becomes "every
  object's model is built in Blender", and the "Migrate or keep" section changes to match.
- **Placeholders:** stay as the fallback (keys unchanged; saves and content need no migration).

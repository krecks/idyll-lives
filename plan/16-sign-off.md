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

## The end

- **Last group:** when the last group leaves Validate, remove the `validate` category and `icon.category.validate`
  (`web/public/assets/manifest.json`).
- **Docs:** update `docs/design/objects.md`. "From 0.3.0 a new object's model is built in Blender" becomes "every
  object's model is built in Blender", and the "Migrate or keep" section changes to match.
- **Placeholders:** stay as the fallback (keys unchanged; saves and content need no migration).

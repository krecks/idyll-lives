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

The seven sinks (Validate › Sinks) are the first group. Before commit 1caf5ec they were:
- `sink`, `sinkPedestal`, `vanityDouble`, `sinkVessel`, `sinkConsole`: bathroom / sinks;
- `farmhouseSink`, `counterSinkDouble`: kitchen / counters.

## Per category

A category is signed off when all its tasks are: its file's tables are all built, back, and committed. Mark each
task done in the category file (strike through its heading, with the commit).

## The end

- **Last group:** when the last group leaves Validate, remove the `validate` category and `icon.category.validate`
  (`web/public/assets/manifest.json`).
- **Docs:** update `docs/design/objects.md`. "From 0.3.0 a new object's model is built in Blender" becomes "every
  object's model is built in Blender", and the "Migrate or keep" section changes to match.
- **Placeholders:** stay as the fallback (keys unchanged; saves and content need no migration).

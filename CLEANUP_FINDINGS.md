# Cleanup findings

Review of the codebase after the module-split series on `chore/code-cleanup`. Scope: dead code,
unused exports and imports, stale comments, naming, leftover debug output, duplicated logic and
test coverage. Every finding below is reproducible with the command quoted beside it.

## 1. A blanket `allow(unused_imports)` hides dead imports

`crates/tools/src/checkglb/against/mod.rs:29` carries `#![allow(unused_imports)]`, a module-wide
lint suppression added while the file was split. It silences unused imports in the module and in
every child module (`compare.rs`, `differ.rs`, `findings.rs`, `main.rs`, `measures.rs`, `tests.rs`),
so a dead import can sit there indefinitely. Remove the allow and the imports it was hiding.

Check: `rg -n 'allow\(unused_imports\)' crates/tools/src/checkglb/against/mod.rs`.

## 2. Exports nothing imports in `web/src`

Found by collecting every `export` in `web/src` and `web/tests` and counting identifier uses
repo-wide (`rg -n '\bNAME\b'`). Nine names are declared and never used, in the module or anywhere
else (including the HTML entry pages and `tools/`):

| File:line | Name |
|---|---|
| `web/src/assets/build.ts:19` | `BUILD_ID` (only `__BUILD_ID__` itself is read, by `QUERY` in the same file) |
| `web/src/game/macro.ts:2829` | `worldExits` (and the `roadEnds` import it alone uses) |
| `web/src/game/porch.ts:16` | `PorchMode` |
| `web/src/game/surfaces.ts:137` | `hostOf` |
| `web/src/game/town.ts:121` | `LOT_SIZES` |
| `web/src/render/quality.ts:253` | `GRAPHICS_PRESETS` |
| `web/src/render/shared/cars.ts:79` | `CAR_AGES` |
| `web/src/render/shared/characters/pose.ts:473` | `bonePosition` |
| `web/src/ui/townedit/ghosts.ts:24` | `NO_GHOSTS` |

Check: `rg -n '\bworldExits\b' web tools` (and the other names) returns the declaration only.
`countTokens` (`web/src/llm/generate.ts:423`) looks unused the same way but is imported by
`tools/llm/check.mjs`, so it stays.

## 3. Doc comments point at files the splits removed

The split series moved code into directories but left comments naming the old files, so a reader
following them finds nothing. A sweep of every comment line in `crates/` that names a `*.rs` file
no longer in the tree found 77 lines in 53 files, all from the same splits:

| Old name | Now |
|---|---|
| `body.rs`, `face.rs`, `hair.rs`, `clothes.rs` | `crates/tools/src/characters/{body,face,hair,clothes}/` |
| `conversation.rs` | `crates/sim-core/src/conversation/` |
| `save.rs` | `crates/sim-core/src/save/` |
| `taxi.rs` | `crates/sim-core/src/taxi/` |
| `traffic.rs` | `crates/sim-core/src/traffic/` |
| `travel.rs` | `crates/sim-core/src/travel/` |
| `view.rs` | `crates/sim-core/src/view/` |
| `things.rs` | `crates/sound/src/things/` |
| `math.rs` | `crates/artkit/src/math/` |
| `crates/tools/src/characters/clothes.rs` (`web/src/render/shared/characters/data.ts:100`) | `crates/tools/src/characters/clothes/` |

`ear.rs`, `head.rs`, `write.rs` and the rest still exist and are left as they are. So are the
`Split from `x.rs`` headers at the top of each new module: they say where the code came from, which
is still true, and they name no file a reader is meant to open.

Check: `ls crates/tools/src/characters/face.rs crates/sim-core/src/view.rs` (both gone).

## 4. ~~A stale save-format number in `land.rs`~~ — not a finding

`crates/sim-core/src/land.rs:24` says the land and the water mask are saved "still 39" while
`crates/sim-core/src/save/mod.rs:185` has `pub const SAVE_VERSION: u32 = 41;`. That reads like a
stale number, but it is not: `crates/sim-core/src/save/mod.rs:124` says "Still 39" of the same
fields, and `crates/sim-core/src/save/mod.rs:130` explains why ("`meta.land` has its land drawn
from the landscape, as before, so nothing changes version"). Both comments mean "this part of the
file has not changed since format 39", which is true of format 41.

Check: `rg -n 'still 39|SAVE_VERSION' crates/sim-core/src/save/mod.rs`.

## Reviewed and left alone

- `PLAN.md`, `CHANGELOG.md` and the release notes name files that have since been split
  (`traffic.rs`, `bus.rs`, `body.rs`, …). They are a historical record of what each release held,
  not navigation aids, and rewriting them would falsify that record.
- `#[allow(dead_code)]` on `crates/tools/src/checkglb/built/read.rs:31` (`pub about: String`) and
  on the two `crates/art` script helpers: the field is read by serde and never used, and the art
  helpers are deliberate spare ways of building a model.
- `println!` in `crates/art` and `crates/art/tests`: the CLI's output, not debug leftovers.
- `console.error` / `console.warn` in `web/src`: the game's error reporting, all prefixed by area.
- No `TODO`, `FIXME`, `XXX` or `HACK` markers in `crates/` or `web/src` (the only `XXXX` hits are
  JSON `\uXXXX` escape handling).

## Deferred

- `crates/sim-core/src/privacy.rs:707` and `crates/sim-core/src/social/defs.rs:230` still say
  `conversation.rs`. Both files are pinned parts (`tools/pins/pins.json`: `sim-privacy`,
  `sim-social-defs`), so the gate holds their sha256 and a comment change fails `pins` until an
  integrator re-accepts them with `cargo pins --update <name> --by <who> --reason <why>`. Two
  comments are not worth re-accepting accepted work for; they go back with that update.

# outside (art-outside)

The group's crate; each family is a folder in `src/<family>/` with its own `NOTES.md`, and imports only `artkit`
and this group's `src/common.rs`, never another family (`tests/registry.rs` checks it).

## Families

yard, outdoor, playground, farm, fields, street_furniture, stalls, shops, civic, civic_works, water_edge

## Shared code

`src/common.rs`: `F` (the game frame), `rod` and `bent` (round bars through game points). `lib.rs` holds `content_object`, which the families' content-fact tests read the content with.

## Who uses it

`outdoor`, `yard` and `civic` use `F`, `rod`, `bent` (`outdoor` and `yard` re-export them from their own `common.rs`; `civic` re-exports `rod`).

## Tests

Unit tests sit in `outdoor`, `civic`, `farm`, `playground` and `shops` (content facts, through `crate::content_object`) and in `lib.rs`.

# packs (art-packs)

The group's crate; each family is a folder in `src/<family>/` with its own `NOTES.md`, and imports only `artkit`
and this group's `src/common.rs`, never another family (`tests/registry.rs` checks it).

## Families

pack_bookworm, pack_cheerful, pack_couch_potato (`pack_couchPotato`), pack_energetic, pack_example, pack_foodie, pack_gloomy, pack_hot_headed (`pack_hotHeaded`), pack_kind, pack_lazy, pack_loner, pack_nature_lover (`pack_natureLover`), pack_neat, pack_outgoing, pack_romantic, pack_slob

## Shared code

`src/common.rs`: `_box` (a rounded-cornered box lofted upright), `fluted` (a ring of points in and out round).

## Who uses it

`pack_slob`, `pack_neat` (dock, laundry) use `_box`; `pack_hot_headed` (calm) and `pack_kind` (home) use `fluted`.

## Tests

Unit tests sit in `pack_example/common.rs` and `pack_slob/common.rs`.

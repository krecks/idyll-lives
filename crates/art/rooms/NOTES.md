# rooms (art-rooms)

The group's crate; each family is a folder in `src/<family>/` with its own `NOTES.md`, and imports only `artkit`
and this group's `src/common.rs`, never another family (`tests/registry.rs` checks it).

## Families

living, dining, office, decor, lighting, bedroom, bedroom_storage, bedroom_dressing

## Shared code

None: the group has no `common.rs`. Each family keeps its own helpers; what two families would share goes in `src/common.rs` (create it and add `mod common;` to `lib.rs`) or, when other groups want it too, in `artkit::kit`.

## Who uses it

None.

## Tests

No unit tests of its own; `crates/art/tests` and `cargo art --all --check` cover the families.

# things (art-things)

The group's crate; each family is a folder in `src/<family>/` with its own `NOTES.md`, and imports only `artkit`
and this group's `src/common.rs`, never another family (`tests/registry.rs` checks it).

## Families

hobbies, kids, held, tabletop, media, fitness, birds

## Shared code

`src/common.rs`: `P`, a game point (x, height, depth) of floats.

## Who uses it

`hobbies`, `kids`, `fitness` and `media` re-export `P` from their own `common.rs`.

## Tests

A unit test sits in `birds/mod.rs`.

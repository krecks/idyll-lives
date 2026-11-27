# nature (art-nature)

The group's crate; each family is a folder in `src/<family>/` with its own `NOTES.md`, and imports only `artkit`
and this group's `src/common.rs`, never another family (`tests/registry.rs` checks it).

## Families

trees, plants, garden_things, garden_decor, ground, street

## Shared code

`src/common.rs`: `mesh` (a part from points and faces, normals bent to the sky) and `sky_normals`, used by `ground` (flowers, markers) and `garden_things` (pond); `cut`, `cut_own`, `lump` and `stone` (boulders roughened, split and cut flat under the ground), used by `ground`'s rock and `garden_decor`'s stonework.

## Who uses it

`ground` (rock, flowers, markers), `garden_things` (pond) and `garden_decor` (garden) use `common`; none of the six families uses another.

## Tests

Unit tests sit beside the code in `ground/common.rs`, `ground/rock.rs`, `garden_decor/common.rs`, `garden_things/common.rs` and `street/common.rs`; the models themselves are held byte for byte by `cargo art --all --check`.

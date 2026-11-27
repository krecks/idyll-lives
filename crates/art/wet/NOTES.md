# wet (art-wet)

The group's crate; each family is a folder in `src/<family>/` with its own `NOTES.md`, and imports only `artkit`
and this group's `src/common.rs`, never another family (`tests/registry.rs` checks it).

## Families

kitchen, kitchen_cooking, kitchen_counters, bathroom, laundry

## Shared code

`src/common.rs`: `pull` (with the `pull!` macro in `lib.rs`) and `Pull`, the bar handle; `shaker`, the shaker front; `water` and `stream`, the taps' water; `COUNTER66`, the counters' drawn depths (`GameFrame::with_depths`).

## Who uses it

`kitchen` and `kitchen_cooking` use `pull!` and `shaker`; `kitchen`, `bathroom` use `water`; `kitchen_cooking` (the coffee machine) and `bathroom` (baths) use `stream`; `kitchen_counters` and `kitchen_cooking` use `COUNTER66`.

## Tests

Unit tests sit in `kitchen_cooking/common.rs` and `bathroom/bath_parts.rs`.

# Objects: what is still open

What is known not to be right yet. What a single object does is on its category page ([README.md](README.md)); this
page collects what cuts across them.

## Still open

### Residents
- **Feet slide at game speeds 4 and 5** (48–81 % of the way: 45 and 90 m/s, faster than any gait steps without
  strobing; A-residents-03, G-25): won't fix, looked at again on 2026-10-29. At 60 frames a second a resident moves
  0.75 m a frame at speed 4 and 1.5 m at speed 5 (the renderer snaps past 1.6 m); a running step is about 1 m and the
  quickest a leg can alternate without flickering is every other frame. The longest walk on a lot, 31 m across the
  review garden, lasts 0.85 s at speed 4, and the following camera keeps the walker in frame for only part of it, so
  the feet can't be read. Speed 4 is also what quiet hours skip at, with the household asleep or out. The clip
  check's baseline (`tools/characters/clip-baseline.json`) is empty: it reports nothing to keep.

### Content and sim
- **Parts with no use, left decorative:** the executive desk's drawer (I-office-02, won't fix) — the last one. The
  laundry basket has no lid and none is planned: it holds clean washing heaped above its
  rim (bathroom review, 2026-10-21); the lidded one is the bedroom's `laundryHamper`, whose lid lifts for sorting.
- **Uses end once every need they raise is full**, by design (I-systems-02); it makes clips easy to miss in checks.

## Notes on the sim

- **`Relationships::partner_of`'s `debug_assert` is kept.** It re-scans the row on every call in test builds, but
  the whole sim-core suite times the same with and without it (towns are ~60 Sims). It catches `.partners` changed
  through `get_mut` instead of `set_partners`.

# Residents: generator, checks and benches

The residents are made in code. Nothing here downloads anything.

- `cargo characters` (Rust, in `crates/tools/src/characters`, no Python): the bodies, faces, hair,
  beards and clothes of both families at every life stage. `cargo characters build` writes
  `web/public/assets/characters` (`rig.json` and the `.bin` files) and then checks it.
- `web/src/render/shared/characters/clips/`: the animation clips, in TypeScript, baked when the game
  loads the character set (no clip file is shipped).
- `check_clips.ts`: checks those clips against the built rig.

The format, the conventions and the clip list are in [docs/characters-format.md](../../docs/characters-format.md).

## Build and check

From the repo root:

```
cargo characters build [out]     # about 6 s; writes the set, then checks it
cargo characters check [set dir] # the set against the contract only
cargo characters diff <a> <b>    # two sets against each other, within the quantisation
bun tools/characters/check_clips.ts [rig.json] [--body male|female|all] [--stage <stage>[,…]|all] [--update-baseline]
cd web && bun run check:clips    # check_clips on both bodies at every life stage, as CI runs it
```

- The build is deterministic (seeded noise only). After a code change that shouldn't touch the
  art, the files in `web/public/assets/characters` come out byte-identical (`git status` stays
  clean); commit them together with the generator change when they do change.
- `cargo characters check` fails on any contract violation: bone names and order, rest frames,
  vertex and triangle budgets, weights, winding, stage topology, morph channels, garment ids and
  cuts, stage heights, the seat drop, eyes in their sockets and uncovered, hair clear of the head,
  bounded morphs.
- `check_clips.ts` fails on NaNs, seams in loops, sliding feet, lengths more than 15% off the
  runtime's, seated clips that leave the seat or feet off the floor, and a crooked `lie`, on the
  bodies (`--body`, default `male`) and life stages (`--stage`, default `youngAdult,baby`) asked
  for; what a stage reproduces is in its header and in docs/characters-format.md (Checking clips).
  The failures known today are in `clip-baseline.json`: they print as `known`, one that is fixed
  prints as `fixed` (take it out, or `--update-baseline`), and only a new one fails.

## Where to change what

The generator's `.rs` files are in `crates/tools/src/characters/`.

| To change | Edit |
|---|---|
| proportions per family and life stage, bone positions | `rig.rs` (`proportions`) |
| the body's shape, skin weights, fields and tone | `body.rs`, `cage.rs`, `subdiv.rs` |
| the head, the face's place | `head.rs` (the head interface the face and hair build on) |
| eyes, lids, brows, lips, teeth, tongue, the expression and viseme shapes | `face.rs` |
| hairstyles and the beard | `hair.rs`, built from `prims.rs` |
| garment cuts and shells | `clothes.rs`, `garments.rs` |
| clips | `clips/*.ts` (`dsl.ts` for the format, `ik.ts` for arm and leg placement) |
| how residents use clips, seats, beds, carried things | `web/src/render/shared/characters/` (`residents.ts`, `seats.ts`, `props.ts`) |
| colours, garments per family, pyjamas | `web/src/render/shared/characters/outfit.ts` |
| shading (skin, painted clothes, eyes, hair) | `web/src/render/pc/characterMaterial.ts`, `web/src/render/pc/shaders/character.ts` |
| moods and micro-expressions | `web/src/render/shared/characters/face.ts` |

## Benches

Run the dev server without rebuilding the Rust packages: in `web/`, `bunx vite` (or `bun run dev`),
then open:

- `/characters.html`: a grid of residents to check bodies, faces, hair, clothes and skins.
  `?stage=all`, `?hair=all`, `?tops=all`, `?skins=all`, `?builds=all`, `?face=moods|visemes`,
  `?clip=walk&t=0.4`, `?view=side|back`, `?closeup=1`, `?family=female`.
- `/poses.html`: residents doing things at real furniture (gardening, beds, seats, eating, desks,
  instruments, cooking, carrying, exercise, conversations). `?only=sleep,chop`, `?seats=sofa,toilet`,
  `?clip=walk` (everyone plays it), `?clip=sitDown&strip=8` (a filmstrip on a man, a woman and a
  child), `?focus=3`.

Both take `?manual` (time moves only through `__bench.advance(s)` / `__poses.advance(s)`) and set
`window.__ready` when drawn, for screenshots from a headless browser. Clip files hot-reload: saving
one re-bakes the clips in the open bench.

In the game itself, `?debug` adds `__characters` (`force = 'walk'` plays a clip on everyone,
`faceDebug`, `stress(n, x, z)` adds `n` animated residents for profiling; `updateMs` is the CPU time
of the residents' update).

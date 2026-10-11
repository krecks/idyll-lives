# Residents: generator, checks and benches

The residents are made in code. Nothing here downloads anything.

- `prim/`: the bodies, faces, hair, beards and clothes of both families at every life stage
  (Python 3 with numpy and scipy). `build.py` writes `web/public/assets/characters`
  (`rig.json` and the `.bin` files) and then runs `check.py`.
- `web/src/render/babylon/characters/clips/`: the animation clips, in TypeScript, baked when the game
  loads the character set (no clip file is shipped).
- `check_clips.ts`: checks those clips against the built rig.

The format, the conventions and the clip list are in [docs/characters-format.md](../../docs/characters-format.md).

## Build and check

From the repo root:

```
python3 -B tools/characters/prim/build.py              # about 25 s; writes the set, then checks it
python3 -B tools/characters/prim/check.py [set dir]    # the set against the contract only
node --experimental-strip-types tools/characters/check_clips.ts [rig.json]
```

- Use `-B`: the repo tracks some `.pyc` files, and running without it rewrites them.
- The build is deterministic (seeded noise only). After a code change that shouldn't touch the
  art, the files in `web/public/assets/characters` come out byte-identical (`git status` stays
  clean); commit them together with the generator change when they do change.
- `check.py` fails on any contract violation: bone names and order, rest frames, vertex and triangle
  budgets, weights, winding, stage topology, morph channels, garment ids and cuts, stage heights,
  the seat drop, eyes in their sockets and uncovered, hair clear of the head, bounded morphs.
- `check_clips.ts` fails on NaNs, seams in loops, sliding feet, lengths more than 15% off the
  runtime's, seated clips that leave the seat, and a crooked `lie`.

## Where to change what

| To change | Edit |
|---|---|
| proportions per family and life stage, bone positions | `prim/rig.py` (`PROPORTIONS`) |
| the body's shape, skin weights, fields and tone | `prim/body.py`, `prim/cage.py`, `prim/subdiv.py` |
| the head, the face's place | `prim/head.py` (the head interface the face and hair build on) |
| eyes, lids, brows, lips, teeth, tongue, the expression and viseme shapes | `prim/face.py` |
| hairstyles and the beard | `prim/hair.py`, built from `prim/prims.py` |
| garment cuts and shells | `prim/clothes.py`, `prim/garments.py` |
| clips | `clips/*.ts` (`dsl.ts` for the format, `ik.ts` for arm and leg placement) |
| how residents use clips, seats, beds, carried things | `web/src/render/babylon/characters.ts` |
| colours, garments per family, pyjamas | `web/src/render/babylon/characters/outfit.ts` |
| shading (skin, painted clothes, eyes, hair) | `web/src/render/babylon/characters/material.ts` |
| moods and micro-expressions | `web/src/render/babylon/characters/face.ts` |

## Benches

Run the dev server without rebuilding the Rust packages: in `web/`, `npx vite` (or `pnpm dev`),
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

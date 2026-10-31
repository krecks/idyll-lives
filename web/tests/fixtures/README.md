# Babylon fixtures

Several tests check the game's own code against Babylon.js 9.30. The checks cover:

- matrix and quaternion maths;
- colour curves and vignette;
- mesh builders, primitive parts and mirror faces;
- the CPU particle system;
- bird and resident matrices.

Until Stage 11 of the PlayCanvas port, these tests ran Babylon directly. Babylon is no longer a dependency, so its outputs now live in `babylon/*.json`, together with the inputs they were made from. `babylon.ts` reads them.

## How the fixtures were made

`record-babylon.ts` wrote them. It ran against the last commit that still had the Babylon renderer, **f15b839b**. That commit has `web/src/render/babylon/` and `@babylonjs/core` 9.30.0 installed. The recording is deterministic.

## Recording them again

Running the script again needs a Babylon reference checkout of that commit with its own `node_modules`:

```sh
git worktree add bench-results/babylon-f15b839b f15b839b
(cd bench-results/babylon-f15b839b/web && bun install)
cd web && bun tests/fixtures/record-babylon.ts --ref ../bench-results/babylon-f15b839b/web
```

An earlier commit won't do: the Babylon renderer gained `particleSystem`, `partMesh` and `mirrorSurface` during the port, and f15b839b is the last commit that has them.

The script takes its inputs from this tree: the look's settings, the particle settings and the bursts. The tests check that the game's particle settings still match the recorded ones. After a deliberate change to those settings, record again.

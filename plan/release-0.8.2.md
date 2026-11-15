# Idyll Lives 0.8.2 — Release Plan

- **Branch:** `release/0.8.2`, cut from `main` at `43398e3c` (v0.8.1) on 2026-11-15; to be merged into `main` by pull request; tag `v0.8.2` on the merge
- **Base:** v0.8.1 at `43398e3c`
- **Release type:** a patch: one fix to 0.8.1's sun shadows on WebGPU; save format 39, as 0.8.1

## 1. What ships

The owner's report (F8, 2026-11-15, WebGPU, High): "the shadows look not correct". The shadows of houses, trees and fences lay across the lawn, away from what cast them; the residents' were right.

- **Cause:** 0.8.1's kept picture of the still casters (#98, `web/src/render/pc/sunShadows.ts`) was a render target without an origin. PlayCanvas makes the sun's map bottom-up (`RENDERTARGET_ORIGIN_BOTTOM`), which WebGPU draws with Y flipped, so the picture was drawn the other way up and copied into the map mirrored. WebGL2 flips neither, which is why #98's check (WebGL2 and WebGPU alike, on a scene whose picture it compared only with itself) didn't see it.
- **Fix:** the picture takes the map's origin, and is made again if they differ. `shadowSquare.test.ts` checks that the picture is drawn the same way up as the map.

## 2. Checks

- The F8 report headless at its camera pose, against `?diag=wholeshadows` (every caster drawn every time): WebGPU 0.00 % of pixels more than 24 levels off (1.90 % before the fix); WebGL2 0.10 % against the WebGPU reference.
- `bun test --isolate tests/shadowSquare.test.ts` (the new test fails without the fix), `bun run check` 0 errors.
- CI on the pull request, the dry run on `main`, and the tag's run, as in the README's **Releasing**.

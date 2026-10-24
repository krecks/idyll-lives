**Play it in your browser:** https://krecks.github.io/idyll-lives/

The game draws with **PlayCanvas** instead of Babylon.js.

- **Renderer:** PlayCanvas 2.23 on WebGPU, with the WebGL2 fallback, replaces Babylon.js 9.30. On the game's own scene PlayCanvas took about a third of Babylon's main-thread time in Chrome and about a ninth in Safari's engine, where Babylon fell below 60 fps (`docs/design/performance.md`). The pictures were matched to Babylon's stage by stage: the same light, shadows, soft look, colour grading, residents, mirrors, build tools, previews and town overview; the camera moves as it did.
- **The menus' town:** standing still, it isn't drawn again until something in it changes, on WebGPU and WebGL2 alike (Babylon's snapshot rendering only spared the processor, on WebGPU).
- **Under the hood:** the engine-neutral parts of the renderer (`web/src/render/shared`) are tested by `bun run test`, and `bun run check:chunks` checks the PlayCanvas internals the renderer patches; both run in CI. Parts of the renderer are ported from Babylon.js (Apache-2.0) and say so; its licence and notice ship in `THIRD-PARTY-LICENSES.txt`. The decoders Babylon needed are gone; PlayCanvas's Basis transcoder decodes the textures.
- **No stalls at the start of play:** the game compiles its shaders behind the loading screen, so the first seconds of play, the first night, a mirror and Build mode no longer stutter while they compile (only a model bought for the first time compiles its own).

### Also
- **Saves:** unchanged (format 32): 0.3.2 and 0.4.0 open each other's saves.
- **Known:** on Low with WebGL2 the graphics processor does up to 15 % more work than it did with Babylon; everywhere else it does the same or less.
- **The sky's light** is made straight from the CC0 HDRI it came from (Poly Haven's Belfast Open Field, `cargo envconv`), instead of from Babylon's packed copy of it; the lighting is unchanged.

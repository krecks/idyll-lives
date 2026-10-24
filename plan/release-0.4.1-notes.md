**Play it in your browser:** https://krecks.github.io/idyll-lives/

Two fixes to the renderer and a tool.

- **Changing graphics settings between games:** the renderer that's replaced now lets its WebGL context go at once, and the new one draws on a fresh canvas. Before, each change left a context behind until the browser cleared it, and after enough of them the browser could drop the one in use. Replacing the renderer on WebGL also no longer throws an error for every model it had loaded.
- **`cargo envconv`:** the sky's HDRI download stops as soon as an answer is longer than the pinned file, instead of writing it all to disk before the checksum refuses it.

### Also
- **Saves:** unchanged (format 32): 0.4.0 and 0.4.1 open each other's saves.
- **Known:** on Low with WebGL2 the graphics processor still does more work than it did with Babylon, most in the wide view (+16–18 %). It was profiled for this release: not the FXAA, but nature's share, which needs switches of its own to go further (`docs/design/performance.md`, Low on WebGL2).

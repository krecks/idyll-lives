/**
 * The part of PlayCanvas the game uses: everything, from its release build by path. The package's
 * own entry (`playcanvas`) has a `development` export condition that Vite's dev server picks,
 * the debug build, which wraps every WebGPU draw in validation error scopes (a GPU-process round
 * trip each: a few frames a second on a full scene). Rendering code imports PlayCanvas from here
 * (eslint forbids the bare package).
 */

export * from 'playcanvas/build/playcanvas/src/index.js';

/**
 * The page's current app: the default of `new Entity()` (its `app` argument), made the newest
 * app by PlayCanvas's `AppBase` constructor. A second app on the page (the previews') puts the
 * game's back (`device.ts` `createSideApp`).
 */
export { getApplication, setApplication } from 'playcanvas/build/playcanvas/src/framework/globals.js';

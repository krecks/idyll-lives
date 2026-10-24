/**
 * The PlayCanvas renderer's own layers: what no camera draws
 * but the sun's shadow map does. PlayCanvas gathers a light's casters from every layer of the
 * composition the light is on, whether a camera renders that layer or not.
 */

import * as pc from './engine';

const shadowLayers = new WeakMap<pc.AppBase, pc.Layer>();

/**
 * The layer of draws that only cast the sun's shadow (the woods near the town drawn whole for the
 * shadow map while the game draws them in groups, the house's lid over its rooms, the low preset's
 * merged object shadows), made once per app. The sun lights it (`SunShadows` puts it on the sun's
 * light; `sun` does too, for a caller that has the sun).
 */
export function shadowOnlyLayer(app: pc.AppBase, sun?: pc.Entity): pc.Layer {
  let layer = shadowLayers.get(app);
  if (!layer) {
    layer = new pc.Layer({ name: 'ShadowOnly' });
    app.scene.layers.push(layer);
    shadowLayers.set(app, layer);
  }
  const light = sun?.light;
  if (light && !light.layers.includes(layer.id)) light.layers = [...light.layers, layer.id];
  return layer;
}

const effectLayers = new WeakMap<pc.AppBase, pc.Layer>();
const overLayers = new WeakMap<pc.AppBase, pc.Layer>();

/**
 * The build effects' place in a frame (after the opaque draws, before the blended ones): build
 * mode's effects (`buildFx.ts`), blended only, so the layer is in the composition only as blended,
 * just before the world's blended draws. Made once per app; the `camera` draws it.
 */
export function effectsLayer(app: pc.AppBase, camera: pc.CameraComponent): pc.Layer {
  let layer = effectLayers.get(app);
  if (!layer) {
    layer = new pc.Layer({ name: 'BuildEffects' });
    const layers = app.scene.layers;
    layers.insertTransparent(layer, layers.getTransparentIndex(layers.getLayerById(pc.LAYERID_WORLD)!));
    effectLayers.set(app, layer);
  }
  if (!camera.layers.includes(layer.id)) camera.layers = [...camera.layers, layer.id];
  return layer;
}

/**
 * The over layer (build mode's thing in hand, `placementGhost.ts`): drawn after the world, the
 * depth cleared before it, so it shows over what stands in front of it, its own parts still in
 * front of one another. In the scene's picture (before the image processing: just before the
 * immediate layer), lit by every light that lights the world (the lights of a layer go into its
 * shaders: the same as the world's, the same shaders, and the same light clusters). Made once per
 * app; the `camera` draws it.
 *
 * (The depth is cleared with the device's clear as the layer starts, on frames it has something to
 * draw: under the game's `CameraFrame` a layer's own `clearDepthBuffer` isn't honoured past the
 * first layer.)
 */
export function overLayer(app: pc.AppBase, camera: pc.CameraComponent): pc.Layer {
  let layer = overLayers.get(app);
  if (!layer) {
    const made = (layer = new pc.Layer({ name: 'BuildOver' }));
    const layers = app.scene.layers;
    layers.insert(made, layers.getOpaqueIndex(layers.getLayerById(pc.LAYERID_IMMEDIATE)!));
    for (const light of app.root.findComponents('light') as pc.LightComponent[]) {
      if (light.layers.includes(pc.LAYERID_WORLD) && !light.layers.includes(made.id)) light.layers = [...light.layers, made.id];
    }
    const clear = { depth: 1, flags: pc.CLEARFLAG_DEPTH };
    // SAFETY: both of PlayCanvas's devices have `clear(options)` (the renderer's own clears go through
    // it: `Renderer.clear`), inside the pass being drawn; the base class doesn't declare it.
    const device = app.graphicsDevice as unknown as { clear(options: typeof clear): void };
    app.scene.on(pc.EVENT_PRERENDER_LAYER, (_camera: pc.CameraComponent, drawn: pc.Layer, transparent: boolean) => {
      if (drawn === made && !transparent && made.meshInstances.length) device.clear(clear);
    });
    overLayers.set(app, made);
  }
  if (!camera.layers.includes(layer.id)) camera.layers = [...camera.layers, layer.id];
  return layer;
}

/**
 * The PlayCanvas renderer's own layers (`shared/layers.ts` has Babylon's masks): what no camera draws
 * but the sun's shadow map does. PlayCanvas gathers a light's casters from every layer of the
 * composition the light is on, whether a camera renders that layer or not.
 */

import * as pc from './engine';

const shadowLayers = new WeakMap<pc.AppBase, pc.Layer>();

/**
 * The layer of draws that only cast the sun's shadow (Babylon's `LAYER_SHADOW_ONLY`: the woods near
 * the town drawn whole for the shadow map while the game draws them in groups), made once per app
 * and put on `sun`'s light.
 */
export function shadowOnlyLayer(app: pc.AppBase, sun: pc.Entity): pc.Layer {
  let layer = shadowLayers.get(app);
  if (!layer) {
    layer = new pc.Layer({ name: 'ShadowOnly' });
    app.scene.layers.push(layer);
    shadowLayers.set(app, layer);
  }
  const light = sun.light;
  if (light && !light.layers.includes(layer.id)) light.layers = [...light.layers, layer.id];
  return layer;
}

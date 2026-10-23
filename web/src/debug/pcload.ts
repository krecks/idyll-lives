/**
 * Dev-only check of the PlayCanvas loaders (`render/pc/loaders.ts`, /pcload.html): loads the glTF
 * `?url=` (a production copy: meshopt geometry) and the KTX2 `?ktx=` (Basis ETC1S) through them and
 * draws both, the model beside a plane wearing the texture. `window.__result` says what loaded
 * (meshes, vertices, the texture's size and format); `__ready` once drawn.
 */

import * as pc from '../render/pc/engine';
import { createApp, createDevice } from '../render/pc/device';
import { containerOptions, setupDecoders } from '../render/pc/loaders';

const params = new URLSearchParams(location.search);
const canvas = document.getElementById('c') as HTMLCanvasElement;
canvas.width = canvas.clientWidth;
canvas.height = canvas.clientHeight;
const device = await createDevice(canvas, params.get('renderer') === 'webgl');
const app = createApp(canvas, device);
setupDecoders();
const result: Record<string, unknown> = { backend: device.isWebGPU ? 'webgpu' : 'webgl2' };

const camera = new pc.Entity('camera');
camera.addComponent('camera', { clearColor: new pc.Color(0.6, 0.75, 0.88) });
camera.setPosition(2.2, 1.6, 2.6);
camera.lookAt(0, 0.5, 0);
app.root.addChild(camera);
const light = new pc.Entity('sun');
light.addComponent('light', { type: 'directional', intensity: 1.5 });
light.setEulerAngles(50, 30, 0);
app.root.addChild(light);
app.scene.ambientLight = new pc.Color(0.4, 0.4, 0.45);

function load(url: string, type: 'container' | 'texture', options?: object): Promise<pc.Asset> {
  return new Promise((resolve, reject) => {
    const asset = new pc.Asset(url, type, { url }, undefined, options as never);
    asset.once('load', () => resolve(asset));
    asset.once('error', (err: unknown) => reject(new Error(`${url}: ${err}`)));
    app.assets.add(asset);
    app.assets.load(asset);
  });
}

try {
  const url = params.get('url');
  if (url) {
    const container = (await load(url, 'container', containerOptions)).resource as pc.ContainerResource;
    const entity = container.instantiateRenderEntity();
    app.root.addChild(entity);
    const instances = (entity.findComponents('render') as pc.RenderComponent[]).flatMap((r) => r.meshInstances);
    result.meshes = instances.length;
    result.vertices = instances.reduce((n, mi) => n + (mi.mesh?.vertexBuffer?.numVertices ?? 0), 0);
  }
  const ktx = params.get('ktx');
  if (ktx) {
    const texture = (await load(ktx, 'texture')).resource as pc.Texture;
    result.texture = { width: texture.width, height: texture.height, format: texture.format, mipmaps: texture.mipmaps };
    const m = new pc.StandardMaterial();
    m.diffuseMap = texture;
    m.update();
    const plane = new pc.Entity('plane');
    plane.addComponent('render', { type: 'plane', material: m });
    plane.setLocalPosition(-1.2, 0, 0);
    app.root.addChild(plane);
  }
} catch (err) {
  result.error = String(err);
}
let frames = 0;
const tick = () => {
  app.update(1 / 60);
  app.render();
  if (++frames === 10) Object.assign(window, { __result: result, __ready: true });
  requestAnimationFrame(tick);
};
requestAnimationFrame(tick);

/** What item pictures and the colour studio's stage share: the product-shot scene, its lights and shade. */

import {
  Color3,
  Color4,
  DirectionalLight,
  DynamicTexture,
  HemisphericLight,
  ImageProcessingConfiguration,
  MeshBuilder,
  Scene,
  StandardMaterial,
  Vector3,
  type AbstractEngine,
  type Mesh,
} from '../babylon/core';
import type { AssetRegistry } from '../../assets/registry';
import { MaterialLibrary } from '../babylon/materials';
import type { StandIn } from '../babylon/models';
import { installNature } from '../babylon/nature';

/** What a thing whose file fails is pictured as (`BuildOptions.standIn`): a plain box on its footprint. */
export const PICTURE_STAND_IN: StandIn = { layer: 'floor', radius: 0.12 };

/**
 * The product-shot scene item pictures (and the colour studio's stage) are drawn in: transparent
 * behind, tone-mapped like the game, lit by the sky's reflections and `lights`.
 */
export async function studioScene(engine: AbstractEngine, assets: AssetRegistry): Promise<Scene> {
  const scene = new Scene(engine);
  scene.clearColor = new Color4(0, 0, 0, 0);
  scene.skipPointerMovePicking = true;
  // Foliage cards turn to face the camera here too (and sway a little in the turntable).
  installNature(scene);
  const ip = scene.imageProcessingConfiguration;
  ip.toneMappingEnabled = true;
  ip.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
  ip.exposure = 1.2;
  ip.contrast = 1.1;
  const lib = MaterialLibrary.for(scene).setAssets(assets);
  const env = assets.get('environment.sky', 'image');
  if (env) await lib.loadEnvironment(env.url);
  lib.setEnvironmentIntensity(0.75);
  lights(scene);
  return scene;
}

/** The 8 corners of the meshes' combined bounding box (model space). */
export function boxCorners(meshes: Mesh[]): Vector3[] {
  const min = new Vector3(Infinity, Infinity, Infinity);
  const max = new Vector3(-Infinity, -Infinity, -Infinity);
  for (const mesh of meshes) {
    if (!mesh.getTotalVertices()) continue;
    mesh.computeWorldMatrix(true);
    const box = mesh.getBoundingInfo().boundingBox;
    min.minimizeInPlace(box.minimumWorld);
    max.maximizeInPlace(box.maximumWorld);
  }
  if (!Number.isFinite(min.x)) return [];
  const out: Vector3[] = [];
  for (const x of [min.x, max.x]) for (const y of [min.y, max.y]) for (const z of [min.z, max.z]) out.push(new Vector3(x, y, z));
  return out;
}

/** A soft round shade under the model, so it stands on something. */
export function contactShadow(scene: Scene): Mesh {
  const n = 128;
  const tex = new DynamicTexture('itemShadow', n, scene, true);
  const ctx = tex.getContext() as CanvasRenderingContext2D;
  const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
  g.addColorStop(0, 'rgba(0,0,0,0.42)');
  g.addColorStop(0.55, 'rgba(0,0,0,0.2)');
  g.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, n, n);
  tex.update();
  tex.hasAlpha = true;
  const mat = new StandardMaterial('itemShadow', scene);
  mat.disableLighting = true;
  mat.diffuseColor = Color3.Black();
  mat.specularColor = Color3.Black();
  mat.emissiveColor = Color3.Black();
  mat.opacityTexture = tex;
  mat.disableDepthWrite = true;
  const mesh = MeshBuilder.CreateGround('itemShadow', { width: 1, height: 1 }, scene);
  mesh.material = mat;
  mesh.position.y = 0.004;
  mesh.isPickable = false;
  return mesh;
}

/** Product-shot lighting: a warm key from the upper front, a cool fill, a rim and the sky. */
function lights(scene: Scene): void {
  const key = new DirectionalLight('itemKey', new Vector3(0.45, -0.75, -0.5), scene);
  key.intensity = 2.6;
  key.diffuse = new Color3(1, 0.96, 0.9);
  const fill = new DirectionalLight('itemFill', new Vector3(-0.8, -0.3, -0.4), scene);
  fill.intensity = 0.9;
  fill.diffuse = new Color3(0.86, 0.91, 1);
  fill.specular = new Color3(0.1, 0.1, 0.12);
  const rim = new DirectionalLight('itemRim', new Vector3(-0.2, -0.4, 0.9), scene);
  rim.intensity = 1.6;
  const sky = new HemisphericLight('itemSky', new Vector3(0, 1, 0), scene);
  sky.intensity = 0.45;
  sky.groundColor = new Color3(0.5, 0.46, 0.42);
  sky.specular = Color3.Black();
}

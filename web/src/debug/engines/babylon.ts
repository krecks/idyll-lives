/**
 * The engine comparison's Babylon.js side (`bench.ts`): the shared scene (`scene.ts`) with the
 * game's Babylon and glTF loader, set up as the game sets it up where that matters to the cost
 * of a frame (pointer picking off, materials frozen, clustered lamps, the game's post-processing
 * pipeline), and nothing of the game's own shading. No snapshot rendering: the game uses it only
 * for the town behind the menus, not in play (`BabylonRenderer.scheduleSnapshot`).
 */

import {
  Bone,
  ClusteredLightContainer,
  Color3,
  Color4,
  ColorCurves,
  DefaultRenderingPipeline,
  DirectionalLight,
  Engine,
  FreeCamera,
  HemisphericLight,
  ImageProcessingConfiguration,
  ImportMeshAsync,
  Matrix,
  Mesh,
  MeshBuilder,
  PBRMaterial,
  Quaternion,
  Scene,
  SceneInstrumentation,
  ShadowGenerator,
  Skeleton,
  SpotLight,
  TransformNode,
  Vector3,
  VertexData,
  WebGPUEngine,
  type AbstractEngine,
  type AbstractMesh,
} from '../../render/babylon/core';
import '../../render/babylon/loaders';
import { LOOK_COLOR, SWUNG, loadBodies, newPose, poseAt, walks, type PartLook } from './residents';
import {
  AMBIENT,
  CAMERA,
  CLEAR,
  FLOOR,
  GARDEN,
  LAMP,
  MODELS,
  SUN,
  TREES,
  cameraAt,
  floorSize,
  gardenPlacements,
  lampPositions,
  orbitRadius,
  placements,
  treePlacements,
  type Driver,
  type DriverOptions,
  type Placement,
} from './scene';

/** Loads each model once, as a template (its root, the `__root__` of the glTF). */
function loadTemplates(urls: string[], scene: Scene): Promise<AbstractMesh[]> {
  return Promise.all(
    urls.map(async (url) => {
      const result = await ImportMeshAsync(url, scene);
      for (const group of result.animationGroups) group.dispose();
      return result.meshes[0];
    }),
  );
}

/** Each placement as a copy of its template (its own meshes sharing the geometry and materials); the templates stay hidden. */
function placeCopies(templates: AbstractMesh[], all: Placement[], drawn: Mesh[]): void {
  for (const root of templates) root.setEnabled(false);
  for (const p of all) {
    const copy = templates[p.model].instantiateHierarchy(null, { doNotInstantiate: true });
    if (!copy) continue;
    copy.position.set(p.x, 0, p.z);
    copy.rotationQuaternion = null;
    copy.rotation.y = p.rotY;
    copy.setEnabled(true);
    for (const mesh of copy.getChildMeshes(false)) if (mesh instanceof Mesh && mesh.getTotalVertices() > 0) drawn.push(mesh);
  }
}

/** Every placement of a template as a matrix in its meshes' thin-instance buffers. */
function placeInstances(templates: AbstractMesh[], all: Placement[], drawn: Mesh[]): void {
  for (const [model, root] of templates.entries()) {
    const mine = all.filter((p) => p.model === model);
    for (const mesh of root.getChildMeshes(false)) {
      if (!(mesh instanceof Mesh) || mesh.getTotalVertices() === 0) continue;
      // A thin instance draws at `instance × world`: placed at P in the world, that is `world × P × world⁻¹`.
      const world = mesh.computeWorldMatrix(true).clone();
      const inverse = world.clone().invert();
      const matrices = new Float32Array(mine.length * 16);
      mine.forEach((p, i) => world.multiply(Matrix.RotationY(p.rotY).multiply(Matrix.Translation(p.x, 0, p.z))).multiply(inverse).copyToArray(matrices, i * 16));
      mesh.thinInstanceSetBuffer('matrix', matrices, 16, true);
      mesh.thinInstanceRefreshBoundingInfo();
      drawn.push(mesh);
    }
  }
}

/** The residents: a template mesh per body part, and per resident a skeleton and copies of its family's parts. */
async function addResidents(count: number, scene: Scene, drawn: Mesh[]): Promise<(t: number) => void> {
  const bodies = await loadBodies();
  const materials = new Map<PartLook, PBRMaterial>();
  const material = (look: PartLook) => {
    let mat = materials.get(look);
    if (!mat) {
      mat = new PBRMaterial(`resident.${look}`, scene);
      mat.albedoColor = new Color3(...LOOK_COLOR[look]);
      mat.metallic = 0;
      mat.roughness = 0.7;
      materials.set(look, mat);
    }
    return mat;
  };
  const templates = bodies.map((body) =>
    body.parts.map((part) => {
      const mesh = new Mesh(`${body.family}.${part.name}`, scene);
      const data = new VertexData();
      data.positions = part.positions;
      data.normals = part.normals;
      data.indices = part.indices;
      data.matricesIndices = Float32Array.from(part.joints);
      data.matricesWeights = part.weights;
      data.applyToMesh(mesh);
      mesh.numBoneInfluencers = 4;
      mesh.material = material(part.look);
      mesh.setEnabled(false);
      return mesh;
    }),
  );
  const people = walks(count).map((walk, i) => {
    const body = bodies[i % bodies.length];
    const node = new TransformNode(`resident${i}`, scene);
    const skeleton = new Skeleton(`resident${i}`, `resident${i}`, scene);
    const bones: Bone[] = [];
    body.bones.forEach((name, b) => {
      const parent = body.parents[b];
      bones.push(new Bone(name, skeleton, parent >= 0 ? bones[parent] : null, Matrix.Translation(body.restT[b * 3], body.restT[b * 3 + 1], body.restT[b * 3 + 2])));
    });
    for (const template of templates[i % bodies.length]) {
      const mesh = template.clone(`${template.name}${i}`, node);
      mesh.skeleton = skeleton;
      mesh.setEnabled(true);
      drawn.push(mesh);
    }
    const swung = SWUNG.map((name) => bones[body.bones.indexOf(name)]);
    node.rotationQuaternion = null;
    return { walk, node, swung, rotations: swung.map(() => new Quaternion()) };
  });
  const pose = newPose();
  return (t) => {
    for (const person of people) {
      poseAt(person.walk, t, pose);
      person.node.position.set(pose.x, 0, pose.z);
      person.node.rotation.y = pose.yaw;
      person.swung.forEach((bone, k) => {
        const q = person.rotations[k];
        q.set(pose.rotations[k * 4], pose.rotations[k * 4 + 1], pose.rotations[k * 4 + 2], pose.rotations[k * 4 + 3]);
        bone.rotationQuaternion = q;
      });
    }
  };
}

export async function babylonDriver({ canvas, api, width, height, rooms, spec }: DriverOptions): Promise<Driver> {
  let engine: AbstractEngine;
  let webgpu: WebGPUEngine | null = null;
  if (api === 'webgpu') {
    webgpu = new WebGPUEngine(canvas, { antialias: true, powerPreference: 'high-performance' });
    await webgpu.initAsync();
    engine = webgpu;
  } else {
    engine = new Engine(canvas, true, { powerPreference: 'high-performance' }, false);
  }
  engine.setSize(width, height);

  const scene = new Scene(engine);
  scene.useRightHandedSystem = true;
  scene.skipPointerMovePicking = true;
  scene.clearColor = new Color4(CLEAR[0], CLEAR[1], CLEAR[2], 1);
  scene.imageProcessingConfiguration.toneMappingEnabled = true;
  scene.imageProcessingConfiguration.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;

  const camera = new FreeCamera('camera', Vector3.Zero(), scene);
  camera.fov = CAMERA.fov;
  camera.minZ = CAMERA.near;
  camera.maxZ = CAMERA.far;

  const ambient = new HemisphericLight('ambient', Vector3.Up(), scene);
  ambient.diffuse = new Color3(AMBIENT, AMBIENT, AMBIENT);
  ambient.groundColor = ambient.diffuse;
  ambient.specular = Color3.Black();
  const sun = new DirectionalLight('sun', new Vector3(...SUN.direction).normalize(), scene);
  sun.intensity = SUN.intensity;

  const size = floorSize(rooms, spec);
  const floor = MeshBuilder.CreateGround('floor', { width: size, height: size }, scene);
  const floorMat = new PBRMaterial('floor', scene);
  floorMat.albedoColor = new Color3(FLOOR, FLOOR, FLOOR);
  floorMat.metallic = 0;
  floorMat.roughness = 1;
  floor.material = floorMat;
  floor.receiveShadows = spec.shadows;

  const drawn: Mesh[] = [];
  const furniture = await loadTemplates(MODELS, scene);
  (spec.instanced ? placeInstances : placeCopies)(furniture, placements(rooms), drawn);
  if (spec.garden) placeCopies(await loadTemplates(GARDEN, scene), gardenPlacements(), drawn);
  if (spec.trees) placeInstances(await loadTemplates(TREES, scene), treePlacements(spec.trees), drawn);
  const animate = spec.residents ? await addResidents(spec.residents, scene, drawn) : () => {};

  if (spec.lamps > 0) {
    const lamps = lampPositions(rooms, spec.lamps).map((at, i) => {
      const lamp = new SpotLight(`lamp${i}`, new Vector3(...at), Vector3.Down(), LAMP.angle, 2, scene);
      lamp.diffuse = new Color3(...LAMP.color);
      lamp.intensity = LAMP.intensity;
      lamp.range = LAMP.range;
      return lamp;
    });
    const cluster = new ClusteredLightContainer('lamps', lamps, scene);
    cluster.maxRange = LAMP.range;
  }

  if (spec.shadows) {
    // The map fitted to every caster, its depth range too (else casters far from the camera's range drop out).
    sun.autoCalcShadowZBounds = true;
    const shadows = new ShadowGenerator(2048, sun);
    shadows.usePercentageCloserFiltering = true;
    shadows.filteringQuality = ShadowGenerator.QUALITY_MEDIUM;
    for (const mesh of drawn) {
      shadows.addShadowCaster(mesh, false);
      mesh.receiveShadows = true;
    }
  }

  // The game's chain (`BabylonRenderer.setupPostProcessing`): MSAA into a half-float target, then
  // one image-processing pass (tone mapping, vignette, colour curves); bloom only in `post: 'bloom'`.
  if (spec.post && spec.post !== 'none') {
    const post = new DefaultRenderingPipeline('post', true, scene, [camera]);
    post.samples = 4;
    post.fxaaEnabled = false;
    post.imageProcessingEnabled = true;
    const ip = post.imageProcessing;
    ip.toneMappingEnabled = true;
    ip.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
    ip.vignetteEnabled = true;
    ip.vignetteStretch = 0.5;
    ip.vignetteColor = new Color4(0.02, 0.02, 0.04, 0);
    ip.colorCurvesEnabled = true;
    ip.colorCurves = new ColorCurves();
    post.bloomEnabled = spec.post === 'bloom';
    post.bloomThreshold = 0.8;
    post.bloomWeight = 0.3;
    post.bloomKernel = 64;
    post.bloomScale = 0.5;
    post.depthOfFieldEnabled = false;
  }

  animate(0);
  await scene.whenReadyAsync();
  for (const material of scene.materials) material.freeze();
  const instrumentation = new SceneInstrumentation(scene);

  const radius = orbitRadius(rooms, spec);
  return {
    api: webgpu ? 'webgpu' : 'webgl2',
    version: Engine.Version,
    frame(t) {
      const { position, target } = cameraAt(t, radius);
      camera.position.set(...position);
      camera.setTarget(new Vector3(...target));
      animate(t);
      engine.beginFrame();
      scene.render();
      engine.endFrame();
    },
    drawCalls: () => instrumentation.drawCallsCounter.current,
    dispose: () => engine.dispose(),
  };
}

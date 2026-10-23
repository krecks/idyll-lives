/**
 * The engine comparison's PlayCanvas side (`bench.ts`): the shared scene (`scene.ts`) with the
 * PlayCanvas engine used from code (no Editor), on its defaults where the game sets nothing
 * (clustered lighting is on in PlayCanvas by default; lamps go through it, as in Babylon).
 * Post-processing is its `CameraFrame`, set up as the game's Babylon chain.
 *
 * The app's own frame loop is stopped once started: the benchmark calls `update` and `render`
 * from its own loop, which is what PlayCanvas's tick does each frame.
 */

// The release build by path: under Vite's dev server the package's `development` condition picks its
// debug build, which wraps every WebGPU draw in validation error scopes (a GPU-process round trip each).
import * as pc from 'playcanvas/build/playcanvas/src/index.js';
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

const DEG = 180 / Math.PI;

function loadContainer(app: pc.AppBase, url: string): Promise<pc.ContainerResource> {
  return new Promise((resolve, reject) => {
    app.assets.loadFromUrl(url, 'container', (err, asset) => (err || !asset ? reject(new Error(`${url}: ${err}`)) : resolve(asset.resource as pc.ContainerResource)));
  });
}

/** The turn that points an entity's -Y (where PlayCanvas lights shine) along `dir`. */
function pointDown(entity: pc.Entity, dir: readonly [number, number, number]): void {
  const from = new pc.Vec3(0, -1, 0);
  const to = new pc.Vec3(...dir).normalize();
  const axis = new pc.Vec3().cross(from, to);
  if (axis.length() < 1e-6) return;
  entity.setRotation(new pc.Quat().setFromAxisAngle(axis.normalize(), Math.acos(from.dot(to)) * DEG));
}

interface ShadowFlags {
  castShadows: boolean;
  receiveShadows: boolean;
}

/** Each placement as its own copy of the model (its own mesh instances, sharing meshes and materials). */
function placeCopies(app: pc.AppBase, containers: pc.ContainerResource[], all: Placement[], flags: ShadowFlags): void {
  for (const p of all) {
    const entity = containers[p.model].instantiateRenderEntity(flags);
    entity.setLocalPosition(p.x, 0, p.z);
    entity.setLocalEulerAngles(0, p.rotY * DEG, 0);
    app.root.addChild(entity);
  }
}

/**
 * One entity per model at the origin; each of its parts draws every placement from a buffer of
 * matrices. A part draws at `world × instance` (PlayCanvas's `transformInstancing` chunk):
 * placed at P in the world, that is `world⁻¹ × P × world`.
 */
function placeInstances(app: pc.AppBase, device: pc.GraphicsDevice, containers: pc.ContainerResource[], all: Placement[], flags: ShadowFlags): void {
  const format = pc.VertexFormat.getDefaultInstancingFormat(device);
  const templates = containers.map((container) => {
    const entity = container.instantiateRenderEntity(flags);
    app.root.addChild(entity);
    return entity;
  });
  app.root.syncHierarchy();
  for (const [model, entity] of templates.entries()) {
    const mine = all.filter((p) => p.model === model);
    for (const render of entity.findComponents('render') as pc.RenderComponent[]) {
      for (const instance of render.meshInstances) {
        const world = instance.node.getWorldTransform();
        const inverse = world.clone().invert();
        const matrices = new Float32Array(mine.length * 16);
        const placed = new pc.Mat4();
        const local = new pc.Mat4();
        mine.forEach((p, i) => {
          placed.setTRS(new pc.Vec3(p.x, 0, p.z), new pc.Quat().setFromEulerAngles(0, p.rotY * DEG, 0), pc.Vec3.ONE);
          matrices.set(local.mul2(inverse, placed).mul(world).data, i * 16);
        });
        instance.setInstancing(new pc.VertexBuffer(device, format, mine.length, { data: matrices }));
      }
    }
  }
}

/** The residents: a mesh and a skin per body part and family, and per resident a hierarchy of bone nodes and a skin instance. */
async function addResidents(app: pc.AppBase, device: pc.GraphicsDevice, count: number, flags: ShadowFlags): Promise<(t: number) => void> {
  const bodies = await loadBodies();
  const materials = new Map<PartLook, pc.StandardMaterial>();
  const material = (look: PartLook) => {
    let mat = materials.get(look);
    if (!mat) {
      mat = new pc.StandardMaterial();
      mat.diffuse = new pc.Color(...LOOK_COLOR[look]);
      mat.useMetalness = true;
      mat.metalness = 0;
      mat.gloss = 0.3;
      mat.update();
      materials.set(look, mat);
    }
    return mat;
  };
  const families = bodies.map((body) => {
    // Rest rotations are the identity: a bone's inverse bind is its rest joint, negated.
    const inverseBinds = body.bones.map((_, b) => new pc.Mat4().setTranslate(-body.restWorld[b * 3], -body.restWorld[b * 3 + 1], -body.restWorld[b * 3 + 2]));
    const skin = new pc.Skin(device, inverseBinds, body.bones);
    const meshes = body.parts.map((part) => {
      const mesh = new pc.Mesh(device);
      const vertices = part.positions.length / 3;
      mesh.setPositions(part.positions);
      mesh.setNormals(part.normals);
      mesh.setVertexStream(pc.SEMANTIC_BLENDINDICES, part.joints, 4, vertices, pc.TYPE_UINT8);
      mesh.setVertexStream(pc.SEMANTIC_BLENDWEIGHT, part.weights, 4, vertices);
      mesh.setIndices(part.indices);
      mesh.update(pc.PRIMITIVE_TRIANGLES);
      mesh.skin = skin;
      return { mesh, look: part.look };
    });
    return { body, skin, meshes };
  });
  const people = walks(count).map((walk, i) => {
    const { body, skin, meshes } = families[i % families.length];
    const entity = new pc.Entity(`resident${i}`);
    app.root.addChild(entity);
    const bones: pc.GraphNode[] = [];
    body.bones.forEach((name, b) => {
      const bone = new pc.GraphNode(name);
      bone.setLocalPosition(body.restT[b * 3], body.restT[b * 3 + 1], body.restT[b * 3 + 2]);
      const parent = body.parents[b];
      (parent >= 0 ? bones[parent] : entity).addChild(bone);
      bones.push(bone);
    });
    const skinInstance = new pc.SkinInstance(skin);
    skinInstance.bones = bones;
    skinInstance.rootBone = entity;
    const meshInstances = meshes.map(({ mesh, look }) => {
      const instance = new pc.MeshInstance(mesh, material(look), entity);
      instance.skinInstance = skinInstance;
      return instance;
    });
    entity.addComponent('render', { meshInstances, ...flags });
    return { walk, entity, swung: SWUNG.map((name) => bones[body.bones.indexOf(name)]) };
  });
  const pose = newPose();
  return (t) => {
    for (const person of people) {
      poseAt(person.walk, t, pose);
      person.entity.setLocalPosition(pose.x, 0, pose.z);
      person.entity.setLocalEulerAngles(0, pose.yaw * DEG, 0);
      person.swung.forEach((bone, k) => bone.setLocalRotation(pose.rotations[k * 4], pose.rotations[k * 4 + 1], pose.rotations[k * 4 + 2], pose.rotations[k * 4 + 3]));
    }
  };
}

export async function playcanvasDriver({ canvas, api, width, height, rooms, spec }: DriverOptions): Promise<Driver> {
  const device = await pc.createGraphicsDevice(canvas, {
    deviceTypes: [api === 'webgpu' ? pc.DEVICETYPE_WEBGPU : pc.DEVICETYPE_WEBGL2],
    antialias: true,
    powerPreference: 'high-performance',
  });
  device.maxPixelRatio = 1;

  const options = new pc.AppOptions();
  options.graphicsDevice = device;
  options.componentSystems = [pc.RenderComponentSystem, pc.CameraComponentSystem, pc.LightComponentSystem];
  options.resourceHandlers = [pc.ContainerHandler, pc.TextureHandler];
  const app = new pc.AppBase(canvas);
  app.init(options);
  app.setCanvasFillMode(pc.FILLMODE_NONE);
  app.setCanvasResolution(pc.RESOLUTION_FIXED, width, height);

  app.scene.ambientLight = new pc.Color(AMBIENT, AMBIENT, AMBIENT);
  app.scene.clusteredLightingEnabled = true;
  app.scene.lighting.shadowsEnabled = false;
  app.scene.lighting.cookiesEnabled = false;
  app.scene.lighting.maxLightsPerCell = 32;

  const camera = new pc.Entity('camera');
  camera.addComponent('camera', {
    clearColor: new pc.Color(CLEAR[0], CLEAR[1], CLEAR[2]),
    fov: CAMERA.fov * DEG,
    nearClip: CAMERA.near,
    farClip: CAMERA.far,
    toneMapping: pc.TONEMAP_ACES,
    gammaCorrection: pc.GAMMA_SRGB,
  });
  app.root.addChild(camera);

  const radius = orbitRadius(rooms, spec);
  const sun = new pc.Entity('sun');
  sun.addComponent('light', {
    type: 'directional',
    intensity: SUN.intensity,
    castShadows: spec.shadows,
    shadowResolution: 2048,
    shadowType: pc.SHADOW_PCF3_32F,
    numCascades: 1,
    // Covers what the orbit (`cameraAt`) sees, as Babylon's sun fits its map to every caster.
    shadowDistance: radius * 2,
    shadowBias: 0.2,
    normalOffsetBias: 0.05,
  });
  pointDown(sun, SUN.direction);
  app.root.addChild(sun);

  const floorMat = new pc.StandardMaterial();
  floorMat.diffuse = new pc.Color(FLOOR, FLOOR, FLOOR);
  floorMat.useMetalness = true;
  floorMat.metalness = 0;
  floorMat.gloss = 0;
  floorMat.update();
  const floor = new pc.Entity('floor');
  floor.addComponent('render', { type: 'plane', material: floorMat, castShadows: false, receiveShadows: spec.shadows });
  const size = floorSize(rooms, spec);
  floor.setLocalScale(size, 1, size);
  app.root.addChild(floor);

  const flags = { castShadows: spec.shadows, receiveShadows: spec.shadows };
  const load = (urls: string[]) => Promise.all(urls.map((url) => loadContainer(app, url)));
  const furniture = await load(MODELS);
  if (spec.instanced) placeInstances(app, device, furniture, placements(rooms), flags);
  else placeCopies(app, furniture, placements(rooms), flags);
  if (spec.garden) placeCopies(app, await load(GARDEN), gardenPlacements(), flags);
  if (spec.trees) placeInstances(app, device, await load(TREES), treePlacements(spec.trees), flags);
  const animate = spec.residents ? await addResidents(app, device, spec.residents, flags) : () => {};

  for (const at of lampPositions(rooms, spec.lamps)) {
    const lamp = new pc.Entity('lamp');
    lamp.addComponent('light', {
      type: 'spot',
      color: new pc.Color(...LAMP.color),
      intensity: LAMP.intensity,
      range: LAMP.range,
      innerConeAngle: (LAMP.angle / 2) * DEG * 0.75,
      outerConeAngle: (LAMP.angle / 2) * DEG,
      castShadows: false,
    });
    lamp.setPosition(...at);
    app.root.addChild(lamp);
  }

  // As the game's Babylon chain (`babylon.ts`): MSAA into an HDR target, then tone mapping,
  // vignette and colour grading in one pass; bloom only in `post: 'bloom'`.
  if (spec.post && spec.post !== 'none' && camera.camera) {
    const frame = new pc.CameraFrame(app, camera.camera);
    frame.rendering.samples = 4;
    frame.rendering.toneMapping = pc.TONEMAP_ACES;
    frame.vignette.intensity = 0.4;
    frame.vignette.inner = 0.5;
    frame.vignette.outer = 1;
    frame.vignette.curvature = 0.5;
    frame.grading.enabled = true;
    frame.bloom.intensity = spec.post === 'bloom' ? 0.03 : 0;
    frame.bloom.blurLevel = 16;
    frame.update();
  }

  animate(0);
  app.start();
  // The benchmark's loop drives the frames from here (see the file's comment).
  cancelAnimationFrame(app.frameRequestId);
  app.frameRequestId = null;

  let last = 0;
  let draws = 0;
  return {
    api: device.isWebGPU ? 'webgpu' : 'webgl2',
    version: pc.version,
    frame(t) {
      const { position, target } = cameraAt(t, radius);
      camera.setPosition(...position);
      camera.lookAt(...target);
      animate(t);
      app.update(Math.max(0, t - last));
      last = t;
      app.render();
      draws = device._drawCallsPerFrame;
      device._drawCallsPerFrame = 0;
    },
    drawCalls: () => draws,
    dispose: () => app.destroy(),
  };
}

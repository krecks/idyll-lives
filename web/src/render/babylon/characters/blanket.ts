/**
 * Bed blankets in Babylon (the quilt's shape and tint: `shared/characters/blanket.ts`): one draped quilt mesh per
 * body type, drawn with one thin instance per Sim asleep in a bed (none drawn at all while nobody is), in that bed's colours.
 *
 * The quilt is a height field draped over the actual lying pose (the body skinned once on the CPU at
 * load): heights are tented and blurred so the cloth bridges between the feet, knees, hips and the
 * hands resting on the belly, then it drapes over both sides and the foot end and has a turned-down
 * sheet band and a thick edge at the chest. Local frame: the Sim's slot on the mattress (x across,
 * y up from the mattress top, z towards the feet; scale 1); the instance matrix fits it to the Sim's
 * size, the slot width and the bed's foot end.
 */

import {
  Color3,
  DynamicTexture,
  MaterialPluginBase,
  Mesh,
  PBRMaterial,
  ShaderLanguage,
  VertexData,
  type Material,
  type Nullable,
  type Scene,
} from '../core';
import { BLANKET_WIDTH, COVER, quiltTint, SHEET, type QuiltGeometry } from '../../shared/characters/blanket';

export { BLANKET_WIDTH, quiltTint };

export interface BlanketShape {
  mesh: Mesh;
  /** Local z of the foot edge (where the foot drape starts), for fitting to the bed. */
  footZ: number;
  /** Local z just past the toes: the quilt beyond it stretches to the bed's foot end (`blanketExt`). */
  toeZ: number;
  /** Local z of the top (chest) edge. */
  topZ: number;
}

/** The quilt mesh of `geometry` (`quiltGeometry`), in `material`. */
export function buildBlanket(scene: Scene, name: string, geometry: QuiltGeometry, material: PBRMaterial): BlanketShape {
  const { positions: pos, uvs: uv, indices: idx, footZ, topZ, toeZ } = geometry;
  const vd = new VertexData();
  vd.positions = pos;
  vd.indices = idx;
  vd.uvs = uv;
  const normals: number[] = [];
  VertexData.ComputeNormals(pos, idx, normals);
  vd.normals = normals;
  const mesh = new Mesh(name, scene);
  vd.applyToMesh(mesh, false);
  mesh.material = material;
  mesh.isPickable = false;
  mesh.receiveShadows = true;
  mesh.alwaysSelectAsActiveMesh = true;
  mesh.doNotSyncBoundingInfo = true;
  return { mesh, footZ, topZ, toeZ };
}

/**
 * Quilt material: a soft quilted cover with a turned-down white sheet band at the top. The cover is
 * drawn grey and each quilt takes its bed's bedding colour over it (`blanketTint`, per instance), so
 * every bed keeps the colour the player chose, under one material.
 */
export function blanketMaterial(scene: Scene, toeZ: number, footZ: number): PBRMaterial {
  const size = 512;
  const tex = new DynamicTexture('simBlanket', { width: size, height: size }, scene, true);
  const ctx = tex.getContext() as CanvasRenderingContext2D;
  // v (texture y) spans 2 m; the top `SHEET` is the sheet.
  const perM = size / 2;
  const cover = COVER.toString(16);
  ctx.fillStyle = `#${cover}${cover}${cover}`;
  ctx.fillRect(0, 0, size, size);
  // Quilting: diamond stitch lines with soft puffed shading between them.
  const step = 0.16 * perM;
  for (let k = -size; k < size * 2; k += step) {
    for (const dir of [1, -1]) {
      ctx.strokeStyle = 'rgba(40,40,40,0.35)';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(k, 0);
      ctx.lineTo(k + dir * size, size);
      ctx.stroke();
      ctx.strokeStyle = 'rgba(255,255,255,0.12)';
      ctx.lineWidth = 6;
      ctx.beginPath();
      ctx.moveTo(k + step / 2, 0);
      ctx.lineTo(k + step / 2 + dir * size, size);
      ctx.stroke();
    }
  }
  // Turned-down sheet with a piped edge.
  // (Canvas rows run bottom-up in texture v.)
  ctx.fillStyle = '#EEEBE4';
  ctx.fillRect(0, size - SHEET * perM, size, SHEET * perM);
  ctx.fillStyle = '#C9C3B8';
  ctx.fillRect(0, size - SHEET * perM - 3, size, 3);
  ctx.fillStyle = '#D9D4CB';
  ctx.fillRect(0, size - 0.03 * perM - 2, size, 2);
  tex.update();
  tex.wrapU = DynamicTexture.CLAMP_ADDRESSMODE;
  tex.wrapV = DynamicTexture.CLAMP_ADDRESSMODE;
  tex.vScale = 0.5;
  const mat = new PBRMaterial('simBlanket', scene);
  mat.albedoTexture = tex;
  mat.metallic = 0;
  mat.roughness = 0.88;
  mat.sheen.isEnabled = true;
  mat.sheen.intensity = 0.45;
  mat.sheen.color = new Color3(1, 1, 1);
  mat.backFaceCulling = false;
  mat.twoSidedLighting = true;
  new BlanketPlugin(mat, toeZ, footZ);
  return mat;
}

let pendingRange: [number, number] = [0, 1];

/**
 * Per-instance `blanketExt` (m, local), two numbers. The first moves the quilt past the toes towards the
 * bed's foot end (stretching the stretch between toes and foot edge), so one mesh fits beds of any length
 * while the part over the body keeps its shape. The second is how far up it is pulled (local z): the quilt
 * from there to its foot end keeps its shape and the rest folds down under that line, flat on the mattress
 * (pulled up as the Sim lies down, pushed down as they get up; far past the top edge when it's all up).
 * Per-instance `blanketTint` (linear rgb, `quiltTint`): its bed's bedding colour, over the cover.
 */
class BlanketPlugin extends MaterialPluginBase {
  private readonly range: [number, number];

  constructor(material: Material, toeZ: number, footZ: number) {
    pendingRange = [toeZ, footZ];
    super(material, 'SimBlanket', 240, { SIMBLANKET: false }, true, true);
    this.range = [toeZ, footZ];
  }

  override getClassName(): string {
    return 'SimBlanketPlugin';
  }

  override isCompatible(): boolean {
    return true;
  }

  override prepareDefines(defines: Record<string, unknown>): void {
    defines.SIMBLANKET = true;
  }

  override getAttributes(attributes: string[]): void {
    attributes.push('blanketExt', 'blanketTint');
  }

  override getCustomCode(shaderType: string, shaderLanguage?: ShaderLanguage): Nullable<{ [pointName: string]: string }> {
    const [a, b] = (this.range ?? pendingRange).map((v) => v.toFixed(4));
    // The tint over the cover, not the sheet band (v: metres from the top edge).
    const sheet = SHEET.toFixed(3);
    if (shaderLanguage === ShaderLanguage.WGSL) {
      if (shaderType !== 'vertex') {
        return {
          CUSTOM_FRAGMENT_DEFINITIONS: `#ifdef SIMBLANKET\nvarying vBlanketTint: vec4<f32>;\n#endif\n`,
          CUSTOM_FRAGMENT_UPDATE_ALBEDO: `#ifdef SIMBLANKET\nsurfaceAlbedo = select(surfaceAlbedo, surfaceAlbedo * fragmentInputs.vBlanketTint.rgb, fragmentInputs.vBlanketTint.w >= ${sheet});\n#endif\n`,
        };
      }
      return {
        CUSTOM_VERTEX_DEFINITIONS: `#ifdef SIMBLANKET\nattribute blanketExt: vec2<f32>;\nattribute blanketTint: vec3<f32>;\nvarying vBlanketTint: vec4<f32>;\n#endif\n`,
        CUSTOM_VERTEX_UPDATE_POSITION: `#ifdef SIMBLANKET\nif (positionUpdated.z < vertexInputs.blanketExt.y) { positionUpdated = vec3<f32>(positionUpdated.x, 0.0, vertexInputs.blanketExt.y); }\npositionUpdated.z += vertexInputs.blanketExt.x * smoothstep(${a}, ${b}, positionUpdated.z);\n#endif\n`,
        CUSTOM_VERTEX_MAIN_END: `#ifdef SIMBLANKET\nvertexOutputs.vBlanketTint = vec4<f32>(vertexInputs.blanketTint, vertexInputs.uv.y);\n#endif\n`,
      };
    }
    if (shaderType !== 'vertex') {
      return {
        CUSTOM_FRAGMENT_DEFINITIONS: `#ifdef SIMBLANKET\nvarying vec4 vBlanketTint;\n#endif\n`,
        CUSTOM_FRAGMENT_UPDATE_ALBEDO: `#ifdef SIMBLANKET\nif (vBlanketTint.w >= ${sheet}) surfaceAlbedo *= vBlanketTint.rgb;\n#endif\n`,
      };
    }
    return {
      CUSTOM_VERTEX_DEFINITIONS: `#ifdef SIMBLANKET\nattribute vec2 blanketExt;\nattribute vec3 blanketTint;\nvarying vec4 vBlanketTint;\n#endif\n`,
      CUSTOM_VERTEX_UPDATE_POSITION: `#ifdef SIMBLANKET\nif (positionUpdated.z < blanketExt.y) positionUpdated = vec3(positionUpdated.x, 0.0, blanketExt.y);\npositionUpdated.z += blanketExt.x * smoothstep(${a}, ${b}, positionUpdated.z);\n#endif\n`,
      CUSTOM_VERTEX_MAIN_END: `#ifdef SIMBLANKET\nvBlanketTint = vec4(blanketTint, uv.y);\n#endif\n`,
    };
  }
}

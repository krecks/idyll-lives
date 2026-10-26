/**
 * Sims as skinned, animated characters (bodies, faces, hair and clothes generated in code by
 * `cargo characters` (crates/tools/src/characters), clips made in code in `characters/clips`;
 * format in docs/characters-format.md): what they do and how they are posed, engine-neutral. The
 * engine draws them through a `ResidentView` (`view.ts`; `pc/characters.ts`).
 * The tables the state machine plays from are in `actions.ts`, `seats.ts`, `socials.ts`,
 * `expressions.ts` and `parts.ts`; a resident's state in `rig.ts`.
 *
 * GPU / performance model:
 * - One mesh per (body, part) — body, eyes, brows, lashes, teeth, tongue, each hairstyle and each garment
 *   (`top.*`, `bottom.*`, `shoes.*`) — drawn instanced (one instance per Sim wearing it).
 * - Skinning runs on the GPU: a float texture holds one row per Sim (all bone matrices, plus
 *   appearance texels, `layout.ts`), and each instance reads "its" row (on PlayCanvas an instance
 *   attribute, `pc/characters.ts`). The CPU poses every visible
 *   Sim once per frame (clip sampling, crossfades, retargeting, look-at, face) and uploads the band of
 *   rows that changed in one write. Only buffer contents change per frame (bone texture, contact-shadow
 *   / ring / blanket instance matrices), never meshes.
 * - Clothes are painted on the body by its shader (each garment's cut over the body's fields);
 *   shells (`top.*`, `bottom.*`, `shoes.*` parts) are added only where the silhouette needs them.
 * - Small face parts (teeth, tongue, lashes, brows) are drawn only on faces big enough on screen to
 *   show them, and faces too small to show expressions skip their morphs (`faceDetail`); only the
 *   instance lists change per frame.
 * - Far from the camera a Sim switches to the low-detail body and shells (`lod.*` parts; with a gap
 *   between switching down and up), and every Sim casts its sun shadow from shadow-only copies of
 *   those (`shadowMeshes`, on the shadow-only layer; none where Sims cast no shadows, `castsShadows`).
 * - Life stages load the first time a Sim needs them (`loadStage`).
 * - No allocations per frame.
 *
 * Animation: a small state machine per Sim, driven by the snapshot: pose, moving, conversation
 * fields, and what the Sim is doing (`object` + `action` tag, mapped to clips, upper-body layers and
 * hand poses in `ACTIONS`; older layouts fall back to guessing from the object the Sim stands at).
 * The clips stand, sit and lie as residents should, so the runtime layers on only what depends on
 * the moment: idle with variations, walk (playback matched to the ground speed, so feet don't
 * slide), sit (fitted to each seat's height and depth, `SEATS`), lie (`lie` turned flat, under a
 * blanket in beds), talk / listen / laugh / argue / fight / hug / kiss, object use (cooking steps
 * have clips of their own), carrying (an upper-body clip, the thing on the palms), a baby picked up
 * out of the crib for a cuddle (posed in the arms of the one holding them, `cuddles`) and procedural
 * touches per activity (`activityPose`). States crossfade over ~0.25 s; turns are smoothed.
 *
 * Face: face morphs (ARKit face units and visemes, mixed per Sim in `characters/face.ts` and added
 * on the GPU before skinning) plus eye bones: irregular blinks, eyes on the conversation partner or
 * now and then the camera with the lids following, expressions from the snapshot `emotion`, and
 * the mouth speaking the voice line's visemes (`setSpeech`), or babbling without one (`animateFace`).
 */

import { paintSlots } from '../../../assets/paint';
import type { AssetRegistry } from '../../../assets/registry';
import type { Content, ItemDef } from '../../../content/content';
import type { FrameState } from '../../../core/bridge';
import { type ObjectPlacement, type PartColors, type SimInfo, type WorldStructure, colorsKey, objectCentre, storeyShown } from '../../../core/protocol';
import { Pose } from '../../../core/snapshot';
import { seatsItself } from '../../../game/surfaces';
import { ENERGY_RATE, type VisemeTrack } from '../../../voice/visemes';
import { GAME_RATE, QUIET_SPEED } from '../gameSpeed';
import { paintColors } from '../modelEntry';
import { type TableRoom, furthestAhead, nearestAhead } from '../tableTuck';
import { ACTIONS, BABY_LIE, BABY_LIE_FADE, BASE_HEIGHT, BUNK_LADDER, CARRY, CROUCH_ONLY, CROUCH_SHOULDER, CUDDLE, DEFAULT_FADE, FINGER_OPEN, GAITS, GAIT_MAX_RATE, GAIT_SWAY, GEARS, GRIPPED, GUITAR, HANDS, HOLD_DROP, HUGGED, IDLE_ONLY, JOINT_OPEN, LANE_SEQ, LANE_SYNC, LEVEL_GAZE, LOOP_STEP, LYING, PUSH_ONLY, RIDES, SET_DOWN, SET_DOWN_SIDE, SIT_HANDS, SNAP_DISTANCE, STALL, STAND_BACK, STATION_PROPS, STATION_SPOTS, STEP_IN, STEP_IN_SPEED, STEP_OUT_S, STEP_OVER, STRADDLED, STRUM, STRUM_SHORT, type StepIn, TOWEL_OUT, USE, WALK_FASTEST, WALK_IN, WALK_PER_TICK, WALK_SLACK, WALK_WAIT } from './actions';
import { LOW_TOP, SIT_LOW } from './clips/low';
import { FLOOR_SITS } from './clips/seated';
import { type BodyData, type CharacterSet, type Clip, loadCharacterSet, loadStage } from './data';
import { BEARING_FLOATS, bearing } from './bodyLanguage';
import { gaitFor } from './gait';
import { TURN_FADE, isTurn, turnClipIn, turnDone, turnOnSpot, turnYaw } from './turns';
import { ASLEEP, COLD, EXPRESSIONS, FRIENDLY, HUG, KISS, LAUGH, MOOD_COLD, MOOD_WARM, NONE, RELAX_ARMS, SOCIAL_FACES, STANCE, type StageDirection } from './expressions';
import { FACE_KEYS, type Face, FaceMixer, NEUTRAL } from './face';
import { type OutfitParams, SHELL_BITS, WEARS, type Wear, outfitFor, writeOutfit } from './outfit';
import { cradled, cuddlePose, onPad, writeCradle } from './cuddles';
import { lyingPose, measureLegs, measureSit, outfitFit, type PoseKit } from './measure';
import { palmAt, placeFrame, placeProp } from './props';
import { DETAIL_PX, FACE_PARTS, LOD_FAR_PX, LOD_NEAR_PX, MORPH_PX, type PartMesh, Range, YOUNG_STAGES, shellBit, skinChunks } from './parts';
import { type Additive, type Placement, PoseScratch, addLocal, addRotation, blendInto, makeAdditive, resetAdditive, sampleClip, skin } from './pose';
import { BABBLE, compose, type Frame, type Player, HEAD_MARGIN, MAX_CHARACTERS, type SimRig, UNIT, advance, alongWay, clamp, clampBuild, clampHeight, copyPlayer, embraceStep, eyesBelowTop, freezePlayer, gettingIn, hash, lerp, lerpAngle, lieBack, lieCentre, multiply, player, ramp, rotationOf, setFrame, shoulderDrop, smooth, speaking, splitKey, unit, wrapAngle } from './rig';
import { type LegSide, SEAT_FIT, feetDown, legTurn, perch, seatSize } from './seatFit';
import { BATH_WEAR, BEDS, BENCH_HIP, BENCH_KNEE, BLANKET_BEDS, BLANKET_MEET, BUNKS, type Bed, CLIP_HEIGHT, CURLED_BEDS, DEFAULT_BED, DEFAULT_SEAT, FEET_UP, FOOTSTOOLS, LEGS_OUT_DROP, LIE_CENTRE, LIE_ENTRY, type LegFit, NO_FEET_UP, ON_FLOOR, PEDAL, type PlacedObject, QUILT_COLOUR, QUILT_PART, REST_BACK, RISING, SEATS, SIDE_BEDS, SIDE_IN, SIT_CLIPS, SIT_FLEX, SIT_FROM, SLING, type Seat, type SeatFit, TUCK, floorSit, seatOf, slingTop } from './seats';
import { AIM, Anim, EMBRACE, KISS_FIT, REFUSED, gesturesOf } from './socials';
import { BLANKET_WIDTH, quiltGeometry, quiltTint } from './blanket';
import { APPEARANCE_TEXELS, MORPH_TEXELS } from './layout';
import type { TemplateCache } from '../templateCache';
import { WALL_HEIGHT } from '../walls';
import type { ResidentProp, ResidentView } from './view';

export { GAME_RATE, QUIET_SPEED } from '../gameSpeed';
export { MAX_CHARACTERS } from './rig';
export { EXPRESSIONS, FRIENDLY, STANCE, type StageDirection } from './expressions';
export type { Face } from './face';

/** How fast an upper-layer clip fades in over the one it replaces (per second; a quarter of a second, less when the
 * action runs fast: scaled by `animRate` like the lower fade). */
const UPPER_FADE_RATE = 4;

/** How fast a rise cut short (`SimRig.riseK`) finishes its step off the seat (per second: a quarter of a second, less
 * when the action runs fast). */
const RISE_CATCH_UP = 4;

/** `q` / `p` = the player's pose: its clip sampled at its time, or the pose it holds (`Player.frozen`). A player
 * with neither (a rig that has not been given a clip yet) leaves `q` / `p` as they were. */
function samplePlayer(set: CharacterSet, p: Player, q: Float32Array, t: Float32Array): void {
  const held = p.frozen;
  if (held) {
    q.set(held.q);
    t.set(held.p);
  } else if (p.clip) sampleClip(set, p.clip, p.time, q, t);
}

export class Characters<M extends object, Q, P extends ResidentProp> {
  private set: CharacterSet | null = null;
  private failed = false;
  private data!: Float32Array;
  /** The pose texture as last uploaded, and which rows were drawn then (see `upload`). */
  private uploaded!: Float32Array;
  private readonly uploadedVisible = new Uint8Array(MAX_CHARACTERS);
  /** The last `update` changed how a drawn Sim looks (moved, breathed, blinked …): their shadows are drawn again. */
  posed = false;
  private width = 0;
  /** The per-Sim face morph channel mixer (null for sets without morphs). */
  private mixer: FaceMixer | null = null;
  private readonly meshes = new Map<string, PartMesh<M>>();
  /** The drawn part meshes by mesh (`rowsOf`). */
  private readonly parts = new WeakMap<object, PartMesh<M>>();
  private sims: (SimRig | null)[] = [];
  private objects: PlacedObject[] = [];
  /** Rows per storey (0: one storey): residents' rows are mapped back onto the ground's. */
  private storeyDepth = 0;

  /** A lot row on the ground storey's rows (where it's drawn, lifted to its storey's height). */
  private groundRow(z: number): number {
    const d = this.storeyDepth;
    return d > 0 ? z - Math.floor(z / d) * d : z;
  }
  private readonly objectById = new Map<number, PlacedObject>();
  /** Pulled up to a table (`TUCK`): the tables the seats face, by seat id (`setTables`), and the seats' ids. */
  private tables = new Map<number, TableRoom>();
  private readonly seatIds = new Map<PlacedObject, number>();
  /** The placed footstools (`PlacedObject.footstool`). */
  private footstools: PlacedObject[] = [];
  /** Where an object's moving seat has its riders now (`ObjectMotion.ride`): world space, from rest. */
  private rideOf: ((objectId: number) => ArrayLike<number> | null) | null = null;
  /** Scratch for `riding`: the placement as a matrix, then moved by the ride. */
  private readonly rideM = new Float32Array(16);
  /** The lanes others have taken on what `freeLane` looks at (scratch). */
  private readonly lanesTaken: number[] = [];
  /** Action tag per `sim.action` index (from the snapshot layout). */
  private actionTags: readonly string[] = [];
  /** The `cuddle` and `change` actions' indexes in `actionTags` (-1: none). */
  private cuddleAction = -1;
  private changeAction = -1;
  private scratch!: PoseScratch;
  /** Pose scratch for the frame: `A` the blend in progress, `B` / `C` the players it blends (`update`, `start`).
   * Allocated by `init`; `update` returns early until then (`set` is null). */
  private qA!: Float32Array;
  private pA!: Float32Array;
  private qB!: Float32Array;
  private pB!: Float32Array;
  private qC!: Float32Array;
  private pC!: Float32Array;
  private add!: Additive;
  /** The posing scratch above, for measuring bodies (`measure.ts`). */
  private kit!: PoseKit;
  private upperMask!: Float32Array;
  private fingerMask!: Float32Array;
  private readonly place: Placement = {
    x: 0,
    y: 0,
    z: 0,
    qx: 0,
    qy: 0,
    qz: 0,
    qw: 1,
    scale: 1,
  };
  private readonly headTmp = new Float32Array(3);
  private readonly palm = new Float32Array(3);
  /** Per Sim row: the grown-up holding them (a baby picked up for a cuddle, `CUDDLE`), -1 none. */
  private readonly heldBy = new Int16Array(MAX_CHARACTERS).fill(-1);
  /** The rows in the order they're posed this frame: a baby in someone's arms after the one holding them. */
  private readonly order = new Int16Array(MAX_CHARACTERS);
  /** Per Sim row holding a baby: the middle of the cradling forearms (x, y, z) as last posed. */
  private readonly cradle = new Float32Array(MAX_CHARACTERS * 3);
  /** Per Sim row: a baby on the crib's changing pad (`CHANGE`): the crib, else null. */
  private readonly onPadOf: (PlacedObject | null)[] = new Array(MAX_CHARACTERS).fill(null);
  /** Scratch for placing a held baby: two orientations and a vector. */
  private readonly cradleQ = new Float32Array(12);
  /** How long a cuddle lasts (game minutes), per object def. */
  private readonly cuddleMinutes = new Map<string, number>();
  private readonly faceTmp: Face = { ...NEUTRAL };
  /** Gaze yaw / pitch (head-relative) from `gazeAt`. */
  private readonly gaze = new Float32Array(2);
  /**
   * Things Sims carry (content `items`, by model) and tools on the station they cook at (`tool.*`):
   * one template per model, one thin instance (matrix) per Sim row; null for a part its model doesn't have.
   */
  private readonly props: TemplateCache<P | null>;
  /** Stops listening for model files that come (`onModelFileArrived`). */
  private readonly unlisten: () => void;
  /** Per Sim row: the model of what it carries and of the tool in front of it, as last drawn. */
  private readonly heldProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  private readonly toolProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  /** Per Sim row: what it holds of the object it uses (content `held`), in the left and the right hand (or both). */
  private readonly leftProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  private readonly rightProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  /** Scratch for `writeHeld`: a frame's axes and origin (x, y, z each). */
  private readonly frame3 = new Float32Array(12);
  /**
   * Blankets: one quilt mesh per body (the base bodies', and each life stage's body as it loads, `addQuilt`);
   * the covered Sims wearing that body packed afresh each frame into its drawn `matrices` (a matrix each) and
   * `ext` (two numbers each), `drawn` of them (`writeBlanket`).
   */
  private readonly blankets = new Map<
    string,
    { quilt: Q; footZ: number; topZ: number; matrices: Float32Array; ext: Float32Array; tint: Float32Array; drawn: number; shown: number; base: boolean }
  >();
  /** The quilts' shared material is made (`ResidentView.quiltMaterial`), for the life stages' quilts made later. */
  private quilted = false;
  /** The beds' bedding colours as their files have them, by `PlacedObject.quilt` key (null: still loading, or none). */
  private readonly beddingColours = new Map<string, string | null>();
  /** Per Sim row: the shells its clothes now leave out (`OutfitParams.bare`, `faceDetail`). */
  private readonly bare = new Uint8Array(MAX_CHARACTERS);
  /** Per Sim row: head top (x, y, z) for speech bubbles and the selection marker. */
  readonly heads = new Float32Array(MAX_CHARACTERS * 3);
  /** Per Sim row: where they stand (x, z), drawn or not (NaN while away). */
  private readonly spots = new Float32Array(MAX_CHARACTERS * 2).fill(NaN);
  /** The storey each Sim stands on (`update`; fractions on the stairs). */
  private readonly storeys = new Float32Array(MAX_CHARACTERS);
  /** Per Sim row, for a conversation they started: how many listen in, and (up to) two of them. */
  private readonly groups = new Int16Array(MAX_CHARACTERS * 3);
  /** Per Sim row, for a conversation they started with others listening in: the middle of the group (x, z). */
  private readonly groupMiddle = new Float32Array(MAX_CHARACTERS * 2);
  /** Per Sim row: projected size (px) of 5 cm of its face, from the last frame (see `faceDetail`). */
  private readonly facePx = new Float32Array(MAX_CHARACTERS).fill(1e6);
  /** Per Sim row: drawn with the low-detail body and clothes. */
  private readonly far = new Uint8Array(MAX_CHARACTERS);
  /** The set has low-detail bodies. */
  private hasLod = false;
  /** Shadow-only copies of the low-detail meshes (all Sims cast shadows from those; none without `castsShadows`). */
  private readonly shadowMeshes = new Map<string, PartMesh<M>>();
  private readonly castsShadows: boolean;
  readonly visible = new Uint8Array(MAX_CHARACTERS);
  private lastNow = 0;
  /** Seconds of animation played: stands still while the game is paused (Sims hold their pose). */
  private clock = 0;
  /** Sit / lie pelvis offsets per body (computed once from the clips). */
  private readonly sitPelvis = new Map<string, SeatFit>();
  /** The same for the floor clips (`sitFloor`), per body, then floor sit (`FloorSit.name`). */
  private readonly floorPelvis = new Map<string, ReadonlyMap<string, SeatFit>>();
  /** Per body, then clip (`LEG_CLIPS`): where its legs are in the clip, for placing the feet (`legTo`). */
  private readonly legFits = new Map<string, Map<string, LegFit>>();
  /** Which way the sun's light goes (the renderer's sun), for the shade under a tree (`shadeSide`). */
  private sunOf: (() => { x: number; z: number }) | null = null;
  /** Where a moving object's loop has got to (0..1 of a round; a bike's pedals), null while none plays. */
  private loopOf: ((objectId: number) => number | null) | null = null;
  /** Scratch: a leg's thigh and calf rotations from `legTo`, and the root's height and forward offset from `inSling`. */
  private readonly legAdd = new Float32Array(2);
  private readonly slingAt = new Float32Array(2);
  /** `?debug`: force a clip on every Sim (`__characters.force = 'sit'`). */
  force: string | null = null;
  /** `?debug`: keep the standing stance layers on a forced clip (as the clip looks in game). */
  forceStance = false;
  /**
   * `?debug`: expression override (`__characters.faceDebug = { smile: 1, jaw: 0.5, closed: 1 }`;
   * `channels: { jawOpen: 1 }` adds raw morph channels, `viseme: 10` holds a viseme).
   */
  faceDebug: (Partial<Face> & { closed?: number; talk?: boolean; channels?: Record<string, number>; viseme?: number }) | null = null;
  /** `?debug`: animation speed override (e.g. to keep animating while the game is paused). */
  debugRate: number | null = null;
  /** CPU time of the last `update` (ms, smoothed), for profiling. */
  updateMs = 0;
  /** Emotion id per snapshot emotion code - 1 (content `emotions`). */
  private emotionIds: readonly string[] = [];
  /** Scratch: how free a body is for its body language this frame (`bodyLanguage`). */
  private readonly freedom = { posture: 0, arms: 0, head: 0 };
  /** Stage directions per Sim row (character creator / portraits only; empty in game). */
  readonly directions: (StageDirection | null)[] = [];
  /** Selected Sim (selection ring), or null. */
  private selected: number | null = null;
  /** Reduce motion: the selection ring keeps still (no spin, no pulse). */
  reducedMotion = false;
  private readonly ringMatrix = new Float32Array(16);
  /** Scratch: the camera (`ResidentView.camera`): position, render height, field of view. */
  private readonly cam = new Float32Array(5);
  /** The camera this frame (`cam`; false: none). */
  private hasCam = false;

  constructor(
    /** The engine that draws them (`ResidentView`). */
    private readonly view: ResidentView<M, Q, P>,
    private readonly assets: AssetRegistry,
    /** For seats content describes (`seat`) and what can be sat or lain on. */
    private readonly content?: Content,
    { refreshProps = false, castsShadows = true }: {
      /**
       * Builds again the props drawn without a model file that came (the game's): the scene then tries
       * failed files in the background (`onModelFileArrived`). Off for the preview studio and the debug pages.
       */
      refreshProps?: boolean;
      /** Sims cast sun shadows (the shadow-only copies are built); off where they cast none (`QualitySettings.characterShadows`). */
      castsShadows?: boolean;
    } = {},
  ) {
    this.castsShadows = castsShadows;
    this.props = view.props;
    // A model file that failed has come: the props drawn without it (small blocks) are built again,
    // and the blocks go once they are (hidden at once: the frame no longer moves them).
    this.unlisten = refreshProps
      ? view.onModelFile((url) => {
          const keys = this.props.invalidate(url);
          if (!keys.length) return;
          this.props.hideStale();
          const dispose = () => this.props.disposeStale();
          void Promise.all(keys.map((key) => this.props.get(key, () => this.view.buildProp(key)))).then(dispose, dispose);
        })
      : () => {};
  }

  /** Loads the character set and creates the pose texture; false if unavailable. */
  async init(): Promise<boolean> {
    if (this.set) return true;
    if (this.failed) return false;
    const entry = this.assets.get('model.sim', 'character');
    if (!entry) {
      this.failed = true;
      return false;
    }
    try {
      this.set = await loadCharacterSet(entry.url);
    } catch (err) {
      console.warn('[render] characters unavailable', err);
      this.failed = true;
      return false;
    }
    const set = this.set;
    const NB = set.bones.length;
    this.width = NB * 4 + APPEARANCE_TEXELS + MORPH_TEXELS;
    this.data = new Float32Array(this.width * MAX_CHARACTERS * 4);
    this.uploaded = new Float32Array(this.data.length);
    this.view.init(set, this.data, this.width, MAX_CHARACTERS);
    if (set.morphs) this.mixer = new FaceMixer(set.morphs);
    this.scratch = new PoseScratch(NB);
    this.qA = new Float32Array(NB * 4);
    this.qB = new Float32Array(NB * 4);
    this.qC = new Float32Array(NB * 4);
    this.pA = new Float32Array(3);
    this.pB = new Float32Array(3);
    this.pC = new Float32Array(3);
    this.add = makeAdditive(NB);
    this.kit = { set, qA: this.qA, pA: this.pA, add: this.add, scratch: this.scratch };
    // Upper body: spine_02 and everything above it (arms, neck, head); spine_01 half.
    this.upperMask = new Float32Array(NB);
    for (let b = 0; b < NB; b++) {
      let p = b;
      let w = 0;
      while (p >= 0) {
        const n = set.bones[p];
        if (n === 'spine_02') {
          w = 1;
          break;
        }
        if (n === 'spine_01' && p === b) w = 0.5;
        p = set.parents[p];
      }
      this.upperMask[b] = w;
    }
    // How far each finger joint opens towards the rest pose (scaled by the hand pose): the knuckles
    // keep more of the clip's curl than the tips, and the curl grows from the index to the little
    // finger, so open hands rest softly curved instead of straight-knuckled with curled tips (a claw).
    this.fingerMask = new Float32Array(NB);
    set.bones.forEach((n, b) => {
      const m = /^(index|middle|ring|pinky|thumb|fingers)_0(\d)/.exec(n);
      if (m) this.fingerMask[b] = FINGER_OPEN[m[1]] * (JOINT_OPEN[Number(m[2]) - 1] ?? 1);
    });
    for (const body of set.bodies.values()) this.measureBody(body);
    const lying = [...set.bodies.values()].map((body) => ({
      body,
      ...lyingPose(this.kit, body),
    }));
    // One stretch range for both quilts (their toes are within a centimetre).
    const toeZ = Math.max(...lying.map((l) => l.feetZ)) + 0.03;
    this.view.quiltMaterial(toeZ, toeZ + 0.29);
    this.quilted = true;
    for (const l of lying) this.addQuilt(l.body, l);
    this.view.createRing(this.ringMatrix);
    if (new URLSearchParams(location.search).has('debug')) {
      Object.assign(window, { __characters: this, __stance: STANCE, __friendly: FRIENDLY });
      this.view.onDispose(() => {
        const debug = window as { __characters?: unknown };
        if (debug.__characters === this) delete debug.__characters;
      });
    }
    return true;
  }

  /** The mesh part of a hairstyle (manifest `model.hair.<style>` may name another part). */
  private hairPart(style = 'short'): string {
    const key = `model.hair.${style}`;
    return (this.assets.has(key, 'character') ? this.assets.get(key, 'character')!.part : undefined) ?? `hair.${style}`;
  }

  /** The base body (young adult) for a gender. */
  private baseBody(gender: string): BodyData {
    const set = this.set!;
    return set.bodies.get(gender === 'female' ? 'female' : 'male') ?? set.bodies.values().next().value!;
  }

  /** Emotion ids in snapshot order (`emotion` field = index + 1), for facial expressions. */
  setEmotions(ids: readonly string[]): void {
    this.emotionIds = ids;
  }

  /** Shows the selection ring under Sim `id` (null hides it). */
  select(id: number | null): void {
    this.selected = id;
  }

  private writeRing(time: number): void {
    const m = this.ringMatrix;
    m.fill(0);
    const id = this.selected;
    const rig = id !== null ? this.sims[id] : null;
    if (rig && rig.visible && !LYING.has(rig.cur.key)) {
      const r = 0.95 * rig.scale * (1 + Math.sin(time * 2.4) * 0.03);
      const a = time * 0.5;
      m[0] = Math.cos(a) * r;
      m[2] = -Math.sin(a) * r;
      m[5] = 1;
      m[8] = Math.sin(a) * r;
      m[10] = Math.cos(a) * r;
      m[12] = rig.x;
      m[13] = 0.03;
      m[14] = rig.z;
      m[15] = 1;
    }
    this.view.ringMoved();
  }

  /** Character meshes in use (shadow casters; small details excluded); none if Sims cast no shadows. */
  casters(): M[] {
    const out: M[] = [];
    if (!this.castsShadows) return out;
    // The low-detail body and clothes' shadow copies; sets without them cast from the full meshes.
    for (const p of this.shadowMeshes.values()) if (p.enabled) out.push(p.mesh);
    if (out.length) return out;
    for (const [key, p] of this.meshes) if (p.enabled && !/:(eyes|brows|lashes|teeth|tongue|shoes\.)/.test(key)) out.push(p.mesh);
    return out;
  }

  /**
   * Builds instances for the world's Sims (meshes and materials are cached). `keep` keeps the
   * animation state (clips, blankets, pyjamas, a baby in someone's arms ...): `'rows'` of every
   * row that stays (previews restyling a Sim without restarting it), `'residents'` of rows that
   * still hold the same resident in the same body (the game: the world is rebuilt whenever
   * something in it changes; a slot taken by someone new, or a body grown into the next life
   * stage, starts afresh). A Sim may carry `outfitId`, the id its outfit is derived from when it
   * differs from its row.
   */
  async build(world: Pick<WorldStructure, 'sims' | 'objects'> & { storeys?: number; storeyDepth?: number; basement?: number }, keep: 'rows' | 'residents' | false = false): Promise<void> {
    if (!(await this.init())) return;
    const set = this.set!;
    // Life stages other than the base bodies' (children, teens, elders …) load on demand, the
    // first time a Sim needs them.
    await Promise.all(
      world.sims.map(async (s) => {
        if (s.gone || s.id >= MAX_CHARACTERS) return null;
        const base = this.baseBody(s.gender);
        const stage = s.appearance?.stage;
        if (stage && stage !== base.stage) {
          const b = await loadStage(set, base, stage).catch((err) => (console.warn('[render] life stage unavailable', base.name, stage, err), null));
          if (b && !this.sitPelvis.has(b.name)) this.measureBody(b);
          // Its own quilt, draped over its own lying pose: a child's feet stand up higher for its size than a
          // grown-up's quilt made smaller covers (their shoes showed through it halfway down the bed).
          if (b && !this.blankets.has(b.name) && this.quilted) this.addQuilt(b, lyingPose(this.kit, b));
        }
      }),
    );
    // Positions arrive in lot rows, which hold every storey (see `WorldStructure.storeys`).
    this.storeyDepth = (world.storeys ?? 1) > 1 || world.basement !== undefined ? (world.storeyDepth ?? 0) : 0;
    const kept = keep ? this.sims : [];
    this.sims = [];
    this.ghosts = [];
    this.data.fill(0);
    this.visible.fill(0);
    this.bare.fill(0);
    const groups = new Map<string, SimInfo[]>();
    const push = (key: string, s: SimInfo) => {
      const list = groups.get(key) ?? [];
      list.push(s);
      groups.set(key, list);
    };
    for (const s of world.sims) {
      if (s.id >= MAX_CHARACTERS || s.gone) continue;
      const base = this.baseBody(s.gender);
      const stage = s.appearance?.stage;
      // The body shaped for the Sim's life stage when there is one (else the base body, scaled).
      const body = stage && stage !== base.stage ? (set.bodies.get(`${base.name}.${stage}`) ?? base) : base;
      const shaped = !!stage && body.stage === stage;
      const hairKey = this.hairPart(s.appearance?.hairStyle);
      const outfitId = (s as SimInfo & { outfitId?: number }).outfitId ?? s.id;
      const day = outfitFor(outfitId, s.gender, s.appearance, body, 'day', undefined, s.outfit);
      // (Pyjamas are the same garments, so the same chunks show at night.)
      for (const part of skinChunks(body, day.parts)) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      for (const part of FACE_PARTS) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      if (body.parts.has(hairKey)) push(`${body.name}:${hairKey}`, s);
      if (s.appearance?.beard && !YOUNG_STAGES.has(body.stage) && body.parts.has('hair.beard')) push(`${body.name}:hair.beard`, s);
      for (const part of day.parts) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      // The low-detail body and clothes (drawn instead when the Sim is far from the camera).
      for (const part of ['lod.body', ...day.parts.map((p) => `lod.${p}`)]) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      const height = clampHeight(s.appearance?.height);
      // Life stage: a body shaped for it keeps its own size (at the base body's scale); otherwise
      // children are the base body made smaller with a bigger head (the session sets these).
      const stageScale = shaped ? 1 : (s.appearance?.stageScale ?? 1);
      const scale = ((BASE_HEIGHT[base.name] ?? 1.72) / base.height) * height * stageScale;
      const build = clampBuild(s.appearance?.build);
      const head = shaped ? 1 : (s.appearance?.stageHead ?? 1);
      const stoop = s.appearance?.stageStoop ?? 0;
      const size = body.height / base.height;
      const who = `${s.name}|${s.generation ?? 0}|${s.gender}|${stage ?? ''}|${body.name}|${outfitId}`;
      const old = kept[s.id];
      // The same rig carries on (pyjamas: `asleep` is worked out again, and the outfit for it written, when next posed).
      const rig: SimRig = old && (keep === 'rows' || old.who === who) ? Object.assign(old, { who, body, scale, size, head, stoop, build, worn: null }) : {
        id: s.id,
        who,
        floorY: 0,
        body,
        scale,
        head,
        stoop,
        build,
        size,
        seed: hash(s.id + 17),
        visible: false,
        x: 0,
        z: 0,
        yaw: 0,
        speed: 0,
        lastPose: -1,
        actTime: 0,
        seq: 0,
        useKey: '',
        useSeq: null,
        tag: '',
        useDef: '',
        useObj: null,
        useId: -1,
        sling: null,
        slingLift: 0,
        shadeYaw: NaN,
        floorSat: '',
        seatTurn: new Float32Array(8),
        tagKnown: false,
        cur: player(),
        prev: player(),
        w: 1,
        fadeRate: 4,
        upper: player(),
        upperPrev: player(),
        upperFade: 1,
        upperW: 0,
        upperTarget: 0,
        lookYaw: 0,
        lookPitch: 0,
        approach: 0,
        feetUp: false,
        feet: new Float32Array(4),
        bearing: new Float32Array(BEARING_FLOATS),
        turnKey: '',
        turnDeg: 0,
        turnFrom: 0,
        turnBy: 0,
        turnStarted: false,
        cuddleT: -1,
        cuddleOut: -1,
        cuddleMin: 0,
        cuddleRate: 1,
        cuddleBaby: -1,
        reachW: 0,
        holdW: 0,
        liftW: 0,
        cuddleStep: 0,
        carryT: -1,
        putT: -1,
        carryRate: 1,
        changeT: -1,
        changeOut: -1,
        changeMin: 0,
        changeRate: 1,
        faceX: NaN,
        faceZ: NaN,
        outfits: null,
        worn: null,
        hands: HANDS.relaxed,
        stepTime: 0,
        emotion: 0,
        socialId: '',
        bed: null,
        bedH: 0,
        bedFwd: 0,
        upperBunk: false,
        tx: 0,
        tz: 0,
        lieAcross: false,
        fromBed: false,
        lieX: 0,
        lieZ: 0,
        lieFwd: 0,
        lieYaw: 0,
        sideYaw: NaN,
        sideOut: 0,
        fromX: 0,
        fromZ: 0,
        rising: false,
        riseX: 0,
        riseZ: 0,
        riseK: 1,
        riseViaX: NaN,
        riseViaZ: NaN,
        walkT: -1,
        walkS: 0,
        walkL: Infinity,
        walkX: 0,
        walkZ: 0,
        viaX: NaN,
        viaZ: NaN,
        walkEX: 0,
        walkEZ: 0,
        walkPX: 0,
        walkPZ: 0,
        walkWait: 0,
        walkYaw: 0,
        walkTurn: 0,
        walkDown: false,
        toweled: -1,
        shower: null,
        sitX: 0,
        sitZ: 0,
        inside: null,
        inT: 0,
        inX: 0,
        inY: 0,
        inZ: 0,
        inLane: 0,
        inFacing: 0,
        inDX: 0,
        inDZ: 0,
        outT: -1,
        outWait: 0,
        inOver: Float32Array.of(0, 0, 1),
        tuck: 0,
        tuckSeat: -1,
        tuckFar: false,
        tuckLow: false,
        cover: 0,
        face: { ...NEUTRAL },
        blinkAt: -1,
        nextBlink: 0,
        eyeYaw: 0,
        eyePitch: 0,
        gazeMode: 0,
        gazeUntil: 0,
        glanceYaw: 0,
        glancePitch: 0,
        audience: null,
        dir: null,
        warmth: 1,
        relaxArms: 0,
        skirt: false,
        faceState: new Float32Array(set.morphs?.channels.length ?? 0),
        faceL: 1 + (hash(s.id * 7 + 3) - 0.5) * 0.2,
        faceR: 1 + (hash(s.id * 7 + 5) - 0.5) * 0.2,
        speech: null,
        speechStart: 0,
        speechSeg: 0,
      };
      this.sims[s.id] = rig;
      // (Day clothes until posed: `wearFor` then picks again.)
      const fit = outfitFit(set, base);
      rig.outfits = Object.fromEntries(WEARS.map((wear) => [wear, outfitFor(outfitId, s.gender, s.appearance, body, wear, fit, s.outfit)])) as Record<Wear, OutfitParams>;
      this.wear(rig, s.id, rig.outfits.day);
    }
    for (const p of this.meshes.values()) this.enablePart(p, false);
    for (const p of this.shadowMeshes.values()) this.enablePart(p, false);
    this.hasLod = [...set.bodies.values()].some((b) => b.parts.has('lod.body'));
    for (const [key, sims] of groups) {
      const [bodyName, part] = splitKey(key);
      const body = set.bodies.get(bodyName)!;
      const pm = this.partMesh(body, part);
      if (!pm) continue;
      const ids = sims.map((x) => x.id);
      this.setInstances(pm, ids);
      this.enablePart(pm, true);
      // Every Sim casts its sun shadow from the low-detail body and clothes (no hair, no face).
      if (this.castsShadows && part.startsWith('lod.')) {
        const sm = this.partMesh(body, part, true);
        if (sm) {
          this.setInstances(sm, ids);
          this.enablePart(sm, true);
        }
      }
    }
    // Objects Sims sit, lie or stand at (for seat heights and object-use animations).
    this.objectById.clear();
    this.objects = world.objects.map((o: ObjectPlacement) => {
      const z = this.groundRow(o.z);
      // Its middle where it stands (things that stand anywhere on their tile); the bounds stay on tiles.
      const centre = objectCentre(o);
      const def = this.content?.object(o.def);
      // (A thing on a top used sitting is used from the seat pulled up to the top, not sat on: `seatUnder` finds that.)
      const restful = !!(SEATS[o.def] || BEDS[o.def]) || (def?.layer !== 'surface' && !!def?.interactions.some((i) => i.pose === 'sit' || i.pose === 'lie'));
      const lift = (o.tall ?? 0) / 100;
      const seat = seatOf(def, o.def);
      const stand = def?.stand ? { h: def.stand.height + lift, motion: def.stand.motion } : null;
      const paint = def?.held ? colorsKey(paintColors(this.assets.get(def.model, 'model'), o.colors)) : '';
      const held = def?.held ? { keys: [def.held.part].flat().map((part) => `${def.model}#${part}${paint ? `@${paint}` : ''}`), grip: def.held.grip } : null;
      const stool = FOOTSTOOLS[o.def];
      const made = def?.footprint ?? [1, 1];
      const footstool = stool ? { hw: (stool[0] / 2) * (o.size ? o.size[0] / (made[0] * 100) : 1), hd: (stool[1] / 2) * (o.size ? o.size[1] / (made[1] * 100) : 1), yaw: ((o.rot * 90 + (o.turn ?? 0)) * Math.PI) / 180 } : null;
      return { def: o.def, rot: o.rot, seat: { ...seat, h: seat.h + lift }, stand, held, lift, restful, builtIn: !!def?.surface && seatsItself(def), quilt: BLANKET_BEDS.has(o.def) ? this.beddingOf(o.def, def?.model, o.colors) : null, footstool, cx: centre.x, cz: z + centre.z - o.z, minX: o.x, minZ: z, maxX: o.x + o.w, maxZ: z + o.d };
    });
    this.footstools = this.objects.filter((o) => o.footstool);
    world.objects.forEach((o, j) => this.objectById.set(o.id, this.objects[j]));
    this.seatIds.clear();
    world.objects.forEach((o, j) => this.tables.has(o.id) && this.seatIds.set(this.objects[j], o.id));
    const beds = world.objects.some((o) => BLANKET_BEDS.has(o.def));
    const bodies = new Set(this.sims.map((r) => r?.body.name));
    const rows = this.sims.length;
    for (const [name, b] of this.blankets) {
      // `update` compacts the covered rows (a town of six must not draw 64 quilts' worth of triangles).
      // (With none, not drawn at all, rather than once as it is, at the origin.)
      b.shown = 0;
      this.view.quiltRows(b.quilt, 0, false);
      // (A life stage's quilt only where someone has that body: no draw call for nobody.)
      this.view.quiltEnabled(b.quilt, rows > 0 && beds && (b.base || bodies.has(name)));
    }
    this.view.uploadPoses();
    this.uploaded.set(this.data);
    this.uploadedVisible.set(this.visible);
    this.posed = true;
  }

  /** `?debug` stress test: `n` extra animated Sims in a grid around (x, z), cloned from Sim 0. */
  private ghosts: { x: number; z: number; clip: string }[] = [];
  stress(n: number, x: number, z: number): void {
    const set = this.set;
    const base = this.sims.find(Boolean);
    if (!set || !base) return;
    const count = this.sims.length;
    const clips = ['idle', 'walk', 'talk', 'foldArms', 'idle', 'walk'];
    // Young adults of both genders in a T-shirt, trousers and sneakers.
    const bodies = [this.baseBody('male'), this.baseBody('female')];
    const clothes = ['top.tee', 'bottom.trousers', 'shoes.sneakers'];
    const parts = [
      ...FACE_PARTS,
      'hair.short',
      ...clothes,
      ...new Set(bodies.flatMap((b) => skinChunks(b, clothes))),
      'lod.body',
      ...clothes.map((c) => `lod.${c}`),
    ];
    this.ghosts = [];
    for (let k = 0; k < n && count + k < MAX_CHARACTERS; k++) {
      const id = count + k;
      this.ghosts.push({
        x: x + (k % 5) * 1.1,
        z: z + Math.floor(k / 5) * 1.1,
        clip: clips[k % clips.length],
      });
      const body = bodies[k % 2];
      const rig: SimRig = {
        ...base,
        id,
        body,
        scale: (BASE_HEIGHT[body.name] ?? 1.72) / body.height,
        head: 1,
        stoop: 0,
        build: 1,
        size: 1,
        cur: player(),
        prev: player(),
        upper: player(),
        upperPrev: player(),
        upperFade: 1,
        visible: true,
        x: 0,
        z: 0,
        yaw: k,
        seed: hash(id),
        face: { ...NEUTRAL },
        faceState: new Float32Array(set.morphs?.channels.length ?? 0),
        speech: null,
      };
      this.sims[id] = rig;
      const look = { body: '#5F86C9', skin: '#C08A64', hair: '#2B2320', hairStyle: 'short' as const, height: 1, top: 'top.tee', bottom: 'bottom.trousers', shoes: 'shoes.sneakers' };
      writeOutfit(outfitFor(id, body.name, look, body), this.data, this.row(id) + set.bones.length * 16);
    }
    // The ghosts' meshes (and the low-detail ones' shadow copies), even where nobody in the world wears them.
    for (const body of bodies) {
      for (const part of parts) {
        for (const shadow of this.castsShadows && part.startsWith('lod.') ? [false, true] : [false]) {
          const pm = this.partMesh(body, part, shadow);
          if (pm && !pm.enabled) {
            pm.all = new Float32Array(0);
            this.enablePart(pm, true);
          }
        }
      }
    }
    for (const [key, pm] of [...this.meshes, ...this.shadowMeshes]) {
      if (!pm.enabled) continue;
      const [bodyName, part] = splitKey(key);
      const ids = [...pm.all.filter((_, j) => j % 4 === 0)];
      this.ghosts.forEach((_, k) => {
        if (bodies[k % 2].name === bodyName && parts.includes(part)) ids.push(count + k);
      });
      this.setInstances(pm, ids);
    }
  }

  private poseGhosts(count: number, dt: number, time: number): void {
    const set = this.set!;
    this.ghosts.forEach((g, k) => {
      const i = count + k;
      const rig = this.sims[i]!;
      if (rig.cur.key !== g.clip) this.start(rig, g.clip, 0.2, 1, true, 0, 0, 0);
      advance(rig.cur, dt);
      rig.w = 1;
      sampleClip(set, rig.cur.clip!, rig.cur.time, this.qA, this.pA);
      blendInto(this.qA, this.pA, set.sourceRestQ, this.pA, 0.6, this.fingerMask);
      resetAdditive(this.add);
      this.procedural(rig, i, 0, 0, null, Pose.Stand, false, time, dt);
      const pl = this.place;
      pl.qx = 0;
      pl.qy = Math.sin(rig.yaw / 2);
      pl.qz = 0;
      pl.qw = Math.cos(rig.yaw / 2);
      pl.x = g.x;
      pl.y = 0;
      pl.z = g.z;
      pl.scale = rig.scale;
      pl.head = rig.head;
      pl.build = rig.build;
      skin(set, rig.body, this.qA, this.pA, this.add, pl, this.scratch, this.data, this.row(i), this.headTmp);
      this.visible[i] = 1;
    });
  }

  /** Stops listening for model files (the renderer goes; the scene's meshes go with it). */
  dispose(): void {
    this.unlisten();
  }

  /** Removes the Sims (end of a session); meshes stay cached. */
  clear(): void {
    this.sims = [];
    this.ghosts = [];
    this.objects = [];
    this.objectById.clear();
    this.visible.fill(0);
    for (const p of this.meshes.values()) this.enablePart(p, false);
    for (const b of this.blankets.values()) this.view.quiltEnabled(b.quilt, false);
    for (let i = 0; i < MAX_CHARACTERS; i++) {
      this.writeProp(this.heldProp, i, null);
      this.writeProp(this.toolProp, i, null);
      this.writeProp(this.leftProp, i, null);
      this.writeProp(this.rightProp, i, null);
    }
    // Props built without their files are built (and their files tried) afresh in the next session.
    this.props.dropFailed();
  }

  // --- carried things and tools ------------------------------------------------------------

  /**
   * A prop's template and matrices; starts loading it the first time (null until loaded, and for a part
   * its model doesn't have). `<model key>#<part>`: a moving part of an object's model, in its own frame
   * (`…@<colours>`: in those colours, `colorsKey`'s).
   */
  private prop(key: string): P | null {
    const known = this.props.built(key);
    if (known !== undefined) return known;
    void this.props.get(key, () => this.view.buildProp(key));
    return null;
  }

  /** Per frame: a prop nobody holds or works with isn't drawn (no draw call for its empty instances). */
  private readonly showProp = (p: P | null, key: string): void => {
    if (!p) return;
    const used = this.heldProp.includes(key) || this.toolProp.includes(key) || this.leftProp.includes(key) || this.rightProp.includes(key);
    // (The `stress` ghosts' rigs are in `sims` already, in the rows after the residents'.)
    this.view.showProp(p, used ? Math.min(this.sims.length, MAX_CHARACTERS) : 0);
  };

  /**
   * What Sim row `i` holds of the object it uses (content `held`; its own part is hidden then, the
   * model's `motions`), placed by the hands as just posed: a barbell across both palms, a dumbbell in
   * each, a guitar's neck in the left hand with its body against the belly, a mop's handle in the hands
   * with its foot on the floor ahead.
   */
  private writeHeld(rig: SimRig, i: number, pose: number, moving: boolean): void {
    const held = !moving && rig.tagKnown && pose !== Pose.Sit && !RISING.has(rig.cur.key) ? rig.useObj?.held : undefined;
    const grip = held && (held.grip === 'bar') === (pose === Pose.Lie) ? held.grip : null;
    const left = this.writeProp(this.leftProp, i, grip === 'pair' ? held!.keys[0] : null);
    const right = this.writeProp(this.rightProp, i, grip ? (held!.keys[grip === 'pair' ? 1 : 0] ?? null) : null);
    if (!grip || !this.set) return;
    const B = this.set.bone;
    const s = rig.scale;
    const size = Math.min(1, s);
    const f = this.frame3;
    const pl = this.palm;
    const fx = Math.sin(rig.yaw);
    const fz = Math.cos(rig.yaw);
    if (grip === 'pair') {
      // A dumbbell in each fist, lying across it (along the Sim's right), level.
      for (const [m, b, side] of [
        [left, B.hand_l, -1],
        [right, B.hand_r, 1],
      ] as const) {
        if (!m) continue;
        palmAt(this.scratch, rig, b, side, pl);
        setFrame(f, fz, 0, -fx, 0, 1, 0, fx, 0, fz, pl[0], pl[1], pl[2]);
        placeFrame(m, i, f, size);
      }
      return;
    }
    if (!right) return;
    palmAt(this.scratch, rig, B.hand_l, -1, pl);
    const lx = pl[0];
    const ly = pl[1];
    const lz = pl[2];
    palmAt(this.scratch, rig, B.hand_r, 1, pl);
    if (grip === 'bar') {
      // Across both palms, level.
      unit(pl[0] - lx, pl[1] - ly, pl[2] - lz);
      const x = UNIT[0];
      const y = UNIT[1];
      const z = UNIT[2];
      unit(-x * y, 1 - y * y, -z * y);
      const ux = UNIT[0];
      const uy = UNIT[1];
      const uz = UNIT[2];
      setFrame(f, x, y, z, ux, uy, uz, y * uz - z * uy, z * ux - x * uz, x * uy - y * ux, (pl[0] + lx) / 2, (pl[1] + ly) / 2, (pl[2] + lz) / 2);
      placeFrame(right, i, f, 1);
      return;
    }
    const top = this.props.built(held!.keys[0])?.top;
    if (!top) return;
    if (grip === 'guitar') {
      // Upright along its neck from where its body rests against the belly (in front of the spine's
      // base, a little to the right) to the left hand; its face to the front.
      // (Out from the spine as far as the body's size: a child's belly is nearer it; clips/uses.ts `GUITAR_BODY`.)
      const t = this.scratch.worldT;
      const g = GUITAR;
      const k = s * rig.size;
      const bx = t[B.spine_01 * 3] + (fx * g.out + fz * g.right) * k;
      const by = t[B.spine_01 * 3 + 1] + (g.up + g.upShort * (1 - this.guitarReach(rig))) * k;
      const bz = t[B.spine_01 * 3 + 2] + (fz * g.out - fx * g.right) * k;
      unit(lx - bx, ly - by, lz - bz);
      const yx = UNIT[0];
      const yy = UNIT[1];
      const yz = UNIT[2];
      const d = fx * yx + fz * yz;
      unit(fx - yx * d, -yy * d, fz - yz * d);
      const zx = UNIT[0];
      const zy = UNIT[1];
      const zz = UNIT[2];
      const along = top[1] * g.body * size;
      setFrame(f, yy * zz - yz * zy, yz * zx - yx * zz, yx * zy - yy * zx, yx, yy, yz, zx, zy, zz, bx - yx * along, by - yy * along, bz - yz * along);
      placeFrame(right, i, f, size);
      return;
    }
    // A pole: its top in the hands (between them, if both are on it), its foot on the floor ahead, as far
    // as it reaches; turned only up and down from facing the way the Sim faces.
    const both = Math.hypot(pl[0] - lx, pl[1] - ly, pl[2] - lz) < 0.35 * s;
    const gx = both ? (pl[0] + lx) / 2 : pl[0];
    const gy = both ? (pl[1] + ly) / 2 : pl[1];
    const gz = both ? (pl[2] + lz) / 2 : pl[2];
    const reach = Math.hypot(top[1], top[2]) * size;
    const high = Math.min(gy - rig.floorY, reach * 0.99);
    const ahead = Math.sqrt(reach * reach - high * high);
    // Its axis from the foot to the top: as built (up, and towards its back), and as held (up, and back to the hands).
    const turn = Math.atan2(-ahead, high) - Math.atan2(top[2], top[1]);
    const c = Math.cos(turn);
    const n = Math.sin(turn);
    setFrame(f, fz, 0, -fx, fx * n, c, fz * n, fx * c, -n, fz * c, gx + fx * ahead, rig.floorY, gz + fz * ahead);
    placeFrame(right, i, f, size);
  }

  /** How long the guitar Sim `rig` plays is for its size (`GUITAR`): 0 short (a ukulele) to 1 long (a guitar; also while it loads). */
  private guitarReach(rig: SimRig): number {
    const top = this.props.built(rig.useObj?.held?.keys[0] ?? '')?.top;
    return top ? clamp((top[1] * Math.min(1, rig.scale) / rig.scale - GUITAR.short) / (GUITAR.long - GUITAR.short), 0, 1) : 1;
  }

  /** Sim row `i` draws prop `key` (null: none) in `slot`; the old one's matrix is cleared. */
  private writeProp(slot: (string | null)[], i: number, key: string | null): Float32Array | null {
    const old = slot[i];
    if (old && old !== key) this.props.built(old)?.matrices.fill(0, i * 16, i * 16 + 16);
    slot[i] = key;
    return key ? (this.prop(key)?.matrices ?? null) : null;
  }

  /**
   * What Sim row `i` carries (`item`, from the snapshot's `held`): on the right palm, or held in
   * front between both palms (content `hold`), level and facing the way they face.
   */
  private writeCarried(rig: SimRig, i: number, item: ItemDef | undefined): void {
    const key = item ? (item.model ?? `item.${item.id}`) : null;
    const m = this.writeProp(this.heldProp, i, key);
    if (!m || !item || !this.set) return;
    const B = this.set.bone;
    const palm = this.palm;
    palmAt(this.scratch, rig, B.hand_r, 1, palm);
    let x = palm[0];
    let y = palm[1];
    let z = palm[2];
    if (item.hold === 'both') {
      palmAt(this.scratch, rig, B.hand_l, -1, palm);
      x = (x + palm[0]) / 2;
      y = (y + palm[1]) / 2;
      z = (z + palm[2]) / 2;
    }
    // The thing rests on the palms (its base a little below their centres).
    placeProp(m, i, x, y - HOLD_DROP * rig.scale, z, rig.yaw, Math.min(1, rig.scale));
  }

  /**
   * What Sim row `i` carried in, put down on the station it works at (`SET_DOWN`; none: in the oven): on its top, to
   * the side of the step's tool (`writeTool`) away from the station's middle, else of the middle.
   */
  private writeSetDown(rig: SimRig, i: number, item: ItemDef | undefined, tag: string, station: PlacedObject | null | undefined): void {
    const m = this.writeProp(this.heldProp, i, item && station ? (item.model ?? `item.${item.id}`) : null);
    if (!m || !station) return;
    const f = (station.rot * Math.PI) / 2;
    const spot = STATION_SPOTS[station.def]?.[tag];
    const top = this.content?.object(station.def)?.surface?.height ?? spot?.[2] ?? this.content?.cooking.defaultSurfaceHeight ?? 0.9;
    // (In the station's own frame: right of its middle, or of the tool's spot, by the side it isn't on.)
    const x = spot ? spot[0] - Math.sign(spot[0] || 1) * SET_DOWN_SIDE : SET_DOWN_SIDE;
    const z = spot ? spot[1] : 0;
    placeProp(m, i, station.cx + x * Math.cos(f) + z * Math.sin(f), rig.floorY + top, station.cz - x * Math.sin(f) + z * Math.cos(f), (station.rot * Math.PI) / 2, 1);
  }

  /**
   * Cuddles (`CUDDLE`): finds the baby each grown-up cuddling holds (lying in the crib they use),
   * advances the cuddle (picking up, holding, putting back as it nears its end; a cuddle cut short
   * lets the baby settle back into the crib) and orders the rows so a held baby is posed after the
   * one holding them.
   */
  private cuddles(frame: FrameState, count: number, dt: number, gameRate: number, animRate: number): void {
    const { curr, layout } = frame;
    const H = layout.headerLen;
    const S = layout.simStride;
    const k = layout.sim;
    const act = k.action !== undefined && k.object !== undefined ? this.cuddleAction : -1;
    const changing = k.action !== undefined && k.object !== undefined ? this.changeAction : -1;
    const step = dt * animRate;
    // Seconds of animation per game minute at this speed.
    const perMinute = gameRate > 0 ? animRate / gameRate : Infinity;
    this.heldBy.fill(-1);
    this.onPadOf.fill(null);
    for (let j = 0; j < count; j++) {
      const rig = this.sims[j];
      if (!rig) continue;
      const o = H + j * S;
      const obj = curr[o + k.object] | 0;
      const moving = curr[o + k.moving] > 0;
      rig.faceX = NaN;
      // Carrying a baby (`CARRY`): lifting them out (standing), then in the arms on the way; put down, laid in.
      const carried = k.carry !== undefined ? (curr[o + k.carry] | 0) - 1 : -1;
      if (carried >= 0 && carried < count && carried !== j && this.sims[carried]?.body.stage === 'baby' && !curr[o + k.away] && !this.force) {
        if (rig.carryT < 0 || rig.cuddleBaby !== carried) {
          rig.carryT = 0;
          rig.putT = -1;
          rig.carryRate = clamp((CUDDLE.pick + 0.4) / (CARRY.lift * perMinute), 1, 6);
          rig.cuddleT = -1;
          rig.cuddleOut = -1;
        }
        rig.cuddleBaby = carried;
        rig.carryT += step * rig.carryRate;
        // (On the way: held, the lift done whatever the speed.)
        const t = moving ? Math.max(rig.carryT, CUDDLE.pick + 1) : rig.carryT;
        rig.reachW = ramp(t, 0.15, 0.9) * (1 - ramp(t, 1.5, 0.9));
        rig.holdW = ramp(t, 1.5, 0.9);
        rig.liftW = clamp((t - 1.35) / 1.05, 0, 1);
        const ob = H + carried * S;
        if (moving) rig.cuddleStep = 0;
        else {
          const d = Math.hypot(curr[ob + k.x] - curr[o + k.x], this.groundRow(curr[ob + k.z]) - this.groundRow(curr[o + k.z]));
          rig.cuddleStep = lerp(Math.max(0, d - CUDDLE.rail - CUDDLE.reach) * ramp(t, 0, 0.7), 0, ramp(t, 1.9, 0.7));
          if (rig.liftW < 1) [rig.faceX, rig.faceZ] = [curr[ob + k.x], this.groundRow(curr[ob + k.z])];
        }
        this.heldBy[carried] = j;
        continue;
      }
      if (rig.carryT >= 0 && rig.cuddleBaby >= 0) {
        // Put down where they go (the high chair, the crib): laid there from the arms.
        if (rig.putT < 0) rig.putT = 0;
        rig.putT += step * rig.carryRate;
        const u = rig.putT;
        rig.reachW = ramp(u, 0.1, 0.9) * (1 - ramp(u, 1.4, 0.8));
        rig.holdW = 1 - ramp(u, 0.1, 0.9);
        rig.liftW = moving ? 0 : 1 - clamp((u - 0.1) / 1.05, 0, 1);
        rig.cuddleStep = 0;
        if (rig.liftW > 0) this.heldBy[rig.cuddleBaby] = j;
        else {
          rig.carryT = -1;
          rig.putT = -1;
          rig.cuddleBaby = -1;
        }
        continue;
      }
      // Changing the baby at the changing table (`CHANGE`).
      let changed = -1;
      if (changing >= 0 && (curr[o + k.action] | 0) === changing && curr[o + k.pose] === Pose.Stand && !moving && !curr[o + k.away] && !this.force) {
        for (let b = 0; b < count && changed < 0; b++) {
          const ob = H + b * S;
          if (b !== j && this.sims[b]?.body.stage === 'baby' && curr[ob + k.pose] === Pose.Lie && (curr[ob + k.object] | 0) === obj && !curr[ob + k.away]) changed = b;
        }
      }
      const crib = changed >= 0 ? (this.objectById.get(obj) ?? null) : null;
      if (crib) {
        const minutes = this.useLength(crib.def, 'change', 10);
        const T = CUDDLE.pick + CUDDLE.put;
        if (rig.changeT < 0 || rig.cuddleBaby !== changed) {
          rig.changeT = 0;
          rig.changeOut = -1;
          rig.changeMin = 0;
          rig.changeRate = clamp((2 * T + 1.5) / (minutes * perMinute), 1, 6);
          rig.cuddleT = -1;
        }
        rig.cuddleBaby = changed;
        rig.changeT += step * rig.changeRate;
        rig.changeMin += dt * gameRate;
        if (rig.changeOut >= 0) rig.changeOut += step * rig.changeRate;
        else if (rig.changeT >= T && (minutes - rig.changeMin) * perMinute * rig.changeRate <= T) rig.changeOut = 0;
        // Out of the cot (into the arms), onto the pad, back off it (into the arms), into the cot.
        const out = rig.changeOut >= 0;
        const t = out ? rig.changeOut : rig.changeT;
        const lifting = t < CUDDLE.pick;
        const u = lifting ? t : t - CUDDLE.pick;
        if (lifting) {
          rig.reachW = ramp(u, 0.15, 0.9) * (1 - ramp(u, 1.5, 0.9));
          rig.holdW = ramp(u, 1.5, 0.9);
          rig.liftW = clamp((u - 1.35) / 1.05, 0, 1);
        } else {
          rig.reachW = ramp(u, 0.1, 0.9) * (1 - ramp(u, 1.4, 0.8));
          rig.holdW = 1 - ramp(u, 0.1, 0.9);
          rig.liftW = 1 - clamp((u - 0.1) / 1.05, 0, 1);
        }
        rig.cuddleStep = 0;
        // On the pad from the moment they're in the arms on the way there until they're in the arms on the way back.
        const onPad = out ? lifting : !lifting;
        if (onPad) this.onPadOf[changed] = crib;
        else {
          const ob = H + changed * S;
          [rig.faceX, rig.faceZ] = [curr[ob + k.x], this.groundRow(curr[ob + k.z])];
        }
        if (rig.liftW > 0) this.heldBy[changed] = j;
        continue;
      }
      if (rig.changeT >= 0) {
        // Cut short: the baby back in the cot at once.
        rig.changeT = -1;
        rig.changeOut = -1;
        rig.liftW = 0;
        rig.holdW = 0;
        rig.reachW = 0;
        rig.cuddleBaby = -1;
      }
      let baby = -1;
      if (act >= 0 && (curr[o + k.action] | 0) === act && curr[o + k.pose] === Pose.Stand && !(curr[o + k.moving] > 0) && !curr[o + k.away] && !this.force) {
        for (let b = 0; b < count && baby < 0; b++) {
          const ob = H + b * S;
          if (b !== j && this.sims[b]?.body.stage === 'baby' && curr[ob + k.pose] === Pose.Lie && (curr[ob + k.object] | 0) === obj && !curr[ob + k.away]) baby = b;
        }
      }
      if (baby >= 0) {
        // Game minutes it lasts.
        const minutes = this.cuddleLength(this.objectById.get(obj)?.def ?? '');
        if (rig.cuddleT < 0 || rig.cuddleBaby !== baby) {
          // Picking up and putting back fit in (faster at speed).
          rig.cuddleT = 0;
          rig.cuddleOut = -1;
          rig.cuddleMin = 0;
          rig.cuddleRate = clamp((CUDDLE.pick + CUDDLE.put + 1.5) / (minutes * perMinute), 1, 6);
        }
        rig.cuddleBaby = baby;
        rig.cuddleT += step * rig.cuddleRate;
        rig.cuddleMin += dt * gameRate;
        if (rig.cuddleOut >= 0) rig.cuddleOut += step * rig.cuddleRate;
        else if (rig.cuddleT >= CUDDLE.pick && (minutes - rig.cuddleMin) * perMinute * rig.cuddleRate <= CUDDLE.put) rig.cuddleOut = 0;
        const t = rig.cuddleT;
        const u = rig.cuddleOut;
        // The feet: at the rail to reach in, a step back from it holding the baby.
        const ob = H + baby * S;
        const d = Math.hypot(curr[ob + k.x] - curr[o + k.x], this.groundRow(curr[ob + k.z]) - this.groundRow(curr[o + k.z]));
        const atRail = Math.max(0, d - CUDDLE.rail - CUDDLE.reach);
        const clear = Math.max(0, d - CUDDLE.rail - CUDDLE.clear);
        if (u < 0) {
          // Picking up: leaning in over the rail, the hands under the baby, lifting them out into the arms.
          rig.reachW = ramp(t, 0.15, 0.9) * (1 - ramp(t, 1.5, 0.9));
          rig.holdW = ramp(t, 1.5, 0.9);
          rig.liftW = clamp((t - 1.35) / 1.05, 0, 1);
          rig.cuddleStep = lerp(atRail * ramp(t, 0, 0.7), clear, ramp(t, 1.9, 0.7));
        } else {
          // Putting back: the same the other way round.
          rig.reachW = ramp(u, 0.1, 0.9) * (1 - ramp(u, 1.4, 0.8));
          rig.holdW = 1 - ramp(u, 0.1, 0.9);
          rig.liftW = 1 - clamp((u - 0.1) / 1.05, 0, 1);
          rig.cuddleStep = lerp(lerp(clear, atRail, ramp(u, 0, 0.6)), 0, ramp(u, 1.5, 0.7));
        }
      } else {
        // Not (or no longer) cuddling: cut short, the arms come down and the baby settles back into the crib.
        rig.cuddleT = -1;
        rig.cuddleOut = -1;
        rig.cuddleStep = 0;
        rig.reachW = Math.max(0, rig.reachW - step * 3);
        rig.holdW = Math.max(0, rig.holdW - step * 1.6);
        rig.liftW = Math.max(0, rig.liftW - step * 1.6);
        // Walking off (given an order), or the baby no longer lying there as a baby: back in the crib at once.
        const held = rig.cuddleBaby >= 0 ? this.sims[rig.cuddleBaby] : null;
        if (curr[o + k.moving] > 0 || held?.body.stage !== 'baby' || curr[H + rig.cuddleBaby * S + k.pose] !== Pose.Lie) rig.liftW = 0;
        if (rig.liftW <= 0) rig.cuddleBaby = -1;
      }
      if (rig.cuddleBaby >= 0 && rig.liftW > 0) this.heldBy[rig.cuddleBaby] = j;
    }
    let n = 0;
    for (let i = 0; i < count; i++) if (this.heldBy[i] < 0) this.order[n++] = i;
    for (let i = 0; i < count; i++) if (this.heldBy[i] >= 0) this.order[n++] = i;
  }

  /** How long the use shown as `anim` at an object of def `def` lasts (game minutes; `fallback` without one). */
  private useLength(def: string, anim: string, fallback: number): number {
    const key = `${def}:${anim}`;
    let m = this.cuddleMinutes.get(key);
    if (m === undefined) {
      m = this.content?.object(def)?.interactions.find((it) => it.anim === anim)?.minutes ?? fallback;
      this.cuddleMinutes.set(key, m);
    }
    return m;
  }

  /** How long a cuddle at an object of def `def` lasts (game minutes; its interaction shown as `cuddle`). */
  private cuddleLength(def: string): number {
    let m = this.cuddleMinutes.get(def);
    if (m === undefined) {
      m = this.content?.object(def)?.interactions.find((it) => it.anim === 'cuddle')?.minutes ?? 15;
      this.cuddleMinutes.set(def, m);
    }
    return m;
  }

  /**
   * The tool on the station Sim row `i` cooks at (a chopping board, a pan, food on the grill; by
   * action tag): at its spot (`STATION_SPOTS`), else on its top, a little towards the cook.
   */
  private writeTool(rig: SimRig, i: number, tag: string, station: PlacedObject | null | undefined): void {
    const key = station ? (STATION_PROPS[tag] ?? null) : null;
    const m = this.writeProp(this.toolProp, i, key);
    if (!m || !station) return;
    const spot = STATION_SPOTS[station.def]?.[tag];
    if (spot) {
      const f = (station.rot * Math.PI) / 2;
      const [x, z, y] = spot;
      placeProp(m, i, station.cx + x * Math.cos(f) + z * Math.sin(f), rig.floorY + y, station.cz - x * Math.sin(f) + z * Math.cos(f), rig.yaw, 1);
      return;
    }
    const def = this.content?.object(station.def);
    const top = def?.surface?.height ?? this.content?.cooking.defaultSurfaceHeight ?? 0.9;
    const dx = rig.x - station.cx;
    const dz = rig.z - station.cz;
    const d = Math.hypot(dx, dz) || 1;
    const toward = Math.min(0.15, d * 0.3);
    placeProp(m, i, station.cx + (dx / d) * toward, rig.floorY + top + 0.005, station.cz + (dz / d) * toward, rig.yaw, 1);
  }

  /** Head-top position of Sim `i` (false if not drawn). */
  head(i: number, out: { x: number; y: number; z: number }): boolean {
    if (i < 0 || i >= MAX_CHARACTERS || !this.visible[i]) return false;
    out.x = this.heads[i * 3];
    out.y = this.heads[i * 3 + 1];
    out.z = this.heads[i * 3 + 2];
    return true;
  }

  /** Whether Sim `i` is drawn standing on the ground storey (not upstairs, on the stairs or in the basement). */
  onGround(i: number): boolean {
    return i >= 0 && i < MAX_CHARACTERS && !!this.visible[i] && Math.abs(this.storeys[i]) < 0.01;
  }

  /** Where Sim `i` stands (x, z), also when not drawn (another lot); false while away or unknown. */
  spot(i: number, out: { x: number; z: number }): boolean {
    if (i < 0 || i >= MAX_CHARACTERS || Number.isNaN(this.spots[i * 2])) return false;
    out.x = this.spots[i * 2];
    out.z = this.spots[i * 2 + 1];
    return true;
  }

  /**
   * Poses every Sim for this frame. `inView(x, z)` decides visibility; contact-shadow matrices
   * (one 4x4 per Sim row) are written into `shadows`.
   */
  /** `storey`: the storey in view; residents above it (or, viewing the basement, not in it) aren't drawn. */
  update(frame: FrameState, inView: (x: number, z: number) => boolean, shadows: Float32Array, storey = Infinity): void {
    const set = this.set;
    if (!set) return;
    const started = performance.now();
    this.hasCam = this.view.camera(this.cam);
    const { prev, curr, alpha, layout, now } = frame;
    let dt = this.lastNow ? (now - this.lastNow) / 1000 : 0;
    this.lastNow = now;
    const stalled = dt > STALL;
    if (dt > 0.1) dt = 0.1;
    if (dt < 0) dt = 0;
    const speed = curr[layout.header.speed] | 0;
    const quiet = layout.header.calm !== undefined && curr[layout.header.calm] > 0;
    const gameSpeed = quiet ? Math.max(speed, QUIET_SPEED) : speed;
    const gameRate = GAME_RATE[gameSpeed] ?? 1;
    // How far a walker can get from `prev` to `curr` (at the faster of their speeds, over the steps between them).
    const prevSpeed = prev[layout.header.speed] | 0;
    const prevGameSpeed = layout.header.calm !== undefined && prev[layout.header.calm] > 0 ? Math.max(prevSpeed, QUIET_SPEED) : prevSpeed;
    const reach = Math.max(SNAP_DISTANCE, WALK_PER_TICK * WALK_FASTEST * WALK_SLACK * Math.max(gameRate, GAME_RATE[prevGameSpeed] ?? 1) * frame.steps);
    const animRate = this.debugRate ?? Math.min(gameRate, 2.5);
    // Paused (the pause button, Buy and Build mode, menus): everyone freezes mid-motion, walk
    // cycles, breathing, blinks and glances included.
    if (gameSpeed === 0 && this.debugRate === null) dt = 0;
    this.clock += dt;
    const time = this.clock;
    const count = Math.min(curr[layout.header.simCount], MAX_CHARACTERS);
    const H = layout.headerLen;
    const S = layout.simStride;
    const k = layout.sim;
    const NB = set.bones.length;
    if (layout.actions && layout.actions !== this.actionTags) {
      this.actionTags = layout.actions;
      this.cuddleAction = layout.actions.indexOf('cuddle');
      this.changeAction = layout.actions.indexOf('change');
    }
    const hasAction = k.action !== undefined && k.object !== undefined;

    // Group conversations: those listening in (role 3, the one who started it as partner), and
    // the middle of each group, everyone in it counted.
    const groups = this.groups;
    const middle = this.groupMiddle;
    groups.fill(0, 0, count * 3);
    for (let j = 0; j < count; j++) {
      const o = H + j * S;
      const l = curr[o + k.partner] | 0;
      if ((curr[o + k.role] | 0) !== 3 || l < 0 || l >= count || groups[l * 3] >= 2) continue;
      groups[l * 3 + 1 + groups[l * 3]++] = j;
    }
    for (let l = 0; l < count; l++) {
      const n = groups[l * 3];
      if (!n) continue;
      const t = curr[H + l * S + k.partner] | 0;
      // The one who started it, their partner and those listening in.
      let sx = 0;
      let sz = 0;
      let members = 0;
      for (let j = 0; j < n + 2; j++) {
        const m = j === 0 ? l : j === 1 ? t : groups[l * 3 + j - 1];
        if (m < 0 || m >= count) continue;
        sx += curr[H + m * S + k.x];
        sz += this.groundRow(curr[H + m * S + k.z]);
        members++;
      }
      middle[l * 2] = sx / members;
      middle[l * 2 + 1] = sz / members;
    }

    // Cuddles: who holds which baby (posed after them, in their arms).
    this.cuddles(frame, count, dt, gameRate, animRate);

    for (const b of this.blankets.values()) b.drawn = 0;
    for (let oi = 0; oi < count; oi++) {
      const i = this.order[oi];
      const rig = this.sims[i];
      const o = H + i * S;
      const cx = curr[o + k.x];
      const cz = this.groundRow(curr[o + k.z]);
      const pz = this.groundRow(prev[o + k.z]);
      const ch = k.height !== undefined ? curr[o + k.height] : 0;
      let tx = cx;
      let tz = cz;
      let th = ch;
      let tyaw = curr[o + k.yaw];
      // A walk's first and last step: moving in one snapshot and standing in the other glides too,
      // or the start and the end of every walk jump at speeds 4-5 (a step is 2.25 m at speed 4).
      const walking = curr[o + k.moving] > 0 || prev[o + k.moving] > 0;
      if (walking ? Math.hypot(cx - prev[o + k.x], cz - pz) < reach : Math.abs(cx - prev[o + k.x]) + Math.abs(cz - pz) < SNAP_DISTANCE) {
        tx = prev[o + k.x] + (cx - prev[o + k.x]) * alpha;
        tz = pz + (cz - pz) * alpha;
        if (k.height !== undefined) th = prev[o + k.height] + (ch - prev[o + k.height]) * alpha;
        tyaw = lerpAngle(prev[o + k.yaw], tyaw, alpha);
      }
      const away = curr[o + k.away] !== 0;
      this.spots[i * 2] = away ? NaN : tx;
      this.spots[i * 2 + 1] = tz;
      this.storeys[i] = th;
      const show = !!rig && !away && inView(tx, tz) && storeyShown(Math.floor(th + 0.01), storey);
      if (!rig || !show) {
        if (this.visible[i]) {
          this.data.fill(0, this.row(i), this.row(i) + NB * 16);
          shadows.fill(0, i * 16, i * 16 + 16);
        }
        if (rig) rig.cover = 0;
        this.visible[i] = 0;
        if (rig) rig.visible = false;
        this.writeProp(this.heldProp, i, null);
        this.writeProp(this.toolProp, i, null);
        this.writeProp(this.leftProp, i, null);
        this.writeProp(this.rightProp, i, null);
        continue;
      }
      const dir = (rig.dir = this.directions[i] ?? null);
      rig.floorY = th * WALL_HEIGHT;
      const pose = curr[o + k.pose];
      const moving = curr[o + k.moving] > 0;
      // Calling someone over (sim-core's `call`): a wave, looking at them.
      const call = k.call === undefined ? 0 : curr[o + k.call] | 0;
      const social = (curr[o + k.anim] | 0) || (call > 0 ? Anim.Wave : 0);
      const role = curr[o + k.role] | 0;
      const partner = call > 0 ? call - 1 : curr[o + k.partner] | 0;
      const outcome = curr[o + k.outcome] | 0;
      rig.emotion = curr[o + k.emotion] | 0;
      rig.socialId = social ? (this.content?.social((curr[o + k.social] | 0) - 1)?.id ?? '') : '';
      const mood = k.mood !== undefined ? curr[o + k.mood] : 1;
      rig.warmth = clamp((mood - MOOD_COLD) / (MOOD_WARM - MOOD_COLD), 0, 1);
      // What the Sim is doing (sim-core's `object` / `action` fields; older layouts: guessed from position).
      let useObj: PlacedObject | null | undefined;
      let tag = '';
      let usingId = -1;
      if (hasAction) {
        const objId = curr[o + k.object];
        usingId = objId | 0;
        const act = curr[o + k.action];
        useObj = objId >= 0 ? (this.objectById.get(objId) ?? null) : null;
        tag = act >= 0 ? (this.actionTags[act | 0] ?? '') : '';
      }

      // Position / speed / facing.
      const appeared = !rig.visible || stalled;
      if (appeared) {
        rig.x = tx;
        rig.z = tz;
        rig.yaw = tyaw;
        rig.speed = 0;
        rig.lastPose = pose;
        // (Nor on the way off a seat they left before: `riseK`.)
        rig.riseK = 1;
      }
      // Sitting or lying down: where they stood, and getting into bed, whether from its side (`sideIn`).
      if ((pose === Pose.Sit || pose === Pose.Lie) && rig.lastPose === Pose.Stand && !appeared) {
        [rig.fromX, rig.fromZ] = [rig.x, rig.z];
        rig.sideYaw = pose === Pose.Lie ? this.sideIn(rig, useObj ?? this.objectAt(tx, tz), tx, tz) : NaN;
      } else if (pose === Pose.Stand || appeared) rig.sideYaw = NaN;
      // (Into bed from its side: onto its edge first, facing out, then in to the slot lying back.)
      const edge = pose === Pose.Lie && !Number.isNaN(rig.sideYaw) && gettingIn(rig) ? rig.sideOut : 0;
      const gx = tx + Math.sin(rig.sideYaw) * edge || tx;
      const gz = tz + Math.cos(rig.sideYaw) * edge || tz;
      const dx = gx - rig.x;
      const dz = gz - rig.z;
      const dist = Math.hypot(dx, dz);
      if (dt > 0) {
        const inst = dist / dt;
        rig.speed += (Math.min(inst, 20) - rig.speed) * (1 - Math.exp(-dt * 10));
      }
      if (moving || dist > SNAP_DISTANCE || appeared) {
        rig.x = gx;
        rig.z = gz;
        rig.walkT = -1;
      } else {
        // A step sim-core takes in one tick, walked (`WALK_IN`): drawn along the way (`walkPlace`), the Sim already there.
        if (rig.walkT < 0 && dist > WALK_IN.min && this.walksIn(rig, i, pose, useObj, gx, gz)) this.startWalk(rig, pose, useObj, gx, gz, tyaw);
        if (rig.walkT >= 0) {
          rig.x = gx;
          rig.z = gz;
        } else {
          // Settle into seats / beds / next to objects instead of popping.
          const f = 1 - Math.exp(-dt * 9);
          rig.x += dx * f;
          rig.z += dz * f;
        }
      }
      let faceYaw = tyaw;
      // Into bed from its side: facing out on its edge, then turned to lie along it as they lie back; out
      // of it to the side sim-core turned them, turned from lying to that as they sit up and rise.
      let turned: number | null = null;
      if (edge) faceYaw = rig.sideYaw;
      else if (pose === Pose.Lie && !Number.isNaN(rig.sideYaw) && rig.cur.key === 'lie' && rig.prev.key === 'sitDown' && rig.w < 1) turned = lerpAngle(rig.sideYaw, tyaw, smooth(rig.w));
      else if (pose === Pose.Stand && rig.fromBed && rig.cur.key === 'standUp' && rig.w < 1) turned = lerpAngle(rig.lieYaw, tyaw, smooth(rig.w));
      // (And on the frame they get up, before `choose` starts it: turned already, the lying body would swing round by
      // that frame's turn and back.)
      else if (pose === Pose.Stand && rig.lastPose === Pose.Lie && rig.bed && !rig.upperBunk && !moving && !appeared) turned = rig.lieYaw;
      // A bed lain on across (`BEDS` `across`): turned a quarter, feet to its right.
      rig.lieAcross = pose === Pose.Lie && !!BEDS[(useObj ?? this.objectAt(rig.x, rig.z))?.def ?? '']?.across;
      if (rig.lieAcross) faceYaw = wrapAngle(tyaw + Math.PI / 2);
      else if (pose === Pose.Sit) {
        const seat = this.seatUnder(rig, useObj)?.seat;
        // A seat facing the object's back (`SEATS` `back`, content `seat.facing`): turned round, facing into it (a
        // dressing table's mirror, an upright piano); or along it (`Seat.along`: a bath), turned a quarter.
        if (seat?.back) faceYaw = wrapAngle(tyaw + Math.PI);
        else if (seat?.along) faceYaw = wrapAngle(tyaw + (seat.along * Math.PI) / 2);
        else if (seat?.shade) faceYaw = this.shadeSide(rig, tyaw);
      }
      if (pose !== Pose.Sit) rig.shadeYaw = NaN;
      // Up against a thing at a lane, squared to it (`StepIn.square`).
      // (From the side they use it from: `sides`, the foosball's far player facing the near one.)
      if (pose === Pose.Stand && useObj && STEP_IN[useObj.def]?.square) faceYaw = wrapAngle(this.stepFacing(rig, useObj, STEP_IN[useObj.def]) + Math.PI);
      // Cuddling: turned to the baby in the crib.
      const cuddled = rig.cuddleT >= 0 && rig.cuddleBaby >= 0 ? this.sims[rig.cuddleBaby] : null;
      if (cuddled) faceYaw = Math.atan2(curr[H + cuddled.id * S + k.x] - rig.x, this.groundRow(curr[H + cuddled.id * S + k.z]) - rig.z);
      // Lifting a baby out of the cot to carry or to change them: turned to the baby.
      else if (!moving && !Number.isNaN(rig.faceX)) faceYaw = Math.atan2(rig.faceX - rig.x, rig.faceZ - rig.z);
      let partnerRig = partner >= 0 && partner < count ? this.sims[partner] : null;
      // Listening in (role 3, a group conversation): `partner` started it; they look at whichever
      // of the two is speaking, and stand the way sim-core turned them (to the middle of the group).
      const listening = role === 3;
      if (listening && partnerRig) {
        const other = curr[H + partner * S + k.partner] | 0;
        if (other >= 0 && other < count && !speaking(Math.min(partner, other), 1, time)) partnerRig = this.sims[other];
      }
      // The two talking face each other; with others listening in, they open up to the group: the
      // body faces its middle (as sim-core turns them), the head and eyes stay on the one speaking,
      // and while speaking themselves they glance round at those listening now and then.
      const leader = role === 1 ? i : role === 2 ? partner : -1;
      const listeners = leader >= 0 && leader < count ? groups[leader * 3] : 0;
      rig.audience = null;
      if (social && !listening && partnerRig && partnerRig.visible && !moving && pose === Pose.Stand) {
        if (listeners) {
          faceYaw = Math.atan2(middle[leader * 2] - rig.x, middle[leader * 2 + 1] - rig.z);
          if (speaking(Math.min(i, partner), role, time)) {
            const pick = Math.floor(hash(Math.floor(time / 1.4 + rig.seed * 3) * 7 + i) * (listeners + 1));
            if (pick > 0) rig.audience = this.sims[groups[leader * 3 + pick]] ?? null;
          }
        } else faceYaw = Math.atan2(partnerRig.x - rig.x, partnerRig.z - rig.z);
      }
      // Walking in: along the way, then turned to the way they end up facing.
      if (rig.walkT >= 0) {
        rig.walkYaw = faceYaw;
        const ahead = this.walkAhead(rig);
        if (!Number.isNaN(ahead)) faceYaw = ahead;
      }
      const dyaw = wrapAngle(faceYaw - rig.yaw);
      // Walkers turn as quickly as the game runs (up to 10x): sped up, the corners of their way come sooner.
      const turn = moving ? clamp(gameRate, 1, 10) : 1;
      const maxTurn = (moving ? 9 : 6) * dt * turn;
      // Standing free, a wide turn steps round (`turnOnSpot`): the yaw follows the turn clip's steps. (To the way the
      // snapshot turns them, not the way the frames between snapshots have got to: sized for the whole turn.)
      const goal = faceYaw === tyaw ? curr[o + k.yaw] : faceYaw;
      const onSpot = dir || turned !== null ? null : turnOnSpot(set, rig, this.standsFree(rig, i, pose, moving, social, useObj, appeared), goal, rig.body.stage === 'child');
      rig.yaw = dir ? wrapAngle(faceYaw) : turned !== null ? wrapAngle(turned) : onSpot !== null ? onSpot : wrapAngle(rig.yaw + clamp(dyaw * (1 - Math.exp(-dt * 12 * turn)), -maxTurn, maxTurn));
      rig.visible = true;
      this.visible[i] = 1;
      rig.tx = cx;
      rig.tz = cz;

      // What it carries (`items` index + 1), and the arms that carry it.
      const held = k.held !== undefined ? curr[o + k.held] | 0 : 0;
      const item = held > 0 && layout.items ? this.content?.item(layout.items[held - 1] ?? '') : undefined;
      // (Working at a station, what they carry is put down: `SET_DOWN`.)
      const setDown = item && !moving && useObj && k.recipe !== undefined && curr[o + k.recipe] > 0 ? SET_DOWN[tag] : undefined;
      const carry = item && !setDown ? (item.hold === 'both' ? 'carry' : 'carryOne') : '';

      // Activity → clip.
      rig.useId = usingId;
      this.choose(rig, i, pose, moving, social, role, outcome, partnerRig, time, dt * animRate, useObj, tag, carry, dt, appeared);
      rig.lastPose = pose;

      // Advance and blend.
      const step = dt * animRate;
      advance(rig.cur, GAITS.has(rig.cur.key) || rig.cur.key === 'jog' ? dt : step);
      advance(rig.prev, GAITS.has(rig.prev.key) || rig.prev.key === 'jog' ? dt : step);
      advance(rig.upper, step);
      advance(rig.upperPrev, step);
      if (rig.turnKey && rig.turnStarted) rig.yaw = turnYaw(rig);
      // Carrying while walking: the arms bob in step with the feet.
      if (rig.cur.clip && GAITS.has(rig.cur.key) && rig.upper.clip && rig.upper.key.startsWith('carry')) rig.upper.time = (rig.cur.time / rig.cur.clip.duration) * rig.upper.clip.duration;
      rig.w = Math.min(1, rig.w + dt * rig.fadeRate * Math.max(1, animRate));
      rig.upperFade = Math.min(1, rig.upperFade + dt * UPPER_FADE_RATE * Math.max(1, animRate));
      rig.upperW += clamp(rig.upperTarget - rig.upperW, -dt * 4, dt * 4);
      if (dir?.still) {
        rig.cur.time = dir.time ?? rig.cur.time;
        rig.w = 1;
        rig.upperW = 0;
      }

      samplePlayer(set, rig.cur, this.qA, this.pA);
      const w = smooth(rig.w);
      if (w < 1 && rig.prev.clip) {
        samplePlayer(set, rig.prev, this.qB, this.pB);
        blendInto(this.qB, this.pB, this.qA, this.pA, w, null);
        this.qA.set(this.qB);
        this.pA.set(this.pB);
      }
      if (rig.upperW > 0.001 && rig.upper.clip) {
        samplePlayer(set, rig.upper, this.qB, this.pB);
        // A new upper clip fades in over the one it replaced, as the lower pair does: at full layer weight, a
        // straight swap would turn an arm by the whole difference in one frame.
        // (`srcQ` / `srcP` alias the scratch below: `blendInto` writes into the pair it is given first.)
        let srcQ = this.qB;
        let srcP = this.pB;
        const f = smooth(rig.upperFade);
        if (f < 1 && rig.upperPrev.clip) {
          samplePlayer(set, rig.upperPrev, this.qC, this.pC);
          blendInto(this.qC, this.pC, this.qB, this.pB, f, null);
          srcQ = this.qC;
          srcP = this.pC;
        }
        blendInto(this.qA, this.pA, srcQ, srcP, smooth(rig.upperW), this.upperMask);
      }

      if (this.force === 'rest') {
        this.qA.set(set.sourceRestQ);
        this.pA.set(set.sourcePelvis);
      }
      // Clothes for what they're doing: pyjamas in bed for the night, a towel in the bath ...
      let wear = this.wearFor(rig, pose, moving || rig.walkT >= 0 || dist > WALK_IN.min, w, useObj, tag);
      // (Out of the shower in the towel: dressed once they're out, `toweled`.)
      if (wear === 'towel' && tag === 'shower') [rig.toweled, rig.shower] = [0, useObj ?? null];
      else if (rig.toweled >= 0) {
        rig.toweled = moving || (rig.walkT < 0 && rig.toweled > TOWEL_OUT) ? -1 : rig.walkT >= 0 ? 0 : rig.toweled + dt * animRate;
        if (rig.toweled >= 0 && pose === Pose.Stand) wear = 'towel';
      }
      const worn = rig.outfits?.[wear];
      if (worn && worn !== rig.worn) this.wear(rig, i, worn);
      // Hand pose: the clips clench fists; open them according to the activity.
      blendInto(this.qA, this.pA, set.sourceRestQ, this.pA, rig.hands, this.fingerMask);

      // Procedural layers.
      resetAdditive(this.add);
      this.procedural(rig, i, social, role, partnerRig, pose, moving, time, dt);
      // A baby in someone's arms: knees drawn up a little (the baby's lying clips have them drawn up already).
      const holder = this.heldBy[i] >= 0 ? this.sims[this.heldBy[i]] : null;
      const inArms = holder?.visible ? smooth(holder.liftW) : 0;
      if (inArms > 0) {
        const B = set.bone;
        const k = inArms * (rig.cur.key === 'lie' ? 1 : 0.3);
        addRotation(this.add, B.thigh_l, 1, 0, 0, -0.55 * k);
        addRotation(this.add, B.thigh_r, 1, 0, 0, -0.4 * k);
        addRotation(this.add, B.calf_l, 1, 0, 0, 0.8 * k);
        addRotation(this.add, B.calf_r, 1, 0, 0, 0.65 * k);
      }

      // Placement (root offsets blend with the clips).
      const rootY = lerp(rig.prev.rootY, rig.cur.rootY, w);
      const rootFwd = lerp(rig.prev.rootFwd, rig.cur.rootFwd, w) + rig.approach;
      const pitch = lerp(rig.prev.rootPitch, rig.cur.rootPitch, w);
      // (At the Sim, or a clip placed in a frame of its own: the bunk's ladder.)
      const at: Frame = rig.cur.frame || rig.prev.frame ? this.framed(rig, w) : rig;
      const sy = Math.sin(at.yaw / 2);
      const cyw = Math.cos(at.yaw / 2);
      const sp = Math.sin(pitch / 2);
      const cp = Math.cos(pitch / 2);
      // q = yaw(Y) * pitch(X)
      const pl = this.place;
      pl.qx = cyw * sp;
      pl.qy = sy * cp;
      pl.qz = -sy * sp;
      pl.qw = cyw * cp;
      pl.x = at.x + Math.sin(at.yaw) * rootFwd;
      pl.y = rootY + rig.floorY;
      pl.z = at.z + Math.cos(at.yaw) * rootFwd;
      this.stepPlace(rig, pl, step);
      if (rig.walkT >= 0) this.walkPlace(rig, pl, at.x + Math.sin(rig.walkYaw) * rootFwd, at.z + Math.cos(rig.walkYaw) * rootFwd, step);
      if (pose === Pose.Sit) {
        [rig.sitX, rig.sitZ] = [pl.x, pl.z];
        rig.riseK = 1;
      } else if (rig.rising && rig.cur.key.startsWith('standUp')) {
        // Rising off a seat where it is, then stepping off to the tile sim-core put them on (`rising`), round a
        // built-in chair's side to a tile behind it (`riseViaX`).
        rig.riseK = smooth(clamp((rig.cur.time / rig.cur.clip!.duration - 0.25) / 0.5, 0, 1));
        alongWay(pl, rig.riseX, rig.riseZ, rig.riseViaX, rig.riseViaZ, pl.x, pl.z, rig.riseK);
      } else if (rig.riseK < 1) {
        // A rise cut short (walking off before it was done): the rest of the way to the tile from where it had got,
        // or the body would cross it in one frame (`blend.test.ts`).
        rig.riseK = Math.min(1, rig.riseK + dt * RISE_CATCH_UP * Math.max(1, animRate));
        alongWay(pl, rig.riseX, rig.riseZ, rig.riseViaX, rig.riseViaZ, pl.x, pl.z, rig.riseK);
      }
      pl.scale = rig.scale;
      pl.head = rig.head;
      pl.build = rig.build;
      // Sitting or lying on a seat that moves (a rocking chair, a swing): along with it.
      const ride = (pose === Pose.Sit || pose === Pose.Lie) && usingId >= 0 && this.rideOf ? this.rideOf(usingId) : null;
      if (ride) this.riding(ride);
      const pad = this.onPadOf[i];
      if (pad) onPad(this.place, this.cradleQ, rig, pad);
      if (inArms > 0) cradled(this.place, this.cradleQ, this.cradle, rig, this.sims[this.heldBy[i]]!, this.heldBy[i], inArms);
      skin(set, rig.body, this.qA, this.pA, this.add, pl, this.scratch, this.data, this.row(i), this.headTmp);
      this.writeSit(rig, i);
      if (rig.cuddleBaby >= 0 && rig.liftW > 0) writeCradle(set.bone, this.scratch.worldT, this.cradle, i, rig);
      this.writeBlanket(rig, i, pose, dt, appeared);
      if (setDown) this.writeSetDown(rig, i, setDown === 'top' ? item : undefined, tag, useObj);
      else this.writeCarried(rig, i, item);
      this.writeHeld(rig, i, pose, moving);
      this.writeTool(rig, i, moving ? '' : tag, useObj);

      // Head top (speech bubbles, selection marker): a little above the head joint.
      const hs = rig.scale * rig.head;
      this.heads[i * 3] = this.headTmp[0];
      this.heads[i * 3 + 1] = this.headTmp[1] + (rig.body.headAbove + HEAD_MARGIN) * hs;
      this.heads[i * 3 + 2] = this.headTmp[2];

      // Contact shadow under standing / sitting Sims.
      // (None on the bunk's ladder: the root is in the bed.)
      const r = pose === Pose.Lie || at !== rig ? 0 : 0.62 * hs;
      const m = i * 16;
      shadows.fill(0, m, m + 16);
      shadows[m] = r;
      shadows[m + 5] = 1;
      shadows[m + 10] = r;
      // (Under them inside a thing they use from inside, and on its floor: `STEP_IN`; rising off a seat, and the rest
      // of a rise cut short: `riseK`.)
      const stepped = !!rig.inside || rig.outT >= 0 || rig.walkT >= 0 || (rig.rising && rig.cur.key.startsWith('standUp')) || rig.riseK < 1;
      shadows[m + 12] = stepped ? pl.x : rig.x;
      shadows[m + 13] = 0.02 + (stepped ? pl.y : rig.floorY);
      shadows[m + 14] = stepped ? pl.z : rig.z;
      shadows[m + 15] = 1;
    }
    if (this.ghosts.length) this.poseGhosts(count, dt, time);
    this.faceDetail(count + this.ghosts.length);
    if (this.shadowsBare) this.bareShadows();
    this.writeRing(this.reducedMotion ? 0 : now * 0.001);
    for (let i = count + this.ghosts.length; i < MAX_CHARACTERS; i++) {
      if (!this.visible[i]) continue;
      this.visible[i] = 0;
      this.data.fill(0, this.row(i), this.row(i) + NB * 16);
      shadows.fill(0, i * 16, i * 16 + 16);
    }
    for (const b of this.blankets.values()) {
      // (Nobody under it now or last frame: nothing to upload.)
      if (!b.drawn && !b.shown) continue;
      b.shown = b.drawn;
      this.view.quiltRows(b.quilt, b.drawn, true);
    }
    this.props.forEachBuilt(this.showProp);
    this.pullChairs();
    this.upload();
    this.updateMs += (performance.now() - started - this.updateMs) * 0.1;
  }

  // --- activity ------------------------------------------------------------------------

  /**
   * Picks the clip for this frame and starts a crossfade when it changes. `useObj` / `tag` come
   * from the snapshot (`undefined` with older layouts: the object is guessed from the position).
   */
  private choose(
    rig: SimRig,
    i: number,
    pose: number,
    moving: boolean,
    social: number,
    role: number,
    outcome: number,
    partner: SimRig | null,
    time: number,
    step: number,
    useObj: PlacedObject | null | undefined,
    tag: string,
    /** The upper-body clip for what the Sim carries ('' = nothing). */
    carry: string,
    /** Seconds since the last frame (0 while paused). */
    dt: number,
    /** Drawn again (back in view, on the storey shown, home from away): no fading in from what they did when last seen. */
    fresh: boolean,
  ): void {
    const pair = partner ? Math.min(i, partner.id) : i;
    const known = useObj !== undefined;
    const act = ACTIONS[tag];
    rig.tag = tag;
    rig.tagKnown = known;
    rig.feetUp = false;
    rig.actTime += step;
    rig.stepTime += step;
    let key = 'idle';
    let rootY = 0;
    let rootFwd = 0;
    let pitch = 0;
    let repeat = false;
    let fade = DEFAULT_FADE;
    let upper = '';
    let rate = 1;
    let hands: number = (typeof act?.hands === 'function' ? act.hands(useObj?.def ?? '') : act?.hands) ?? HANDS.relaxed;
    // Step towards a partner (hugs, kisses, fights): eased towards `reach` at `reachRate`.
    let reach = 0;
    let reachRate = 4;
    // Using a thing from inside or up against it this frame (`stepIn`).
    let inside = false;
    // Placed in a frame of its own this frame (`Player.frame`: the bunk's ladder).
    let frame: Frame | null = null;

    if (this.force) {
      key = this.force;
    } else if (pose === Pose.Lie) {
      const obj = (known ? useObj : null) ?? this.objectAt(rig.x, rig.z);
      const bed = (obj && BEDS[obj.def]) || DEFAULT_BED;
      rig.bed = obj;
      // A bunk's second sleeper (a hair to the right of its centre, see sim-core `slot_position`) is on top
      // (by where the snapshot puts them: the drawn place may still be on its way there).
      const bunk = obj && BUNKS[obj.def];
      const right = (x: number, z: number) => (obj ? (x - obj.cx) * Math.cos((obj.rot * Math.PI) / 2) - (z - obj.cz) * Math.sin((obj.rot * Math.PI) / 2) : 0);
      const across = right(rig.x, rig.z);
      rig.upperBunk = !!bunk && right(rig.tx, rig.tz) > 0;
      rig.bedH = (bunk ? bunk[rig.upperBunk ? 1 : 0] : bed.h) + (obj?.lift ?? 0);
      const climb = rig.upperBunk && obj ? this.sizedClip(rig, BUNK_LADDER.up) : '';
      // (Lain on across, they face along it: moved back to its middle from a slot to one side.)
      rig.bedFwd = (bed.fwd ?? 0) - (rig.lieAcross ? across : 0);
      rig.useDef = obj?.def ?? '';
      rig.useObj = obj ?? null;
      if (climb && ((rig.lastPose === Pose.Stand && rig.cur.key !== climb) || (rig.cur.key === climb && rig.cur.time < rig.cur.clip!.duration - 0.1))) {
        // Into the upper bunk: up its ladder from the tile in front, lain down at the end (`BUNK_LADDER`).
        key = climb;
        rootY = obj!.lift ?? 0;
        fade = BUNK_LADDER.fade;
        frame = this.bedFrame(obj!);
        if (rig.cur.key !== climb) rig.actTime = 0;
      } else if ((rig.lastPose === Pose.Stand && rig.cur.key !== 'sitDown') || (rig.cur.key === 'sitDown' && rig.cur.time < rig.cur.clip!.duration - 0.35)) {
        // Getting in: sit down on the edge first; the crossfade to lying then leans the Sim back
        // (the root pitch blends with the clips).
        const sp = this.sitPelvis.get(rig.body.name)!;
        key = 'sitDown';
        rootY = rig.bedH - sp.y * rig.scale;
        // (From its side: on its edge, where they are drawn, facing out: `sideIn`.)
        rootFwd = (Number.isNaN(rig.sideYaw) ? LIE_ENTRY * rig.scale + rig.bedFwd : 0) - sp.z * rig.scale;
        fade = 0.35;
        if (rig.cur.key !== 'sitDown') rig.actTime = 0;
      } else if (rig.body.stage === 'baby') {
        // A baby: on the back in the middle of the crib, curled up asleep or kicking awake (its own
        // clips, made standing like `lie` and turned flat the same way).
        key = !known || tag === 'sleep' || tag === 'nap' ? BABY_LIE.asleep : BABY_LIE.awake;
        rootY = rig.bedH + lieBack(rig);
        rootFwd = lieCentre(rig);
        pitch = -Math.PI / 2;
        fade = LYING.has(rig.cur.key) ? BABY_LIE_FADE : 0.45;
        rig.lieX = rig.x;
        rig.lieZ = rig.z;
      } else {
        // On the back, head towards the head of the bed (local -z), centred on the slot: the
        // `lie` clip (made standing) turned flat and slowed down.
        key = obj && CURLED_BEDS.has(obj.def) ? 'lieCurl' : obj && SIDE_BEDS.has(obj.def) ? 'lieSide' : 'lie';
        rootY = rig.bedH + lieBack(rig);
        rootFwd = lieCentre(rig) + rig.bedFwd;
        pitch = -Math.PI / 2;
        rate = 0.3;
        // In a hammock: down in the sling's dip, the head up on the pillow (`inSling`).
        const at = bed.sling ? this.inSling(rig, bed.sling, obj?.lift ?? 0, rootFwd) : null;
        rig.sling = at ? bed.sling! : null;
        if (at) [rootY, rootFwd] = at;
        fade = rig.cur.key === 'sitDown' ? 0.9 : rig.cur.key.startsWith('bunkUp') ? BUNK_LADDER.lie : 0.45;
        // (Where it lies, for getting up: by then the Sim has already moved off the bed.)
        rig.lieX = rig.x;
        rig.lieZ = rig.z;
        rig.lieYaw = rig.yaw;
      }
      hands = tag === 'lift' ? HANDS.grip : HANDS.open;
    } else if (pose === Pose.Sit) {
      const obj = known ? this.seatUnder(rig, useObj) : this.objectAt(rig.x, rig.z);
      const seat = obj?.seat ?? DEFAULT_SEAT;
      // A seat on the floor (a floor cushion) is sat on cross-legged, by its own clips.
      const floor = floorSit(seat.h - (obj?.lift ?? 0));
      rig.floorSat = floor;
      const sp = (floor ? this.floorPelvis.get(rig.body.name)?.get(floor) : this.sitPelvis.get(rig.body.name))!;
      rootY = seat.h - sp.y * rig.scale;
      // (Facing the object's back, its forward is the Sim's back.)
      rootFwd = (seat.back ? -seat.fwd : seat.fwd) - sp.z * rig.scale + (floor ? 0 : this.perch(rig, seat, sp));
      fade = 0.35;
      rig.useDef = obj?.def ?? '';
      rig.useObj = obj ?? null;
      const ride = known && obj ? RIDES[obj.def] : undefined;
      if (social) key = speaking(pair, role, time) ? 'sitTalk' : 'sit';
      else if (ride) {
        // Astride a ride, the hands on its grips (`RIDES`).
        key = this.sizedClip(rig, ride.sizes);
        hands = HANDS.grip;
      } else if (act?.sit) key = typeof act.sit === 'function' ? act.sit(rig.useDef) : act.sit;
      else if (!known && obj && SIT_HANDS.has(obj.def)) key = 'sitHands';
      else key = 'sit';
      if (floor) key = SIT_CLIPS[floor].sit;
      const down = SIT_CLIPS[floor].down;
      // (Walked in to the seat first, `WALK_IN`: sitting down once there.)
      if ((rig.lastPose === Pose.Stand || rig.walkDown) && rig.cur.key !== down) {
        key = down;
        rig.actTime = 0;
      } else if (rig.cur.key === down && rig.cur.time < rig.cur.clip!.duration - 0.3) {
        key = down;
      }
      rootFwd += this.tuckIn(rig, obj, seat, key !== down, step, fresh);
      // (At a low top, a kids' chair at a coffee table: bent over it, the size for their height.)
      if (key === 'sitHands' && rig.tuckLow) key = this.sizedClip(rig, SIT_LOW);
      else if (key === 'sitHands' && rig.tuckFar) key = 'sitHandsFar';
      // Feet up on a footstool in front, slid forward in the seat as far as that takes.
      if (!social && key !== down && !seat.legsOut && !seat.pedals && this.footstoolFor(rig, key, rootY, rootFwd)) {
        reach = rig.feet[3];
        reachRate = 1.5;
      }
      if (!social) {
        const up = known ? (typeof act?.upper === 'function' ? act.upper(rig.useDef) : act?.upper) : obj?.def === 'diningTable' ? 'eat' : undefined;
        // Eating / drinking now and then; holding a book or phone throughout.
        if (up === 'eat' || up === 'drink') upper = Math.floor((rig.actTime + rig.seed * 9) / 4) % 3 === 0 ? up : '';
        else if (up) upper = up;
      }
    } else if (moving) {
      // Walking, the cycle matched to the ground speed; faster than the walk can step (a sped-up
      // game, short legs), jogging along, and faster than that running (`GEARS`): the slowest gait
      // that needn't go faster than its pace, a slower one than now only well within it.
      // (The run's beat quicker on shorter legs: `runMaxRate`.)
      const gait = gaitFor(rig.speed, rig.body.pelvisScale * rig.scale, GEARS.indexOf(rig.cur.key), this.gaitSpeed);
      key = GEARS[gait.gear];
      rate = gait.rate;
      fade = GAITS.has(rig.cur.key) ? 0.3 : 0.2;
      hands = HANDS.relaxed;
      rig.useObj = null;
    } else if (social) {
      const me = speaking(pair, role, time);
      hands = HANDS.relaxed;
      // Done with the thing they used: its prop (a guitar, a mop) isn't in their hands while they talk.
      rig.useObj = null;
      switch (social) {
        case Anim.Talk:
        case Anim.Laugh:
        case Anim.Flirt:
        case Anim.Argue: {
          // The social's own gestures and how it's taken (`GESTURES`).
          const g = gesturesOf(social, rig.socialId);
          const nod = hash(Math.floor(time / 3) + i) < 0.4;
          const listen = g.listen ?? (outcome === 1 && nod ? 'yes' : outcome === 2 && nod ? 'no' : 'idle');
          if (role === 1) key = outcome === 2 && g.flop ? g.flop : me ? (g.speak ?? 'talk') : outcome === 1 && g.good === 'laugh' ? 'laugh' : listen;
          else key = outcome === 2 && g.bad ? g.bad : outcome === 1 && g.good && (!me || g.good === 'laugh') ? g.good : me ? (g.answer ?? 'talk') : listen;
          // (The chat's talk and head shakes, quicker in an argument.)
          if (social === Anim.Argue && (key === 'talk' || key === 'no')) rate = 1.25;
          break;
        }
        case Anim.Fight: {
          // The attacker steps in with each punch, the defender gives ground as it lands
          // (eased like the hug's approach).
          const beat = (time * 1.1) % 1;
          const lunge = Math.sin(beat * Math.PI);
          reach = role === 1 ? 0.04 + 0.1 * lunge : -0.03 - 0.09 * lunge;
          reachRate = 8;
          key = role === 1 ? (Math.floor(time * 1.1) % 2 ? 'punchCross' : 'punchJab') : 'hit';
          repeat = true;
          hands = HANDS.grip;
          break;
        }
        case Anim.Hug:
        case Anim.Kiss:
          if (outcome === 2) {
            // Refused (`REFUSED`): leaning away, a hand up, a step back; the one who asked crestfallen.
            key = role === 1 ? 'rebuffed' : 'refuse';
            reach = role === 1 ? 0 : -REFUSED.back;
            reachRate = 3;
            break;
          }
          // Arms around each other (`procedural`), stepped in until the chests meet (a kiss: the faces).
          key = 'idle';
          reach = embraceStep(rig, partner, social === Anim.Hug ? 0 : EMBRACE.kiss);
          reachRate = 6;
          break;
        case Anim.Wave:
          // Goodbye: both wave, again and again while it lasts.
          key = 'wave';
          repeat = true;
          break;
        default:
          key = me ? 'talk' : 'idle';
      }
      if (social !== Anim.Talk && social !== Anim.Fight) rig.actTime = 0;
    } else {
      // Standing object use: the action's clip sequence (or, without action data, the object in front).
      const obj = known ? useObj : this.objectInFront(rig);
      rig.useDef = obj?.def ?? '';
      rig.useObj = obj ?? null;
      // Used from inside or up against it (`STEP_IN`): the clip for their height.
      // (With every lane taken by others, from where sim-core stood them: three at the bus stop's timetable.)
      const entry = known && obj && (!STEP_IN[obj.def]?.tag || STEP_IN[obj.def].tag === tag) ? STEP_IN[obj.def] : undefined;
      const into = entry && obj && (rig.inside === entry || !Number.isNaN(this.freeLane(rig, entry, obj))) ? entry : undefined;
      const fitted = into && obj ? this.stepSeq(rig, into, obj) : undefined;
      // (Bouncing and vibrating on the equipment, content `stand`: the idle with `activityPose` on top.)
      const seq = fitted?.length
        ? fitted
        : known
        ? obj && HUGGED[obj.def]
          ? CROUCH_ONLY
          : obj?.held?.grip === 'guitar'
          ? this.guitarReach(rig) < 0.5
            ? STRUM_SHORT
            : STRUM
          : obj && GRIPPED[obj.def]
          ? this.gripped(rig, GRIPPED[obj.def])
          : obj?.stand?.motion || obj?.held?.grip === 'pair'
          ? IDLE_ONLY
          : obj?.held?.grip === 'pole'
            ? PUSH_ONLY
          : act?.stand
            ? typeof act.stand === 'function'
              ? act.stand(useObj?.def ?? '', rig.body.height * rig.scale)
              : act.stand
            : undefined
        : obj
          ? USE[obj.def]
          : undefined;
      const useKey = seq ? `${tag}:${obj?.def ?? ''}` : '';
      if (seq && seq.length) {
        if (rig.useKey !== useKey) {
          rig.useKey = useKey;
          rig.useSeq = seq;
          rig.seq = 0;
          rig.actTime = 0;
          rig.stepTime = 0;
        }
        key = seq[rig.seq % seq.length];
        repeat = true;
        // Up to the crib's rail to pick the baby up, back from it holding them (`cuddles`).
        if (tag === 'cuddle') {
          reach = rig.cuddleStep;
          reachRate = 6;
        } else if (obj && STAND_BACK[obj.def]) {
          // Back from the door it opens (or in to a small thing), eased like the hug's approach.
          reach = -STAND_BACK[obj.def];
        } else if (obj && HUGGED[obj.def]) {
          // Up to the teddy, to hug it.
          reach = HUGGED[obj.def].reach;
        }
        const on = obj?.stand;
        // (Holding its grips: `GRIPPED`.)
        if (obj && GRIPPED[obj.def]) hands = HANDS.grip;
        if (on && obj && tag !== 'cuddle') {
          // Up on the equipment (content `stand`; Sims use it from the tile in front): forward to its
          // middle, on what they stand on (a treadmill's belt, facing the console).
          rootFwd = Math.hypot(obj.cx - rig.x, obj.cz - rig.z);
          rootY = on.h;
          if (on.motion === 'bounce') {
            // Bouncing on the mat: the feet leave it for a little under half of each bounce.
            const b = Math.sin(((rig.actTime * 1.6) % 1) * Math.PI);
            rootY += Math.max(0, b * 0.75 - 0.3) * 0.55;
          } else if (on.motion === 'buzz') {
            // On the vibration plate: a fine buzz.
            rootY += Math.sin(rig.actTime * 90) * 0.004;
          }
        }
        if (into && obj) {
          [rootFwd, rootY, key, rate] = this.stepIn(rig, obj, into, key, step, dt);
          inside = true;
        }
      } else {
        rig.useKey = '';
        rig.useSeq = null;
        // Idle with occasional variations.
        const phase = Math.floor((rig.actTime + rig.seed * 20) / 9);
        const r = hash(phase * 7 + i);
        key = rig.actTime < 6 ? 'idle' : r < 0.18 ? 'foldArms' : 'idle';
        if (!tag || tag === 'idle') hands = HANDS.rest;
      }
    }
    if (!inside && rig.inside) this.stepOut(rig, fresh);
    // Turning on the spot (`turnOnSpot`): its clip on the legs and body until it has played, then what they do. The
    // step towards a thing or a partner waits for it (stepping forward while turning would swing the body round).
    if (rig.turnKey && turnDone(rig)) rig.turnKey = '';
    if (rig.turnKey) {
      key = rig.turnKey;
      fade = TURN_FADE;
      rate = 1;
      repeat = false;
      reach = rig.approach;
    }
    rig.approach += (reach - rig.approach) * Math.min(1, step * reachRate);
    const dir = rig.dir;
    if (dir?.clip) {
      key = dir.clip;
      repeat = false;
      fade = 0.3;
      rate = 1;
      upper = '';
    }
    if (dir?.hands !== undefined) hands = dir.hands;
    // (From the floor, its own clip: `sitFloor`.)
    const up = rig.lastPose === Pose.Sit ? SIT_CLIPS[rig.floorSat].up : RISING.has(rig.cur.key) && rig.cur.key.startsWith('standUp') ? rig.cur.key : 'standUp';
    // Up from the upper bunk: down its ladder to the tile in front (`BUNK_LADDER`), sim-core waiting for them.
    const down = rig.upperBunk && rig.bed && pose === Pose.Stand && !this.force ? this.sizedClip(rig, BUNK_LADDER.down) : '';
    const climbDown = !!down && !moving && (rig.lastPose === Pose.Lie || (rig.cur.key === down && rig.cur.time < rig.cur.clip!.duration - 0.25));
    if (climbDown) {
      if (rig.cur.key !== down) rig.actTime = 0;
      key = down;
      rootY = rig.bed!.lift ?? 0;
      rootFwd = 0;
      fade = BUNK_LADDER.lie;
      repeat = false;
      upper = '';
      frame = this.bedFrame(rig.bed!);
    } else if (down) rig.upperBunk = false;
    // (Getting up out of bed this frame: `lieFwd`.)
    let outOfBed = false;
    if (climbDown) {
      // (Not standing up as from another bed.)
    } else if (!moving && pose === Pose.Stand && (rig.lastPose === Pose.Sit || rig.lastPose === Pose.Lie) && rig.cur.key !== up && !this.force) {
      // Rising from a seat (or sitting up out of bed: the crossfade from lying leans the Sim up).
      key = up;
      rig.fromBed = rig.lastPose === Pose.Lie && !!rig.bed;
      outOfBed = rig.fromBed;
      if (rig.fromBed) fade = 0.9;
      // (Off a seat: rising where it is, then stepping off to the tile: `riseX`.)
      rig.rising = rig.lastPose === Pose.Sit;
      [rig.riseX, rig.riseZ] = [rig.sitX, rig.sitZ];
      [rig.riseViaX, rig.riseViaZ] = rig.rising && rig.useObj?.builtIn ? this.cornerOf(rig.useObj, rig.sitX, rig.sitZ, rig.yaw, rig.tx, rig.tz) : [NaN, NaN];
    } else if (rig.cur.key === up && !moving && pose === Pose.Stand && rig.cur.time < rig.cur.clip!.duration - 0.25) {
      key = up;
    }
    if (key !== 'standUp') rig.fromBed = false;
    if (key !== up) rig.rising = false;
    if (pose !== Pose.Sit) rootFwd += this.tuckOut(rig, key, step);
    // Carrying something while walking or standing about (not while using a station with it).
    if (carry && pose === Pose.Stand && !upper && !dir?.clip && key !== up && (moving || !rig.useSeq) && social !== Anim.Hug && social !== Anim.Fight) {
      upper = carry;
      hands = HANDS.hold;
      if (key === 'foldArms') key = 'idle';
    }
    const fx = Math.sin(rig.yaw);
    const fz = Math.cos(rig.yaw);
    if (rig.fromBed && rig.bed) {
      // Sitting on the foot of the bed, then stepping forward onto the Sim's tile while rising.
      const bed = rig.bed;
      const half = (Math.abs(fx) * (bed.maxX - bed.minX) + Math.abs(fz) * (bed.maxZ - bed.minZ)) / 2;
      const edge = (bed.cx - rig.x) * fx + (bed.cz - rig.z) * fz + half - 0.22 * rig.scale;
      const t = rig.cur.key === 'standUp' ? rig.cur.time / rig.cur.clip!.duration : 0;
      rootFwd = edge * (1 - smooth(clamp((t - 0.3) / 0.6, 0, 1)));
    }
    // Walking in (`WALK_IN`): standing while a door opens, walking, then standing while turned to face the right way;
    // the seat's height and the use's clips once there.
    if (rig.walkT >= 0) {
      const walking = rig.walkT >= rig.walkWait && rig.walkS < rig.walkL;
      key = walking ? 'walk' : 'idle';
      rate = walking ? WALK_IN.pace * (dt > 0 ? step / dt : 1) : 1;
      repeat = false;
      fade = 0.25;
      rootY = 0;
      pitch = 0;
      upper = '';
      hands = HANDS.relaxed;
    } else if (rig.walkDown && rig.cur.key.startsWith('sitDown')) rig.walkDown = false;
    // (A little quicker at slow game speeds than the animation; still while paused.)
    rig.hands += (hands - rig.hands) * (1 - Math.exp(-(step * 5 + dt * 1.2)));

    // Sequenced object use: the next clip when a one-shot ends, or after LOOP_STEP for loops.
    const cur = rig.cur;
    const seq = rig.useSeq;
    const done = cur.clip && (cur.clip.loop ? rig.stepTime >= Math.max(LOOP_STEP, cur.clip.duration) : cur.time >= cur.clip.duration - 0.2);
    if (cur.key === key && repeat && seq && done) {
      rig.seq++;
      rig.stepTime = 0;
      key = seq[rig.seq % seq.length];
      // The same one-shot again restarts it (crossfaded); a loop simply continues.
      if (key !== cur.key || !cur.clip!.loop) this.start(rig, key, 0.3, rate, repeat, rootY, rootFwd, pitch);
    } else if (cur.key === key && repeat && done && !cur.clip!.loop) {
      // Repeated one-shots (fights).
      this.start(rig, key, 0.3, rate, repeat, rootY, rootFwd, pitch);
    } else if (cur.key !== key || (key === rig.turnKey && !rig.turnStarted)) {
      // (A new turn starts its clip afresh, even over the end of the same one.)
      rig.stepTime = 0;
      this.start(rig, key, fade, rate, repeat, rootY, rootFwd, pitch);
    } else {
      cur.rate = rate;
      cur.rootY = rootY;
      cur.rootFwd = rootFwd;
      cur.rootPitch = pitch;
    }
    rig.cur.frame = frame;
    if (key === rig.turnKey) rig.turnStarted = true;
    // A ride in time with its loop (`RIDES` `sync`: the swing's pumping legs).
    if (pose === Pose.Sit && key === rig.cur.key && rig.useId >= 0 && RIDES[rig.useDef]?.sync) {
      const phase = this.loopOf?.(rig.useId);
      if (phase != null) rig.cur.time = phase * rig.cur.clip!.duration;
    }
    // (Down the bunk's ladder from lying: the lying body fading out lies where it lay, in the bed's frame.)
    if (frame && LYING.has(rig.prev.key) && rig.lastPose === Pose.Lie) rig.prev.frame = frame;

    // The lying body fading out lies where it lay as the Sim moves off the bed: its root as it was when they got up
    // (`lieFwd`: the lie's, or a held pose's where a lie still fading in was cut short; the lie's alone would move
    // that one 7 cm in a frame, `blend.test.ts`).
    if (rig.fromBed && LYING.has(rig.prev.key)) {
      if (outOfBed) rig.lieFwd = rig.prev.rootFwd;
      rig.prev.rootFwd = (rig.lieX - rig.x) * fx + (rig.lieZ - rig.z) * fz + rig.lieFwd;
    }

    // Upper-body layer.
    if (upper) {
      if (rig.upper.key !== upper) {
        // The clip it replaces keeps playing under the new one, faded out as `upperFade` rises; a fade cut short
        // holds the pose it had reached, as the lower pair's does (`start`).
        if (rig.upperFade < 1 && rig.upperPrev.clip && rig.upper.clip) this.hold(rig.upperPrev, rig.upper, smooth(rig.upperFade));
        else copyPlayer(rig.upper, rig.upperPrev);
        rig.upperFade = 0;
        rig.upper.key = upper;
        rig.upper.clip = this.clip(upper);
        rig.upper.time = 0;
        rig.upper.rate = 1;
      }
      rig.upperTarget = 1;
    } else {
      rig.upperTarget = 0;
    }

    if (fresh) {
      copyPlayer(rig.cur, rig.prev);
      rig.w = 1;
      rig.upperFade = 1;
      rig.upperW = rig.upperTarget;
      rig.approach = reach;
      rig.hands = hands;
    }
  }

  /**
   * Lying down on `bed` at its slot (`sx`, `sz`, sim-core's): from its side (they stood beside it, sim-core's
   * `approach: side`), the way they face sitting on its edge first, facing out (`sideOut`: how far out from
   * the slot their hips go, `SIDE_IN` in from the edge); NaN from its foot (`LIE_ENTRY`), for a bed lain on
   * across, a hammock's sling and a bunk's upper sleeper (up its ladder).
   */
  private sideIn(rig: SimRig, bed: PlacedObject | null, sx: number, sz: number): number {
    const def = bed ? BEDS[bed.def] : undefined;
    if (!bed || def?.across || def?.sling || rig.body.stage === 'baby') return NaN;
    const f = (bed.rot * Math.PI) / 2;
    const across = (x: number, z: number) => (x - bed.cx) * Math.cos(f) - (z - bed.cz) * Math.sin(f);
    if (BUNKS[bed.def] && across(sx, sz) > 0) return NaN;
    const [ax, az] = [rig.fromX - sx, rig.fromZ - sz];
    const side = ax * Math.cos(f) - az * Math.sin(f);
    const along = ax * Math.sin(f) + az * Math.cos(f);
    if (Math.abs(side) <= Math.abs(along)) return NaN;
    const s = Math.sign(side);
    const half = (Math.abs(Math.cos(f)) * (bed.maxX - bed.minX) + Math.abs(Math.sin(f)) * (bed.maxZ - bed.minZ)) / 2;
    rig.sideOut = Math.max(0.1, half - s * across(sx, sz) - SIDE_IN * rig.scale);
    return wrapAngle(f + (s * Math.PI) / 2);
  }

  /** Holding the grips of a thing stood on (`GRIPPED`): its clip for the resident's height, as a sequence of one. */
  private gripped(rig: SimRig, sizes: readonly { clip: string; min: number }[]): readonly string[] {
    const clip = this.sizedClip(rig, sizes);
    let one = LANE_SEQ.get(clip);
    if (!one) LANE_SEQ.set(clip, (one = [clip]));
    return one;
  }

  /** The clip of `sizes` (the bunk's ladder, `BUNK_LADDER`; a ride's, `RIDES`; `SIT_LOW`) for this resident's height. */
  private sizedClip(rig: SimRig, sizes: readonly { clip: string; min: number }[]): string {
    return this.forHeight(rig, sizes, ({ min }) => min).clip;
  }

  /** The first of `sizes` (tallest first) this resident is as tall as (`min`: the shortest it is for), else the last. */
  private forHeight<T>(rig: SimRig, sizes: readonly T[], min: (size: T) => number): T {
    const h = rig.body.height * rig.scale;
    return sizes.find((size) => h >= min(size)) ?? sizes[sizes.length - 1];
  }

  /** The upper bunk's frame (`BUNK_LADDER`): its slot (sim-core `slot_position`, a hair to the right of the middle), facing the foot end. */
  private bedFrame(bed: PlacedObject): Frame {
    const f = (bed.rot * Math.PI) / 2;
    return { x: bed.cx + Math.cos(f) * 0.02, z: bed.cz - Math.sin(f) * 0.02, yaw: f };
  }

  /** Where the blend of the two players is placed: at the Sim, or between it and a frame of their own (`Player.frame`). */
  private framed(rig: SimRig, w: number): Frame {
    const a = rig.prev.frame ?? rig;
    const b = rig.cur.frame ?? rig;
    return { x: lerp(a.x, b.x, w), z: lerp(a.z, b.z, w), yaw: a.yaw + wrapAngle(b.yaw - a.yaw) * w };
  }

  /**
   * Whether the step sim-core just had a standing resident take in one tick (to `gx`, `gz`) is walked (`WALK_IN`):
   * sitting down (not into a bath or a seat with the legs out) or standing at a new place, not a baby or one held,
   * not on directions, nor using a thing from inside (`STEP_IN`), rising, climbing a bunk's ladder or out of bed.
   */
  private walksIn(rig: SimRig, i: number, pose: number, useObj: PlacedObject | null | undefined, gx: number, gz: number): boolean {
    if (rig.body.stage === 'baby' || this.heldBy[i] >= 0 || rig.dir || rig.inside || rig.outT >= 0 || rig.rising || rig.fromBed || rig.cur.key.startsWith('bunk')) return false;
    if (pose === Pose.Stand) return rig.lastPose === Pose.Stand && !RISING.has(rig.cur.key);
    // (Sat down a moment ago: the snapshot's glide onto the seat had only just begun.)
    if (pose !== Pose.Sit || !(rig.lastPose === Pose.Stand || (rig.cur.key.startsWith('sitDown') && rig.cur.time < 0.3))) return false;
    const obj = this.objectAt(gx, gz) ?? useObj;
    const seat = obj?.seat;
    // (Nor onto a seat in a frame, sat astride: a bike's saddle, a rocking horse; walked to, the legs would go
    // through it. They settle onto it as before: `STRADDLED`.)
    return !seat?.along && !seat?.legsOut && !seat?.pedals && !STRADDLED.has(obj?.def ?? '');
  }

  /** Starts walking in (`walksIn`) from where they are drawn to `gx`, `gz` (a seat's: round a built-in chair, `cornerOf`). */
  private startWalk(rig: SimRig, pose: number, useObj: PlacedObject | null | undefined, gx: number, gz: number, yaw: number): void {
    rig.walkT = 0;
    rig.walkS = 0;
    rig.walkL = Infinity;
    rig.walkTurn = 0;
    rig.walkYaw = yaw;
    [rig.walkX, rig.walkZ] = [rig.x, rig.z];
    [rig.walkPX, rig.walkPZ] = [rig.x, rig.z];
    [rig.walkEX, rig.walkEZ] = [gx, gz];
    rig.walkDown = pose === Pose.Sit;
    // (Into a shower with a door, or out of it as the use ends: once it has slid open.)
    // (Stepping out: the shower they were under, the use's end and their step out of it may come a tick apart.)
    const shower = pose !== Pose.Stand ? null : useObj ? useObj : rig.toweled >= 0 && rig.shower ? rig.shower : rig.useObj;
    const door = shower ? WALK_WAIT[shower.def] : undefined;
    rig.walkWait = !door ? 0 : useObj ? door.in : door.out;
    const seat = pose === Pose.Sit ? (this.objectAt(gx, gz) ?? useObj) : null;
    [rig.viaX, rig.viaZ] = seat?.builtIn ? this.cornerOf(seat, gx, gz, yaw, rig.x, rig.z) : [NaN, NaN];
    if (door && shower) {
      // (Through its doorway, in and out.)
      const f = (shower.rot * Math.PI) / 2;
      const [right, ahead] = door.door;
      [rig.viaX, rig.viaZ] = [shower.cx + Math.cos(f) * right + Math.sin(f) * ahead, shower.cz - Math.sin(f) * right + Math.cos(f) * ahead];
    }
  }

  /** The way a resident walking in faces now (`walkPlace`): along the way (to its corner, then its end), NaN once there. */
  private walkAhead(rig: SimRig): number {
    if (rig.walkS >= rig.walkL) return NaN;
    const corner = !Number.isNaN(rig.viaX) && rig.walkS < Math.hypot(rig.viaX - rig.walkX, rig.viaZ - rig.walkZ);
    const [x, z] = corner ? [rig.viaX, rig.viaZ] : [rig.walkEX, rig.walkEZ];
    const [dx, dz] = [x - rig.walkPX, z - rig.walkPZ];
    return Math.hypot(dx, dz) > 0.05 ? Math.atan2(dx, dz) : NaN;
  }

  /**
   * Walking in (`WALK_IN`): drawn where they stand while waiting, along the way to (`ex`, `ez`: where they are
   * drawn once there, facing `walkYaw`) while walking, there while turning to face that way; then done.
   */
  private walkPlace(rig: SimRig, pl: Placement, ex: number, ez: number, step: number): void {
    rig.walkT += step;
    [rig.walkEX, rig.walkEZ] = [ex, ez];
    const via = !Number.isNaN(rig.viaX);
    const first = via ? Math.hypot(rig.viaX - rig.walkX, rig.viaZ - rig.walkZ) : 0;
    rig.walkL = via ? first + Math.hypot(ex - rig.viaX, ez - rig.viaZ) : Math.hypot(ex - rig.walkX, ez - rig.walkZ);
    if (rig.walkT >= rig.walkWait && rig.walkS < rig.walkL) {
      rig.walkS = Math.min(rig.walkL, rig.walkS + step * WALK_IN.pace * this.clip('walk').speed * rig.body.pelvisScale * rig.scale);
    } else if (rig.walkS >= rig.walkL) {
      rig.walkTurn += step;
      if (rig.walkTurn >= WALK_IN.turn || Math.abs(wrapAngle(rig.walkYaw - rig.yaw)) < 0.05) rig.walkT = -1;
    }
    alongWay(pl, rig.walkX, rig.walkZ, rig.viaX, rig.viaZ, ex, ez, rig.walkL > 0 ? rig.walkS / rig.walkL : 1);
    [rig.walkPX, rig.walkPZ] = [pl.x, pl.z];
  }

  /**
   * The corner beside a built-in chair (its seat at `sx`, `sz`, facing `yaw`, on `obj`: a dining set's, a desk's)
   * that a step to or from a tile behind it (`tx`, `tz`) goes round, `WALK_IN.side` m to the chair's side away from
   * the set's middle; NaN with the tile not behind it.
   */
  private cornerOf(obj: PlacedObject, sx: number, sz: number, yaw: number, tx: number, tz: number): [number, number] {
    const [fx, fz] = [Math.sin(yaw), Math.cos(yaw)];
    const [rx, rz] = [fz, -fx];
    if ((tx - sx) * fx + (tz - sz) * fz > -0.25) return [NaN, NaN];
    const side = Math.sign((sx - obj.cx) * rx + (sz - obj.cz) * rz) || Math.sign((tx - sx) * rx + (tz - sz) * rz) || 1;
    return [sx + rx * side * WALK_IN.side, sz + rz * side * WALK_IN.side];
  }

  /** The size of `STEP_IN` entry `into` for this resident's height: [height, clips, where]. */
  private stepFit(rig: SimRig, into: StepIn): StepIn['by'][number] {
    return this.forHeight(rig, into.by, ([min]) => min);
  }

  /** The way `rig` faces using `obj` up against it (`STEP_IN`): its front, or (`sides`) the side of it they stand on, front or back. */
  private stepFacing(rig: SimRig, obj: PlacedObject, into: StepIn): number {
    const f = (obj.rot * Math.PI) / 2;
    return into.sides && (rig.x - obj.cx) * Math.sin(f) + (rig.z - obj.cz) * Math.cos(f) < 0 ? wrapAngle(f + Math.PI) : f;
  }

  /**
   * The lane of `into`'s (`lanes`, to the right of the middle of the side they use it from) nearest where
   * sim-core stood them (each in front of their own slot) that no one else using the same thing from that
   * side has taken; NaN with every one taken (they use it from where they stand), its middle without lanes.
   */
  private freeLane(rig: SimRig, into: StepIn, obj: PlacedObject): number {
    if (!into.lanes) return 0;
    const f = this.stepFacing(rig, obj, into);
    const taken = this.lanesTaken;
    taken.length = 0;
    for (const r of this.sims) if (r && r !== rig && r.inside === into && r.useId === rig.useId && Math.abs(wrapAngle(r.inFacing - f)) < 0.1) taken.push(r.inLane);
    const across = (rig.x - obj.cx) * Math.cos(f) - (rig.z - obj.cz) * Math.sin(f);
    let best = NaN;
    for (const x of into.lanes) if (!taken.includes(x) && !(Math.abs(x - across) >= Math.abs(best - across))) best = x;
    return best;
  }

  /** The clips played using `into` (`STEP_IN`): its sequence for the resident's height, or (`perLane`) the one for their lane. */
  private stepSeq(rig: SimRig, into: StepIn, obj: PlacedObject): readonly string[] {
    const seq = this.stepFit(rig, into)[1];
    if (!into.perLane || !into.lanes) return seq;
    const lane = Math.max(0, into.lanes.indexOf(rig.inside === into ? rig.inLane : this.freeLane(rig, into, obj)));
    const clip = seq[lane % seq.length];
    let one = LANE_SEQ.get(clip);
    if (!one) LANE_SEQ.set(clip, (one = [clip]));
    return one;
  }

  /**
   * Using a thing from inside or up against it (`STEP_IN`) from the tile in front: back a little while its
   * door opens (`back`, the first `wait` s), then walking in to where they use it, up onto its floor on the
   * way. The root's forward offset and height, the clip (standing about while waiting, walking, or `key`)
   * and its rate.
   */
  private stepIn(rig: SimRig, obj: PlacedObject, into: StepIn, key: string, step: number, dt: number): [number, number, string, number] {
    if (rig.inside !== into) {
      rig.inside = into;
      rig.inT = 0;
      rig.outT = -1;
      rig.inFacing = this.stepFacing(rig, obj, into);
      rig.inLane = this.freeLane(rig, into, obj);
    } else rig.inT += step;
    const at = this.stepFit(rig, into)[2];
    const f = rig.inFacing;
    // The tile's distance from the object's middle, along the way they face it; the way in from stood back.
    const ahead = (rig.x - obj.cx) * Math.sin(f) + (rig.z - obj.cz) * Math.cos(f);
    const back = into.back ?? 0;
    const dist = ahead - at.to + back;
    // And sideways to its lane (along the right of the side they use it from).
    const across = into.lanes ? rig.inLane - ((rig.x - obj.cx) * Math.cos(f) - (rig.z - obj.cz) * Math.sin(f)) : 0;
    const way = Math.hypot(dist, across);
    const span = Math.max(0.3, way / STEP_IN_SPEED);
    const k = smooth(clamp((rig.inT - (into.wait ?? 0)) / span, 0, 1));
    const fwd = -back * smooth(clamp(rig.inT / 0.4, 0, 1)) + dist * k;
    let y = (at.up ?? 0) * smooth(clamp((k - 0.35) / 0.3, 0, 1));
    // Up over a rim on the way (and, stepping out, back over it: `stepPlace`).
    const over = into.over;
    if (over) {
      y += over.h * smooth(clamp(1 - Math.abs(ahead - fwd - over.at) / STEP_OVER, 0, 1));
      rig.inOver.set([(over.at - at.to) / Math.max(0.01, ahead - at.to), over.h, Math.max(0.01, ahead - at.to)]);
    } else rig.inOver.set([0, 0, 1]);
    rig.inDX = Math.cos(f) * across * k;
    rig.inDZ = -Math.sin(f) * across * k;
    // (Waiting to step in, standing about.)
    if (k <= 0) return [fwd, y, into.wait || into.perLane ? 'idle' : key, 1];
    // (Already there, or as good as: no steps.) Waiting for their turn, the use's sequence holds too.
    if (k >= 1 || way < 0.05) {
      if (rig.cur.key === key || this.inTurn(rig, into)) return [fwd, y, key, 1];
      rig.stepTime = 0;
      return [fwd, y, 'idle', 1];
    }
    // Walking, the cycle matched to the step's pace (the walk runs on real time); backwards, facing it, when
    // the place is further out than the tile (out on the court from the hoop).
    const legs = rig.body.pelvisScale * rig.scale;
    const rate = dt > 0 ? ((way / span) * (step / dt)) / (this.clip('walk').speed * legs) : 1;
    return [fwd, y, 'walk', clamp(rate, 0.35, GAIT_MAX_RATE) * (dist < -Math.abs(across) ? -1 : 1)];
  }

  /**
   * Whether `rig`, stepped in to `into`, may start its round (`StepIn.perLane`): the lanes' rounds keep out
   * of each other's way only in step (the climbing frame's second lane waits on the deck while the first
   * slides down the pole), so with someone already going round, it waits standing at its lane until their
   * round comes back to its start (`LANE_SYNC` s into it).
   */
  private inTurn(rig: SimRig, into: StepIn): boolean {
    if (!into.perLane) return true;
    for (const r of this.sims) {
      // (Only those on the same side: `sides`.)
      if (!r || r === rig || r.inside !== into || r.useId !== rig.useId || Math.abs(wrapAngle(r.inFacing - rig.inFacing)) > 0.1 || !LANE_SEQ.has(r.cur.key) || !r.cur.clip) continue;
      if (r.cur.time % r.cur.clip.duration > LANE_SYNC) return false;
    }
    return true;
  }

  /** The use of a thing used from inside (`STEP_IN`) is over: back out to the tile in front after its `out` seconds (`stepPlace`); none when drawn afresh. */
  private stepOut(rig: SimRig, fresh: boolean): void {
    rig.outT = fresh ? -1 : 0;
    rig.outWait = rig.inside?.out ?? 0;
    rig.inside = null;
  }

  /** Where a resident using a thing from inside is drawn (kept for stepping out), and stepping out: from there to where they are now (`STEP_OUT_S`). */
  private stepPlace(rig: SimRig, pl: Placement, step: number): void {
    if (rig.inside) {
      pl.x += rig.inDX;
      pl.z += rig.inDZ;
      [rig.inX, rig.inY, rig.inZ] = [pl.x, pl.y, pl.z];
      return;
    }
    if (rig.outT < 0) return;
    const k = smooth(clamp((rig.outT - rig.outWait) / STEP_OUT_S, 0, 1));
    const [rim, high, way] = rig.inOver;
    pl.x = lerp(rig.inX, pl.x, k);
    pl.y = lerp(rig.inY, pl.y, k) + high * smooth(clamp(1 - (Math.abs(k - rim) * way) / STEP_OVER, 0, 1));
    pl.z = lerp(rig.inZ, pl.z, k);
    rig.outT = k < 1 ? rig.outT + step : -1;
  }

  /**
   * Body language for the Sim's emotion (`bodyLanguage.ts`), as far as what they do leaves the body free: standing
   * free or walking, all of it (but the arms, carrying); using a thing standing (its clips' hands on it), inside one,
   * in a hug, a kiss, a fight or a cuddle, the head only; seated, the head and, sat plainly, some of the back; lying, a
   * baby or directed (the creator, portraits), none.
   */
  private bodyLanguage(rig: SimRig, i: number, social: number, pose: number, moving: boolean, refused: boolean, dt: number): void {
    const B = this.set!.bone;
    const free = this.freedom;
    const key = rig.cur.key;
    free.posture = free.arms = 0;
    free.head = 1;
    if (rig.dir || rig.body.stage === 'baby' || pose === Pose.Lie || this.force) free.head = 0;
    else if (pose === Pose.Sit) free.posture = key === 'sit' || key === 'sitTalk' ? 0.3 : 0;
    else if (!(
      social === Anim.Hug ||
      social === Anim.Kiss ||
      social === Anim.Fight ||
      refused ||
      rig.cuddleT >= 0 ||
      rig.liftW > 0 ||
      this.heldBy[i] >= 0 ||
      rig.inside ||
      (!moving && !!rig.useSeq?.includes(key))
    )) {
      free.posture = 1;
      free.arms = rig.upperW > 0.05 ? 0 : 1;
    }
    const walking = key === 'walk' && rig.cur.clip;
    bearing(rig.bearing, this.emotionIds[rig.emotion - 1] ?? '', free, walking ? (rig.cur.time / rig.cur.clip!.duration) % 1 : 0, walking ? smooth(rig.w) : 0, dt, this.add, B);
  }

  /**
   * Standing free this frame, so that a wide turn can step round (`turnOnSpot`): on their feet and still, not walked
   * in, inside a thing, up on one, getting up or climbing down, nor holding or lifting a baby or in an embrace or a
   * fight (those place the body themselves); a baby never.
   */
  private standsFree(rig: SimRig, i: number, pose: number, moving: boolean, social: number, useObj: PlacedObject | null | undefined, appeared: boolean): boolean {
    if (pose !== Pose.Stand || moving || appeared || this.force || rig.body.stage === 'baby' || this.heldBy[i] >= 0) return false;
    if (rig.walkT >= 0 || rig.inside || rig.outT >= 0 || useObj?.stand || rig.rising || rig.fromBed || rig.riseK < 1) return false;
    if (rig.cuddleT >= 0 || rig.liftW > 0 || !Number.isNaN(rig.faceX)) return false;
    if (social === Anim.Hug || social === Anim.Kiss || social === Anim.Fight) return false;
    // (Getting up, or down the bunk's ladder, until its clip is done: `choose` keeps it.)
    if (rig.lastPose !== Pose.Stand || RISING.has(rig.cur.key) || rig.cur.key.startsWith('bunk')) return false;
    return true;
  }

  /** A gait clip's ground speed at rate 1 on the reference legs (`gaitFor`), undefined for one the set doesn't have. */
  private readonly gaitSpeed = (key: string): number | undefined => this.set?.clips.get(key)?.speed;

  private clip(key: string): Clip {
    const set = this.set!;
    // (Turns on the spot are baked the first time they're played: `turnClipIn`.)
    return set.clips.get(key) ?? turnClipIn(set, key) ?? set.clips.get('idle')!;
  }

  /** Length (s) of the clip played for `key` (0 before the set has loaded). */
  clipDuration(key: string): number {
    return this.set ? this.clip(key).duration : 0;
  }

  /** Moves this frame's placement (`place`) by `ride`, a seat's move from rest in world space. */
  private riding(ride: ArrayLike<number>): void {
    const pl = this.place;
    const m = this.rideM;
    // (The ported `compose`, `multiply` and `rotationOf` of `rig.ts`, step for step in the same float32 matrices.)
    compose(1, 1, 1, pl.qx, pl.qy, pl.qz, pl.qw, pl.x, pl.y, pl.z, m);
    multiply(m, ride, m);
    rotationOf(m, pl);
    pl.x = m[12];
    pl.y = m[13];
    pl.z = m[14];
  }

  /** The tables seats face, with what is in front of them (`tableRooms`), by seat id: set with each world. */
  setTables(tables: Map<number, TableRoom>): void {
    this.tables = tables;
    this.seatIds.clear();
    for (const [id] of tables) {
      const o = this.objectById.get(id);
      if (o) this.seatIds.set(o, id);
    }
  }

  /** How far each chair pulled up to a table is drawn pulled in now (m, along its facing), by seat id (`TUCK`). */
  readonly pulled = new Map<number, number>();

  /**
   * Seated on a chair that faces a table (`tables`): how far it is pulled in now (m), easing in once they
   * are `settled` (sat down), at once when drawn afresh (`TUCK`).
   */
  private tuckIn(rig: SimRig, obj: PlacedObject | null, seat: Seat, settled: boolean, step: number, fresh: boolean): number {
    const id = obj ? this.seatIds.get(obj) : undefined;
    const room = id === undefined ? undefined : this.tables.get(id);
    if (id !== rig.tuckSeat) rig.tuck = 0;
    rig.tuckSeat = id ?? -1;
    let goal = 0;
    rig.tuckFar = false;
    rig.tuckLow = false;
    if (obj && room && settled && !seat.back && room.top > seat.h && room.top - seat.h <= TUCK.rise) {
      const s = (rig.body.height * rig.scale) / CLIP_HEIGHT;
      rig.tuckLow = room.top - seat.h < LOW_TOP * s;
      const yaw = (obj.rot * Math.PI) / 2;
      const [dx, dz] = [rig.x - obj.cx, rig.z - obj.cz];
      const right = dx * Math.cos(yaw) - dz * Math.sin(yaw);
      const hips = dx * Math.sin(yaw) + dz * Math.cos(yaw) + seat.fwd;
      const edge = nearestAhead(room, right, 0.12 * s, room.top - 0.04, room.top + 0.01);
      // (The knees no further than the middle of the top: someone may sit across it.)
      const middle = (edge + furthestAhead(room, right, 0.12 * s, room.top - 0.04, room.top + 0.01)) / 2;
      const knees = nearestAhead(room, right, 0.16 * s, TUCK.toeUp, seat.h + TUCK.kneeUp * s);
      const toes = nearestAhead(room, right, 0.16 * s, 0, TUCK.toeUp);
      const most = Math.min(edge - TUCK.edge * s, Math.min(knees, middle) - TUCK.knees * s - TUCK.clear, toes - TUCK.toes * s - TUCK.clear) - hips;
      if (Number.isFinite(edge)) goal = clamp(most, 0, TUCK.max);
      rig.tuckFar = Number.isFinite(edge) && edge - hips - goal > TUCK.far * s;
    }
    rig.tuck = fresh ? goal : rig.tuck + (goal - rig.tuck) * Math.min(1, step * TUCK.rate);
    return rig.tuck;
  }

  /** Off the chair pulled up to a table: it is pushed back as they rise (`TUCK`); how far in it still is while they do. */
  private tuckOut(rig: SimRig, key: string, step: number): number {
    if (rig.tuckSeat < 0) return 0;
    rig.tuck *= 1 - Math.min(1, step * TUCK.back);
    if (rig.tuck < 0.002) [rig.tuck, rig.tuckSeat] = [0, -1];
    return key === 'standUp' ? rig.tuck : 0;
  }

  /** The chairs pulled in now (`pulled`): each as far as the least pulled-in of those on it. */
  private pullChairs(): void {
    this.pulled.clear();
    for (const rig of this.sims) {
      if (!rig?.visible || rig.tuckSeat < 0 || rig.tuck <= 0.001) continue;
      const was = this.pulled.get(rig.tuckSeat);
      this.pulled.set(rig.tuckSeat, was === undefined ? rig.tuck : Math.min(was, rig.tuck));
    }
  }

  /** Where residents sitting or lying on an object are carried by its moving seat (`ObjectMotion.ride`). */
  setRide(rideOf: ((objectId: number) => ArrayLike<number> | null) | null): void {
    this.rideOf = rideOf;
  }

  /** Which way the sun's light goes (the renderer's `sun.direction`): the shade under a tree is on that side of it (`shadeSide`). */
  setSun(sunOf: (() => { x: number; z: number }) | null): void {
    this.sunOf = sunOf;
  }

  /** How far round an object's loop is (`ObjectMotion.loopPhase`: the bike's pedals the feet follow). */
  setLoop(loopOf: ((objectId: number) => number | null) | null): void {
    this.loopOf = loopOf;
  }

  private start(rig: SimRig, key: string, fade: number, rate: number, repeat: boolean, rootY: number, rootFwd: number, pitch: number): void {
    const clip = this.clip(key);
    const { prev, cur } = rig;
    if (rig.w < 1 && prev.clip) {
      // A fade cut short. Neither end of it is where the pose had got to, so the fading-out player holds the pose
      // the blend had reached (`hold`) and fades out of that: dropping to either end would jump by the blend's
      // remaining distance, up to half the whole fade (`blend.test.ts`).
      const w = smooth(rig.w);
      const heldY = lerp(prev.rootY, cur.rootY, w);
      const heldFwd = lerp(prev.rootFwd, cur.rootFwd, w);
      const heldPitch = lerp(prev.rootPitch, cur.rootPitch, w);
      const at = prev.frame || cur.frame ? this.framed(rig, w) : null;
      this.hold(prev, cur, w);
      // (`copyPlayer` took the incoming clip's root: put back the one the held pose was blended at.)
      prev.rootY = heldY;
      prev.rootFwd = heldFwd;
      prev.rootPitch = heldPitch;
      if (at) prev.frame = at;
    } else {
      copyPlayer(cur, prev);
    }
    const sameLoop = cur.key === key && clip.loop;
    // Walking into a jog and back: in step (both cycles start at the left heel's strike).
    const inStep = GAITS.has(key) && GAITS.has(cur.key) && cur.clip ? ((cur.time / cur.clip.duration) % 1) * clip.duration : -1;
    cur.clip = clip;
    cur.key = key;
    // Walk cycles of the previous walk keep their phase.
    cur.time = sameLoop ? cur.time : inStep >= 0 ? inStep : key === 'idle' || key === 'sit' || key === BABY_LIE.asleep || key === BABY_LIE.awake ? (rig.seed * 7) % clip.duration : 0;
    cur.rate = rate;
    cur.repeat = repeat;
    cur.rootY = rootY;
    cur.rootFwd = rootFwd;
    cur.rootPitch = pitch;
    if (!prev.clip) copyPlayer(cur, prev);
    rig.w = 0;
    rig.fadeRate = 1 / Math.max(0.05, fade);
  }

  /** `prev` takes over from `cur` (`copyPlayer`) holding the pose their blend at `w` (smoothed) had reached. */
  private hold(prev: Player, cur: Player, w: number): void {
    const set = this.set!;
    samplePlayer(set, prev, this.qB, this.pB);
    samplePlayer(set, cur, this.qC, this.pC);
    blendInto(this.qB, this.pB, this.qC, this.pC, w, null);
    copyPlayer(cur, prev);
    freezePlayer(prev, this.qB, this.pB);
  }

  /** Look-at, flirting, kissing and hugging at the partner's height, punches aimed at theirs, on top of the clips. */
  private procedural(
    rig: SimRig,
    i: number,
    social: number,
    role: number,
    partner: SimRig | null,
    pose: number,
    moving: boolean,
    time: number,
    dt: number,
  ): void {
    const set = this.set!;
    const B = set.bone;
    const add = this.add;
    // Look at the conversation partner (head and neck turn, limited).
    let wantYaw = 0;
    let wantPitch = 0;
    const dir = rig.dir;
    if (dir?.lookAt) {
      // Directed: the head turns partway towards the point (the eyes do the rest).
      const dx = dir.lookAt.x - this.heads[i * 3];
      const dz = dir.lookAt.z - this.heads[i * 3 + 2];
      const dy = dir.lookAt.y - this.heads[i * 3 + 1];
      wantYaw = clamp(wrapAngle(Math.atan2(dx, dz) - rig.yaw), -1.1, 1.1) * 0.45;
      wantPitch = clamp(-Math.atan2(dy, Math.hypot(dx, dz)), -0.4, 0.4) * 0.45;
    } else if (dir?.still) {
      // Portraits: straight ahead.
    } else if (social && partner && partner.visible && pose !== Pose.Lie) {
      const ph = (rig.audience?.visible ? rig.audience : partner).id * 3;
      const dx = this.heads[ph] - this.heads[i * 3];
      const dz = this.heads[ph + 2] - this.heads[i * 3 + 2];
      const dy = this.heads[ph + 1] - this.heads[i * 3 + 1];
      wantYaw = clamp(wrapAngle(Math.atan2(dx, dz) - rig.yaw), -1.1, 1.1);
      wantPitch = clamp(-Math.atan2(dy, Math.hypot(dx, dz)), -0.4, 0.4);
    } else if (!moving && pose !== Pose.Lie && !this.force && rig.cuddleT < 0) {
      // Idle glances.
      const g = Math.floor(time / 3.7 + rig.seed * 5);
      wantYaw = (hash(g * 13 + i) - 0.5) * 0.9 * (hash(g * 5 + i) < 0.5 ? 1 : 0);
    }
    const f = dir?.still ? 1 : 1 - Math.exp(-dt * 5);
    rig.lookYaw += (wantYaw - rig.lookYaw) * f;
    rig.lookPitch += (wantPitch - rig.lookPitch) * f;
    if (Math.abs(rig.lookYaw) > 1e-3) {
      addRotation(add, B.neck_01, 0, 1, 0, rig.lookYaw * 0.4);
      addRotation(add, B.Head, 0, 1, 0, rig.lookYaw * 0.6);
    }
    if (Math.abs(rig.lookPitch) > 1e-3) addRotation(add, B.Head, 1, 0, 0, rig.lookPitch * 0.7);

    if (pose !== Pose.Lie && this.force !== 'rest') {
      // Elders stoop: the upper back rounds, the head lifts to look ahead.
      if (rig.stoop > 0) {
        addRotation(add, B.spine_03, 1, 0, 0, 0.2 * rig.stoop);
        addRotation(add, B.neck_01, 1, 0, 0, 0.08 * rig.stoop);
        addRotation(add, B.Head, 1, 0, 0, -0.12 * rig.stoop);
      }
      this.stance(rig, pose, moving, dt);
      // Walking: women's hips sway more (the pelvis rolls over each standing leg, the chest
      // counters it), as in real gaits; one roll each way per stride.
      const walkW = (rig.cur.key === 'walk' ? smooth(rig.w) : 0) * (GAIT_SWAY[rig.body.family] ?? 0);
      if (walkW > 0 && rig.cur.clip) {
        const roll = Math.sin((rig.cur.time / rig.cur.clip.duration) * Math.PI * 2) * walkW;
        addRotation(add, B.pelvis, 0, 0, 1, roll);
        addRotation(add, B.spine_02, 0, 0, 1, -roll * 0.7);
        addRotation(add, B.spine_03, 0, 0, 1, -roll * 0.4);
      }
      if (!dir?.still) {
        // Breathing (a slow lift of the chest, the head kept level; quicker when worked up) and a
        // faint sway of the head: nobody stands perfectly still.
        const rate = social === Anim.Argue || social === Anim.Fight || moving ? 2.2 : 1.55;
        const breath = Math.sin(time * rate + rig.seed * 6);
        addRotation(add, B.spine_02, 1, 0, 0, -0.014 * breath);
        addRotation(add, B.spine_03, 1, 0, 0, -0.008 * breath);
        addRotation(add, B.Head, 1, 0, 0, 0.012 * breath + 0.012 * Math.sin(time * 0.53 + rig.seed * 11));
        addRotation(add, B.Head, 0, 1, 0, 0.018 * Math.sin(time * 0.37 + rig.seed * 17));
      }
    }
    // (A refused hug or kiss plays its own clips, `REFUSED`: nobody leans in or holds anyone.)
    const refused = rig.cur.key === 'refuse' || rig.cur.key === 'rebuffed';
    if (this.force !== 'rest') this.bodyLanguage(rig, i, social, pose, moving, refused, dt);
    switch (social) {
      case Anim.Flirt:
        // A head tilt and a sway of the hips for the one listening (the flirt's own clip has its own).
        if (rig.cur.key === 'flirt') break;
        addRotation(add, B.Head, 0, 0, 1, 0.16 + Math.sin(time * 1.7) * 0.04);
        addRotation(add, B.pelvis, 0, 0, 1, Math.sin(time * 1.3) * 0.04);
        break;
      case Anim.Kiss: {
        if (refused) break;
        // To the other's lips at any two heights (`KISS`): the taller bends and bows the head, the shorter
        // lifts the face (the look at the other's head does the rest); the taller's arms round the waist,
        // the shorter's up round the shoulders (of two alike, the one who asked holds the waist).
        const dh = partner ? rig.body.height * rig.scale - partner.body.height * partner.scale : 0;
        const taller = dh > 0.01 || (Math.abs(dh) <= 0.01 && role === 1);
        const up = Math.max(0, dh);
        const down = Math.max(0, -dh);
        addRotation(add, B.spine_02, 1, 0, 0, KISS_FIT.lean + Math.min(KISS_FIT.most, up * KISS_FIT.bend));
        addRotation(add, B.spine_03, 1, 0, 0, Math.min(KISS_FIT.most, up * KISS_FIT.bend) * 0.5 - Math.min(KISS_FIT.most, down * KISS_FIT.lift) * 0.3);
        addRotation(add, B.neck_01, 1, 0, 0, Math.min(KISS_FIT.most, up * KISS_FIT.nod) - Math.min(KISS_FIT.most, down * KISS_FIT.lift));
        addRotation(add, B.Head, 0, 0, 1, role === 1 ? 0.22 : -0.22);
        addRotation(add, B.Head, 1, 0, 0, 0.1 - Math.min(KISS_FIT.most, down * KISS_FIT.lift) * 0.5);
        if (taller) this.hugArms(KISS_FIT.waist, time + role / 1.5, KISS_FIT.waistWrap);
        else this.hugArms(KISS_FIT.neck, time + role / 1.5, KISS_FIT.neckWrap);
        break;
      }
      case Anim.Hug: {
        if (refused) break;
        // Bent down over a smaller one (a child), the arms lowered to go round them (`EMBRACE`).
        const drop = shoulderDrop(rig, partner);
        addRotation(add, B.spine_02, 1, 0, 0, 0.1 + drop * EMBRACE.bend);
        // Both heads turned and leant to their own right, so they go past each other (cheek by cheek), not into each other.
        addRotation(add, B.spine_03, 0, 0, 1, -EMBRACE.aside);
        addRotation(add, B.Head, 0, 1, 0, 0.5);
        // Arms around the other: one Sim over the shoulders, the other a little lower, around the back.
        this.hugArms(Math.max(EMBRACE.low, (role === 1 ? 1.3 : 1.05) - drop * EMBRACE.lower), time + role / 1.5, EMBRACE.wrap + drop * EMBRACE.wrapSmall);
        break;
      }
      case Anim.Argue:
        addRotation(add, B.spine_03, 1, 0, 0, 0.08);
        break;
      case Anim.Fight: {
        const isAttacker = role === 1;
        if (isAttacker) {
          // Attacker: body leans forward into punches, which are aimed at the other's head (`AIM`): bent
          // down over a shorter one, the arms lowered the rest of the way; up at a taller one.
          addRotation(add, B.spine_02, 1, 0, 0, 0.15);
          addRotation(add, B.spine_01, 1, 0, 0, 0.1);
          const drop = partner ? (rig.body.height * rig.scale - partner.body.height * partner.scale) * AIM.head : 0;
          const angle = clamp(Math.atan2(drop, AIM.reach * rig.scale), -AIM.most, AIM.most);
          const bend = angle > 0 ? angle * AIM.bend : angle * AIM.straighten;
          addRotation(add, B.spine_01, 1, 0, 0, bend * 0.5);
          addRotation(add, B.spine_02, 1, 0, 0, bend * 0.5);
          addRotation(add, B.upperarm_l, 1, 0, 0, angle - bend);
          addRotation(add, B.upperarm_r, 1, 0, 0, angle - bend);
        } else {
          // Defender: flinch back, forearms up to guard the face.
          addRotation(add, B.spine_02, 1, 0, 0, -0.12);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.45);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.45);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -1.5);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -1.5);
        }
        break;
      }
    }
    if (!social && !this.force && !dir?.clip) this.activityPose(rig, pose, time);
    if ((rig.reachW > 0 || rig.holdW > 0) && pose === Pose.Stand && !this.force) cuddlePose(this.add, this.set!.bone, rig, time);
    // Lying: a bench press, or resting (see the `lie` clip).
    if (pose === Pose.Lie && rig.cur.key === 'lie' && rig.tag === 'lift') {
      const feet = rig.bed ? BEDS[rig.bed.def]?.feet : undefined;
      if (feet) this.benchLegs(rig, feet);
      // Bench press: arms push the bar up and lower it to the chest.
      const press = 0.5 - 0.5 * Math.cos(time * 2.2 + rig.seed * 6);
      for (const [ua, la, sgn] of [
        [B.upperarm_l, B.lowerarm_l, 1],
        [B.upperarm_r, B.lowerarm_r, -1],
      ] as const) {
        addRotation(add, ua, 0, 0, 1, sgn * 0.55);
        addRotation(add, ua, 1, 0, 0, -1.2 - press * 0.1);
        addRotation(add, la, 1, 0, 0, -1.4 + press * 1.3);
      }
    } else if (pose === Pose.Lie && rig.cur.key === 'lie') {
      // (The clip breathes, with the hands on the belly.) The head turns slowly to one side, each
      // Sim its own way (asleep: deeper, less movement).
      const asleep = rig.tag === 'sleep' || rig.tag === 'nap';
      const headTurn = (rig.seed - 0.5) * 0.7 * (asleep ? 1 : 1.5) * (rig.sling ? 0.5 : 1);
      addRotation(add, B.Head, 0, 1, 0, headTurn + Math.sin(time * 0.2 + rig.seed * 8) * 0.02);
      if (rig.sling) this.slingPose(rig, rig.sling);
      // Subtle leg adjustment for more natural lying.
      else if (!asleep) {
        const legShift = Math.sin(time * 0.25 + rig.seed * 7) * 0.03;
        addRotation(add, B.thigh_l, 1, 0, 0, legShift);
        addRotation(add, B.thigh_r, 1, 0, 0, -legShift);
      }
    }
    dir?.pose?.(add, B, time);
    this.animateFace(rig, i, social, role, partner, pose, moving, time, dt);
  }

  /**
   * Arms round someone or something hugged: raised forward `raise` (rad), elbows wrapping in by `wrap` (rad;
   * the teddy's neck, the default, is held looser than a body is, `EMBRACE`); a slow squeeze.
   * (Model space: +z forward, the left arm on -x; x rotations raise / bend both sides alike. Spread first,
   * then raise: the other order would only roll the raised arm.)
   */
  private hugArms(raise: number, time: number, wrap = 0.95): void {
    const B = this.set!.bone;
    const add = this.add;
    const squeeze = Math.sin(time * 1.5) * 0.05;
    addRotation(add, B.upperarm_l, 0, 0, 1, -0.35);
    addRotation(add, B.upperarm_r, 0, 0, 1, 0.35);
    addRotation(add, B.upperarm_l, 1, 0, 0, -raise);
    addRotation(add, B.upperarm_r, 1, 0, 0, -raise);
    addRotation(add, B.lowerarm_l, 0, 1, 0, wrap + squeeze);
    addRotation(add, B.lowerarm_r, 0, 1, 0, -wrap - squeeze);
  }

  /**
   * Procedural layers for activities the clips only approximate: reclining, instruments,
   * meditation, gazes at plants and trees, and exercise equipment. Model space: +z is forward,
   * the left limbs are on -x; a negative x rotation raises an arm forward (or bends an elbow /
   * knee forward), on both sides alike.
   */
  private activityPose(rig: SimRig, pose: number, time: number): void {
    const B = this.set!.bone;
    const add = this.add;
    const def = rig.useDef;
    const key = rig.cur.key;
    const seed = rig.seed * 6;
    // (Standing up, the seated legs' fit fades out with the seated pose; it starts afresh on the next seat.)
    const onFloor = ON_FLOOR.has(key);
    if (pose === Pose.Stand && key === 'standUp') this.seatLegsOff(rig);
    else rig.seatTurn.fill(0);
    if (pose === Pose.Sit && !RISING.has(key)) {
      const seat = rig.useObj?.seat;
      if (onFloor) {
        // (Cross-legged on the floor: the clip's legs.)
      } else if (seat?.pedals) this.pedal(rig, seat.pedals);
      else if (seat?.legsOut) this.legsOut(rig, seat.room);
      else if (rig.feetUp) this.feetUp(rig);
      else if (rig.tag !== 'meditate') this.seatLegs(rig, rig.tag === 'relax' || rig.tag === 'nap' || rig.tag === 'watch');
      switch (rig.tag) {
        case 'sit':
          // Just sitting: upright, but against a slanted back (`recline`).
          if (seat?.recline !== undefined) this.recline(seat.recline);
          break;
        case 'relax':
        case 'nap':
        case 'watch':
          // Sinking back into the seat (or against its slanted back, `recline`, split over the spine as `legsOut`
          // does, the head kept looking ahead), legs stretched out a little (`seatLegs`, or all the way,
          // `legsOut`), arms on the armrests. (On the floor, nothing to sink back into: the clip.)
          if (onFloor) break;
          if (seat?.recline !== undefined) this.recline(seat.recline);
          else {
            addRotation(add, B.spine_01, 1, 0, 0, -0.14);
            addRotation(add, B.Head, 1, 0, 0, 0.1);
          }
          addRotation(add, B.upperarm_l, 0, 0, 1, -0.18);
          addRotation(add, B.upperarm_r, 0, 0, 1, 0.18);
          break;
        case 'bath': {
          // Leaning back against the end of the tub, arms along the rim.
          addRotation(add, B.spine_01, 1, 0, 0, -0.3);
          addRotation(add, B.Head, 1, 0, 0, 0.2);
          addRotation(add, B.upperarm_l, 0, 0, 1, -0.95);
          addRotation(add, B.upperarm_r, 0, 0, 1, 0.95);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -0.15);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -0.15);
          break;
        }
        case 'toilet':
          // Leaning forward, the hands sliding down the thighs (the clip rests them there).
          addRotation(add, B.spine_01, 1, 0, 0, 0.3);
          addRotation(add, B.Head, 1, 0, 0, -0.15);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.15);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.15);
          addRotation(add, B.lowerarm_l, 1, 0, 0, 0.1);
          addRotation(add, B.lowerarm_r, 1, 0, 0, 0.1);
          break;
        case 'meditate': {
          // Cross-legged and upright, hands resting on the knees, slow deep breaths (on the floor, the clip's legs).
          const breath = Math.sin(time * 0.8 + seed) * 0.025;
          addRotation(add, B.spine_01, 1, 0, 0, 0.18 + breath);
          addRotation(add, B.spine_02, 1, 0, 0, -breath);
          addRotation(add, B.Head, 1, 0, 0, 0.06);
          if (onFloor) break;
          addRotation(add, B.thigh_l, 0, 1, 0, -0.75);
          addRotation(add, B.thigh_r, 0, 1, 0, 0.75);
          addRotation(add, B.calf_l, 0, 0, 1, 1.25);
          addRotation(add, B.calf_r, 0, 0, 1, -1.25);
          addRotation(add, B.upperarm_l, 0, 0, 1, -0.4);
          addRotation(add, B.upperarm_r, 0, 0, 1, 0.4);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.3);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.3);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -0.1);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -0.1);
          break;
        }
        case 'read':
          addRotation(add, B.Head, 1, 0, 0, 0.18);
          break;
        case 'music':
          if (def.includes('cello')) {
            // Knees apart around the body of the cello, the left hand up on its neck, the right
            // drawing the bow back and forth.
            const bow = Math.sin(time * 2.2 + seed);
            addRotation(add, B.thigh_l, 0, 1, 0, -0.35);
            addRotation(add, B.thigh_r, 0, 1, 0, 0.35);
            addRotation(add, B.upperarm_l, 1, 0, 0, -0.95);
            addRotation(add, B.upperarm_l, 0, 0, 1, -0.25);
            addRotation(add, B.lowerarm_l, 1, 0, 0, -1.3);
            addRotation(add, B.lowerarm_l, 0, 1, 0, 0.6);
            addRotation(add, B.upperarm_r, 1, 0, 0, -0.55);
            addRotation(add, B.upperarm_r, 0, 0, 1, 0.45 + bow * 0.12);
            addRotation(add, B.lowerarm_r, 1, 0, 0, -0.9);
            addRotation(add, B.lowerarm_r, 0, 1, 0, -0.7 - bow * 0.45);
            addRotation(add, B.Head, 0, 0, 1, 0.12);
          } else if (def.includes('drum')) {
            // Sticks: the forearms strike in turn.
            const l = Math.max(0, Math.sin(time * 9 + seed));
            const r = Math.max(0, Math.sin(time * 9 + seed + Math.PI));
            addRotation(add, B.lowerarm_l, 1, 0, 0, -0.55 * l);
            addRotation(add, B.lowerarm_r, 1, 0, 0, -0.55 * r);
            addRotation(add, B.Head, 1, 0, 0, Math.sin(time * 4.5) * 0.05);
          } else {
            // Keys: swaying with the music.
            addRotation(add, B.spine_02, 0, 0, 1, Math.sin(time * 1.1 + seed) * 0.04);
          }
          break;
        case 'tinker':
          if (def === 'potteryWheel') {
            // Knees apart either side of the splash pan, as potters sit, the wheel head between them.
            addRotation(add, B.thigh_l, 0, 1, 0, -0.55);
            addRotation(add, B.thigh_r, 0, 1, 0, 0.55);
          }
          break;
      }
      return;
    }
    if (pose !== Pose.Stand) return;
    const grip = rig.useObj?.held?.grip;
    switch (rig.tag) {
      case 'lift':
        if (grip === 'pair') {
          // Curls: the upper arms at the sides, a dumbbell in each fist coming up in turn.
          const l = 0.5 - 0.5 * Math.cos(time * 2.4 + seed);
          const r = 0.5 - 0.5 * Math.cos(time * 2.4 + seed + Math.PI);
          addRotation(add, B.upperarm_l, 0, 0, 1, 0.12);
          addRotation(add, B.upperarm_r, 0, 0, 1, -0.12);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.1 - 0.15 * l);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.1 - 0.15 * r);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -0.35 - 1.6 * l);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -0.35 - 1.6 * r);
          addRotation(add, B.Head, 1, 0, 0, 0.12);
        }
        break;
      case 'relax': {
        const hugged = HUGGED[def];
        if (!hugged) break;
        // Hugging the teddy, crouched: the arms round its neck, the cheek on its head.
        const shoulder = rig.body.height * rig.scale * CROUCH_SHOULDER;
        addRotation(add, B.Head, 0, 1, 0, 0.5);
        this.hugArms(0.35 + 0.45 * clamp((hugged.at + 0.35 - shoulder) / 0.3, 0, 1), time);
        break;
      }
      case 'smell':
        // Nose down to the flowers.
        addRotation(add, B.spine_03, 1, 0, 0, 0.12);
        addRotation(add, B.Head, 1, 0, 0, 0.3);
        break;
      case 'admire':
        // Looking up into the crown.
        addRotation(add, B.spine_03, 1, 0, 0, -0.08);
        addRotation(add, B.neck_01, 1, 0, 0, -0.25);
        addRotation(add, B.Head, 1, 0, 0, -0.4 + Math.sin(time * 0.4 + seed) * 0.05);
        break;
      case 'talk':
      case 'look':
      case 'listen':
        // Talking to a plant, looking at a bird bath: eyes down to the object (into a mirror, level: `LEVEL_GAZE`).
        if (def && !LEVEL_GAZE.has(def)) addRotation(add, B.Head, 1, 0, 0, 0.2);
        break;
      case 'exercise':
        if (rig.useObj?.stand?.motion === 'bounce') {
          // Arms swing up in the air, knees give on landing.
          const b = Math.sin(((rig.actTime * 1.6) % 1) * Math.PI);
          addRotation(add, B.upperarm_l, 0, 0, 1, -1.3 * b);
          addRotation(add, B.upperarm_r, 0, 0, 1, 1.3 * b);
          const land = Math.max(0, 0.45 - b);
          addRotation(add, B.thigh_l, 1, 0, 0, -0.8 * land);
          addRotation(add, B.thigh_r, 1, 0, 0, -0.8 * land);
          addRotation(add, B.calf_l, 1, 0, 0, 1.6 * land);
          addRotation(add, B.calf_r, 1, 0, 0, 1.6 * land);
          addRotation(add, B.foot_l, 1, 0, 0, -0.8 * land);
          addRotation(add, B.foot_r, 1, 0, 0, -0.8 * land);
        } else if (rig.useObj?.stand?.motion === 'buzz') {
          // Knees soft on the vibrating plate, a little buzz through the body.
          const buzz = Math.sin(time * 70) * 0.012;
          addRotation(add, B.thigh_l, 1, 0, 0, -0.3);
          addRotation(add, B.thigh_r, 1, 0, 0, -0.3);
          addRotation(add, B.calf_l, 1, 0, 0, 0.6);
          addRotation(add, B.calf_r, 1, 0, 0, 0.6);
          addRotation(add, B.foot_l, 1, 0, 0, -0.3);
          addRotation(add, B.foot_r, 1, 0, 0, -0.3);
          addRotation(add, B.Head, 1, 0, 0, buzz);
        } else if (def.includes('sunrise')) {
          // A slow morning stretch: arms up overhead and down again.
          const up = smooth(0.5 - 0.5 * Math.cos(time * 0.9 + seed));
          addRotation(add, B.upperarm_l, 1, 0, 0, -2.7 * up);
          addRotation(add, B.upperarm_r, 1, 0, 0, -2.7 * up);
          addRotation(add, B.spine_02, 1, 0, 0, -0.12 * up);
          addRotation(add, B.Head, 1, 0, 0, -0.2 * up);
        }
        break;
    }
  }

  /**
   * Pedalling (content `seat.pedals`): each foot on its pedal, the ball of the foot on it, in time with
   * the object's loop (the bike's `pedals_turn`; without one playing, at its pace from the use's start).
   */
  private pedal(rig: SimRig, pedals: NonNullable<Seat['pedals']>): void {
    const fits = this.legFits.get(rig.body.name)?.get(rig.cur.key);
    const obj = rig.useObj;
    if (!fits || !obj) return;
    const B = this.set!.bone;
    const s = rig.scale;
    const w = smooth(rig.w);
    // The axle ahead of the Sim (the object's middle, then the pedals' offset from it).
    const ahead = (obj.cx - rig.x) * Math.sin(rig.yaw) + (obj.cz - rig.z) * Math.cos(rig.yaw) + pedals.fwd;
    const phase = (rig.useId >= 0 ? this.loopOf?.(rig.useId) : null) ?? (rig.actTime / PEDAL.turn) % 1;
    for (const [fit, thigh, calf, foot, half] of [
      [fits.l, B.thigh_l, B.calf_l, B.foot_l, 0.5],
      [fits.r, B.thigh_r, B.calf_r, B.foot_r, 0],
    ] as const) {
      // Round from the bottom (the right pedal's at rest) to the back, the top and the front.
      const a = (phase + half) * Math.PI * 2;
      const y = pedals.h - pedals.crank * Math.cos(a) + PEDAL.ankle[0] * s;
      const z = ahead - pedals.crank * Math.sin(a) + PEDAL.ankle[1] * s;
      const turn = this.legTo(fit, (y - rig.cur.rootY) / s, (z - rig.cur.rootFwd) / s);
      addRotation(this.add, thigh, 1, 0, 0, turn[0] * w);
      addRotation(this.add, calf, 1, 0, 0, turn[1] * w);
      // The foot stays about as level as in the clip, its toes dipping as the pedal goes down.
      addRotation(this.add, foot, 1, 0, 0, (-(turn[0] + turn[1]) * 0.75 + 0.15 * Math.sin(a)) * w);
    }
  }

  /**
   * Sitting against a tree (`Seat.shade`, the tree facing `tyaw`): the way to face, out from the trunk on its
   * front or a side, whichever the sun's light falls towards (the crown's shadow), the front unless a side is
   * clearly more so; chosen as they sit down and kept. (Not behind it: they'd walk through the trunk to get there.)
   */
  private shadeSide(rig: SimRig, tyaw: number): number {
    if (!Number.isNaN(rig.shadeYaw)) return rig.shadeYaw;
    const sun = this.sunOf?.();
    const len = sun ? Math.hypot(sun.x, sun.z) : 0;
    let best = 0;
    if (sun && len > 0.05) {
      const towards = (turn: number) => (Math.sin(tyaw + turn) * sun.x + Math.cos(tyaw + turn) * sun.z) / len;
      for (const turn of [Math.PI / 2, -Math.PI / 2]) if (towards(turn) > towards(best) + 0.3) best = turn;
    }
    rig.shadeYaw = wrapAngle(tyaw + best);
    return rig.shadeYaw;
  }

  /**
   * On a deep seat (`Seat.front`), how far forward (m) the hips sit for the knees to clear its front
   * (characters/seatFit.ts `perch`): a child well forward on a sofa, a grown-up a little on a deep one.
   */
  private perch(rig: SimRig, seat: Seat, sp: SeatFit): number {
    const fit = this.legFits.get(rig.body.name)?.get('sit');
    if (seat.front === undefined || seat.back || seat.along || seat.legsOut || seat.pedals || !fit) return 0;
    const s = rig.scale;
    return perch(fit.l, sp.z, (seat.front - seat.fwd) / s, seatSize(rig.body.height)) * s;
  }

  /**
   * Seated: each foot down on the floor under the knee (characters/seatFit.ts `feetDown`), the thigh sloping
   * off the seat's front to reach it, the feet hanging where it can't (a child on a grown-up's chair); on a low
   * seat the knees come up. `stretch`: relaxing, the feet further out. Kept in `rig.seatTurn` for standing up.
   */
  private seatLegs(rig: SimRig, stretch: boolean): void {
    const fits = this.legFits.get(rig.body.name);
    const fit = fits?.get(rig.cur.key) ?? fits?.get('sit');
    if (!fit) return;
    const B = this.set!.bone;
    const s = rig.scale;
    const w = smooth(rig.w);
    const lift = rig.useObj?.lift ?? 0;
    const floor = fit.ankle - (rig.cur.rootY - lift) / s;
    const out = stretch ? SEAT_FIT.stretch * seatSize(rig.body.height) : 0;
    const turns = rig.seatTurn;
    // A footrest (`Seat.rest`): the ankles `REST_BACK` behind its front and as high over it as standing.
    const rest = rig.useObj?.seat.rest;
    const restY = rest ? fit.ankle + (rest[0] + lift - rig.cur.rootY) / s : 0;
    const restZ = rest ? (rest[1] - REST_BACK * seatSize(rig.body.height) - rig.cur.rootFwd) / s : 0;
    for (const [k, side, thigh, calf, foot] of [
      [0, fit.l, B.thigh_l, B.calf_l, B.foot_l],
      [4, fit.r, B.thigh_r, B.calf_r, B.foot_r],
    ] as const) {
      let [y, z] = feetDown(side, floor, out * (k ? 0.85 : 1));
      // (Hanging over the floor, onto the footrest if they reach it.)
      if (rest && y > floor + 0.01 && Math.hypot(restY - side.hy, restZ - side.hz) < side.l1 + side.l2 - 0.01) [y, z] = [restY, restZ - (k ? 0.02 : 0)];
      const turn = this.legTo(side, y, z);
      // The foot about as level as in the clip, its toes dropping a little where it hangs.
      const hang = y > floor + 0.01 ? 0.25 : 0;
      turns[k] = turn[0];
      turns[k + 1] = turn[1];
      turns[k + 2] = -(turn[0] + turn[1]) + hang;
      this.bendLeg(turns, k, thigh, calf, foot, w);
    }
  }

  /** Standing up from a seat: `seatLegs`' turns fading out with the seated pose. */
  private seatLegsOff(rig: SimRig): void {
    const w = 1 - smooth(rig.w);
    if (w <= 0) return;
    const B = this.set!.bone;
    const t = rig.seatTurn;
    for (const [k, thigh, calf, foot] of [
      [0, B.thigh_l, B.calf_l, B.foot_l],
      [4, B.thigh_r, B.calf_r, B.foot_r],
    ] as const) {
      this.bendLeg(t, k, thigh, calf, foot, w);
      addRotation(this.add, thigh, 0, 0, 1, t[k + 3] * w);
    }
  }

  /** A leg bent at the hip, knee and ankle (about x) by the turns kept for it in `rig.seatTurn` (`turns[k..k + 2]`), at weight `w`. */
  private bendLeg(turns: Float32Array, k: number, thigh: number, calf: number, foot: number, w: number): void {
    addRotation(this.add, thigh, 1, 0, 0, turns[k] * w);
    addRotation(this.add, calf, 1, 0, 0, turns[k + 1] * w);
    addRotation(this.add, foot, 1, 0, 0, turns[k + 2] * w);
  }

  /**
   * Lying on a bench to lift (`Bed.feet`: the weight bench), the feet down on the floor either side of it, `out` m
   * from its middle: each thigh out to the side and down off the pad, the shin down to the floor under the knee,
   * the foot flat on it (as a bench press is done). In the lying body's own space (`lie`, the standing convention
   * turned flat: its z up, its y towards the head), so planar leg IK (`legTo`) and then the leg turned out about
   * the hip. Kept in `rig.seatTurn` for getting up.
   */
  private benchLegs(rig: SimRig, out: number): void {
    const fit = this.legFits.get(rig.body.name)?.get('lie');
    if (!fit) return;
    const B = this.set!.bone;
    const s = rig.scale;
    const w = smooth(rig.w);
    // The ankle as high as standing, over the floor under the bench.
    const floor = fit.ankle + ((rig.bed?.lift ?? 0) - rig.cur.rootY) / s;
    const turns = rig.seatTurn;
    for (const [k, side, sx, thigh, calf, foot] of [
      [0, fit.l, -1, B.thigh_l, B.calf_l, B.foot_l],
      [4, fit.r, 1, B.thigh_r, B.calf_r, B.foot_r],
    ] as const) {
      // Out to the side and towards the feet (the knee a little short of the thigh's length on), down to the floor.
      const vx = sx * (out / s - BENCH_HIP * seatSize(rig.body.height));
      const vy = -side.l1 * BENCH_KNEE;
      const across = Math.hypot(vx, vy);
      const turn = this.legTo(side, side.hy - across, floor);
      turns[k] = turn[0];
      turns[k + 1] = turn[1];
      // The foot level on the floor (the toes towards the bench's foot end).
      turns[k + 2] = -0.35;
      turns[k + 3] = Math.atan2(vx, -vy);
      this.bendLeg(turns, k, thigh, calf, foot, w);
      addRotation(this.add, thigh, 0, 0, 1, turns[k + 3] * w);
    }
  }

  /** Leaning back by `angle` (rad) against a seat's slanted back (`Seat.recline`), split over the spine as `legsOut` does, the head tipped forward to keep looking ahead. */
  private recline(angle: number): void {
    const B = this.set!.bone;
    addRotation(this.add, B.spine_01, 1, 0, 0, -0.71 * angle);
    addRotation(this.add, B.spine_02, 1, 0, 0, -0.29 * angle);
    addRotation(this.add, B.Head, 1, 0, 0, 0.64 * angle);
  }

  /**
   * Legs out along the seat (content `seat.legs`: a chaise): nearly straight, the heels on it, the toes up;
   * leant back against the back, the head up to look along the legs. In a short tub (`Seat.room`) the heels
   * stop at its end and the knees come up.
   */
  private legsOut(rig: SimRig, room = Infinity): void {
    const fits = this.legFits.get(rig.body.name)?.get(rig.cur.key);
    if (!fits) return;
    const B = this.set!.bone;
    const w = smooth(rig.w);
    const reach = room / rig.scale;
    addRotation(this.add, B.spine_01, 1, 0, 0, -0.2 * w);
    addRotation(this.add, B.spine_02, 1, 0, 0, -0.08 * w);
    addRotation(this.add, B.Head, 1, 0, 0, 0.18 * w);
    for (const [fit, thigh, calf, foot, bend] of [
      [fits.l, B.thigh_l, B.calf_l, B.foot_l, 0.96],
      [fits.r, B.thigh_r, B.calf_r, B.foot_r, 0.93],
    ] as const) {
      // The ankles a heel's height above the seat, the hips' joints being higher (`seatDrop`).
      const turn = this.legTo(fit, fit.hy - LEGS_OUT_DROP, fit.hz + Math.min((fit.l1 + fit.l2) * bend, reach));
      addRotation(this.add, thigh, 1, 0, 0, turn[0] * w);
      addRotation(this.add, calf, 1, 0, 0, turn[1] * w);
      addRotation(this.add, foot, 1, 0, 0, -(turn[0] + turn[1]) * 0.25 * w);
    }
  }

  /**
   * Whether a Sim sitting on `rig.useObj` (playing `key`, its root at `rootY` and `rootFwd`) puts their feet up on
   * a footstool (`FOOTSTOOLS`), and where (`rig.feetUp`, `rig.feet`): one in front of them, nobody sitting on it,
   * its near half within the legs' reach once slid forward in the seat (`FEET_UP`); the nearest. Not when sitting
   * on one, facing a seat's back (a dressing table), pulled up to a table, at the desk clips, nor for an
   * instrument, a bath or the toilet (`NO_FEET_UP`).
   */
  private footstoolFor(rig: SimRig, key: string, rootY: number, rootFwd: number): boolean {
    const own = rig.useObj;
    const fit = this.legFits.get(rig.body.name)?.get(key)?.l;
    if (key !== 'sit' || !fit || !this.footstools.length || own?.footstool || own?.seat.back || rig.tuck > 0.01 || NO_FEET_UP.has(rig.tag)) return false;
    const s = rig.scale;
    const fx = Math.sin(rig.yaw);
    const fz = Math.cos(rig.yaw);
    // The hips' joints (forward of the Sim's place, height), and how far the ankles go from them.
    const hipFwd = rootFwd + fit.hz * s;
    const hipY = rootY + fit.hy * s;
    const legs = (fit.l1 + fit.l2) * s;
    let best = Infinity;
    for (const o of this.footstools) {
      const f = o.footstool!;
      const dx = o.cx - rig.x;
      const dz = o.cz - rig.z;
      const ahead = dx * fx + dz * fz;
      const right = dx * fz - dz * fx;
      // Its half depth along the Sim's facing and half width across it (a box turned any way).
      const turn = rig.yaw - f.yaw;
      const along = Math.abs(Math.cos(turn)) * f.hd + Math.abs(Math.sin(turn)) * f.hw;
      const across = Math.abs(Math.sin(turn)) * f.hd + Math.abs(Math.cos(turn)) * f.hw;
      const near = ahead - along;
      if (near <= hipFwd || Math.abs(right) > across + FEET_UP.side * 0.5) continue;
      // The feet on its top towards the hips' line, the ankles over it.
      const side = clamp(right, -Math.max(0, across - 0.12), Math.max(0, across - 0.12));
      const up = o.seat.h + FEET_UP.ankle * s - hipY;
      const most = legs * FEET_UP.reach;
      if (Math.abs(side) > FEET_UP.side || Math.abs(up) >= most) continue;
      // As far in as the legs reach, slid forward if need be.
      const out = Math.sqrt(Math.max(0, most * most - up * up - side * side));
      const heel = Math.min(near + Math.min(FEET_UP.heel, along), hipFwd + out + FEET_UP.slide);
      if (heel < near + FEET_UP.edge || heel - hipFwd < legs * FEET_UP.near) continue;
      const slide = Math.max(0, heel - hipFwd - out);
      if (slide >= best) continue;
      // Nobody sitting on it.
      if (this.sims.some((r) => r && r !== rig && r.useObj === o && r.lastPose === Pose.Sit)) continue;
      best = slide;
      rig.feet.set([o.seat.h + FEET_UP.ankle * s, heel, side, slide]);
    }
    rig.feetUp = best < Infinity;
    return rig.feetUp;
  }

  /**
   * Feet up on a footstool (`footstoolFor`): the legs out to it, the heels on its top, the toes up, turned
   * towards it if it's to one side; leant back a little, and slouched back as far as they slid forward.
   */
  private feetUp(rig: SimRig): void {
    const fits = this.legFits.get(rig.body.name)?.get(rig.cur.key);
    if (!fits) return;
    const B = this.set!.bone;
    const w = smooth(rig.w);
    const s = rig.scale;
    const [ankle, heel, side] = rig.feet;
    // (Where the root is now, slid forward as far as it has got.)
    const y = (ankle - rig.cur.rootY) / s;
    const z = (heel - rig.cur.rootFwd - rig.approach) / s;
    const slouch = 0.08 + rig.approach * FEET_UP.slouch;
    addRotation(this.add, B.spine_01, 1, 0, 0, -slouch * w);
    addRotation(this.add, B.Head, 1, 0, 0, slouch * 0.6 * w);
    // (Kept in `rig.seatTurn`, as `seatLegs` keeps its, to fade out standing up.)
    const turns = rig.seatTurn;
    for (const [k, fit, thigh, calf, foot] of [
      [0, fits.l, B.thigh_l, B.calf_l, B.foot_l],
      [4, fits.r, B.thigh_r, B.calf_r, B.foot_r],
    ] as const) {
      const flat = Math.hypot(z - fit.hz, side / s);
      const legTurn = this.legTo(fit, y, fit.hz + flat);
      turns[k] = legTurn[0];
      turns[k + 1] = legTurn[1];
      turns[k + 2] = -(legTurn[0] + legTurn[1]) * 0.3;
      turns[k + 3] = 0;
      this.bendLeg(turns, k, thigh, calf, foot, w);
      addRotation(this.add, thigh, 0, 1, 0, Math.atan2(side / s, z - fit.hz) * w);
    }
  }

  /**
   * Relaxed standing (see `STANCE`): over a skirt the hanging arms move out to clear it, faded in
   * over a few tenths of a second on the clips whose arms hang and out again for everything else.
   */
  private stance(rig: SimRig, pose: number, moving: boolean, dt: number): void {
    const B = this.set!.bone;
    const standing = pose === Pose.Stand && !moving && (!this.force || this.forceStance);
    const arms = standing && rig.skirt && (RELAX_ARMS.has(rig.cur.key) || isTurn(rig.cur.key)) && !(rig.upperW > 0.05) && rig.cuddleT < 0 ? 1 : 0;
    rig.relaxArms += (arms - rig.relaxArms) * (rig.dir?.still ? 1 : Math.min(1, dt * 5));
    const a = smooth(clamp(rig.relaxArms, 0, 1));
    if (a > 1e-3) {
      addRotation(this.add, B.upperarm_l, 0, 0, 1, STANCE.skirt * a);
      addRotation(this.add, B.upperarm_r, 0, 0, 1, -STANCE.skirt * a);
    }
  }

  /**
   * Face: blinks at irregular intervals, eyes that look at the conversation partner (or now and
   * then at the camera / around), an expression from the Sim's emotion and conversation, and the
   * mouth moving while it speaks (to the voice line's visemes when there is one). The eyes are
   * bones (local rotations in the head's frame); the rest are face morphs, mixed by `FaceMixer` and
   * written into the Sim's row after the appearance texels.
   */
  private animateFace(
    rig: SimRig,
    i: number,
    social: number,
    role: number,
    partner: SimRig | null,
    pose: number,
    moving: boolean,
    time: number,
    dt: number,
  ): void {
    const set = this.set!;
    const B = set.bone;
    const mixer = this.mixer;
    if (!mixer || B.eye_l === undefined) return;
    const add = this.add;
    const pair = partner ? Math.min(i, partner.id) : i;
    const asleep = pose === Pose.Lie && LYING.has(rig.cur.key) && (!rig.tagKnown || rig.tag === 'sleep' || rig.tag === 'nap');

    // Target expression: the emotion, then the conversation / activity on top.
    const tgt = this.faceTmp;
    // The friendly resting face, then the emotion on top.
    const emotion = this.emotionIds[rig.emotion - 1] ?? '';
    Object.assign(tgt, NEUTRAL);
    const warmth = COLD.has(emotion) ? 0 : rig.warmth;
    if (warmth > 0) for (const key of FACE_KEYS) tgt[key] += (FRIENDLY[key] ?? 0) * warmth;
    Object.assign(tgt, EXPRESSIONS[emotion] ?? NONE);
    let eyesClosed = 0;
    let talking = false;
    const reaction = social ? SOCIAL_FACES[rig.cur.key] : undefined;
    if (reaction) {
      // How it was taken (`GESTURES`, `REFUSED`): put off, crestfallen, embarrassed, unamused.
      Object.assign(tgt, reaction);
    } else if (social) {
      const me = speaking(pair, role, time);
      switch (social) {
        case Anim.Laugh:
          // The teller grins through the joke; whoever laughs, laughs.
          if (rig.cur.key === 'laugh') {
            Object.assign(tgt, LAUGH);
            tgt.jaw = 0.1 + 0.2 * Math.abs(Math.sin(time * 11 + i));
          } else {
            tgt.smile = Math.max(tgt.smile, 0.5);
            talking = me;
          }
          break;
        case Anim.Flirt:
          Object.assign(tgt, EXPRESSIONS.flirty);
          talking = me;
          break;
        case Anim.Argue:
        case Anim.Fight:
          Object.assign(tgt, EXPRESSIONS.angry);
          talking = me || social === Anim.Fight;
          break;
        case Anim.Kiss:
          Object.assign(tgt, KISS);
          eyesClosed = 1;
          break;
        case Anim.Hug:
          Object.assign(tgt, HUG);
          eyesClosed = 0.85;
          break;
        default:
          talking = me;
          // Listening: a friendly face (outcome 1 = going well, 2 = badly).
          if (!me) tgt.smile = Math.max(tgt.smile, 0.35);
      }
    } else if (rig.tag === 'sing' || (rig.tag === 'talk' && rig.cur.key === 'talk')) {
      talking = true;
      tgt.smile = Math.max(tgt.smile, 0.4);
    }
    if (rig.tag === 'meditate') eyesClosed = 1;
    if (rig.tag === 'read' || rig.tag === 'type' || rig.tag === 'write' || rig.tag === 'paint' || rig.tag === 'cook' || rig.tag === 'wash')
      tgt.gazeDown = Math.max(tgt.gazeDown, 0.55);
    if (asleep) {
      eyesClosed = 1;
      Object.assign(tgt, NEUTRAL, ASLEEP);
    }
    const dir = rig.dir;
    if (dir?.face) Object.assign(tgt, dir.face);
    const dbg = this.faceDebug;
    if (dbg) {
      Object.assign(tgt, NEUTRAL, dbg);
      eyesClosed = dbg.closed ?? 0;
      talking = !!dbg.talk;
    }
    const f = dir?.still ? 1 : 1 - Math.exp(-dt * 6);
    const face = rig.face;
    for (const k of FACE_KEYS) face[k] += (tgt[k] - face[k]) * f;

    // A voice line takes over from the conversation's turn-taking.
    const speech = rig.speech;
    const now = performance.now() / 1000;
    const voiced = !!speech && now - rig.speechStart < (speech.times[speech.times.length - 1] ?? 0) + 0.3;
    if (voiced) talking = true;

    // Blinks: every 1.5 - 6 s at rest, more often while talking (sometimes twice), ~0.17 s each.
    if (time >= rig.nextBlink) {
      rig.blinkAt = time;
      const r = hash(Math.floor(time * 10) + i * 7919);
      const gap = talking ? 1 + 2.6 * hash(Math.floor(time * 13) + i * 31) : 1.5 + 4.5 * hash(Math.floor(time * 13) + i * 31);
      rig.nextBlink = time + (r < 0.15 ? 0.32 : gap);
    }
    const bt = (time - rig.blinkAt) / 0.17;
    const blink = !dir?.still && bt >= 0 && bt < 1 ? (bt < 0.4 ? bt / 0.4 : 1 - (bt - 0.4) / 0.6) : 0;
    const closed = Math.max(eyesClosed, smooth(Math.min(1, blink)));

    // Gaze: the partner's eyes; otherwise, now and then, the camera or a glance around.
    let gy = 0;
    let gp = 0.35 * face.gazeDown;
    const ex = this.heads[i * 3];
    const eyeY = this.heads[i * 3 + 1] - eyesBelowTop(rig);
    const ez = this.heads[i * 3 + 2];
    if (time >= rig.gazeUntil) {
      const r = hash(Math.floor(time * 7) + i * 131);
      const mode = social && partner ? (r < 0.82 ? 1 : 3) : moving ? 0 : r < 0.3 ? 2 : r < 0.65 ? 3 : 0;
      // A big shift of gaze often comes with a blink.
      if (mode !== rig.gazeMode && hash(Math.floor(time * 11) + i * 3) < 0.4 && time - rig.blinkAt > 0.5) rig.nextBlink = time;
      rig.gazeMode = mode;
      rig.gazeUntil = time + 1.2 + 2.8 * hash(Math.floor(time * 5) + i * 17);
      rig.glanceYaw = (hash(Math.floor(time * 3) + i) - 0.5) * 0.7;
      rig.glancePitch = (hash(Math.floor(time * 3) + i + 5) - 0.6) * 0.3;
    }
    if (!asleep && closed < 0.99) {
      const look = rig.audience?.visible ? rig.audience : partner;
      if (rig.gazeMode === 1 && look?.visible) {
        this.gazeAt(rig, ex, eyeY, ez, this.heads[look.id * 3], this.heads[look.id * 3 + 1] - eyesBelowTop(look), this.heads[look.id * 3 + 2]);
        gy = this.gaze[0];
        gp = this.gaze[1];
      } else if (rig.gazeMode === 2) {
        const cam = this.cam;
        if (this.hasCam && this.gazeAt(rig, ex, eyeY, ez, cam[0], cam[1], cam[2])) {
          gy = this.gaze[0];
          gp = this.gaze[1];
        } else {
          gy = rig.glanceYaw;
          gp = rig.glancePitch;
        }
      } else if (rig.gazeMode === 3) {
        gy = rig.glanceYaw;
        gp = rig.glancePitch + 0.3 * face.gazeDown;
      }
    }
    if (dir?.lookAt && !asleep && closed < 0.99 && this.gazeAt(rig, ex, eyeY, ez, dir.lookAt.x, dir.lookAt.y, dir.lookAt.z)) {
      gy = this.gaze[0];
      gp = this.gaze[1];
    } else if (dir?.still) {
      gy = 0;
      gp = 0.35 * face.gazeDown;
    }
    gy = clamp(gy, -0.42, 0.42);
    gp = clamp(gp, -0.28, 0.32);
    // Saccades: quick, not smoothed over seconds.
    const fe = dir?.still ? 1 : 1 - Math.exp(-dt * 22);
    rig.eyeYaw += (gy - rig.eyeYaw) * fe;
    rig.eyePitch += (gp - rig.eyePitch) * fe;

    // Eye bones (local axes = the head's: x right, y up, z forward; +x rotation turns the front down).
    addLocal(add, B.eye_l, 0, 1, 0, rig.eyeYaw);
    addLocal(add, B.eye_l, 1, 0, 0, rig.eyePitch);
    addLocal(add, B.eye_r, 0, 1, 0, rig.eyeYaw);
    addLocal(add, B.eye_r, 1, 0, 0, rig.eyePitch);

    // Morphs: the expression (lids following the eyes), then speech. Faces too small on screen to
    // show them skip their morphs (in the vertex shader too).
    const morphRow = this.row(i) + set.bones.length * 16 + APPEARANCE_TEXELS * 4;
    if (this.facePx[i] < MORPH_PX && !dir && !dbg) {
      mixer.off(this.data, morphRow);
      return;
    }
    mixer.begin();
    mixer.mood(tgt.mood, 1, rig.faceL, rig.faceR);
    mixer.expression(tgt, rig.faceL, rig.faceR);
    if (!talking && !asleep && !dir?.still && !dbg) mixer.micro(time, rig.seed, 1);
    mixer.settle(rig.faceState, f);
    mixer.eyes(closed, rig.eyePitch);
    if (voiced) {
      mixer.speaking(1);
      this.speak(rig, speech!, now - rig.speechStart);
    } else if (talking) {
      mixer.speaking(1);
      this.babble(rig, i, time);
    } else if (dbg?.viseme) {
      mixer.viseme(dbg.viseme, 1, 1);
    }
    if (dbg?.channels) mixer.raw(dbg.channels);
    mixer.write(this.data, morphRow);
  }

  /**
   * Visemes of the voice line at `t` (s from its first sample): the mouth moves a little ahead of
   * the sound, and each shape blends in from the last over ~70 ms; quiet stretches open less.
   */
  private speak(rig: SimRig, track: VisemeTrack, t: number): void {
    const mixer = this.mixer!;
    const times = track.times;
    const n = track.visemes.length;
    if (!n) return;
    const tt = t + 0.04;
    let k = rig.speechSeg;
    if (k >= n || tt < times[k]) k = 0;
    while (k < n - 1 && tt >= times[k + 1]) k++;
    rig.speechSeg = k;
    if (tt < times[0] - 0.08 || tt > times[n] + 0.08) return;
    const e = track.energy;
    const energy = e.length ? e[Math.min(e.length - 1, Math.max(0, Math.floor(t * ENERGY_RATE)))] : 1;
    // Fade in before the first segment and out after the last.
    const edge = Math.min(1, (tt - times[0] + 0.08) / 0.08, (times[n] + 0.08 - tt) / 0.08);
    const a = smooth(clamp((tt - times[k]) / 0.07, 0, 1));
    mixer.viseme(track.visemes[k], a * edge, energy);
    if (k > 0 && a < 1) mixer.viseme(track.visemes[k - 1], (1 - a) * edge, energy);
  }

  /** Talking without a voice line: syllable-like mouth shapes, in phrases. */
  private babble(rig: SimRig, i: number, time: number): void {
    const rate = 5.5;
    const s = time * rate + rig.seed * 40;
    const k = Math.floor(s);
    const f = s - k;
    const phrase = Math.sin(time * 1.9 + rig.seed * 9);
    if (phrase < -0.55) return;
    const env = Math.sin(Math.PI * f) * (0.55 + 0.45 * Math.min(1, phrase + 0.55));
    const v = BABBLE[Math.floor(hash(k * 7 + i * 131) * BABBLE.length)];
    this.mixer!.viseme(v, env, 0.6);
  }

  /** Lip sync: `track` plays on Sim row `index` from `startTime` (s, `performance.now` clock); null stops it. */
  setSpeech(index: number, track: VisemeTrack | null, startTime: number): void {
    const rig = this.sims[index];
    if (!rig) return;
    rig.speech = track && track.visemes.length ? track : null;
    rig.speechStart = startTime;
    rig.speechSeg = 0;
  }

  // --- helpers --------------------------------------------------------------------------

  private row(i: number): number {
    return i * this.width * 4;
  }

  /**
   * Uploads the pose texture's rows that changed since the last upload (Sims drawn now, or just
   * hidden), as one band of rows: a few of the 64 most frames, none while the game is paused.
   */
  private upload(): void {
    const stride = this.width * 4;
    const data = this.data;
    const uploaded = this.uploaded;
    let lo = -1;
    let hi = -1;
    for (let i = 0; i < MAX_CHARACTERS; i++) {
      if (!this.visible[i] && !this.uploadedVisible[i]) continue;
      this.uploadedVisible[i] = this.visible[i];
      const end = (i + 1) * stride;
      let k = i * stride;
      while (k < end && data[k] === uploaded[k]) k++;
      if (k === end) continue;
      uploaded.set(data.subarray(k, end), k);
      if (lo < 0) lo = i;
      hi = i;
    }
    this.posed = lo >= 0;
    if (lo >= 0) this.view.uploadPoseRows(lo, hi - lo + 1);
  }

  /** The seat a sitting Sim is on: the object they use if it's a seat, else the seat under them
   * (content `seatNear`: watching TV from the sofa). */
  private seatUnder(rig: SimRig, useObj: PlacedObject | null | undefined): PlacedObject | null {
    if (useObj && useObj.restful) return useObj;
    const under = this.objectAt(rig.x, rig.z);
    return under ?? useObj ?? null;
  }

  private objectAt(x: number, z: number): PlacedObject | null {
    let best: PlacedObject | null = null;
    let bestD = Infinity;
    for (const o of this.objects) {
      if (x < o.minX - 0.05 || x > o.maxX + 0.05 || z < o.minZ - 0.05 || z > o.maxZ + 0.05) continue;
      const d = Math.abs(x - o.cx) + Math.abs(z - o.cz);
      if (o.restful && d < bestD) {
        best = o;
        bestD = d;
      }
    }
    return best;
  }

  /** The object a standing Sim faces (within reach), if any. */
  private objectInFront(rig: SimRig): PlacedObject | null {
    const fx = rig.x + Math.sin(rig.yaw) * 0.55;
    const fz = rig.z + Math.cos(rig.yaw) * 0.55;
    let best: PlacedObject | null = null;
    let bestD = 0.6;
    for (const o of this.objects) {
      if (!USE[o.def]) continue;
      const qx = clamp(fx, o.minX, o.maxX);
      const qz = clamp(fz, o.minZ, o.maxZ);
      const d = Math.hypot(fx - qx, fz - qz);
      if (d < bestD) {
        best = o;
        bestD = d;
      }
    }
    return best;
  }

  /** Head-relative eye yaw / pitch towards a point (into `gaze`); false if it's behind the Sim. */
  private gazeAt(rig: SimRig, ex: number, ey: number, ez: number, tx: number, ty: number, tz: number): boolean {
    const dx = tx - ex;
    const dz = tz - ez;
    this.gaze[0] = wrapAngle(Math.atan2(dx, dz) - rig.yaw - rig.lookYaw);
    this.gaze[1] = -Math.atan2(ty - ey, Math.hypot(dx, dz)) - rig.lookPitch * 0.7;
    return Math.abs(this.gaze[0]) < 1.4;
  }

  /**
   * What a Sim wears now (`Wear`): pyjamas for a night's sleep (lying in a bed, `sleep`), the shoes off
   * for a nap on a bed (a thing slept in; on a sofa or a lounger, or outdoors, the day clothes stay as they
   * are), a towel in the shower and sat in a bath (swimwear in a hot tub, `BATH_WEAR`), else the day clothes.
   * They change as they lie back (the crossfade half done), sit down in the water or stop under the shower.
   */
  private wearFor(rig: SimRig, pose: number, moving: boolean, w: number, useObj: PlacedObject | null | undefined, tag: string): Wear {
    if (moving) return 'day';
    if (pose === Pose.Lie) {
      if (!LYING.has(rig.cur.key) || w <= 0.5) return 'day';
      // (A lying Sim whose use isn't known is taken to sleep, as its face and its quilt are.)
      if (!rig.tagKnown || tag === 'sleep') return 'night';
      return tag === 'nap' && rig.bed && this.sleptIn(rig.bed.def) ? 'rest' : 'day';
    }
    const def = useObj?.def ?? '';
    const water = BATH_WEAR[def] ?? (tag === 'bath' || tag === 'shower' ? 'towel' : null);
    return water && (pose === Pose.Sit || tag === 'shower') ? water : 'day';
  }

  /** Things slept in (a use tagged `sleep`: beds, the crib), by def (`wearFor`). */
  private readonly beds = new Map<string, boolean>();
  private sleptIn(def: string): boolean {
    let bed = this.beds.get(def);
    if (bed === undefined) this.beds.set(def, (bed = !!this.content?.object(def)?.interactions.some((x) => x.tags?.includes('sleep'))));
    return bed;
  }

  /** Puts `worn` (one of its `outfits`) on Sim `i`: its appearance texels, and the shells it doesn't draw (`bare`). */
  private wear(rig: SimRig, i: number, worn: OutfitParams): void {
    rig.worn = worn;
    writeOutfit(worn, this.data, this.row(i) + this.set!.bones.length * 16);
    if (this.bare[i] !== worn.bare) this.shadowsBare = true;
    this.bare[i] = worn.bare;
    rig.skirt = worn.parts.includes('bottom.skirt') && !(worn.bare & SHELL_BITS.bottom);
  }

  /**
   * How far Sim `i`'s thighs are turned forward from its hips, from its skinning matrices just written (0 at
   * rest .. 1 as `sit` has them, `SIT_FLEX`; both thighs' mean, at its body's size): appearance texel 4 b, how much
   * of the shells' seated corrective it wears (a skirt on the lap, not standing out round the seat; material.ts).
   */
  private writeSit(rig: SimRig, i: number): void {
    const set = this.set!;
    const d = this.data;
    const row = this.row(i);
    const p = row + set.bone.pelvis * 16;
    let flex = 0;
    for (const thigh of [set.bone.thigh_l, set.bone.thigh_r]) {
      // The thigh's rest down (0, -1, 0) as posed, in the pelvis' rest frame (columns: the matrices are column-major).
      const o = row + thigh * 16;
      const y = -(d[p + 4] * d[o + 4] + d[p + 5] * d[o + 5] + d[p + 6] * d[o + 6]);
      const z = -(d[p + 8] * d[o + 4] + d[p + 9] * d[o + 5] + d[p + 10] * d[o + 6]);
      flex += Math.max(0, Math.atan2(z, -y));
    }
    // (From `SIT_FROM`: a stride's swing of the legs leaves the skirt as it hangs.)
    d[row + set.bones.length * 16 + 18] = clamp((flex / 2 - SIT_FROM) / (SIT_FLEX - SIT_FROM), 0, 1) * rig.size;
  }

  /** Someone's `bare` changed: the shells' shadow copies are drawn again for the Sims that wear them now (`bareShadows`). */
  private shadowsBare = false;

  /**
   * The shells' shadow copies drawn for the Sims whose clothes now draw them (the drawn shells are left out
   * per frame, `faceDetail`; the shadow copies keep their rows, so only when someone's `bare` changes).
   */
  private bareShadows(): void {
    this.shadowsBare = false;
    for (const pm of this.shadowMeshes.values()) {
      if (!pm.shell || !pm.enabled) continue;
      const ids = [...pm.all.filter((_, j) => j % 4 === 0)].filter((id) => !(pm.shell & this.bare[id]));
      if (ids.length === pm.count && ids.every((id, j) => pm.settings[j * 4] === id)) continue;
      // (Nobody: not drawn, its rows kept.)
      if (ids.length) {
        const settings = new Float32Array(ids.length * 4);
        ids.forEach((id, j) => (settings[j * 4] = settings[j * 4 + 1] = id));
        pm.settings = settings;
      }
      pm.count = ids.length;
      this.view.partRows(pm.mesh, pm.settings, pm.count, ids.length ? 'new' : 'count');
    }
  }

  /**
   * Blanket over a Sim asleep in a bed: fitted to the slot (half of a double bed, or a single bed)
   * and to the bed's foot end, pulled up from the foot end over ~0.5 s once the Sim lies flat (and
   * pushed back down as they get up); not drawn otherwise. A quilt is drawn only for the Sims covered
   * now, packed afresh each frame to the front of its buffer: the thin instances cost the sleepers, not
   * every resident in the town (and not the buffer's capacity).
   */
  private writeBlanket(rig: SimRig, i: number, pose: number, dt: number, fresh: boolean): void {
    // Its body's own quilt (a life stage's, draped over its own pose), else its family's base body's.
    const own = this.blankets.get(rig.body.name);
    const blanket = own ?? this.blankets.get(rig.body.family);
    if (!blanket) return;
    const bed = rig.bed;
    // Only once the body lies flat (the crossfade from sitting on the edge done): pulled up while the legs
    // still swing up onto the bed, it showed the shoes through it halfway down the bed.
    const flat = rig.w > 0.97 || rig.cover > 0;
    const under = pose === Pose.Lie && LYING.has(rig.cur.key) && flat && !!bed && BLANKET_BEDS.has(bed.def) && (rig.tagKnown ? rig.tag === 'sleep' || rig.tag === 'nap' : true);
    rig.cover = fresh ? (under ? 1 : 0) : clamp(rig.cover + (under ? dt * 2 : -dt * 3), 0, 1);
    if (rig.cover <= 0 || !bed) return;
    const d = blanket.drawn++;
    const m = blanket.matrices;
    const o = d * 16;
    const s = rig.scale;
    const fx = Math.sin(rig.yaw);
    const fz = Math.cos(rig.yaw);
    // Getting up: the Sim is already off the bed; the quilt stays on its slot as it slides off.
    const x = rig.fromBed ? rig.lieX : rig.x;
    const z = rig.fromBed ? rig.lieZ : rig.z;
    // Bed extent across the Sim (slot width) and the distance to its foot end.
    const across = Math.abs(fz) * (bed.maxX - bed.minX) + Math.abs(fx) * (bed.maxZ - bed.minZ);
    // Two to a wide bed: each quilt half the bed and a little more, moved a little towards its middle, so the
    // two meet over it (the one lifted a hair over the other) with no strip of the bed's own cover between.
    const wide = across >= 1.8;
    // (Tucked in between a bed's sides: `Bed.tuck`.)
    const tuck = BEDS[bed.def]?.tuck;
    const slot = tuck ? tuck[0] : wide ? across / 2 + BLANKET_MEET * 2 : Math.min(across - 0.08, 1.0);
    const side = wide ? Math.sign((x - bed.cx) * fz - (z - bed.cz) * fx) * BLANKET_MEET : 0;
    const tx = fx > 1e-3 ? (bed.maxX - x) / fx : fx < -1e-3 ? (bed.minX - x) / fx : Infinity;
    const tz = fz > 1e-3 ? (bed.maxZ - z) / fz : fz < -1e-3 ? (bed.minZ - z) / fz : Infinity;
    const foot = Math.min(tx, tz) - (BEDS[bed.def]?.foot ?? 0.05);
    // A body smaller than the one the quilt was made for (a child's with no quilt of its own yet): the quilt
    // shrunk to it along the body and in height, from its feet, so its head is out (the width stays the bed's).
    const zs = own ? s : s * rig.size;
    const z0 = lieCentre(rig) - LIE_CENTRE * zs;
    // The quilt past the toes reaches the bed's foot end (local units, at the Sim's scale); tucked in, as close
    // as just past the toes.
    blanket.ext[d * 2] = clamp((foot - z0) / zs - blanket.footZ, tuck ? -0.3 : -0.22, 1.2);
    const c = smooth(rig.cover);
    // Pulled up: the quilt from its foot end to this line (local z) at full height, the rest folded away
    // under it (`BlanketPlugin`), so nothing shows through it on the way up or down.
    blanket.ext[d * 2 + 1] = c >= 1 ? -9 : blanket.footZ - (blanket.footZ - blanket.topZ + 0.02) * c;
    const sx = (slot / BLANKET_WIDTH) * (0.9 + 0.1 * c);
    // In the bed's own bedding colour.
    quiltTint(bed.quilt?.colour ?? (bed.quilt && this.beddingColours.get(bed.quilt.key)) ?? QUILT_COLOUR, blanket.tint, d * 3);
    // Two Sims in one bed: offset the quilts a hair so their tops don't z-fight where they meet.
    const lift = (i % 2) * 0.004;
    // Rows: x axis, y axis, z axis, translation (the game's row-vector layout, `shared/math.ts`).
    m[o] = fz * sx;
    m[o + 1] = 0;
    m[o + 2] = -fx * sx;
    m[o + 3] = 0;
    m[o + 4] = 0;
    m[o + 5] = zs;
    m[o + 6] = 0;
    m[o + 7] = 0;
    m[o + 8] = fx * zs;
    m[o + 9] = 0;
    m[o + 10] = fz * zs;
    m[o + 11] = 0;
    const r = (bed.rot * Math.PI) / 2;
    const right = tuck ? tuck[1] : 0;
    m[o + 12] = x - fz * side + fx * z0 + Math.cos(r) * right;
    m[o + 13] = rig.bedH + lift;
    m[o + 14] = z + fx * side + fz * z0 - Math.sin(r) * right;
    m[o + 15] = 1;
  }

  /** A body's quilt (`blankets`), draped over its lying pose, every row's instance hidden. */
  private addQuilt(body: BodyData, lying: { positions: Float32Array; chestZ: number; feetZ: number }): void {
    const geometry = quiltGeometry(lying.positions, lying.chestZ, lying.feetZ);
    const matrices = new Float32Array(MAX_CHARACTERS * 16);
    const ext = new Float32Array(MAX_CHARACTERS * 2);
    const tint = new Float32Array(MAX_CHARACTERS * 3);
    const quilt = this.view.createQuilt(`simBlanket:${body.name}`, geometry, matrices, ext, tint);
    this.blankets.set(body.name, { quilt, footZ: geometry.footZ, topZ: geometry.topZ, matrices, ext, tint, drawn: 0, shown: 0, base: body.name === body.family });
  }

  /**
   * A bed's bedding (`PlacedObject.quilt`): the part `QUILT_PART` names or its "Bedding", in the colour the
   * player gave it, else the file's own (looked up once per model, `beddingColours`); null without one.
   */
  private beddingOf(id: string, model: string | undefined, colors: PartColors | undefined): PlacedObject['quilt'] {
    const entry = model ? this.assets.get(model, 'model') : undefined;
    const part = QUILT_PART[id] ?? paintSlots(entry).find((s) => s.label === 'Bedding')?.material;
    if (!part) return null;
    const key = `${model}#${part}`;
    const colour = colors?.[part];
    if (!colour && !this.beddingColours.has(key)) {
      this.beddingColours.set(key, null);
      this.view.fileColour(entry, part).then(
        (c) => this.beddingColours.set(key, c ?? null),
        () => this.beddingColours.delete(key),
      );
    }
    return { key, colour };
  }

  /** A body's seated pelvis offsets (`sitPelvis`, `floorPelvis`) and its legs in the clips (`legFits`). */
  private measureBody(body: BodyData): void {
    this.sitPelvis.set(body.name, measureSit(this.kit, body));
    this.floorPelvis.set(body.name, new Map(FLOOR_SITS.map((f) => [f.name, measureSit(this.kit, body, SIT_CLIPS[f.name].sit)])));
    this.legFits.set(body.name, measureLegs(this.kit, body));
  }

  /**
   * Turns a leg (`fit`, its clip's) so its ankle reaches (`y`, `z`) (body space, scale 1; the knee bent
   * forward, out of reach straightened towards it): the thigh's and calf's rotations about x into
   * `legAdd`, added to the clip's (see `addRotation`: a positive x rotation swings a limb from +z
   * towards -y).
   */
  private legTo(fit: LegSide, y: number, z: number): Float32Array {
    legTurn(fit, y, z, this.legAdd);
    return this.legAdd;
  }

  /**
   * Lying in a hammock's sling (`Bed.sling`; root `fwd0` as on a bed): the root's height and forward
   * offset that put the hips down in the dip with the back on the sling and, the upper body raised by
   * `rig.slingLift` (rad, about the spine's base), the head on the pillow. Null without the body's fit.
   */
  private inSling(rig: SimRig, sling: NonNullable<Bed['sling']>, lift: number, fwd0: number): Float32Array | null {
    const fit = this.legFits.get(rig.body.name)?.get('lie');
    if (!fit) return null;
    const s = rig.scale;
    const dy = (fit.head[0] - fit.spine[0]) * s;
    const dz = (fit.head[1] - fit.spine[1]) * s;
    const reach = Math.hypot(dy, dz);
    const lean = Math.atan2(dz, dy);
    // The head as far above the pillow as above a mattress.
    const want = slingTop(sling, SLING.crown - sling.head) + lift + lieBack(rig) + fit.head[1] * s + SLING.pillow;
    const out = this.slingAt;
    let fwd = fwd0;
    for (let k = 0; k < 3; k++) {
      // Along the sling (from its middle towards the feet): the spine's base, which the root follows down.
      const base = fwd - fit.spine[0] * s;
      out[0] = slingTop(sling, base) + lift + lieBack(rig) + SLING.back;
      const rise = clamp(Math.asin(clamp((want - out[0] - fit.spine[1] * s) / reach, -1, 1)) - lean, 0, SLING.lift);
      rig.slingLift = rise;
      // Moved along so the head, raised, is over the pillow.
      fwd += SLING.crown - sling.head - (base - (dy * Math.cos(rise) - dz * Math.sin(rise)));
    }
    out[1] = fwd;
    return out;
  }

  /**
   * The pose in a hammock's sling (`inSling`): the back curved up from the hips to the raised head, the
   * knees up and the heels on the sling inside the spreader bar.
   */
  private slingPose(rig: SimRig, sling: NonNullable<Bed['sling']>): void {
    const fits = this.legFits.get(rig.body.name)?.get('lie');
    if (!fits) return;
    const B = this.set!.bone;
    const add = this.add;
    const w = smooth(rig.w);
    const rise = rig.slingLift * w;
    addRotation(add, B.spine_01, 1, 0, 0, rise * 0.5);
    addRotation(add, B.spine_02, 1, 0, 0, rise * 0.3);
    addRotation(add, B.spine_03, 1, 0, 0, rise * 0.2);
    const s = rig.scale;
    const lift = rig.bed?.lift ?? 0;
    const { rootY, rootFwd } = rig.cur;
    for (const [fit, thigh, calf, foot, short] of [
      [fits.l, B.thigh_l, B.calf_l, B.foot_l, 0],
      [fits.r, B.thigh_r, B.calf_r, B.foot_r, 0.05],
    ] as const) {
      // Along the sling: the hip, and the ankle as far as the leg reaches, short of the spreader bar.
      const hip = rootFwd - fit.hy * s;
      const at = Math.min(sling.half - SLING.feet - short, hip + (fit.l1 + fit.l2) * s * 0.97);
      const turn = this.legTo(fit, (rootFwd - at) / s, (slingTop(sling, at) + lift + SLING.ankle * s - rootY) / s);
      addRotation(add, thigh, 1, 0, 0, turn[0] * w);
      addRotation(add, calf, 1, 0, 0, turn[1] * w);
      addRotation(add, foot, 1, 0, 0, -(turn[0] + turn[1]) * 0.4 * w);
    }
  }

  /**
   * The mesh of a part of a body (`shadow`: its shadow-only copy, drawn into the sun's shadow map
   * but not by the camera).
   */
  private partMesh(body: BodyData, part: string, shadow = false): PartMesh<M> | null {
    const key = `${body.name}:${part}`;
    const meshes = shadow ? this.shadowMeshes : this.meshes;
    const cached = meshes.get(key);
    if (cached) return cached;
    if (!body.parts.has(part)) return null;
    const mesh = this.view.createPart(body, part, shadow);
    const range = shadow ? Range.Always : part.startsWith('lod.') ? Range.Far : part === 'body' || /^(body|top|bottom|shoes)\./.test(part) ? Range.Near : Range.Always;
    const pm: PartMesh<M> = { mesh, enabled: true, settings: new Float32Array(0), count: 0, all: new Float32Array(0), detail: shadow ? 0 : (DETAIL_PX[part] ?? 0), range, shell: shellBit(part) };
    meshes.set(key, pm);
    if (!shadow) this.parts.set(mesh, pm);
    return pm;
  }

  /** Enables or disables a part mesh. */
  private enablePart(pm: PartMesh<M>, on: boolean): void {
    pm.enabled = on;
    this.view.partEnabled(pm.mesh, on);
  }

  /**
   * The Sim rows a part mesh draws now: one each four numbers of `rows`, `count` of them (its
   * instances); null for any other mesh. (The mirrors draw only the residents they show.)
   */
  rowsOf(mesh: object): { rows: Float32Array; count: number } | null {
    const pm = this.parts.get(mesh);
    return pm ? { rows: pm.settings, count: pm.count } : null;
  }

  /** Draws `pm` for Sim rows `ids` (instances). */
  private setInstances(pm: PartMesh<M>, ids: readonly number[]): void {
    const n = ids.length;
    const settings = new Float32Array(n * 4);
    ids.forEach((id, j) => {
      settings[j * 4] = settings[j * 4 + 1] = id;
    });
    pm.settings = settings;
    pm.all = settings.slice();
    pm.count = n;
    // Parts drawn by distance rewrite their rows each frame.
    this.view.setPartRows(pm.mesh, settings, !(pm.detail === 0 && pm.range === Range.Always));
  }

  /**
   * How big each Sim's face is on screen (`facePx`: px per 5 cm), and the small face parts (teeth,
   * tongue, lashes, brows) drawn only on the faces big enough to show them.
   */
  private faceDetail(count: number): void {
    const c = this.cam;
    const px = this.facePx;
    // (No camera, or one without perspective: full detail.)
    if (!this.hasCam || Number.isNaN(c[4])) {
      px.fill(1e6);
    } else {
      const k = (0.05 * c[3]) / (2 * Math.tan(c[4] / 2));
      for (let i = 0; i < count; i++) {
        const rig = this.sims[i];
        if (!rig || !this.visible[i]) {
          px[i] = 0;
          continue;
        }
        // Directed Sims (the creator's stage, portraits drawn by their own cameras): full detail.
        if (rig.dir) {
          px[i] = 1e6;
          continue;
        }
        const d = Math.hypot(this.heads[i * 3] - c[0], this.heads[i * 3 + 1] - c[1], this.heads[i * 3 + 2] - c[2]);
        px[i] = (k * rig.scale * rig.head) / Math.max(d, 0.05);
      }
    }
    // Low detail far from the camera (with a gap between switching down and up).
    const far = this.far;
    for (let i = 0; i < count; i++) {
      if (far[i] && px[i] > LOD_NEAR_PX) far[i] = 0;
      else if (!far[i] && px[i] < LOD_FAR_PX && this.hasLod) far[i] = 1;
    }
    for (const pm of this.meshes.values()) {
      if ((!pm.detail && pm.range === Range.Always) || !pm.enabled) continue;
      const all = pm.all;
      const out = pm.settings;
      const near = pm.range === Range.Near;
      const ranged = pm.range !== Range.Always;
      let n = 0;
      let changed = false;
      for (let j = 0; j < all.length; j += 4) {
        const id = all[j];
        if (px[id] < pm.detail) continue;
        if (ranged && (far[id] === 1) === near) continue;
        if (pm.shell & this.bare[id]) continue;
        if (out[n * 4] !== id) {
          out[n * 4] = out[n * 4 + 1] = id;
          changed = true;
        }
        n++;
      }
      if (changed || pm.count !== n) {
        pm.count = n;
        this.view.partRows(pm.mesh, out, n, 'rows');
      }
    }
  }
}

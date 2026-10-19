/**
 * Resident voices in a game (docs/design/voices.md). Watches the render snapshot and decides
 * who says what, then plays the line once the voice worker has made it, and hands its mouth
 * shapes to the renderer while it plays (`Renderer.setSpeech`).
 *
 * Only two kinds of resident speak:
 * - the one whose panel is open (`game.inspected`), and whoever they are talking to;
 * - both sides of a conversation the player started from the social menu.
 *
 * Lines are hand-written (`lines.ts`), or written by a small language model from who the
 * resident is and what's going on (`talk/`, the `voiceWriter` setting), from what happened. With subtitles on, the speech bubble shows the line while it plays.
 *
 * Everyone speaks with Supertonic 3 on the GPU (`voices.ts`); babies don't speak. If lines keep
 * coming late or slow the frames, voices pause for the session.
 *
 * Presentation only: nothing goes back to the simulation. Work per frame is a cheap check; the
 * snapshot is read about ten times a second.
 */

import type { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import type { WorldStructure } from '../core/protocol';
import { llmReady, llmStatus, unloadLlm } from '../llm';
import type { Renderer } from '../render/types';
import { settings } from '../settings/settings.svelte';
import { deviceTest, llmWrites, LlmWriter, loadTalk, WRITE_MS } from '../talk';
import { game, toast } from '../ui/state.svelte';
import { fill, loadLines, Lines, speechPieces, type Line } from './lines';
import { playClip, updateVoiceVolume, type Playing } from './player';
import { SAMPLE_RATE } from './protocol';
import { hasClip, speak, unloadVoice, voiceReady, voiceStatus, type Clip } from './service.svelte';
import type { VisemeTrack } from './visemes';
import { voiceFor, withTone, type VoiceParams } from './voices';
import { scripted, type LineRequest } from './writer';

const TICK_MS = 100;
/** At most this many voices at once, and lines being made at once. */
const MAX_VOICES = 2;
const MAX_IN_FLIGHT = 2;
/** A line not ready by then is dropped: the moment has passed. */
const MAX_WAIT_MS = 4000;
/** The same for a line the language model writes (it has `WRITE_MS` of it, speech the rest). */
const MAX_WAIT_LLM_MS = 6000;
/** Lines being written by the language model at once (it writes one after another). */
const MAX_WRITING = 2;
/** A line made ahead while its resident walks over is dropped if they haven't arrived by then. */
const AHEAD_MS = 60_000;
/** Lines this late, so many times in a row, pause voices for the session. */
const LATE_RUN = 3;
/** Frames this slow, in this share of the frames drawn while speech is being made, pause them too. */
const SLOW_FRAME_MS = 50;
const SLOW_SHARE = 0.1;
const FRAME_WINDOW = 120;
/** An answer waits this long at most for the other side to finish their line. */
const ANSWER_WAIT_MS = 15000;
/** A subtitle stays this long after its line ends. */
const SUBTITLE_LINGER_MS = 1200;
/** Quiet time after a resident's own thought before the next one. */
const THOUGHT_GAP_MS = 6000;
/** How long a conversation the player ordered may take to begin (residents may walk over first). */
const PLAYER_TALK_MS = 60000;
/** Not every new action or mood is worth a line. */
const ACTION_CHANCE = 0.6;
const EMOTION_CHANCE = 0.5;

interface SimState {
  social: number;
  outcome: number;
  action: number;
  /** What they use: object definition and interaction indices (-1: nothing, or an older layout). */
  def: number;
  interaction: number;
  emotion: number;
  thought: number;
  /** Calling out someone's name: their id + 1, else 0. */
  call: number;
  /** When this resident may next say a thought (ms, performance clock). */
  quietUntil: number;
}

export interface VoiceDirectorStats {
  speaking: number;
  inFlight: number;
  /** Lines the language model is writing, and lines made ahead for residents on their way somewhere. */
  writing: number;
  ahead: number;
  spoken: number;
  dropped: number;
  language: 'off' | 'english' | 'paused';
  writer: 'lines' | 'llm';
}

/** Where a line's text shows while it plays (the speech bubbles). */
export interface Subtitles {
  say(id: number, text: string, ms: number): void;
}

/** What the voice director needs from the game besides the snapshot. */
export interface VoiceSources {
  /** The lot in view (rooms, for where residents are). */
  world: () => WorldStructure | null;
  subtitles?: Subtitles;
}

interface Speaking {
  id: number;
  playing: Playing;
  /** The renderer row (snapshot index) its mouth shapes went to. */
  row: number;
  track: VisemeTrack;
  /** The whole line was said (or stopped). */
  done: Promise<void>;
}

export class VoiceDirector {
  private lines: Lines | null = null;
  private linesLanguage = '';
  private llmWriter: LlmWriter | null = null;
  /** The resident whose persona the language model last read ahead. */
  private primed: number | null = null;
    /** Lines the language model is writing, by resident (a newer event replaces theirs). */
  private readonly writing = new Map<number, AbortController>();
  /**
   * Lines made ahead, by resident: what they set out to use (`useKey`), whether the dice said
   * to speak, and the line with its speech on the way (null: nothing written ahead; the
   * animation, unknown while walking, may still find a hand-written line on arrival).
   */
  private readonly ahead = new Map<number, { key: string; speak: boolean; line: Promise<Line | null> | null; at: number; control?: AbortController }>();
  private readonly sims = new Map<number, SimState>();
  /** Each resident's snapshot index at the last tick. */
  private readonly rows = new Map<number, number>();
  private readonly speaking: Speaking[] = [];
  private inFlight = 0;
  /** Lines played, and lines given up on (engine busy, too late, too many voices). */
  private spoken = 0;
  private dropped = 0;
  /** Speech came late or slowed the game: no voices for the rest of the session. */
  private paused = false;
  private lateRun = 0;
  private lastFrame = 0;
  /** Frames drawn while speech was being made, and those over `SLOW_FRAME_MS`; the same while not. */
  private readonly frames = { busy: 0, busySlow: 0, idle: 0, idleSlow: 0 };
  private lastTick = 0;
  private counter = 0;
  private disposed = false;
  /** The conversation the player started: these two speak until it ends. */
  private playerTalk: { a: number; b: number; until: number; started: boolean } | null = null;
  private readonly head = { x: 0, y: 0, z: 0 };
  private readonly screen = { x: 0, y: 0 };
  /** With `?debug`: the last lines chosen, as `id key: text` (`window.__voices.heard`). */
  readonly heard: string[] = [];
  private readonly debug = new URLSearchParams(location.search).has('debug');

  constructor(
    private readonly content: Content,
    private readonly renderer: Renderer,
    private readonly sources: VoiceSources = { world: () => null },
  ) {
    if (this.debug) Object.assign(window, { __voices: this });
  }

  /** The language model's writer, with `?debug` its log (`__voices.writer.log`). */
  get writer(): LlmWriter | null {
    return this.llmWriter;
  }


  /** Lines for the voice language, (re)loaded when it changes. */
  private ensureLines(): void {
    const language = settings.voiceLanguage;
    if (this.linesLanguage === language) return;
    this.linesLanguage = language;
    this.lines = null;
    this.llmWriter = null;
    void loadLines(language).then((data) => {
      if (data && !this.disposed && this.linesLanguage === language) this.lines = new Lines(data, this.content.voice[language]);
    });
    void loadTalk(language, this.content).then((text) => {
      if (!text || this.disposed || this.linesLanguage !== language) return;
      this.llmWriter = new LlmWriter(text, { content: this.content, world: this.sources.world }, () => this.lines);
    });
  }

  /** The player asked resident `a` to start a conversation with `b`. */
  playerConversation(a: number, b: number): void {
    this.playerTalk = { a, b, until: performance.now() + PLAYER_TALK_MS, started: false };
  }

  update(frame: FrameState): void {
    const now = frame.now;
    this.watchFrames(now);
    if (now - this.lastTick < TICK_MS) return;
    this.lastTick = now;
    if (!settings.voices || this.paused) {
      this.stopAll();
      return;
    }
    this.ensureLines();
    updateVoiceVolume();

    const { curr, layout } = frame;
    const k = layout.sim;
    const count = curr[layout.header.simCount];
    const row = (i: number) => layout.headerLen + i * layout.simStride;
    const idAt = (i: number) => curr[row(i) + k.id];

    this.rows.clear();
    for (let i = 0; i < count; i++) this.rows.set(idAt(i), i);
    for (const [id, a] of this.ahead) if (now - a.at > AHEAD_MS || !this.rows.has(id)) this.dropAhead(id);

    // Who may speak this tick.
    const voiced = new Set<number>();
    const inspected = game.inspected;
    // A newly picked resident's persona is read ahead by the language model.
    if (inspected !== this.primed && inspected !== null && llmWrites() && llmReady() && this.llmWriter) {
      const at = this.rows.get(inspected);
      if (at !== undefined) {
        this.primed = inspected;
        this.llmWriter.prime({ speaker: inspected, row: at, partner: -1, kind: 'emotion', emotion: 'happy', seed: 0 }, frame);
      }
    }
    const talk = this.playerTalk;
    for (let i = 0; i < count; i++) {
      const o = row(i);
      const id = curr[o + k.id];
      if (id === inspected || (talk && (id === talk.a || id === talk.b))) {
        voiced.add(id);
        const partner = curr[o + k.partner];
        if (curr[o + k.social] > 0 && partner >= 0 && partner < count) voiced.add(idAt(partner));
      }
    }
    if (talk) {
      const busy = [talk.a, talk.b].some((id) => {
        for (let i = 0; i < count; i++) if (idAt(i) === id) return curr[row(i) + k.social] > 0;
        return false;
      });
      if (busy) talk.started = true;
      else if (talk.started || now > talk.until) this.playerTalk = null;
    }

    // Nothing is said while the device test runs (it measures the voices and the model).
    const quiet = game.mode !== 'live' || document.hidden || !this.lines || deviceTest.running;
    const fast = game.speed >= 3;
    for (let i = 0; i < count; i++) {
      const o = row(i);
      const id = curr[o + k.id];
      const next: SimState = {
        social: curr[o + k.social],
        outcome: curr[o + k.outcome],
        action: curr[o + k.action],
        def: k.objectDef === undefined ? -1 : curr[o + k.objectDef],
        interaction: k.interaction === undefined ? -1 : curr[o + k.interaction],
        emotion: curr[o + k.emotion],
        thought: k.thought === undefined ? 0 : curr[o + k.thought],
        call: k.call === undefined ? 0 : curr[o + k.call],
        quietUntil: 0,
      };
      const prev = this.sims.get(id);
      next.quietUntil = prev?.quietUntil ?? 0;
      this.sims.set(id, next);
      if (!prev || quiet || !voiced.has(id)) continue;
      // At high speed only the conversation the player started is voiced.
      const playerSide = !!talk && (id === talk.a || id === talk.b || this.isPartnerOf(curr, row, k, i, talk));
      if (fast && !playerSide) continue;
      this.setOut(prev, next, frame, i, id, now);
      const e = this.event(prev, next, frame, i, id, now);
      if (e) this.write(e, frame, now);
    }

    this.pan(frame);
  }

  /** A resident's voice (babies don't speak). */
  private voiceOf(id: number): VoiceParams | null {
    const info = game.roster.find((s) => s.id === id);
    if (!info || info.stage === 'baby') return null;
    return voiceFor(id, info.gender, info.stage, info.appearance?.voice);
  }

  /** For the debug overlay. */
  stats(): VoiceDirectorStats {
    const language = !settings.voices ? 'off' : this.paused ? 'paused' : 'english';
    return {
      speaking: this.speaking.length,
      inFlight: this.inFlight,
      spoken: this.spoken,
      dropped: this.dropped,
      language,
      writing: this.writing.size,
      ahead: this.ahead.size,
      writer: llmWrites() ? 'llm' : 'lines',
    };
  }

  /** Counts slow frames while speech is being made, against those while it isn't. */
  private watchFrames(now: number): void {
    const dt = now - this.lastFrame;
    this.lastFrame = now;
    // Gaps from a hidden tab or a paused game say nothing about voices.
    if (dt <= 0 || dt > 1000 || document.hidden || this.paused) return;
    const f = this.frames;
    const slow = dt > SLOW_FRAME_MS ? 1 : 0;
    if ((this.inFlight > 0 && voiceStatus.state === 'ready') || this.writing.size > 0) {
      f.busy++;
      f.busySlow += slow;
    } else {
      f.idle++;
      f.idleSlow += slow;
    }
    if (f.busy < FRAME_WINDOW) return;
    const busyShare = f.busySlow / f.busy;
    const idleShare = f.idle ? f.idleSlow / f.idle : 0;
    // Only when speech makes it worse: a game that is slow anyway isn't the voices' fault.
    if (busyShare > SLOW_SHARE && busyShare > idleShare * 2) this.pause('frames');
    Object.assign(f, { busy: 0, busySlow: 0, idle: 0, idleSlow: 0 });
  }

  private pause(why: 'late' | 'frames'): void {
    if (this.paused) return;
    this.paused = true;
    unloadVoice();
    if (llmStatus.state !== 'off') unloadLlm();
    toast(
      why === 'late'
        ? 'Resident voices are paused for now: speech came too slowly on this computer.'
        : 'Resident voices are paused for now: making speech slowed the game down.',
      6000,
    );
  }

  dispose(): void {
    this.disposed = true;
    for (const w of this.writing.values()) w.abort();
    this.writing.clear();
    this.stopAll();
  }

  private isPartnerOf(
    curr: Float32Array,
    row: (i: number) => number,
    k: FrameState['layout']['sim'],
    i: number,
    talk: { a: number; b: number },
  ): boolean {
    const partner = curr[row(i) + k.partner];
    if (curr[row(i) + k.social] <= 0 || partner < 0) return false;
    const pid = curr[row(partner) + k.id];
    return pid === talk.a || pid === talk.b;
  }

  /**
   * A resident setting out to use something (the object and interaction show while they walk,
   * the animation only once there): the dice decide now whether they'll speak, and if so the
   * line is written and its speech made on the way, so it plays the moment they arrive
   * (`event`). A plan given up on the way is forgotten.
   */
  private setOut(prev: SimState, next: SimState, frame: FrameState, i: number, id: number, now: number): void {
    const going = next.action < 0 && next.def >= 0 && next.interaction >= 0;
    const key = going ? useKey(next) : '';
    const have = this.ahead.get(id);
    // Off somewhere else before arriving: that plan is forgotten.
    if (have && have.key !== key && next.action < 0) this.dropAhead(id);
    if (!going || this.ahead.get(id)?.key === key || (next.def === prev.def && next.interaction === prev.interaction)) return;
    // In their quiet time after a line: decided on arrival instead.
    if (now < next.quietUntil) return;
    const seed = id * 7919 + ++this.counter;
    if (!chance(seed + 1, ACTION_CHANCE)) {
      this.ahead.set(id, { key, speak: false, line: null, at: now });
      return;
    }
    const object = this.content.objectList[next.def];
    const interaction = object?.interactions[next.interaction]?.id;
    const req: LineRequest = { speaker: id, row: i, partner: -1, seed, kind: 'use', object: object?.id, interaction };
    const writer = llmWrites() ? this.llmWriter : null;
    let line: Promise<Line | null> | null = null;
    let control: AbortController | undefined;
    if (!writer) {
      const l = this.lines ? scripted(this.lines, req) : null;
      if (l) line = Promise.resolve(l);
    } else if (llmReady() && this.writing.size < MAX_WRITING) {
      this.writing.get(id)?.abort();
      control = new AbortController();
      const own = control;
      this.writing.set(id, own);
      line = writer
        .write(req, frame, own.signal)
        .catch(() => null)
        .then((l) => {
          if (this.writing.get(id) === own) this.writing.delete(id);
          return own.signal.aborted ? null : l;
        });
    }
    this.ahead.set(id, { key, speak: true, line, at: now, control });
    // The speech too, so arriving plays it from the cache.
    if (line) void line.then((l) => l && this.make(id, l));
  }

  /** Forgets the line made ahead for resident `id` (and stops writing it, if it still is). */
  private dropAhead(id: number): void {
    const a = this.ahead.get(id);
    if (!a) return;
    this.ahead.delete(id);
    if (a.control && this.writing.get(id) === a.control) {
      a.control.abort();
      this.writing.delete(id);
    }
  }

  /** What just changed for resident `id` (row `i`) that is worth a line, if anything. `line`: one made ahead. */
  private event(prev: SimState, next: SimState, frame: FrameState, i: number, id: number, now: number): { req: LineRequest; thought: boolean; line?: Promise<Line | null> } | null {
    const { curr, layout } = frame;
    const k = layout.sim;
    const o = layout.headerLen + i * layout.simStride;
    const seed = id * 7919 + ++this.counter;
    const partner = curr[o + k.partner];
    const partnerId = partner >= 0 ? curr[layout.headerLen + partner * layout.simStride + k.id] : -1;
    const role = curr[o + k.role];
    // Conversations: the one who starts says the opener, the other answers once the outcome shows (those listening in, role 3, say nothing).
    const base = { speaker: id, row: i, partner: partnerId, seed };
    if (next.social > 0) {
      const def = this.content.social(next.social - 1);
      if (!def) return null;
      if (role === 1 && next.social !== prev.social) return { req: { ...base, kind: 'social', social: def.id, phase: 'start' }, thought: false };
      if (role === 2 && prev.outcome === 0 && next.outcome > 0) {
        return { req: { ...base, kind: 'social', social: def.id, phase: next.outcome === 1 ? 'good' : 'bad' }, thought: false };
      }
      return null;
    }
    // Calling someone over: their name.
    if (next.call > 0 && next.call !== prev.call) {
      const name = game.roster.find((s) => s.id === next.call - 1)?.name.split(' ')[0];
      return name ? { req: { ...base, partner: next.call - 1, kind: 'call', name }, thought: false } : null;
    }
    const alone = { ...base, partner: -1 };
    if (now < next.quietUntil) return null;
    if (next.thought > 0 && next.thought !== prev.thought) {
      const subject = layout.sim.thoughtSubject === undefined ? 0 : curr[o + layout.sim.thoughtSubject];
      return { req: { ...alone, kind: 'thought', thought: next.thought, thoughtSubject: subject }, thought: true };
    }
    if (next.emotion > 0 && next.emotion !== prev.emotion && chance(seed, EMOTION_CHANCE)) {
      const emotion = this.content.emotions[next.emotion - 1];
      if (emotion) return { req: { ...alone, kind: 'emotion', emotion: emotion.id }, thought: true };
    }
    // Starting something, or going on to another thing with the same animation (a snack, then a meal).
    const started = next.action !== prev.action || next.def !== prev.def || next.interaction !== prev.interaction;
    if (next.action >= 0 && started) {
      // Arrived where they set out to: what was decided (and made) on the way holds.
      const ahead = this.ahead.get(id);
      const arrived = !!ahead && ahead.key === useKey(next);
      if (arrived) this.ahead.delete(id);
      if (arrived ? !ahead.speak : !chance(seed + 1, ACTION_CHANCE)) return null;
      const object = next.def >= 0 ? this.content.objectList[next.def] : undefined;
      const interaction = next.interaction >= 0 ? object?.interactions[next.interaction]?.id : undefined;
      const req: LineRequest = { ...alone, kind: 'use', object: object?.id, interaction, tag: layout.actions[next.action] };
      return { req, thought: true, line: arrived && ahead.line ? ahead.line : undefined };
    }
    return null;
  }

  /**
   * The line for an event: hand-written at once, or written by the language model (one at a
   * time per resident, a newer event replacing theirs); then said.
   */
  private write({ req, thought, line: ahead }: { req: LineRequest; thought: boolean; line?: Promise<Line | null> }, frame: FrameState, now: number): void {
    const id = req.speaker;
    const requested = performance.now();
    if (ahead) {
      // Made on the way over: said from the cache (or as soon as it's there).
      void ahead.then((line) => {
        if (line && !this.disposed) this.say(id, req.row, line, req.partner, thought, performance.now(), requested, MAX_WAIT_MS);
      });
      return;
    }
    // (A name called out is said as it is.)
    const writer = llmWrites() && req.kind !== 'call' ? this.llmWriter : null;
    if (!writer) {
      const line = this.lines ? scripted(this.lines, req) : null;
      if (line) this.say(id, req.row, line, req.partner, thought, now, requested, MAX_WAIT_MS);
      return;
    }
    // Nothing is written while the model loads: the moment passes.
    if (!llmReady()) return;
    this.writing.get(id)?.abort();
    this.writing.delete(id);
    if (this.writing.size >= MAX_WRITING) {
      this.dropped++;
      return;
    }
    const state = this.sims.get(id);
    if (thought && state) state.quietUntil = now + THOUGHT_GAP_MS;
    const control = new AbortController();
    this.writing.set(id, control);
    void writer
      .write(req, frame, control.signal)
      .catch(() => null)
      .then((line) => {
        if (this.writing.get(id) === control) this.writing.delete(id);
        if (control.signal.aborted || this.disposed) return;
        if (!line) {
          // Not written in time counts as late; a line that didn't pass doesn't.
          if (performance.now() - requested >= WRITE_MS) this.timed(Infinity, MAX_WAIT_LLM_MS);
          else this.dropped++;
          return;
        }
        this.say(id, req.row, line, req.partner, thought, performance.now(), requested, MAX_WAIT_LLM_MS);
      });
  }

  /**
   * Says a line: its speech from the cache or made, then played, a piece at a time
   * (`speechPieces`): a long line starts once its first piece is made, the next one being made
   * while one plays. `requested`: when the moment was; `budget`: how late its first piece may be.
   */
  private say(id: number, index: number, line: Line, partnerId: number, thought: boolean, now: number, requested: number, budget: number): void {
    const spoken = this.spokenAs(id, line, partnerId);
    if (!spoken) return;
    const { text, voice, pieces } = spoken;
    if (this.debug && this.heard.push(`${id} ${line.key}: ${text}`) > 50) this.heard.shift();
    // A line said before plays at once, even while the engine is unloaded.
    const cached = pieces.every((p) => hasClip(p, voice));
    if (!cached) {
      // Nothing new is said while the engine loads: the moment passes.
      if (!voiceReady()) return;
      if (this.inFlight >= MAX_IN_FLIGHT) {
        this.dropped++;
        return;
      }
    }
    const state = this.sims.get(id);
    if (thought && state) state.quietUntil = now + THOUGHT_GAP_MS;
    // Each piece is asked for once the one before is made, so another line's can go between.
    let given = false;
    const clips: Promise<Clip>[] = [];
    for (const piece of pieces) {
      const before = clips[clips.length - 1] ?? Promise.resolve();
      clips.push(before.then(() => (given ? Promise.reject(new Error('not said')) : speak(piece, voice))));
    }
    for (const c of clips) c.catch(() => {});
    if (!cached) {
      this.inFlight++;
      void Promise.allSettled(clips).then(() => this.inFlight--);
      void clips[0].then(() => this.timed(performance.now() - requested, budget)).catch(() => {});
    }
    let entry: Speaking | null = null;
    let ended = () => {};
    const play = async () => {
      const first = await clips[0];
      const late = performance.now() - requested;
      // An answer waits for the other side to finish (within reason).
      const before = this.speaking.filter((s) => s.id !== id);
      if (before.length && !thought) await Promise.race([Promise.all(before.map((s) => s.done)), wait(ANSWER_WAIT_MS)]);
      if (this.disposed || !settings.voices) return;
      if (late > budget + 3000 || this.speaking.length >= MAX_VOICES) {
        this.dropped++;
        return;
      }
      for (const [i, piece] of pieces.entries()) {
        const { samples, visemes } = i ? await clips[i] : first;
        // Stopped meanwhile (`stopAll`), or voices switched off.
        if (this.disposed || !settings.voices || (entry && !this.speaking.includes(entry))) return;
        // Where the resident is now (rows move as residents come and go).
        const row = entry?.row ?? this.rows.get(id) ?? index;
        const playing = playClip(samples, this.panFor(row));
        if (!playing) return;
        if (entry) {
          entry.playing = playing;
          entry.track = visemes;
        } else {
          this.spoken++;
          entry = { id, playing, row, track: visemes, done: new Promise((resolve) => (ended = resolve)) };
          this.speaking.push(entry);
        }
        this.renderer.setSpeech(row, visemes, playing.startTime);
        if (settings.subtitles) this.sources.subtitles?.say(id, piece, (samples.length / SAMPLE_RATE) * 1000 + SUBTITLE_LINGER_MS);
        await playing.ended;
      }
    };
    void play()
      .catch(() => {})
      .finally(() => {
        given = true;
        ended();
        if (!entry) return;
        const at = this.speaking.indexOf(entry);
        if (at >= 0) this.speaking.splice(at, 1);
        this.quiet(entry.row);
        if (thought && state) state.quietUntil = Math.max(state.quietUntil, performance.now() + THOUGHT_GAP_MS);
      });
  }

  /** A line as resident `id` says it: the names filled in, their voice with the line's tone, and the pieces it's spoken in. */
  private spokenAs(id: number, line: Line, partnerId: number): { text: string; voice: VoiceParams; pieces: string[] } | null {
    const own = this.voiceOf(id);
    if (!own) return null;
    const text = fill(line.text, this.firstName(id), partnerId >= 0 ? this.firstName(partnerId) : '');
    return { text, voice: withTone(own, line.tone), pieces: speechPieces(text) };
  }

  /** Makes a line's speech into the cache without playing it (for a resident on their way somewhere). */
  private make(id: number, line: Line): void {
    const spoken = this.spokenAs(id, line, -1);
    if (!spoken || !voiceReady() || this.inFlight >= MAX_IN_FLIGHT) return;
    const { voice, pieces } = spoken;
    if (pieces.every((p) => hasClip(p, voice))) return;
    this.inFlight++;
    void pieces
      .reduce((before, piece) => before.then(() => speak(piece, voice)).then(() => {}), Promise.resolve())
      .catch(() => {})
      .finally(() => this.inFlight--);
  }

  /** How long a line took to make (waiting included); too many late ones in a row pause voices. */
  private timed(ms: number, budget: number): void {
    this.lateRun = ms > budget ? this.lateRun + 1 : 0;
    if (this.lateRun >= LATE_RUN) this.pause('late');
  }

  private firstName(id: number): string {
    return (game.roster.find((s) => s.id === id)?.name ?? '').split(' ')[0];
  }

  /** Stops the mouth shapes on `row` unless another line is playing there. */
  private quiet(row: number): void {
    if (!this.speaking.some((s) => s.row === row)) this.renderer.setSpeech(row, null, 0);
  }

  /** Stereo position from where the resident is on screen. */
  private panFor(index: number): number {
    if (!this.renderer.simHead(index, this.head) || !this.renderer.project(this.head.x, this.head.y, this.head.z, this.screen)) return 0;
    const width = window.innerWidth || 1;
    const pan = Math.max(-1, Math.min(1, (this.screen.x / width) * 2 - 1)) * 0.7;
    // Web Audio throws on a non-finite value (and that would stop the frame).
    return Number.isFinite(pan) ? pan : 0;
  }

  private pan(frame: FrameState): void {
    if (!this.speaking.length) return;
    const { curr, layout } = frame;
    const count = curr[layout.header.simCount];
    for (const s of this.speaking) {
      for (let i = 0; i < count; i++) {
        if (curr[layout.headerLen + i * layout.simStride + layout.sim.id] === s.id) {
          s.playing.setPan(this.panFor(i));
          // Their row moved: the mouth shapes go with them.
          if (i !== s.row) {
            const old = s.row;
            s.row = i;
            this.quiet(old);
            this.renderer.setSpeech(i, s.track, s.playing.startTime);
          }
          break;
        }
      }
    }
  }

  private stopAll(): void {
    for (const w of this.writing.values()) w.abort();
    this.writing.clear();
    this.ahead.clear();
    const playing = this.speaking.splice(0);
    for (const s of playing) {
      s.playing.stop();
      this.renderer.setSpeech(s.row, null, 0);
    }
  }
}

/** What a resident is using or walking to use, as one key. */
function useKey(s: SimState): string {
  return `${s.def}:${s.interaction}`;
}

function chance(seed: number, p: number): boolean {
  let h = Math.imul(seed ^ 0x5bd1e995, 0x27d4eb2d);
  h ^= h >>> 15;
  return (h >>> 0) / 4294967296 < p;
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

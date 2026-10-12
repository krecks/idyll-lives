/**
 * Speech bubbles above Sims in a conversation, and thought bubbles (a planned block skipped or
 * kept, wanting somewhere to do it, a goal reached, a room loved or disliked, something broken,
 * an accident), as DOM elements over the canvas. With subtitles, a spoken line shows as text in
 * the bubble while it plays (`say`; a bubble of its own if there's no icon).
 * Per frame only `transform` is written (compositor-only); icons and text change only when a
 * conversation starts, its outcome is revealed or a line starts or ends. No allocations per frame.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import type { Renderer } from '../render/types';

const MAX_SIMS = 64;
/** Thought kinds (snapshot `thought`) to bubble tones. */
const THOUGHT_TONES = ['', 'bad', 'wish', 'good', 'love', 'good', 'bad', 'bad', 'bad', 'bad', 'good', 'wish'];

interface Bubble {
  /** Positioned element (transform written per frame). */
  el: HTMLDivElement;
  /** Visual body (animated independently of the position). */
  body: HTMLDivElement;
  icon: HTMLSpanElement;
  /** The spoken line's text. */
  text: HTMLSpanElement;
  /** Current icon key and text, to avoid redundant DOM writes. */
  key: string;
  shown: string;
  visible: boolean;
}

/** A line being said: its text, until when (ms, performance clock). */
interface Said {
  text: string;
  until: number;
}

export class BubbleLayer {
  private readonly root: HTMLDivElement;
  private readonly bubbles: Bubble[] = [];
  private readonly head = { x: 0, y: 0, z: 0 };
  private readonly screen = { x: 0, y: 0 };
  /** Lines being said, by resident id. */
  private readonly said = new Map<number, Said>();

  constructor(
    container: HTMLElement,
    private readonly renderer: Renderer,
    private readonly content: Content,
    private readonly assets: AssetRegistry,
  ) {
    this.root = document.createElement('div');
    this.root.className = 'bubble-layer';
    container.appendChild(this.root);
  }

  /** Shows `text` in resident `id`'s bubble for `ms`. */
  say(id: number, text: string, ms: number): void {
    this.said.set(id, { text, until: performance.now() + ms });
  }

  update(frame: FrameState): void {
    const { curr, layout } = frame;
    const k = layout.sim;
    const count = Math.min(curr[layout.header.simCount], MAX_SIMS);
    const now = performance.now();
    for (const [id, s] of this.said) if (s.until < now) this.said.delete(id);
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      const social = curr[o + k.social];
      const thought = social > 0 || k.thought === undefined ? 0 : curr[o + k.thought];
      const text = this.said.size ? (this.said.get(curr[o + k.id])?.text ?? '') : '';
      const key =
        social > 0
          ? this.iconFor(social - 1, curr[o + k.role], curr[o + k.outcome])
          : thought > 0
            ? this.thoughtIcon(thought, curr[o + (k.thoughtSubject ?? 0)])
            : '';
      const bubble = key || text ? this.bubble(i) : this.bubbles[i];
      if (!bubble) continue;
      if ((!key && !text) || !this.renderer.simHead(i, this.head) || !this.renderer.project(this.head.x, this.head.y, this.head.z, this.screen)) {
        this.hide(bubble);
        continue;
      }
      if (bubble.shown !== text) {
        bubble.shown = text;
        bubble.text.textContent = text;
        bubble.body.classList.toggle('has-text', !!text);
        bubble.icon.style.display = key ? '' : 'none';
        if (!key) bubble.body.dataset.tone = 'topic';
      }
      if (bubble.key !== key) {
        bubble.key = key;
        bubble.icon.style.setProperty('--icon', `url('${this.assets.icon(key).url}')`);
        bubble.body.dataset.tone = thought > 0 ? THOUGHT_TONES[thought] : key.startsWith('icon.bubble.') ? key.slice(12) : 'topic';
        bubble.body.classList.toggle('thought', thought > 0);
        bubble.body.classList.remove('pop');
        void bubble.body.offsetWidth; // restart the pop animation
        bubble.body.classList.add('pop');
      }
      if (!bubble.visible) {
        bubble.visible = true;
        bubble.el.style.display = '';
      }
      bubble.el.style.transform = `translate3d(${this.screen.x}px, ${this.screen.y}px, 0)`;
    }
    for (let i = count; i < this.bubbles.length; i++) if (this.bubbles[i]) this.hide(this.bubbles[i]);
  }

  dispose(): void {
    this.root.remove();
  }

  /**
   * A thought's icon: the activity (skipped, wanted, kept), the goal reached, the room (loved,
   * disliked), the broken thing, the need behind the accident or the thing they'd like to buy.
   */
  private thoughtIcon(kind: number, subject: number): string {
    switch (kind) {
      case 4:
        return this.content.goals[subject]?.icon || 'icon.bubble.good';
      case 5:
      case 6:
        return roomIcon(this.content, subject);
      case 7:
        return this.content.objectList[subject]?.icon || 'icon.bubble.bad';
      case 11:
        // Something they'd like to buy.
        return this.content.objectList[subject]?.icon || 'icon.ui.funds';
      case 8: {
        const need = this.content.accidents[subject]?.need;
        return this.content.needs.find((n) => n.id === need)?.icon || 'icon.bubble.bad';
      }
      case 9:
        // A baby crying for what they need most.
        return this.content.needs[subject]?.icon || 'icon.bubble.bad';
      default:
        return this.content.activities[subject]?.icon ?? 'icon.bubble.good';
    }
  }

  /** The initiator shows the topic; the other Sim (and anyone listening in, role 3) shows their reaction once it's revealed. */
  private iconFor(social: number, role: number, outcome: number): string {
    const def = this.content.social(social);
    if (!def) return '';
    if (role === 1) return def.icon;
    if (outcome === 0) return '';
    if (outcome === 2) return def.category === 'mean' ? 'icon.bubble.angry' : 'icon.bubble.bad';
    if (def.category === 'romantic') return 'icon.bubble.love';
    if (def.category === 'mean') return 'icon.bubble.angry';
    return 'icon.bubble.good';
  }

  private bubble(i: number): Bubble {
    let b = this.bubbles[i];
    if (!b) {
      const el = document.createElement('div');
      el.className = 'speech-bubble';
      el.style.display = 'none';
      const body = document.createElement('div');
      body.className = 'bubble-body';
      const icon = document.createElement('span');
      icon.className = 'bubble-icon';
      const text = document.createElement('span');
      text.className = 'bubble-text';
      body.append(icon, text);
      el.appendChild(body);
      this.root.appendChild(el);
      b = this.bubbles[i] = { el, body, icon, text, key: '', shown: '', visible: false };
    }
    return b;
  }

  private hide(b: Bubble): void {
    if (!b.visible) return;
    b.visible = false;
    b.key = '';
    b.shown = '';
    b.text.textContent = '';
    b.body.classList.remove('has-text');
    b.icon.style.display = '';
    b.el.style.display = 'none';
  }
}

/** The icon of a room thought's subject (sim-core `rooms::room_subject`: kind × 8 + factor). */
export function roomIcon(content: Content, subject: number): string {
  const kind = Math.floor(subject / 8);
  if (kind === 0) return 'icon.category.garden';
  return content.roomKinds[kind - 1]?.icon || 'icon.ui.room';
}

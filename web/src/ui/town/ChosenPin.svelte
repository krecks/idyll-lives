<script lang="ts">
  import Icon from '../Icon.svelte';
  import { townView } from '../../render/shared/townView';
  import { settings } from '../../settings/settings.svelte';
  import { townMap } from '../townMap.svelte';
  import { chosenView } from './chosenView.svelte';
  import { townHud } from './townHud.svelte';

  /**
   * The pin over the building the town card is about (clicked in the play view): its icon, its name and how many are
   * inside. In the town view a place's own tag stands there instead. Placed each frame through `townView.project`,
   * written straight to the element; kept on the screen (without its stem) when its point is off it, as close up over
   * a tall house its roof's top is.
   */
  /** Margin (px) kept from the screen's edge; the pill's height and its stem (px) over its point. */
  const EDGE = 8;
  const TALL = 44;
  /** The pin's point, as a share of its lift over the ground, tried in turn until one is on the screen. */
  const LIFTS = [1, 0.5, 0];
  const chosen = $derived(chosenView.chosen);
  const shown = $derived(!!chosen && !!chosenView.anchor && !(townHud.on && chosen.kind === 'place'));
  const count = $derived(chosenView.inside ? chosenView.inside.names.length + chosenView.inside.others : 0);
  let pin = $state<HTMLDivElement>();

  $effect(() => {
    const el = pin;
    if (!shown || !el) return;
    let raf = 0;
    let at = '';
    const loop = () => {
      raf = requestAnimationFrame(loop);
      const a = chosenView.anchor;
      const ground = townMap.source.ground;
      // (Over its roof; off the screen there, as close up over a tall house, lower down its walls.)
      let p: [number, number, number] | null = null;
      if (a && townView.project) {
        const y = ground?.(a.x, a.z) ?? 0;
        for (const f of LIFTS) if ((p = townView.project(a.x, y + a.lift * f, a.z))) break;
      }
      let next = '';
      let edge = false;
      if (p) {
        const scale = settings.uiScale || 1;
        const half = ((el.firstElementChild as HTMLElement | null)?.offsetWidth ?? 0) / 2;
        const x = Math.min(Math.max(p[0] / scale, EDGE + half), innerWidth / scale - EDGE - half);
        const y = Math.min(Math.max(p[1] / scale, EDGE + TALL), innerHeight / scale - EDGE);
        edge = Math.abs(x - p[0] / scale) > 1 || Math.abs(y - p[1] / scale) > 1;
        next = `translate(${x.toFixed(1)}px, ${y.toFixed(1)}px)`;
      }
      if (next === at) return;
      at = next;
      el.style.transform = next;
      el.style.visibility = next ? 'visible' : 'hidden';
      el.classList.toggle('edge', edge);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  });
</script>

{#if shown && chosen}
  <div class="pin" bind:this={pin} aria-hidden="true">
    <div class="tag">
      <span class="kind"><Icon name={chosenView.icon} size={13} /></span>
      <b>{chosen.title}</b>
      {#if count > 0}<span class="count"><Icon name="icon.need.social" size={11} />{count}</span>{/if}
    </div>
  </div>
{/if}

<style>
  /* (Hidden until a frame places it.) */
  .pin {
    position: absolute;
    left: 0;
    top: 0;
    visibility: hidden;
    will-change: transform;
    pointer-events: none;
  }
  /* The town tags' pill on a stem (`TownTags.svelte`), with the building's icon. */
  .tag {
    position: absolute;
    left: 0;
    bottom: 12px;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px 3px 3px;
    border-radius: var(--radius-pill);
    background: var(--tag-bg);
    color: var(--text-strong);
    box-shadow: var(--tag-shadow);
    white-space: nowrap;
    font-size: 12px;
    line-height: 1.2;
  }
  .tag::after {
    content: '';
    position: absolute;
    left: 50%;
    bottom: -11px;
    width: 2px;
    height: 11px;
    margin-left: -1px;
    border-radius: 1px;
    background: var(--tag-bg);
  }
  /* (Moved onto the screen from its point: no stem pointing at something it doesn't stand over.) */
  .pin:global(.edge) .tag::after {
    display: none;
  }
  .kind {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
  }
  b {
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .count {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
  }
</style>

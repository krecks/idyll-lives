<script lang="ts">
  import type { ObjectDef } from '../../content/content';
  import Icon from '../Icon.svelte';
  import { money } from '../format';
  import { services } from '../services';
  import { game } from '../state.svelte';
  import { collectionOf, paintable, percent, summarize } from './catalog';

  /**
   * What a catalog item is worth to the household, at a glance: floats above the card pointed at.
   * (The whole story, with the turntable, upgrades and things to do, is `ItemDetail`'s, for things owned.)
   */
  let { def, at, lovers = [], owned = 0 }: { def: ObjectDef; /** Horizontal position over the dock, 0..1. */ at: number; lovers?: string[]; owned?: number } = $props();

  const content = services.content;
  const info = $derived(summarize(content, def));
  const price = $derived(def.price ?? 0);
  const short = $derived(game.creative ? 0 : Math.max(0, price - game.funds));
  const category = $derived.by(() => {
    const c = content.buyCategories.find((c) => c.id === def.category);
    const group = c?.groups?.find((g) => g.id === def.group)?.label;
    return c && group ? `${c.label} · ${group}` : c?.label;
  });
  const collection = $derived(collectionOf(content, def));
  const skillLabel = (id: string) => content.skill(id)?.label ?? id;
  const skillIcon = (id: string) => content.skill(id)?.icon ?? '';
  const billsRate = content.economy.rent?.billsRate ?? 0;
  const addedBills = $derived(game.creative ? 0 : Math.round(price * billsRate));
  const facts = $derived(
    [
      info.slots > 1 ? `Fits ${info.slots} at once` : '',
      info.useCost ? `+${money(info.useCost.min)} per use` : '',
      paintable(services.assets, def) ? 'Any colour' : '',
      owned > 0 ? `You have ${owned}` : '',
      addedBills > 0 ? `~${money(addedBills)}/wk bills` : '',
    ].filter(Boolean),
  );
</script>

<div class="card glass" style="--at:{at}" role="tooltip">
  <div class="head">
    <div>
      <b>{def.name}</b>
      {#if category}<span class="eyebrow">{category}</span>{/if}
    </div>
    <span class="price tabular" class:short={short > 0}>{game.creative ? 'Free' : money(price)}</span>
  </div>
  {#if short > 0}<p class="warn">{money(short)} short</p>{/if}
  {#if def.description}<p class="desc">{def.description}</p>{/if}

  {#if info.boosts.length || info.drains.length}
    <ul class="needs">
      {#each info.boosts.slice(0, 4) as b (b.need.id)}
        <li>
          <Icon name={b.need.icon} size={13} />
          <span class="label">{b.need.label}</span>
          <span class="bar"><span style="width:{Math.min(1, b.perUse) * 100}%"></span></span>
          <span class="num tabular">{percent(b.perUse)}</span>
        </li>
      {/each}
      {#each info.drains.slice(0, 2) as b (b.need.id)}
        <li class="drain">
          <Icon name={b.need.icon} size={13} />
          <span class="label">{b.need.label}</span>
          <span class="bar"><span style="width:{Math.min(1, -b.perUse) * 100}%"></span></span>
          <span class="num tabular">{percent(b.perUse)}</span>
        </li>
      {/each}
    </ul>
  {/if}

  {#if info.trains.length || info.betterWith.length || info.feelings.length || lovers.length || collection}
    <div class="chips">
      {#if lovers.length}<span class="chip love">♥ {lovers.join(' & ')}</span>{/if}
      {#each info.trains as t (t.skill)}
        <span class="chip skill"><Icon name={skillIcon(t.skill)} size={11} />Trains {skillLabel(t.skill)}</span>
      {/each}
      {#each info.betterWith as s (s)}
        <span class="chip better"><Icon name={skillIcon(s)} size={11} />Better with {skillLabel(s)}</span>
      {/each}
      {#each info.feelings.slice(0, 3) as f (f.id)}
        <span class="chip feel">{#if f.icon}<Icon name={f.icon} size={11} />{:else}✦{/if}{f.label}</span>
      {/each}
      {#if collection}<span class="chip"><Icon name={collection.icon} size={11} />{collection.label}</span>{/if}
    </div>
  {/if}

  {#if facts.length}<p class="facts">{facts.join(' · ')}</p>{/if}
</div>

<style>
  .card {
    position: absolute;
    bottom: 0;
    left: clamp(150px, calc(var(--at) * 100%), calc(100% - 150px));
    transform: translateX(-50%);
    width: 290px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    background: var(--glass-strong);
    pointer-events: none;
    animation: appear 160ms var(--ease) both;
    z-index: 2;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
  }
  .head > div {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  b {
    font-size: 13px;
  }
  .price {
    color: var(--good);
    font-weight: 700;
    font-size: 13px;
    white-space: nowrap;
  }
  .short,
  .warn {
    color: var(--bad);
  }
  .warn {
    font-weight: 600;
  }
  p {
    margin: 0;
  }
  .desc {
    color: var(--text-muted);
    line-height: 1.35;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .needs li {
    display: grid;
    grid-template-columns: 13px 64px 1fr auto;
    align-items: center;
    gap: 6px;
  }
  .bar {
    height: 5px;
    border-radius: 3px;
    background: var(--hairline);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    border-radius: 3px;
    background: var(--good);
  }
  .drain {
    color: var(--text-muted);
  }
  .drain .bar span {
    background: var(--bad);
  }
  .num {
    font-weight: 650;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 7px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    font-weight: 600;
    font-size: 11px;
    color: var(--text-muted);
  }
  .chip.skill {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .chip.better {
    background: rgba(240, 181, 74, 0.2);
    color: #9a6a10;
  }
  .chip.feel {
    background: rgba(193, 123, 224, 0.16);
    color: #8a44ad;
  }
  .chip.love {
    background: rgba(224, 96, 126, 0.14);
    color: #c2405f;
  }
  .facts {
    color: var(--text-muted);
    font-size: 11px;
  }
  @keyframes appear {
    from {
      opacity: 0;
      translate: 0 6px;
    }
  }
</style>

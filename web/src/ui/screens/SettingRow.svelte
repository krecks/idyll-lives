<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from '../Icon.svelte';

  /**
   * One setting: its name, a short hint (one line), badges ("Next load", "Experimental") and the
   * control. The longer story, when there is one, is `details`, behind an ⓘ.
   */
  let {
    title,
    hint,
    details,
    badges = [],
    disabled = false,
    children,
  }: { title: string; hint?: string; details?: string; badges?: string[]; disabled?: boolean; children: Snippet } = $props();
</script>

<div class="row" class:disabled>
  <div class="text">
    <b>
      {title}
      {#each badges as b (b)}<em class:experimental={b === 'Experimental'}>{b}</em>{/each}
      {#if details}<span class="more" role="img" aria-label={details} title={details}><Icon name="icon.ui.info" size={14} /></span>{/if}
    </b>
    {#if hint}<span>{hint}</span>{/if}
  </div>
  <div class="control">{@render children()}</div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 12px 0;
  }
  .row + :global(.row) {
    border-top: 1px solid var(--hairline);
  }
  .disabled .text {
    opacity: 0.5;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  b {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
  }
  .more {
    display: inline-grid;
    color: var(--text-muted);
    cursor: help;
  }
  .more:hover {
    color: var(--accent);
  }
  .text > span {
    color: var(--text-muted);
    font-size: 12.5px;
    line-height: 1.4;
  }
  em {
    font-style: normal;
    font-size: 10.5px;
    font-weight: 650;
    letter-spacing: 0.02em;
    color: var(--text-muted);
    background: var(--hairline);
    padding: 1px 7px;
    border-radius: var(--radius-pill);
  }
  em.experimental {
    color: #9a6a10;
    background: rgba(240, 181, 74, 0.2);
  }
  .control {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
  }
</style>

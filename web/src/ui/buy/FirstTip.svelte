<script lang="ts">
  import type { Snippet } from 'svelte';
  import { settings } from '../../settings/settings.svelte';
  import Icon from '../Icon.svelte';

  /** A tip shown the first time a panel opens, until it's dismissed (Settings › Interface brings them back). */
  let { id, children }: { id: string; children: Snippet } = $props();

  const seen = $derived(settings.tipsSeen.includes(id));
  function dismiss() {
    settings.tipsSeen = [...settings.tipsSeen, id];
  }
</script>

{#if !seen}
  <aside class="tip glass" aria-label="Tip">
    <span class="icon"><Icon name="icon.ui.info" size={18} /></span>
    <p>{@render children()}</p>
    <button class="btn small" onclick={dismiss}>Got it</button>
  </aside>
{/if}

<style>
  .tip {
    position: absolute;
    left: 0;
    bottom: 0;
    width: min(380px, 100%);
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 10px;
    padding: 10px 12px;
    font-size: 12px;
    line-height: 1.45;
    background: var(--glass-strong);
    pointer-events: auto;
    animation: rise var(--slow) var(--ease) both;
  }
  .icon {
    color: var(--accent);
    padding-top: 1px;
  }
  p {
    margin: 0;
  }
  .btn {
    grid-column: 2;
    justify-self: end;
    height: 26px;
    padding: 0 12px;
    font-size: 12px;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
</style>

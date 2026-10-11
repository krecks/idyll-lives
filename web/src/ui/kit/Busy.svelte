<script lang="ts">
  /**
   * Something is being worked on: a spinner, what it is, and a bar when how far it got is known
   * (`progress` 0..1; none: the bar sweeps).
   */
  let { label = '', progress = null }: { label?: string; progress?: number | null } = $props();
  const percent = $derived(progress === null ? null : Math.round(Math.max(0, Math.min(1, progress)) * 100));
</script>

<div class="busy" role="status" aria-live="polite">
  <div class="line">
    <span class="spinner" aria-hidden="true"></span>
    {#if label}<span class="label">{label}</span>{/if}
    {#if percent !== null}<span class="percent">{percent} %</span>{/if}
  </div>
  <div class="bar" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent ?? undefined} aria-label={label || 'Working'}>
    {#if percent === null}
      <div class="sweep"></div>
    {:else}
      <div class="fill" style:transform="scaleX({percent / 100})"></div>
    {/if}
  </div>
</div>

<style>
  .busy {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 220px;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .percent {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--text);
  }
  .bar {
    position: relative;
    height: 4px;
    border-radius: var(--radius-pill);
    background: var(--hairline);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
    transform-origin: left;
    transition: transform 240ms var(--ease);
  }
  .sweep {
    position: absolute;
    inset: 0 auto 0 0;
    width: 35%;
    border-radius: inherit;
    background: var(--accent);
    animation: sweep 1.1s ease-in-out infinite;
  }
  @keyframes sweep {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(290%);
    }
  }
</style>

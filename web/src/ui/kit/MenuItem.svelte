<script lang="ts">
  import Icon from '../Icon.svelte';

  /**
   * One entry of a menu over the scene (the title screen, the pause menu): dark glass, light text.
   * `primary` is the one to press (white), `quiet` a smaller entry without an icon, `danger` red.
   */
  let {
    label,
    icon,
    variant = 'default',
    arrow = false,
    badge,
    disabled = false,
    type = 'button',
    onclick,
  }: {
    label: string;
    icon?: string;
    variant?: 'default' | 'primary' | 'quiet' | 'danger';
    /** A → at the end (the way forward). */
    arrow?: boolean;
    badge?: string | number;
    disabled?: boolean;
    type?: 'button' | 'submit';
    onclick?: () => void;
  } = $props();
</script>

<button class="item {variant}" {type} {disabled} {onclick}>
  {#if icon && variant !== 'quiet'}<Icon name={icon} size={18} />{/if}
  <span class="label">{label}</span>
  {#if badge !== undefined}<span class="badge">{badge}</span>{/if}
  {#if arrow}<span class="arrow" aria-hidden="true">→</span>{/if}
</button>

<style>
  .item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 14px;
    height: 54px;
    padding: 0 18px;
    border-radius: 16px;
    font-size: 17px;
    font-weight: 600;
    text-align: left;
    color: inherit;
    transition:
      background var(--fast) ease,
      transform var(--fast) var(--ease),
      box-shadow var(--fast) var(--ease);
    animation: enter 600ms var(--ease) both;
  }
  .item:nth-child(2) {
    animation-delay: 50ms;
  }
  .item:nth-child(3) {
    animation-delay: 100ms;
  }
  .item:nth-child(4) {
    animation-delay: 150ms;
  }
  .item:nth-child(5) {
    animation-delay: 200ms;
  }
  .item:hover:not(:disabled),
  .item:focus-visible {
    background: rgba(255, 255, 255, 0.12);
    transform: translateX(4px);
  }
  .item:not(.primary):hover:not(:disabled)::before,
  .item:not(.primary):focus-visible::before {
    content: '';
    position: absolute;
    left: 0;
    top: 14px;
    bottom: 14px;
    width: 3px;
    border-radius: 2px;
    background: var(--menu-mark, #8fe07a);
  }
  .item:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .primary {
    height: 60px;
    margin-bottom: 6px;
    background: linear-gradient(180deg, #ffffff, #eef1f6);
    color: var(--text-strong);
    box-shadow: 0 12px 34px rgba(0, 0, 0, 0.3);
  }
  .primary:hover:not(:disabled),
  .primary:focus-visible {
    background: linear-gradient(180deg, #ffffff, #eef1f6);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.36);
  }
  .quiet {
    height: 46px;
    font-size: 15px;
    font-weight: 550;
    color: rgba(246, 242, 234, 0.74);
  }
  .danger {
    color: #ff9d8f;
    --menu-mark: #ff9d8f;
  }
  .arrow {
    margin-left: auto;
    font-size: 20px;
    transition: transform var(--fast) var(--ease);
  }
  .item:hover .arrow {
    transform: translateX(3px);
  }
  .badge {
    margin-left: auto;
    min-width: 24px;
    height: 22px;
    padding: 0 7px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-pill);
    background: rgba(255, 255, 255, 0.16);
    font-size: 12px;
    font-weight: 700;
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(14px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .item {
      animation: none;
    }
  }
</style>

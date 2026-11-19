/**
 * The money each change of the household's funds brings or takes, floating off for a moment: off
 * the funds in Live (`TopBar.svelte`), from the pointer in Build (`buy/BuyFx.svelte`).
 */

import { untrack } from 'svelte';
import { money } from './format';

export class MoneyFloats<At extends object = object> {
  /** The changes showing, oldest first; `at`: where each floats from. */
  shown = $state<(At & { id: number; amount: number })[]>([]);
  private next = 0;
  private funds: number | null = null;

  /** `ms`: how long each shows; `most`: how many show at once. */
  constructor(
    private readonly ms: number,
    private readonly most = Infinity,
  ) {}

  /**
   * The funds now: the change since the last ones floats off, if `show` (`at` says where from). Returns the
   * change that floated off (0: none).
   */
  note(funds: number, show: boolean, at: () => At = () => ({}) as At): number {
    // (Untracked: an effect calling this reruns only when the funds, or `show`, change.)
    return untrack(() => {
      const was = this.funds;
      this.funds = funds;
      if (!show || was === null || Math.round(funds - was) === 0) return 0;
      const id = this.next++;
      const amount = funds - was;
      this.shown = [...this.shown, { ...at(), id, amount }].slice(-this.most);
      setTimeout(() => (this.shown = this.shown.filter((f) => f.id !== id)), this.ms);
      return amount;
    });
  }
}

/** A change of money: "+$120", "−$40". */
export function moneyChange(amount: number): string {
  return `${amount > 0 ? '+' : '−'}${money(Math.abs(amount))}`;
}

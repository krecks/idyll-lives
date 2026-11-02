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

  /** The funds now: the change since the last ones floats off, if `show` (`at` says where from). */
  note(funds: number, show: boolean, at: () => At = () => ({}) as At): void {
    // (Untracked: an effect calling this reruns only when the funds, or `show`, change.)
    untrack(() => {
      const was = this.funds;
      this.funds = funds;
      if (!show || was === null || Math.round(funds - was) === 0) return;
      const id = this.next++;
      this.shown = [...this.shown, { ...at(), id, amount: funds - was }].slice(-this.most);
      setTimeout(() => (this.shown = this.shown.filter((f) => f.id !== id)), this.ms);
    });
  }
}

/** A change of money: "+$120", "−$40". */
export function moneyChange(amount: number): string {
  return `${amount > 0 ? '+' : '−'}${money(Math.abs(amount))}`;
}

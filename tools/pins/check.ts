/**
 * Pins: accepted parts of the game held by their sha256, so work on something else can't change them
 * by the way (docs/design/living-town.md, Pins). Each pin in `tools/pins/pins.json` names a file and,
 * optionally, a part of it:
 *
 * - `region`: the lines between `pin:<region> begin` and `pin:<region> end` comments (exclusive);
 * - `const`: an `export const <NAME>` statement, from its line to where its brackets close.
 *
 *   bun tools/pins/check.ts                       check every pin (exit 1 on a difference)
 *   bun tools/pins/check.ts --list                every pin and whether it holds
 *   bun tools/pins/check.ts --update <name> --by <who> --reason <why>
 *                                                 take a pin's present content as accepted
 *   bun tools/pins/check.ts --add <name> --file <path> [--region <r> | --const <NAME>] --by <who> --reason <why>
 *
 * Only the integrator updates or adds pins; every update is listed in the run's end report.
 */
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';

export interface Pin {
  name: string;
  file: string;
  region?: string;
  const?: string;
  sha256: string;
  by: string;
  reason: string;
}

const ROOT = resolve(dirname(new URL(import.meta.url).pathname), '../..');
const PINS = join(ROOT, 'tools/pins/pins.json');

/** The pinned text of a file's content, or an error message. */
export function pinnedText(content: string, pin: Pick<Pin, 'region' | 'const'>): string | { error: string } {
  if (pin.region) {
    const lines = content.split('\n');
    const begin = lines.findIndex((l) => l.includes(`pin:${pin.region} begin`));
    const end = lines.findIndex((l) => l.includes(`pin:${pin.region} end`));
    if (begin < 0 || end < 0 || end < begin) return { error: `region ${pin.region}'s begin and end markers are missing` };
    return lines.slice(begin + 1, end).join('\n');
  }
  if (pin.const) {
    const start = content.search(new RegExp(`^export const ${pin.const}\\b`, 'm'));
    if (start < 0) return { error: `no \`export const ${pin.const}\`` };
    let depth = 0;
    let opened = false;
    for (let i = start; i < content.length; i++) {
      const c = content[i];
      if (c === '(' || c === '{' || c === '[') {
        depth++;
        opened = true;
      } else if (c === ')' || c === '}' || c === ']') depth--;
      else if (c === ';' && depth === 0) return content.slice(start, i + 1);
      if (opened && depth === 0 && content[i + 1] === '\n' && c !== ',') {
        // A statement without its semicolon ends where its brackets close at a line's end.
        const rest = content.slice(i + 1).match(/^\n\s*([.;?)]|as\b)/);
        if (!rest) return content.slice(start, i + 1);
      }
    }
    return { error: `\`export const ${pin.const}\` never closes` };
  }
  return content;
}

const sha = (text: string) => createHash('sha256').update(text).digest('hex');

function hashOf(pin: Pick<Pin, 'file' | 'region' | 'const'>): string | { error: string } {
  let content: string;
  try {
    content = readFileSync(join(ROOT, pin.file), 'utf8');
  } catch {
    return { error: `${pin.file} is missing` };
  }
  const text = pinnedText(content, pin);
  return typeof text === 'string' ? sha(text) : text;
}

const load = (): Pin[] => JSON.parse(readFileSync(PINS, 'utf8')).pins;
const save = (pins: Pin[]) => writeFileSync(PINS, `${JSON.stringify({ pins }, null, 2)}\n`);

function arg(name: string): string | undefined {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 ? process.argv[i + 1] : undefined;
}

function need(name: string): string {
  const v = arg(name);
  if (!v) {
    console.error(`--${name} is needed`);
    process.exit(2);
  }
  return v;
}

function main(): void {
  const pins = load();
  const update = arg('update');
  const add = arg('add');
  if (update || add) {
    const by = need('by');
    const reason = need('reason');
    if (add) {
      if (pins.some((p) => p.name === add)) throw new Error(`pin ${add} exists: use --update`);
      const pin: Pin = { name: add, file: need('file'), sha256: '', by, reason };
      const region = arg('region');
      const cnst = arg('const');
      if (region) pin.region = region;
      if (cnst) pin.const = cnst;
      pins.push(pin);
    }
    const pin = pins.find((p) => p.name === (update ?? add));
    if (!pin) throw new Error(`no pin ${update}`);
    const h = hashOf(pin);
    if (typeof h !== 'string') throw new Error(`${pin.name}: ${h.error}`);
    Object.assign(pin, { sha256: h, by, reason });
    save(pins);
    console.log(`${add ? 'Pinned' : 'Updated'} ${pin.name}: ${h.slice(0, 12)}`);
    return;
  }
  const list = process.argv.includes('--list');
  let broken = 0;
  for (const pin of pins) {
    const h = hashOf(pin);
    const ok = h === pin.sha256;
    if (!ok) broken++;
    const what = `${pin.file}${pin.region ? ` (region ${pin.region})` : pin.const ? ` (${pin.const})` : ''}`;
    if (list || !ok) {
      const why = ok ? 'holds' : typeof h === 'string' ? 'CHANGED' : `BROKEN: ${h.error}`;
      console.log(`${ok ? '  ' : '✗ '}${pin.name}: ${what}: ${why}`);
    }
  }
  if (broken) {
    console.error(
      `\n${broken} pinned part(s) changed. Pinned parts are accepted work: put them back, or have the integrator ` +
        'take the change with `bun tools/pins/check.ts --update <name> --by <who> --reason <why>`.',
    );
    process.exit(1);
  }
  console.log(`${pins.length} pins hold.`);
}

if (import.meta.main) main();

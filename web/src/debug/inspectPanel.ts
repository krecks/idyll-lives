/**
 * The model inspector's side panel (`inspect.ts`, /inspect.html): what it says of the model shown, as
 * HTML, and the gate's report coloured. Not part of the build.
 */

import type { ModelEntry } from '../assets/types';
import type { Content, ObjectDef } from '../content/content';
import { withGlows, type ModelTemplate } from '../render/pc/models';
import type { Shown } from './clipBench';

/** What a template draws: triangles, vertices and draw calls (its glows' too). */
export interface Counts {
  triangles: number;
  vertices: number;
  draws: number;
}

export function counts(t: ModelTemplate): Counts {
  let triangles = 0;
  let vertices = 0;
  for (const { mesh } of t.meshes) {
    triangles += Math.floor((mesh.primitive[0]?.count ?? 0) / 3);
    vertices += mesh.vertexBuffer?.numVertices ?? 0;
  }
  return { triangles, vertices, draws: withGlows(t).length };
}

/** A template's box at rest (m, model space: x right, y up, z its front), from its meshes' boxes. */
export interface Box {
  lo: [number, number, number];
  hi: [number, number, number];
}

export function restBox(t: ModelTemplate): Box | null {
  if (!t.meshes.length) return null;
  const lo: [number, number, number] = [Infinity, Infinity, Infinity];
  const hi: [number, number, number] = [-Infinity, -Infinity, -Infinity];
  for (const { mesh } of t.meshes) {
    const a = mesh.aabb.getMin();
    const b = mesh.aabb.getMax();
    [a.x, a.y, a.z].forEach((v, k) => (lo[k] = Math.min(lo[k], v)));
    [b.x, b.y, b.z].forEach((v, k) => (hi[k] = Math.max(hi[k], v)));
  }
  return { lo, hi };
}

/** Text made safe for HTML. */
export function esc(s: unknown): string {
  return String(s).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}

const m = (v: number) => `${v.toFixed(3)} m`;
const n = (v: number) => v.toLocaleString('en');
const rows = (pairs: [string, string][]) => `<table>${pairs.map(([k, v]) => `<tr><td>${esc(k)}</td><td>${v}</td></tr>`).join('')}</table>`;
const section = (title: string, body: string, open = true) => `<details${open ? ' open' : ''}><summary>${esc(title)}</summary>${body}</details>`;

/** The family a model's file is in (`assets/models/<family>/…`), or what stands for one. */
export function familyOf(entry: ModelEntry | undefined): string {
  if (!entry?.url) return '(no file)';
  return /\/models\/([^/]+)\//.exec(entry.url)?.[1] ?? '(elsewhere)';
}

/** Where a model's file is, as the page fetches it (no origin, no query). */
export function filePath(url: string | undefined): string {
  if (!url) return '';
  try {
    return new URL(url, location.href).pathname;
  } catch {
    return url;
  }
}

export interface PanelModel {
  key: string;
  entry: ModelEntry | undefined;
  /** A file dropped onto the page (its own materials, no manifest entry). */
  dropped?: string;
  template: ModelTemplate;
  far: ModelTemplate | null;
  clips: Shown[];
  /** The content objects that draw it. */
  defs: ObjectDef[];
}

/** The panel's HTML for `model`; rows marked `data-mesh`/`data-part` isolate what they name (`inspect.ts`). */
export function panelHtml(model: PanelModel, content: Content | null): string {
  const { key, entry, template, far, clips, defs } = model;
  const extra = entry as (ModelEntry & { materials?: Record<string, { finish?: string; color?: string }>; motions?: unknown; accept?: Record<string, string>; fit?: number[] }) | undefined;
  const out: string[] = [];
  out.push(`<h2>${esc(key.replace(/^model\./, ''))}</h2>`);
  out.push(`<div class="sub">${esc(model.dropped ? `dropped file: ${model.dropped} (its own materials, no finishes)` : key)}</div>`);
  if (entry?.url && !model.dropped) out.push(`<div class="sub">${esc(filePath(entry.url))}</div>`);
  if (template.boxed || template.failed?.length || !template.meshes.length) {
    out.push(`<p class="sub" style="color:var(--fail)">${esc(template.failed?.length ? `failed to load: ${template.failed.map(filePath).join(', ')}` : 'nothing to draw (no file)')}</p>`);
  }

  // What it is in the content.
  const def = defs[0];
  const info: [string, string][] = [['Family', esc(familyOf(entry))]];
  if (defs.length) info.push(['Objects', defs.map((d) => esc(d.id)).join(', ')]);
  else if (!model.dropped) info.push(['Objects', '<span class="muted">none draws it</span>']);
  if (def) {
    info.push(['Layer', esc(def.layer ?? 'floor')]);
    if (def.footprint) info.push(['Footprint', esc(`${def.footprint[0]} × ${def.footprint[1]} m`)]);
    if (def.layer === 'surface' && content) info.push(['Radius', m(content.radius(def))]);
    if (def.outdoors) info.push(['Outdoors', 'yes']);
  }
  if (extra?.fit) info.push(['Fit', esc(extra.fit.join(' × '))]);
  if (entry?.scale) info.push(['Scale', esc(entry.scale)]);
  if (entry?.rotationY) info.push(['Turned', esc(`${entry.rotationY}°`)]);
  out.push(section('Info', rows(info)));

  // Its size.
  const box = restBox(template);
  if (box) {
    const size: [string, string][] = [
      ['Width (x)', m(box.hi[0] - box.lo[0])],
      ['Height (y)', m(box.hi[1] - box.lo[1])],
      ['Depth (z)', m(box.hi[2] - box.lo[2])],
      ['x', `${box.lo[0].toFixed(3)} … ${box.hi[0].toFixed(3)}`],
      ['y', `${box.lo[1].toFixed(3)} … ${box.hi[1].toFixed(3)}`],
      ['z', `${box.lo[2].toFixed(3)} … ${box.hi[2].toFixed(3)} <span class="muted">(+z its front)</span>`],
    ];
    out.push(section('Size at rest', rows(size)));
  }

  // What it costs to draw.
  const full = counts(template);
  const stats: [string, string][] = [
    ['Triangles', n(full.triangles)],
    ['Vertices', n(full.vertices)],
    ['Draw calls', `${full.draws}${full.draws > 3 ? ' <span style="color:var(--warn)">(over 3)</span>' : ''}`],
  ];
  if (far) {
    const f = counts(far);
    const share = full.triangles ? Math.round((f.triangles / full.triangles) * 100) : 0;
    stats.push(['Far triangles', `${n(f.triangles)} <span class="${share > 50 ? '' : 'muted'}" ${share > 50 ? 'style="color:var(--warn)"' : ''}>(${share} %)</span>`]);
    stats.push(['Far draw calls', String(f.draws)]);
  }
  out.push(section('Stats', rows(stats)));

  // Its meshes: per moving part, one per (tint, finish) group (named `<key>|<part>:<tint>@<finish>`,
  // `models.ts` `PartGroups.build`), and one per file material the entry doesn't map onto a finish.
  const meshRows = template.meshes
    .map((mesh, i) => {
      // (After the key, which may hold any of these: a dropped file's.)
      const own = (mesh.name.startsWith(key) ? mesh.name.slice(key.length) : mesh.name).split('|').pop() ?? '';
      const finish = own.includes('@') ? own.slice(own.lastIndexOf('@') + 1) : null;
      const tint = /:([^#@]+)/.exec(own)?.[1];
      // (Without a finish: the plain one, or a file material drawn as it is.)
      const what = [finish ?? (mesh.material.name.replace(/^material\.finish\./, '') || 'plain'), tint && `tint ${tint}`].filter(Boolean).join(' · ');
      const part = template.motion?.parts[i] ?? -1;
      const tris = Math.floor((mesh.mesh.primitive[0]?.count ?? 0) / 3);
      return `<tr class="row" data-mesh="${i}"><td>${i}</td><td>${esc(what)}${part >= 0 ? ` <span class="muted">· part ${esc(template.motion?.names?.[part] ?? part)}</span>` : ''} <span class="muted">${n(tris)} tris</span></td></tr>`;
    })
    .join('');
  let materials = `<table>${meshRows}</table><div class="muted">Hover a row to see that mesh alone; click to keep it.</div>`;
  if (extra?.materials) {
    materials += `<div style="margin-top:6px"><b>File materials → finishes</b></div>`;
    materials += rows(Object.entries(extra.materials).map(([name, o]) => [name, esc([o.finish?.replace(/^material\.finish\./, ''), o.color].filter(Boolean).join(' · ') || '—')]));
  }
  if (entry?.paint?.length) {
    materials += `<div style="margin-top:6px"><b>Paint</b></div>`;
    materials += rows(entry.paint.map((p) => (typeof p === 'string' ? [p, p] : [p.material, esc(p.label)]) as [string, string]));
  }
  out.push(section(`Meshes (${template.meshes.length})`, materials));

  // Moving parts and clips.
  if (template.motion?.names?.length || clips.length) {
    const parts = (template.motion?.names ?? []).map((name, p) => `<tr class="row" data-part="${p}"><td>${p}</td><td>${esc(name)}</td></tr>`).join('');
    const clipRows = clips
      .map((c) => {
        const how = [c.kind, c.delay ? `delay ${c.delay}s` : '', c.every ? `every ${c.every}s` : ''].filter(Boolean).join(', ');
        return `<tr class="row" data-clip="${esc(c.name)}"><td>${esc(c.name)}</td><td>${c.clip.duration.toFixed(2)} s · ${esc(how)} <span class="muted">· ${c.clip.tracks.length} part${c.clip.tracks.length === 1 ? '' : 's'}</span></td></tr>`;
      })
      .join('');
    let body = '';
    if (parts) body += `<div><b>Moving parts</b> <span class="muted">(click to see one alone)</span></div><table>${parts}</table>`;
    if (clipRows) body += `<div style="margin-top:6px"><b>Clips</b> <span class="muted">(click to play one)</span></div><table>${clipRows}</table>`;
    out.push(section('Motion', body));
  }

  if (extra?.accept) out.push(section('Accepted by the gate', rows(Object.entries(extra.accept).map(([k, why]) => [k, esc(why)]))));
  if (entry) out.push(section('Manifest entry', `<pre>${esc(JSON.stringify(entry, null, 2))}</pre>`, false));
  if (!model.dropped) {
    out.push(section('Gate', `<div class="bar"><button class="pill" id="gate">Run cargo check-glb</button><span id="gate-state" class="muted"></span></div><pre id="gate-out" hidden></pre>`));
  }
  return out.join('');
}

/** The gate's report as HTML, its `ok`, `FAIL`, warnings and accepted findings coloured. */
export function gateHtml(text: string): string {
  return text
    .split('\n')
    .map((line) => {
      const t = line.trimStart();
      const cls = /^FAIL\b|error/i.test(t) ? 'fail' : /^ok\b/.test(t) ? 'ok' : /^(warning|warn)\b|^accepted\b/i.test(t) ? 'warn' : '';
      return cls ? `<span class="${cls}">${esc(line)}</span>` : esc(line);
    })
    .join('\n');
}

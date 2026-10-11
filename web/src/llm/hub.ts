/**
 * A model from Hugging Face, described for the LLM worker (`ModelSpec`): given a repository
 * (`onnx-community/SmolLM2-135M-Instruct-ONNX`, or its URL), finds its 4-bit ONNX build, its
 * tokenizer and its chat format, pinned to the repository's current revision. Checks only what
 * can be told from the files; whether it runs is found when it loads.
 */

import { detectChatFormat, turnEnds } from './chat';
import type { ModelFile, ModelSpec } from './models';

const HUB = 'https://huggingface.co';
/** The 4-bit builds looked for, in order (onnx-community's layout, and Optimum's). */
const GRAPHS = ['onnx/model_q4.onnx', 'model_q4.onnx'];
/** Bigger than this isn't a small model (and wouldn't fit the browser's memory well). */
const MAX_BYTES = 1.5e9;

interface Sibling {
  rfilename: string;
  size?: number;
  lfs?: { sha256?: string };
}

/** "owner/name" from a repository name or URL, or null. */
export function repoOf(input: string): string | null {
  const m = /^(?:https?:\/\/huggingface\.co\/)?([\w.-]+\/[\w.-]+?)(?:\/.*)?$/.exec(input.trim());
  return m ? m[1] : null;
}

/** The model in a Hugging Face repository; throws with a reason if it can't be used. */
export async function describeHubModel(input: string): Promise<ModelSpec> {
  const repo = repoOf(input);
  if (!repo) throw new Error('Give a Hugging Face repository, such as onnx-community/SmolLM2-135M-Instruct-ONNX.');
  const res = await fetch(`${HUB}/api/models/${repo}?blobs=true`);
  if (res.status === 401 || res.status === 404) throw new Error(`${repo} wasn't found (or it's private).`);
  if (!res.ok) throw new Error(`Hugging Face answered ${res.status}.`);
  const info = (await res.json()) as { sha: string; siblings: Sibling[]; cardData?: { license?: string }; gated?: boolean | string };
  if (info.gated) throw new Error(`${repo} needs you to accept its terms on Hugging Face first; the game can't download it.`);
  const files = new Map(info.siblings.map((s) => [s.rfilename, s]));
  const file = (name: string): ModelFile | null => {
    const s = files.get(name);
    return s ? [name, s.size ?? 0, s.lfs?.sha256] : null;
  };
  const graph = GRAPHS.map(file).find((f): f is ModelFile => !!f);
  if (!graph) throw new Error(`${repo} has no 4-bit ONNX build (onnx/model_q4.onnx).`);
  if ([...files.keys()].some((n) => n.startsWith(`${graph[0]}_data_`))) throw new Error(`${repo}'s weights are split into several files; only one is supported.`);
  const data = file(`${graph[0]}_data`) ?? undefined;
  const tokenizer = file('tokenizer.json');
  const tokenizerConfig = file('tokenizer_config.json');
  if (!tokenizer || !tokenizerConfig) throw new Error(`${repo} has no tokenizer.json and tokenizer_config.json.`);
  const bytes = graph[1] + (data?.[1] ?? 0) + tokenizer[1];
  if (bytes > MAX_BYTES) throw new Error(`${repo} is ${Math.round(bytes / 1e6)} MB: too big for residents' lines (at most ${MAX_BYTES / 1e9} GB).`);

  const base = `${HUB}/${repo}/resolve/${info.sha}/`;
  const config = (await (await fetch(base + 'tokenizer_config.json')).json()) as { chat_template?: unknown; bos_token?: unknown; eos_token?: unknown };
  let template = typeof config.chat_template === 'string' ? config.chat_template : '';
  if (!template && files.has('chat_template.jinja')) template = await (await fetch(base + 'chat_template.jinja')).text();
  const token = (t: unknown) => (typeof t === 'string' ? t : t && typeof t === 'object' && 'content' in t ? String((t as { content: unknown }).content) : null);
  const chat = detectChatFormat(template, token(config.bos_token));
  if (!chat) throw new Error(`${repo}'s chat format isn't one the game can write (ChatML or Gemma).`);
  const eos = token(config.eos_token);
  const name = repo.split('/')[1].replace(/-ONNX$/i, '');
  return {
    id: `hf:${repo}`,
    label: name,
    about: `${repo}, added from Hugging Face.`,
    version: `hf:${repo}@${info.sha.slice(0, 7)}`,
    url: base,
    tokenizer,
    tokenizerConfig,
    graph,
    data,
    chat,
    stop: [...new Set([...turnEnds(chat), ...(eos ? [eos] : [])])],
    defaults: { maxTokens: 32, temperature: 0.7, topK: 40, topP: 0.9, repetitionPenalty: 1.15, seed: 1, stopAtNewline: true },
    licence: info.cardData?.license ?? "see the model's page",
    added: true,
  };
}

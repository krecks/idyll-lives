/**
 * The words for asking the model, per language (`content/talk/<lang>.json`), with what content
 * packs add (`talk` in a pack, `Content.talk`) laid over them.
 */

import { withBuild } from '../assets/build';
import { mergeDeep, type Content } from '../content/content';
import type { TalkText } from './prompt';

const files = new Map<string, Promise<TalkText | null>>();

export function loadTalk(language: string, content?: Content): Promise<TalkText | null> {
  let file = files.get(language);
  if (!file) {
    file = fetch(withBuild(`${import.meta.env.BASE_URL}content/talk/${language}.json`))
      .then((r) => (r.ok ? (r.json() as Promise<TalkText>) : null))
      .catch(() => null);
    files.set(language, file);
  }
  const packs = content?.talk[language];
  return packs ? file.then((t) => (t ? (mergeDeep(t as unknown as Record<string, unknown>, packs as Record<string, unknown>) as unknown as TalkText) : null)) : file;
}

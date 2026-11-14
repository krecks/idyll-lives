/**
 * The words for asking the model, per language (`content/talk/<lang>.json`), with what content
 * packs add (`talk` in a pack, `Content.talk`) laid over them.
 */

import { dataUrl } from '../assets/bundle';
import { mergeDeep, type Content } from '../content/content';
import type { TalkText } from './prompt';

const files = new Map<string, Promise<TalkText | null>>();

export function loadTalk(language: string, content?: Content): Promise<TalkText | null> {
  let file = files.get(language);
  if (!file) {
    file = dataUrl(`${import.meta.env.BASE_URL}content/talk/${language}.json`)
      .then((url) => fetch(url))
      .then((r) => (r.ok ? (r.json() as Promise<TalkText>) : null))
      .catch(() => null);
    files.set(language, file);
  }
  const packs = content?.talk[language];
  return packs ? file.then((t) => (t ? mergeDeep(t, packs) : null)) : file;
}

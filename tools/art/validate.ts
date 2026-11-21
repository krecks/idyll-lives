/**
 * The Khronos glTF validator over model files, for `cargo check-glb --validate`
 * (crates/tools/src/checkglb/validate.rs, which decides what fails):
 *
 *     bun tools/art/validate.ts <file.glb>…
 *
 * Prints a JSON line for each file, in the order given: `{"file", "issues": [{"code", "severity",
 * "pointer", "message"}]}` (severity 0 an error, 1 a warning, 2 information, 3 a hint, every issue
 * the validator reports), or `{"file", "error"}` for a file it could not read or validate.
 *
 * The validator is web/package.json's `gltf-validator` (`bun install` in web/), imported by its path:
 * nothing resolves packages from tools/.
 */
import { readFile } from "node:fs/promises";

type Issue = { code: string; severity: number; pointer?: string; message: string };
type Validator = {
  validateBytes(data: Uint8Array, options: { maxIssues: number; writeTimestamp: boolean }): Promise<{
    issues: { messages: Issue[] };
  }>;
};

/** Files validated at once: the reads overlap, the validating is one thread. */
const AT_ONCE = 8;

const { validateBytes } = (await import(
  new URL("../../web/node_modules/gltf-validator/index.js", import.meta.url).href
)) as Validator;

async function validate(file: string): Promise<object> {
  try {
    const bytes = new Uint8Array(await readFile(file));
    // (maxIssues 0: every issue, not the first hundred.)
    const report = await validateBytes(bytes, { maxIssues: 0, writeTimestamp: false });
    const issues = report.issues.messages.map((m) => ({
      code: m.code,
      severity: m.severity,
      pointer: m.pointer ?? "",
      message: m.message,
    }));
    return { file, issues };
  } catch (e) {
    return { file, error: String(e) };
  }
}

const files = process.argv.slice(2);
const out: object[] = new Array(files.length);
let next = 0;
async function worker(): Promise<void> {
  while (next < files.length) {
    const i = next++;
    out[i] = await validate(files[i]);
  }
}
await Promise.all(Array.from({ length: AT_ONCE }, worker));
process.stdout.write(out.map((o) => JSON.stringify(o) + "\n").join(""));

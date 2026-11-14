#!/usr/bin/env bash
# Which of CI's gated jobs (.github/workflows/ci.yml: rust, web, tools) a file can affect, and so
# which of them a change has to run, and the hash of the files each job reads (its passed result is
# remembered under it). One map for both, in `jobs_for` below: a path it doesn't know runs every job.
#
#   tools/ci-changes.sh changed <base> [<head>]   the jobs a change from <base> to <head> (default
#                                                 HEAD) runs, as `rust=true` lines (ci.yml's
#                                                 `changes` job writes them to its outputs); the
#                                                 files and their jobs on stderr
#   tools/ci-changes.sh all                       every job (a push to main, a tag, a run by hand)
#   tools/ci-changes.sh key <job> [<commit>]      the hash of the files <job> reads at <commit>
#                                                 (default HEAD), with SLOW_TESTS
#   tools/ci-changes.sh jobs <path>...            each path's jobs (to check the map)
#
# The map comes from what each step of tools/ci-local.sh opens:
#   rust   fmt and clippy over the whole workspace: crates/, Cargo.toml and .lock, .cargo/, the
#          toolchain; clippy builds sim-core's tests and examples, which `include_str!`
#          web/public/content files; the sim tests read web/public/content and web/public/assets
#          (the manifests, and every model they name must be there).
#   web    the clip check: tools/characters (check_clips.ts, transitions.ts, clip-baseline.json),
#          web/src's clips, furniture.json and actions.ts, web/public/assets/characters/rig.json;
#          the renderer tests: web/ (src, tests, public, package.json and bun.lock for
#          `bun install`); doors, tickRates and snapshotLayout read crates/sim-core/src, the slow
#          sampleWorlds runs tools/worlds/sample.ts.
#   tools  ll-tools (on sim-core) and its tests: crates/, Cargo.*, .cargo/, the toolchain; web/ but
#          web/tests: web/public (assets, content, the voice and LLM licences), web/src (the
#          pinned regions, actions.ts, audio/world.ts, the patched shaders), web/package.json and
#          bun.lock (the chunk check's PlayCanvas); tools/pins/pins.json (pinned files in web/src
#          and crates/sim-core/src), tools/render/chunks.json, tools/art/blender/centres.json
#          (check-glb), docs/review/reference.json (`review sizes` and `uses`).
# ci.yml itself (the jobs, bun-version, SLOW_TESTS), tools/ci-local.sh and this script run every
# job. Read by no step (checked by grep), so no job: README.md, CHANGELOG.md (pages.yml's version
# check reads it, on a tag), PLAN.md, plan/, LICENSE, docs/ but review/reference.json, .gitignore,
# .git-blame-ignore-revs, pages.yml, crates/tools/README.md, tools/characters/README.md, tools/art/
# but centres.json (only `cargo art` and `check-glb --preview` run Blender), and the tools no job
# runs: tools/bench, llm, voice, audio, and tools/housegen (only `generated`'s, which runs on a tag
# or by hand, on everything).
set -euo pipefail
cd "$(dirname "$0")/.."

JOBS="rust web tools"

# The gated jobs a path can affect, space-separated ("" for none), into J (no subshell: `key` asks
# for every tracked file).
jobs_for() {
  case "$1" in
    .github/workflows/ci.yml | tools/ci-local.sh | tools/ci-changes.sh) J="rust web tools" ;;
    rust-toolchain.toml | Cargo.toml | Cargo.lock | .cargo/*) J="rust tools" ;;
    crates/tools/README.md) J="" ;;
    crates/sim-core/src/*) J="rust web tools" ;;
    crates/*) J="rust tools" ;;
    web/public/content/* | web/public/assets/*) J="rust web tools" ;;
    web/tests/*) J="web" ;;
    web/*) J="web tools" ;;
    tools/characters/README.md) J="" ;;
    tools/characters/* | tools/worlds/*) J="web" ;;
    tools/pins/* | tools/render/* | tools/art/blender/centres.json | docs/review/reference.json) J="tools" ;;
    README.md | CHANGELOG.md | PLAN.md | LICENSE | plan/* | docs/* | .gitignore | .git-blame-ignore-revs | \
      .github/workflows/pages.yml | tools/art/* | tools/bench/* | tools/llm/* | \
      tools/voice/* | tools/audio/* | tools/housegen/*)
      J="" ;;
    # Anything else: every job, until the map says otherwise.
    *) J="rust web tools" ;;
  esac
}

has() { case " $1 " in *" $2 "*) return 0 ;; esac; return 1; }

sha256() { if command -v sha256sum >/dev/null; then sha256sum; else shasum -a 256; fi | cut -d' ' -f1; }

case "${1:-}" in
  jobs)
    shift
    for p in "$@"; do
      jobs_for "$p"
      printf '%s: %s\n' "$p" "${J:-none}"
    done
    ;;
  all)
    for j in $JOBS; do echo "$j=true"; done
    ;;
  changed)
    base=${2:?changed <base> [<head>]}
    head=${3:-HEAD}
    run=""
    n=0
    # (Without rename detection: a rename is its old path deleted and its new one added, both counted.)
    while IFS= read -r -d '' p; do
      n=$((n + 1))
      jobs_for "$p"
      echo "  $p: ${J:-none}" >&2
      for j in $J; do has "$run" "$j" || run="$run $j"; done
    done < <(git diff --name-only --no-renames -z "$base" "$head" --)
    echo "$n changed files; jobs to run:${run:- none}" >&2
    for j in $JOBS; do if has "$run" "$j"; then echo "$j=true"; else echo "$j=false"; fi; done
    ;;
  key)
    job=${2:?key <job> [<commit>]}
    has "$JOBS" "$job" || { echo "no job $job (jobs: $JOBS)" >&2; exit 2; }
    commit=${3:-HEAD}
    # Every tracked file the job reads, by its blob hash and mode (a new or gone file changes it
    # too), then the slow tests' switch: the same hash, the same inputs.
    {
      while IFS= read -r -d '' entry; do
        p=${entry#*$'\t'}
        jobs_for "$p"
        if has "$J" "$job"; then printf '%s\0' "$entry"; fi
      done < <(git ls-tree -r -z --full-tree "$commit")
      printf 'job=%s\0SLOW_TESTS=%s\0' "$job" "${SLOW_TESTS:-}"
    } | sha256
    ;;
  *)
    echo "usage: tools/ci-changes.sh changed <base> [<head>] | all | key <job> [<commit>] | jobs <path>..." >&2
    exit 2
    ;;
esac

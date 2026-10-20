# Smarter thoughts (research)

*Status: research note for 0.3.0. Nothing here ships in 0.3.0; the utility AI keeps deciding and
saves keep their format.*

Residents already think in fixed kinds: the planner records a `Thought` when a planned block is
skipped or kept, a goal is reached, a room is loved or disliked, something breaks, a baby cries, or
dinner is ready (`crates/sim-core/src/planner.rs`, `ThoughtKind`). The snapshot sends its kind and
subject (`sim::THOUGHT`, `sim::THOUGHT_SUBJECT`) and the UI draws a bubble. Voices turn the same
moments into spoken lines after the fact (`web/src/talk/`), but the decision that led there is made
by the utility AI in `crates/sim-core/src/world.rs` (`pick_autonomous`, `ai::choose`).

This note asks what it would take for the optional language model to make those thoughts *about
the actual choice*, and at most nudge the choice itself, without giving up determinism, frame rate
or the game's ability to run without a model.

## Where the model sits

The model stays outside `sim-core`, in the web worker that already writes spoken lines
(`web/src/llm/`, LFM2.5 350M on ONNX Runtime Web). `sim-core` must not link a tokenizer or wait on
a worker: it is also the code that runs a century in a test and a town at low detail on a player's
machine.

The flow for one decision:

1. The sim builds its candidate list exactly as it does now and keeps the top three, with their
   scores and the facts behind them (need, object, room, plan, trait, recent failures).
2. The snapshot or a side channel carries that shortlist to the web. It is read-only; the sim has
   already made a safe choice and keeps running.
3. The web asks the model, with a short prompt: "A, B or C, and one sentence on why?" It returns a
   letter and a thought.
4. If the answer arrives before the resident's next decision point, the host sends it back as a
   command. `sim-core` checks that the candidate is still in the current top three and that the
   resident has not started something else, then applies it like a player command.
5. If it arrives late, or the model is off or slow, the command is dropped. The utility AI's
   choice stands. The thought can still be attached to that choice, because it describes the
   moment rather than changing it.

The thought text never enters the simulation. It is presentation, like a voice line. Only the
letter (the chosen candidate) is an input.

## What the model sees

Keep the packet small and already in words, the way `talk/context.ts` builds a line's moment:

- who the resident is: life stage, traits, perks, job or school;
- what they need and how they feel, at the same low-need threshold the voices use;
- the plan block and goal they are following, if any;
- the three candidates: the interaction's label, the object and room, the need it serves, its
  expected gain and duration, and why the others lost;
- the last few things that happened to them (a failure, a broken object, a crowded bathroom),
  bounded to a handful of lines.

Do **not** send the whole world, the object list, or every resident. The shortlist is the point:
the model ranks options the sim already considers legal, so it cannot invent an action, walk
through a wall, or use an object another resident reserved.

The prompt is not the line-writing prompt. It needs a compact decision rubric and a one-token
answer (`A`, `B`, `C`, or `-` for "no preference"), then optionally a separate thought sentence.
Asking for one letter first keeps the expensive part short and makes a truncated reply easy to
reject.

## Determinism

The simulation is deterministic for a seed (`deterministic_for_same_seed` in `world.rs`): the same
seed, content and commands produce the same ticks. A model is not deterministic across devices,
versions or sampling settings, so its output must enter as an *input*, never as hidden state.

The rule:

- **No model call inside a tick.** The sim never blocks on the worker and never reads model output
  from a global.
- **Every accepted re-rank is a command.** A new `Command::Choose` (or `Rerank`) names the
  resident, the decision it belongs to and the candidate index. `World::apply` validates it against
  the live shortlist and rejects a stale one. Applying a command is already how the game handles
  every other outside input, and it is the only place the choice can change.
- **Late or absent answers are not inputs.** If the deadline passes, the sim's own weighted choice
  is final. A reply that arrives afterwards is dropped, not queued for the next decision, so a
  slow machine and a fast one replay the same commands.
- **The thought text is never replayed.** It is regenerated (or cached for display) on the client.
  Saves do not need model text to reproduce behaviour.

For a true replay, the command journal has to contain the accepted `Choose` commands in order,
alongside the player's. There is no command journal today: saves store state, and tests replay by
re-running the same seed and commands. Adding one is a save-format change and therefore out of
scope for 0.3.0. Until then, "recorded" means recorded by the host for the current session; a save
after the fact already contains the outcome (the resident's task and thought), which is enough to
load and continue.

If a future release adds a journal, the format can be a list of `{ tick, command }`, with the
existing `Command` enum reused. The model's text stays out of it.

## Cost and fallback

- **One short call, not one per tick.** A resident makes an autonomous choice every few game
  minutes at most; at normal speed that is seconds apart. Coalesce: one in flight per resident, and
  no call while the resident is asleep, away, or already doing something the player asked for.
- **Only the player's household and the inspected resident.** The same set that has full needs and
  plans in the snapshot (`talk/context.ts`), and the only ones whose thoughts are shown. Neighbours
  keep the utility AI.
- **Cache the persona and rules.** The line writer already keeps the unchanged prompt prefix in
  the model's state (`llm/generate.ts`); a decision prompt should do the same. A cached prefill plus
  a one-token answer is the cheap case. The thought sentence is the optional second call.
- **Target under ~400 ms** for the letter on the reference machine, inside the existing 3.5 s
  line budget. This is a target to measure, not a measured result; the current writer's numbers
  are for 1–5 sentence lines, not a three-way choice.
- **Fallback is the game as it is today.** No WebGPU, no model downloaded, model off, worker
  crashed, reply late: `pick_autonomous` chooses exactly as before, and the thought bubble shows
  the existing `ThoughtKind`. Voices may still speak a scripted line. Nothing is blocked and no
  toast is needed for a missed re-rank.
- **A manual "Voices › Language model" setting stays the switch.** If the player turned the model
  off, thoughts are not written either. Do not add a second download or a second worker.

## What to measure before building it

1. **Latency**: cached prefill and one-letter decode for a 500–900 token decision prompt on the
   reference M3 Pro, and on a machine without WebGPU (the fallback must be the utility AI, not a
   slow CPU call).
2. **Agreement**: how often the model's letter matches the utility AI's choice, and whether the
   disagreements are plausible rather than destructive (the "eat at 3 a.m." problem).
3. **Cost per game hour**: calls per resident per hour at normal speed, with coalescing, and the
   share of frames affected. The line writer's watchdog already pauses voices when the model costs
   too much; a decision loop needs the same guard, or it will compete with rendering.
4. **Determinism**: replay a recorded session with the model on, then with it off and the accepted
   `Choose` commands replayed. The ticks must match. A stale command must be rejected, not applied
   to a different shortlist.

## What not to do

- Do not let the model choose from the whole interaction list. It cannot know reservations,
  reachability or content rules, and a bad choice is a bug the player sees.
- Do not wait for the model before the resident acts. The sim is real-time; a stalled resident is
  worse than a plain choice.
- Do not put model scores into the utility function as a hidden bias. That makes the same seed
  behave differently on two machines with no record of why.
- Do not write thought text into the save. It is presentation and would grow saves for no
  behavioural gain.

## Next step

A prototype behind a debug flag (`?thoughts=model`), using the existing `llm/` worker and a
`Command::Choose` that is only accepted from the debug host. Measure latency and agreement on the
benchmark village before deciding whether the re-rank is worth a save-format change. If the
latency or the choices are poor, keep the model to *describing* the utility AI's choice (the
"thoughts" half) and leave the decision alone; that half needs no new command and no save change.

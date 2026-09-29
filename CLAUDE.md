# assimilator-visualizer

Renders a 3D video of one Assimilator simulation run, for presentations, from a CLI that
a harness can call. Rust, Bevy (headless), ffmpeg. The engine is `../assimilator` and the
harness is `../assimilator-harness`; both are **read-only** from this repo, and other
sessions work in them. What this project needs from either is written down as a request
in a spec's open questions, never done there. The seed brief is
`~/dev/ivapo/sim-3D-visualizer/idea.md`.

**Never run the engine inside its repo** (a run writes `results.db` and `fcd/` into the
project). Copy a project to a scratch folder and run it there.

**Disk is tight (about 100 GB free).** Keep Bevy features trimmed, don't enable
`dynamic_linking` by default, and don't create extra worktrees without asking.

**Licensed MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`): every crate sets `license = "MIT OR Apache-2.0"` in its `Cargo.toml`.

## Development flow

This repo is developed spec-driven. Two artifacts, one job each:

- **`specs/<name>_spec.md`** — why we decided something, and the plan. Append-only once
  `accepted`; it does not track the code and may drift from it.
- **`rules/<subsystem>.md`** — what is true right now. It **does** track the code, and is
  corrected against its own sources rather than rewritten from scratch — freely, with no
  dated note. Each one declares its own `sources`, `covers` and `max_lines`.

The methodology is `/Users/ivapo/dev/main/spec-driven-dev/spec-authoring.md` — the
frontmatter schema, the phase gate, the review loop; every `§N` below refers to it. Spec
ids are `vis-NNN`, allocated as `max(existing) + 1` and never reused; `.spec-lint.yaml`'s
`id_prefix` is the authoritative home for the prefix.

**The observable this project produces is: a video file rendered from one simulation
run.** Every phase says whether it produces one; a phase that does not is argued for
explicitly.

**The engine side of any boundary has one home in the engine repo** (asm-001 §10 for the
harness boundary). Specs here cite engine files as `file:symbol` and state only what this
repo requires and does.

**Before drafting or changing a spec, read `specs/INDEX.md`. Before changing a
subsystem, read `rules/INDEX.md`.** Both are generated from frontmatter by
`spec-lint --write-index` — never hand-edit them. The linter is
`/Users/ivapo/dev/main/spec-driven-dev/bin/spec-lint` (not on `PATH`).

**A phase is not cleared to build until its own review round has converged** — that is
`reviewed` on the phase, not `status` on the document. Run `/review-spec <spec> --phase N`.

**When a conversation settles on a feature, work §6.1's ordered test before assuming a new
document.** Step 0 asks whether a decision changed at all; step 2 — append a phase to the
spec that owns the subject — is the commonest real answer, and the one a fresh context is
least likely to reach for.

**"Implement Phase N of `specs/X`" carries two standing plan steps and a close-out** (§3).
The plan states a **commit plan** — a phase is one plan, one push, and as many commits as
the work wants — and a **reconciliation step** naming which `rules/` files, which
user-facing documentation and which stanza the phase changes, or "none needed" with a
reason. When the exit gate passes, **write that phase's `shipped` date** into `phases[]`:
`/review-spec` owns `reviewed`, and nothing else owns `shipped`.

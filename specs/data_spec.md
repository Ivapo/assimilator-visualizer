---
id: vis-003
title: data
note: >
  The run's data on screen, in render and view. Phase 1 draws the signals as the engine's
  dashboard does: one dot per signalised approach link at its stop line, green, amber or
  red, a fixed size on screen with a black outline, by default in both cameras;
  `--no-signals` gives today's frames and `view`'s `L` hides and shows them. The states
  are replayed with the engine's own signal code at the pin, stepped once through the
  run into a timeline, and checked against the run's FCD. Phase 2 (shipped 2026-10-20)
  draws a HUD in the top-left corner of every frame: the sim clock, the vehicles on the
  road and their mean speed, from the FCD's rows at each frame's time, by default;
  `--no-hud` gives Phase 1's frames and `view`'s `H` hides and shows it. Phase 3 (drafted)
  colours each link by a measure of its FCD rows over the past window ending at the
  frame's time, off by default: `--link-colours` turns it on, `view`'s `C` cycles it.
status: accepted
last_updated: 2026-10-20

phases:
  - name: "Phase 1 — Signals: the engine's signal states as dots at the stop lines, by default"
    reviewed: 2026-10-09
    shipped: 2026-10-18
    cut: null
    by: null
  - name: "Phase 2 — HUD: the clock, the vehicles on the road and their mean speed, top-left, by default"
    reviewed: 2026-10-19
    shipped: 2026-10-20
    cut: null
    by: null
  - name: "Phase 3 — Link colours: each link by a measure over the past window, off by default"
    reviewed: null
    shipped: null
    cut: null
    by: null

extends: null
supersedes: null
superseded_by: null
related: [vis-001, vis-002]
reference: >
  Engine: ~/dev/main/assimilator at the pin of vis-001 §2.2.2 (90b39292), read-only, from
  cargo's checkout of that rev. Its signal code (crates/core/src/systems/signal.rs) is
  what Phase 1 replays, and its dashboard's signal layer is what Phase 1 draws: the
  positions from crates/geometry/src/frame_collect.rs, and the look copied from the web
  front end, web/src/canvas/vehicleRenderer.ts (drawSignals) and web/src/canvas/colors.ts,
  read at 90b39292 on 2026-10-09. Out of scope from the dashboard: its junction panel
  (phase, time remaining), crossing and conflict-point overlays, and its light palette.
  Phase 2 reads the dashboard's status bar (web/src/components/UnifiedStatusBar.tsx,
  SimStats) and its speed formats, and the engine's own count and mean speed
  (crates/output/src/runner.rs, crates/core/src/simulation.rs), at 90b39292 on 2026-10-19;
  it copies no value from them.
---

# Data

## 1. Goal

Show the run's own data in its video, beginning with the signals. A video of Midtown at a
red light shows a queue that waits, then leaves. Nothing on screen says why. With a dot at
each stop line, in the colour the engine gave that approach at that moment, a viewer sees
the queue form on red and leave on green.

**The observable is vis-001's: a video file rendered from one simulation run.** Here it is
the run's video with a dot at every signalised approach, green, amber or red as the engine
had it, in the orthographic render and in a keyframed flight.

```
# Signals are on by default, on any network with a signalised junction.
assimilator-video render --project <project> --scenario baseline --seed 42 --out run.mp4
assimilator-video render --project <project> --scenario baseline --seed 42 \
                         --buildings <dir>/buildings.geojson --camera flight.toml --out city.mp4

# Today's frames, byte for byte.
assimilator-video render … --no-signals

# L hides and shows them; --no-signals starts with them hidden.
assimilator-video view --project <project> --scenario baseline --seed 42
```

Nothing new is read from outside the run: the dots come from the scenario's resolved
network, which `render` already reads (vis-001 §2.1), and from the run's timestep in
`results.db`. They are stepped through the engine's own code (§2.6).

*(2026-10-19, Phase 2 round 1)* From Phase 2, every render also draws the HUD in the
top-left corner (§2.20). "Today's frames, byte for byte" above are then `--no-signals
--no-hud`; `--no-signals` alone gives them with the HUD.

Rejected candidates for the observable:
- *The timeline of signal states.* It is an input to a frame, as vis-002 §1 says of the
  buildings cache.
- *The `view` window with dots.* A tool for writing flights (vis-001 §2.9), which shows
  what `render` will draw.

### 1.1 Non-goals

- **No simulation.** The engine is never run, and its plan logic is never written again
  here: the states come from stepping its own `SignalState` (§2.6, decision 3).
- **Not in Phase 1:** the HUD and link colours (§2.3); a legend; a junction's phase name
  or time remaining (the dashboard's junction panel).
- **One colour per approach link**, as the dashboard and the engine's vehicles have it
  (§2.8). No colour per lane or per movement, and no colour on the stop lines: vis-002's
  per-lane stop lines stay white (§2.17).
- **No signal heads or poles.** A dot is flat colour, a fixed size on screen (§2.11).
- **Fixed-time plans only.** A run whose signals were forced from outside (the Python
  API's `set_signal`) would not replay. No such run writes FCD at the pin (§2.15), and
  recording the states in the outputs is an engine request (OQ-2).
- **Micro runs only** in Phase 1. A meso run is refused with signals on (§2.13, OQ-3).
- **No change** to the boxes, the streets, the buildings, see-through, the camera or the
  credit line. Nothing in `view`'s readout or the keyframe line.

## 2. Design

### 2.1 A spec of its own (decision, recorded)

Decided by the user, 2026-10-09 (§2.2, decision 1), and checked against the methodology's
§6.1:
- **Step 0:** decisions change. A new kind of thing is drawn from the run, and its states
  come from the engine's code, replayed. A default changes too: every render of a network
  with a signal draws dots.
- **Step 1:** nothing shipped is removed or contradicted. `--no-signals` gives today's
  frames byte for byte (gates 1 and 2). *(2026-10-09, round 1)* One default does change,
  for a kind of run no gate here has drawn: a meso run is refused with signals on, and
  renders as today with `--no-signals` (§2.13, OQ-3, gate 15).
- **Step 2:** vis-001 owns rendering one run's files, and its roadmap item 5, "Data"
  (vis-001 §2.7: "links colored by speed or flow; HUD with clock, legend, one chart;
  signal states from the plans"), names this work. By subject alone vis-001 could take a
  phase. The user chose a spec of its own: the run's data on screen is a line of work
  several phases long (signals, a HUD, link colours), each with its own source in the run
  (the engine's code; the FCD's aggregates), and vis-001 is done (7 of 7 phases shipped).
  vis-002 took vis-001's item 4 the same way (vis-002 §2.1).

So this is a new spec, `extends: null`, `related: [vis-001, vis-002]`. vis-001 §2.7 item 5
is **narrowed** here and gets one dated note pointing here, written in Phase 1's
close-out; this draft does not edit vis-001. Its "legend" and "one chart" are in no plan
(§2.3).

### 2.2 The user's decisions (decision, recorded)

Decided by the user, 2026-10-09, before drafting:
1. **A new spec, vis-003 "data"** (the run's data on screen). **Phase 1: signals.**
   Roadmap only, not drafted now:
   - **Phase 2, a HUD:** top-left, the sim clock, the cars on the road and their mean
     speed; on by default, with a flag to turn it off;
   - **later, link colours** as an optional overlay, off by default.
2. **Signals as the engine's dashboard shows them:**
   - one dot per signalised approach link, at its stop line (lane 0's centre);
   - green, amber or red;
   - a fixed size on screen, with an outline;
   - on by default in `render` and `view`, orthographic and perspective;
   - a flag turns them off and gives today's frames, byte-identical;
   - a key shows and hides them in `view`.
3. **The states are replayed with the engine's own signal code at the pin,** so the
   fixtures' runs need no engine request.

Decided by the user, 2026-10-09, on the draft, before review (the answers to OQ-1 and
OQ-4, and one confirmation). The probe's 1080p videos with dots (`runs/on-orbit.mp4`,
`on-city.mp4` and `on-ortho-city.mp4` in `scratch/vis003p1-probe/`, §2.19) were put to the
user before the answers:
4. **The dot is the dashboard's size, scaled with the frame** (OQ-1's (a), §2.10): a fill
   of 4.5 px and a ring to 5.5 px at 1080 lines.
5. **In the orthographic render a dot under a roof stays hidden,** as the boxes are (OQ-4,
   as drafted, §2.12).
6. **The key is `L`** (§2.13, as drafted, confirmed).

OQ-2 and OQ-3 stay open as drafted: deferred, and no answer was asked. *(2026-10-09,
round 1)* Both are still open, and each gained facts in review: OQ-2 what forces it, OQ-3
its class (a design call, the user's) and the shape of a meso run's row.

Decided by the user, 2026-10-09, after the review converged (the answer on OQ-3):
7. **A meso run with signals on is refused in Phase 1,** as the spec has it (§2.13), with
   `--no-signals` as the way through. The user takes this as deferring: **OQ-3 stays open
   for meso dots,** to take up when a meso fixture exists. No scope, gate or prediction
   changes.

The rest of this section is the draft's proposal. It settles:
- the replay and its clock (§2.4–§2.6), and a check of it against the FCD (§2.7);
- what the link-level colour means, permitted movements included (§2.8);
- where each dot is (§2.9), how it looks (§2.10, OQ-1, answered (a)), and how it keeps its
  size in both cameras (§2.11);
- depth, ties and determinism (§2.12; OQ-4, answered as drafted);
- the flag, the key (`L`, confirmed) and their errors (§2.13), and the scripts that read
  references (§2.14);
- a run whose signals were forced (§2.15, OQ-2), and a pin move (§2.16).

*(2026-10-20)* Decided by the user, 2026-10-20, before Phase 3 was drafted (decision 1's
"later, link colours"; designed in §2.21, which does not reopen them):
8. **Link colours are vis-003 Phase 3, one phase.** If it measures too big for one review
   and one build, the draft proposes a split as an open question and does not split it
   itself.
9. **Off by default.** One flag turns them on, in `render` and `view`, in both cameras, and
   takes a measure: speed as a share of the link's speed limit (the default), speed, cars
   per lane-km, or stopped cars. Without the flag, frames are byte-identical to Phase 2's.
10. **The window is the user's,** in seconds, default 60, and past only: it ends at the
    frame's time. A window shorter than the run's FCD row interval is refused with an
    error; the interval itself means "right now".
11. **Per link by default;** a flag draws per lane.
12. **A link with no car in the window:** for the speed measures (no speed to average) it
    stays today's road; for the count measures (a real zero) it takes the lowest step. No
    flag for it.
13. **In `view`, `C` cycles** off → each measure → off.

The flags' and the measures' names were left to the draft (§2.21.11).

### 2.3 Roadmap (not yet phases)

Each item becomes a phase appended here when it is drafted and reviewed (decision 1).
- **Phase 2 — a HUD.** In the top-left corner: the sim clock, the cars on the road and
  their mean speed, from the FCD. On by default, with an off flag. Not designed here. What
  this draft notes for it:
  - "on the road" and "mean speed" need a definition (the rows of the snapshot at `t`,
    or the boxes drawn at `t`);
  - it is text in the image, as the credit line is (vis-002 §2.14.5), at the other corner.
- **Later — link colours.** Each link coloured by a measure from the run, as an optional
  overlay, off by default. Not designed here.
- **Not planned:** vis-001 §2.7 item 5's legend and chart.

*(2026-10-19)* Phase 2 is drafted: §2.20 and §4's Phase 2. Its two notes above are
answered there: "on the road" is the rows of the snapshot at `t` (§2.20.4), and the text
is drawn as the credit line is, in the opposite corner (§2.20.9).

*(2026-10-20)* Phase 2 shipped 2026-10-20. "Later — link colours" is drafted as Phase 3:
§2.21 and §4's Phase 3. "Not planned" above is vis-001's legend and chart: a legend for the
cars' colours and a chart of the run. Phase 3 draws a legend of its own, for the link
colours only, while they are on (§2.21.9).

### 2.4 What the run records: no signal state

Checked on both fixtures at engine `90b39292`, read-only:
- **`results.db`** has the run tables and none for signals. Midtown: `runs` 1 row,
  `trips` 828, every other table empty. urban_grid: `runs` 1, `trips` 91,
  `detector_intervals` 75. No table holds a phase or a colour.
- **The FCD** has `time`, `vehicle_id`, `link_id`, `lane`, `position`, `speed`,
  `acceleration`, `vehicle_class` and `vehicle_length` (vis-001 §2.1 and §2.2.2,
  `rules/inputs.md`). No signal column.
- **The plans** are in the scenario's resolved network: each signalised junction's
  `signal_plan`, with `cycle_time`, `offset` and its `phases`. Each phase has a green
  `duration`, `amber_time`, `all_red_time`, `green_movements` and `permitted_movements`.
  A scenario's `apply` set of plans is merged into that network by
  `crates/config/src/project.rs:resolve_scenario`, which `render` already calls.
- **The run's clock** is in `runs.resolved_config`:
  `{"scenario": …, "sim_params": {"duration", "random_seed", "timestep", "warm_up"}}`,
  written by `crates/output/src/runner.rs:run_to_results_db`. Midtown: duration 1200,
  timestep 0.1, warm_up 180. urban_grid: 300, 0.1, 30. `runs.mode` is `micro` in both.
  `runs` has `PRIMARY KEY (scenario, seed)`, so a run has one row.
- **A meso run's row is another shape.** `crates/output/src/runner.rs:run_meso` writes
  `{"scenario": …, "mode": "meso", "sim_params": {"duration", "warm_up", "random_seed"},
  "meso": …}`, with **no `sim_params.timestep`**, and `runs.mode` `meso` (§2.13).

| | Midtown | urban_grid |
|---|---|---|
| Signalised junctions (others) | **83** (23) | **9** (0) |
| Plans: phases | 81 with 2, **2 with 1** | 9 with 2 |
| Each phase: green + amber + all-red | 25 + 3 + 2 s | 25 + 3 + 2 s |
| `cycle_time` | 60 (81), **30 (2)** | 60 |
| `offset` | 0 (83) | **0 (4), 5 (2), 10 (3)** |
| Movements at signals; ids in `green_movements` / `permitted_movements` | 444; 291 / 152 | 108; 72 / 36 |
| Signalised approach links | **188** | **36** |

So a signal's state at a moment is in no file. It is a function of the plan, the
timestep and the step count, and the engine's code computes it (§2.5).

### 2.5 The engine's signal code at the pin

`crates/core/src/systems/signal.rs:SignalState` holds each signalised junction's phase
index, sub-state (green, amber, all-red) and elapsed time. The parts used here:
- **`SignalState::new(&NetworkData)`**: every junction with a `signal_plan`, at phase 0,
  green, elapsed 0.
- **`advance(dt)`**: per junction, `elapsed += dt`; when `elapsed >= duration` (or the
  amber or all-red time) it subtracts that time and moves on: green → amber → all-red →
  the next phase's green, wrapping.
- **`link_signal_color(&NetworkData, &LinkId)`**: `"green"`, `"amber"` or `"red"` for an
  approach link (§2.8).
- **`signalized_approach_links()`**: every `(node, from_link)` of a movement at a
  signalised junction, sorted by node then link (the engine's hash-order audit, asm-016).
- **`force_phase(node, phase)`**: external control; it freezes a junction at a phase
  until the next call (§2.15).

All of it is public, in `assimilator-core`, already a dependency (vis-001 §2.2.1). The
simulation calls `advance` once a step, before perception
(`crates/core/src/simulation.rs:Simulation::step_decide`); `step_execute` moves the
vehicles and then adds `dt` to the clock.

#### 2.5.1 It reads neither `cycle_time` nor `offset`

`advance` walks each phase's own three durations. Nothing in `SignalState` reads
`cycle_time` or `offset`:
- **Every junction starts at phase 0, green, at time 0.** urban_grid's offsets of 5 and 10 s
  change nothing: its nine signals switch together, as Midtown's 83 do (§2.19).
- **`cycle_time` is checked only by the routing graph's validation**
  (`crates/network/src/validation.rs:validate_config`, rule 4: the phases must sum to it).
  The simulation builds that graph only for rerouters, and logs and drops its error when
  there are none (`crates/core/src/simulation.rs:Simulation::new`). So a plan whose
  phases do not sum to its `cycle_time` would run, at the sum of its phases.

This is the engine's behaviour, and the replay follows it because it is the engine's code.
A replay written from the plan (the seed brief's "signal state can be rebuilt from time")
would honour the offsets and get urban_grid wrong: measured, it puts **90 of urban_grid's
422 junction entries under a red that lasted the whole second before them**, where the
engine's own code puts none (§2.7, which states how the offsets were applied). This repo needs nothing from the engine here. The user
may want to tell the engine's sessions that the offsets are not applied.

*(2026-10-09)* The engine's `main` at `6400b9aa` (2026-10-09) still reads neither `offset`
nor `cycle_time` in `crates/core/src/systems/signal.rs` (checked read-only): the file is
the pin's, byte for byte (the same git blob, `a4d71ecc`). `offset` is not named in it, and
`cycle_time` only where `junction_phase_info` copies the plan's value into its snapshot
for the dashboard's panel; no step and no colour uses either. A fact, not a question: the
replay is the pin's code either way, and a pin move is §2.16.

#### 2.5.2 Midtown's two 30 s plans

The brief asked what the engine does with Midtown's two plans of `cycle_time` 30. Read from
the resolved network: **each has one phase, 25 + 3 + 2 s, which sums to 30**, so there is
no mismatch. The engine cycles that one phase: green 25 s, amber 3 s, all-red 2 s, green
again. Every movement at those two junctions is in that phase, protected or permitted, so
**all their approaches are green, amber and red together**, every 30 s. Measured: no
approach in either fixture is red for the whole run, and Midtown's red runs are 32 s (a
two-phase plan's other phase plus its own all-red) or 2 s (these two junctions' all-red).
The one exception is the red a second-phase approach starts the run with: 30 s, the first
phase alone (91 approaches in Midtown, 18 in urban_grid).

#### 2.5.3 The switching times are the engine's steps, not the plan's seconds

The state changes only at a step, and `elapsed` carries a float sum. Stepping the engine's
code shows where that matters:
- **Both fixtures' plans** (25 + 3 + 2 at 0.1 s) switch exactly on the plan's seconds,
  through the whole run: every green lasts 250 steps, every amber 30, every red 320 (or
  20, at Midtown's two one-phase junctions), but for a second-phase approach's first red,
  from step 0, which lasts 300.
- **A synthetic plan of 20 + 3 + 2 and 15 + 3 + 2** (gate 5) does not: its second phase
  turns amber at step 401 (40.1 s), not 400, and the first phase's green comes back at
  step 451 (45.1 s), not 450. The sum of 0.1 s steps falls a hair short of 15 s, so the
  test `elapsed >= duration` fails one step longer.

So a timeline computed from the plan's seconds can be off by a step where the engine's is
not. Stepping the engine's code reproduces it exactly, at no cost in code here (§2.6).

### 2.6 The replay: a timeline, stepped once through the engine's code

**The clock.** The engine's run loop (`crates/output/src/runner.rs:run_to_results_db`)
steps until `time ≥ duration − 1e-9`, and writes an FCD snapshot after a step whenever
`time ≥ next_fcd_time − 1e-9`, at the clock's value then. So:
- **after step `k`**, the clock reads `T_k`, the engine's own float sum of `k` timesteps
  (`T_0 = 0`), and the signals are in state `S_k`, the state after `k` calls to
  `advance`, the one the vehicles perceived during step `k`;
- **an FCD row at time `T_k`** shows the vehicles after step `k`, under `S_k`, the state
  they perceived in that step;
- *(2026-10-09, round 1; the draft said the dashboard's frame at `T_k` shows `S_k` too)*
  **the dashboard pairs them one step apart.** Its loops call `step_decide`, then
  `crates/geometry/src/frame_collect.rs:collect_frame`, then `step_execute` (the server's
  sim loop in `crates/server/src/server.rs`, and the wasm front end's `step_n`). So its
  frame carries the clock `T_{k−1}`, the vehicles after step `k − 1`, and `S_k`. The rule
  below keeps the FCD's pairing, the vehicles with the state they obeyed: §2.7 checks it
  and every gate is keyed to it. A dot here turns amber at 25.0 s of the clock, where the
  dashboard's clock reads 24.9 s;
- measured, **every FCD time is bit-equal to a `T_k`** summed here as the engine sums
  it: 1,199 of 1,199 in Midtown (1.0999999999999999 to 1199.100000000005) and 291 of 291 in
  urban_grid. The `warm_up` changes nothing for the signals.

**The rule: at sim time `t`, a frame shows `S_k` for the last step with `T_k ≤ t + 1e-6`**
(`S_0` before the first step), held between steps. At an FCD time it is that row's state. With both fixtures' plans the colours change exactly on the plan's
seconds: an approach turns amber at 25.0 s, not 24.9 s. The `1e-6` is vis-001's snapshot
tolerance (`rules/inputs.md`). Between steps, the state shown is up to 0.1 s behind the one
the vehicles are reacting to, which is under 3 frames at 30 fps and 1×.

**The proposal: a timeline built once.** When a render or `view` starts:
- `SignalState::new(&placement.data)` and `signalized_approach_links()`;
- `advance(dt)` for every step of the run, with `dt` the run's `timestep` (§2.13), as the
  engine's loop counts them: 12,000 steps in Midtown, 3,000 in urban_grid;
- after each step, `link_signal_color` for every approach, keeping only the changes: per
  approach, a list of `(k, colour)` from `k = 0`, and the times `T_k`.

A frame at `t` finds `k` by a binary search of `T_k` and each approach's colour by a binary
search of its list. It is built from the engine's code alone, so a pin move that changes
how signals step changes the timeline with it (§2.16).

| | Midtown | urban_grid |
|---|---|---|
| Steps; approaches | 12,000; 188 | 3,000; 36 |
| Changes kept | 11,708 (94 KB) | 576 (5 KB) |
| Build, median of 9 in one process (fastest – slowest) | **261 ms** (251–272) | **21 ms** (21.4–21.7) |
| of which `advance` alone | 25 ms | 0.8 ms |
| A frame's colours: 1,800 frames × every approach | 10.2 ms in all (5.6 µs a frame) | 2.4 ms |
| Approach-steps green / amber / red | 42.6 / 5.1 / 52.3 % | 41.7 / 5.0 / 53.3 % |

**The alternative: step a `SignalState` with the frames.** It needs no list, and a
render's frames go forward in time, so each frame would step from the last one. But `view`
goes back: the slider, `Shift+←` and a restart at `from` (vis-001 §2.9.1, §2.10). Each step
back would build a new state and step it from 0 to `t`: 25 ms on Midtown at its end, more than
a frame at 60 fps, on every frame of a backward scrub, and more on a longer or larger run
(it grows with steps × junctions). The
timeline answers any `t` in microseconds, both ways, and `render` and `view` share it.

**Its cost is paid once,** before the first frame: a quarter of a second on Midtown, whose
renders take 48–88 s (vis-002 §2.18.15). Most of it is `link_signal_color` called for every
approach at every step. A cheaper build is possible: one `junction_phase_info` per junction
per step, re-evaluating a junction's approaches only on a step where its sub-state
changed. Measured, it keeps the same changes in 224 ms on Midtown and 14 ms on
urban_grid. It is not proposed: it saves 37 ms once per render, and the build above asks
`link_signal_color` itself, the function the vehicles perceive (§2.7), at every step, with
no second reading of the phase.

*(2026-10-09, round 1)* The draft gave 259 ms (245–261) and 22 ms for the build, 18 and
1.1 ms for `advance`, and 355 ms for the cheaper build, "slower". The probe's kept output
(`out/stats-midtown.txt`, `out/stats-urban_grid.txt`) has the numbers given here, and they
replace the draft's. No gate is keyed to any of them.

### 2.7 The check against the FCD

The replay is the engine's code, so the question a check answers is whether the clock
lines up (§2.6), on these two runs. The FCD can say, because the engine's vehicles obey the
very colour the dot shows: perception takes `link_signal_color` for the vehicle's link
(`crates/core/src/systems/perception.rs:compute_junction_ahead`, its `ControlType::Signal` arm),
and red stops them.

**When a vehicle enters a junction.** While a micro vehicle crosses a junction, its rows
stay on the approach with `position` frozen at its entry value, and `speed` stays live
(vis-001 OQ-2, asm-020 §15.4). So for each visit to a signalised approach that is followed
by a row on another link, the entry happened in the window between the row before the
first moving row of the visit's trailing frozen run (equal `position`, `speed ≥ 0.1`) and
that row: 1 s, 10 steps, in every case here (7,536 and 422 of them). The window's colours
are the approach's states `S_k` over its 10 steps. Its four classes: **green** (any green
step), **amber** (amber and no red), **amber then red**, and **red throughout**.

**The first stopped boxes.** vis-002's rule for them (its Phase 6 gate 7): a row under
0.1 m/s, first on its `(link, lane)`, within 5 m of its stop line; an episode is a
vehicle's run of such rows with no break over 1.5 s. Here at signals only, with the colour
at each episode's first row, and the colour at the vehicle's next row after the episode.

**Measured, on both whole runs:**

| | Midtown | urban_grid |
|---|---|---|
| Junction entries at signals | **7,536** | **422** |
| green / amber / amber then red / **red throughout** | 6,896 / 570 / 70 / **0** | 373 / 49 / 0 / **0** |
| First stopped boxes: episodes (rows) | **2,490** (48,896) | **129** (2,555) |
| colour at the episode's first row: red / amber / green | 2,301 / 27 / 162 | 118 / 0 / 11 |
| colour at the next row after it: green / amber / red / no next row | 2,406 / 10 / 7 / 67 | 107 / 0 / 0 / 22 |
| Moving off after the light turns green, most common delay | **1.10 s**, 2,194 of 2,406 | **0.10 s**, 96 of 96 that waited through red |

- **No vehicle enters on a red that lasted its whole window**, in either run.
- **Of Midtown's 70 "amber then red"** windows, the vehicle's speeds at the two ends put
  the entry in the amber part in 69. The 70th puts it 0.05 s into the red (988.05 s
  against 988.0 s), within that estimate's error.
- **The 2,194 and the 96 are the drivers' reaction time.** Midtown's car class has
  `reaction_time: 1.0`, urban_grid's 0. A vehicle stopped at a red moves off 1.0 s after
  green, so its first moving row is the one 1.1 s after the change (rows fall at x.1 s);
  in urban_grid, the next row, 0.1 s after.
- **The greens at the first stopped row** are a vehicle that stops first at the line while
  its link is green. By the movement it then takes and the phase's lists: Midtown 107
  permitted left turns (waiting for a gap), 39 protected throughs, 15 protected rights and
  1 with no next link; urban_grid 3, 1 and 7.
- **Midtown's 7 that move on under red** do not enter: 4 creep up to the line from about
  5 m back, and 3 are already in the junction, held there, with their entry in an earlier
  window.

**The check bites.** The same counts with the replay shifted, and with the plans' offsets
applied. *(2026-10-09, round 1: the method, stated; the draft's "early" and "late" were the
wrong way round.)*
- **Shifted by `s` steps:** `S_{k+s}` is read wherever the check reads `S_k`, the index
  clamped to the run's steps. With `s` above 0 the dots switch early, with `s` below 0
  late.
- **The offsets applied:** for a junction with `offset` `o`, `S_{k − round(o/dt)}` is read,
  clamped at `S_0`: the engine's timeline delayed by the plan's offset, as a replay that
  honoured the offsets would have it. (Wrapped round the cycle and not clamped, the count
  is the same 90: not in the probe's `out/shifts.txt`, but found by two reviewers'
  own implementations in rounds 1 and 2. No gate is keyed to it.)

For each: junction entries on red throughout; first stopped boxes whose next row is under
red; the most common delay from green to moving off (with its count).

| Replay | Midtown | urban_grid |
|---|---|---|
| **The engine's (`s` = 0, as proposed)** | **0; 7; 1.10 s (2,194)** | **0; 0; 0.10 s (96)** |
| `s` = −5 (the dots 0.5 s late) | 0; 7; 0.60 s (2,194) | **6**; **96**; 2.60 s (2) |
| `s` = −10 (1 s late) | 0; 7; 0.10 s (2,194) | **6**; **96**; 2.10 s (2) |
| `s` = +5 (0.5 s early) | 0; 7; 1.60 s (2,194) | 0; 0; 0.60 s (96) |
| `s` = +10 (1 s early) | **70**; 7; 2.10 s (2,194) | 0; 0; 1.10 s (96) |
| `s` = +300 (half a cycle) | **7,221**; **2,388**; 1.10 s (22) | **422**; **107**; none |
| The offsets applied (0, 5, 10 s) | (Midtown's are all 0) | **90**; **46**; 0.10 s (53) |

So a mis-wired clock shows as entries on red, as queues moving off under red, or as a
delay off by the shift from the drivers' reaction time; and the offsets as 90 entries on
red. Gate 7 runs this check, with the shifted and offset cases
as its own control.

### 2.8 One colour per approach link, and permitted movements

**The dashboard colours a link, and the engine's vehicles see a link.**
`link_signal_color` takes the junction's current phase and asks whether any movement from
the link is in its `green_movements` **or** `permitted_movements`. If one is, the colour
is the phase's sub-state: green, amber, or red during all-red. If none is, red. The
dashboard draws that string (§2.9), and the vehicles perceive the same string (§2.7). So a
link-level dot shows exactly what the engine's traffic obeys. A colour per movement or per
lane would show what the engine does not use.

**What permitted movements mean for it.** A permitted movement (a left turn that must
yield) counts as green. So a dot can be green while the left turners on its link wait for
a gap: those are the 107 green first stops in Midtown (§2.7). In both fixtures a link's
permitted movement is never its only green one in a phase: of the (approach, phase)
pairs, **0 of 372** in Midtown and **0 of 72** in urban_grid are green through permitted
movements alone. 127 of Midtown's approaches and all 36 of urban_grid's have a permitted
movement, each beside a protected one in the same phase.

**Where the link colour hides a red movement.** In Midtown, **2 of 372** (approach,
phase) pairs show green while one movement from the link is in no list. Both are at one
junction, and each of those two movements is in no phase of its plan, so it is never
green. The dot shows the link's green, which is what the engine's vehicles on that link
see. No approach in either fixture is green in two phases.

### 2.9 Where each dot is

As the dashboard places it (`crates/geometry/src/frame_collect.rs:collect_signal_state`):
- **one per `(node, link)` of `signalized_approach_links()`**, in that order;
- at `s = max(link_length − stop_line_offset, 0)`, the link's own (base) offset, not the
  lane's, on **lane 0's centre**: `LinkGeometryIndex::interpolate(link, s, 0)`, which is
  `Placement::place(link, s, 0)` here (`rules/inputs.md`);
- an approach the engine cannot place is left out, as the dashboard leaves it out.

**On the drawn streets** (vis-002 Phase 6), measured: **188 of 188** of Midtown's dots and
**35 of 36** of urban_grid's lie at the midpoint of their approach's lane-0 stop line's
junction-facing edge, within 0.000000 m. That midpoint is the centroid of the approach's
one `Kind::Stop` marking for `(link, lane 0)` in `Streets::build`, moved 0.2 m (half the
line's 0.4 m) along the link's heading at its end. The 36th is the approach whose lane 0 has a 5 m
stop-line delta (vis-002 §2.17.7): its dot is 5.0000 m ahead of lane 0's own line, on the
edge of the other lanes' line, as on the dashboard.

| | Midtown | urban_grid |
|---|---|---|
| Dots (approach links); none left out | **188** | **36** |
| Approaches per signalised junction | 2 at 62, 3 at 20, 4 at 1 | 4 at 9 |
| Closest two dots | 6.537 m | 17.553 m |
| Pairs of dots that overlap in the orthographic frame: closer than a dot's width there (§2.10), 17.6 m in Midtown and 12.6 m in urban_grid | **107** | **0** |

### 2.10 How a dot looks (OQ-1)

Copied from the front end at `90b39292`, each value with its file and line:
- `drawSignals` (`web/src/canvas/vehicleRenderer.ts` l. 151) draws, for each signal, an arc
  of **`radius = 5`** (l. 158) CSS pixels at the projected point, **filled** with
  `SIGNAL_COLORS[state] ?? SIGNAL_RED` (l. 166; the map is l. 144–148: `green`,
  `amber`, `red`), then **stroked** with `VEHICLE_STROKE` (l. 168) at **`lineWidth = 1`**
  (l. 169). No opacity is set for it.
- The dark palette, the dashboard's default (vis-002 §2.18.4), `web/src/canvas/colors.ts`:
  **`SIGNAL_RED` `#ef4444`** (239, 68, 68) l. 84, **`SIGNAL_AMBER` `#f59e0b`** (245, 158,
  11) l. 85, **`SIGNAL_GREEN` `#22c55e`** (34, 197, 94) l. 86, **`VEHICLE_STROKE`
  `#000000`** l. 121. The light palette (l. 146–148, 183) is not copied.
- A stroke of 1 px centred on a radius-5 path covers 4.5 to 5.5 px. Drawn after the fill,
  it leaves **a fill of radius 4.5 and a black ring out to 5.5**.
- The dashboard draws the signals after the network and under the vehicles: its layer 4
  (`web/src/components/CanvasMapView.tsx` l. 1708–1711) on the 2D canvas, and the
  vehicles on a WebGL canvas above it (l. 2159–2164).

**The proposal: the dashboard's dot at 1080 lines, scaled with the frame,** as the credit
line scales (vis-002 §2.14.5): `s = min(H, 9W/16)/1080`, a fill of radius `4.5·s` and a
black ring to `5.5·s` pixels, in the dark palette's sRGB values exactly, opaque and unlit:

| Frame | `s` | Fill radius | Outline to | Midtown ortho: dot across | urban_grid ortho |
|---|---|---|---|---|---|
| 1280×720 (`view`'s default) | 0.667 | 3.0 px | 3.67 px | 17.6 m | 12.6 m |
| 1920×1080 | 1 | 4.5 px | 5.5 px | 17.6 m (`k` 1.6009) | 12.6 m (`k` 1.1481) |
| 3840×2160 | 2 | 9 px | 11 px | 17.6 m | 12.6 m |

- **Scaled, not fixed in pixels,** so a 4K video and a 1080p one look the same, and `view`
  shows what `render` will draw at the same framing (`view` uses its logical pixels). At
  2160p the dot is the dashboard's on a 2× display.
- **The same in metres across sizes** in the orthographic frame, since `k` falls as the
  frame grows. So the overlaps (§2.9) do not depend on the frame size.
- **Its colours are near the slow boxes'.** Red (239, 68, 68) is near the box colour under
  2 m/s (230, 57, 70), and amber (245, 158, 11) near 2–5 m/s's (244, 132, 45). A queue at a
  red dot is red boxes beside a red dot. The black ring parts them, and on the dashboard
  the vehicles have their own palette. These are the user's decided colours (decision 2),
  shown at gate 14.

The size is the user's to judge (OQ-1). Recommended: as above. *(2026-10-09, user)* OQ-1
is answered (a), as drafted (decision 4), after the probe's 1080p videos with dots.

### 2.11 A fixed size on screen, in both cameras

A dot is a disc in the 3D scene, rebuilt every frame, whose world radius is chosen so that
it covers exactly its pixel radius on screen. Each disc is 32 segments: a fan of 32
triangles for the fill and a ring of 32 quads for the outline, 97 vertices and 96
triangles. A 32-gon is within 0.053 px of the circle at 2160p (11 px, the outer ring).

**Orthographic** (`render` without `--camera`): flat on the road, at `LIFT_M` = **0.03 m**,
radius `4.5·s·k` and `5.5·s·k` metres. Every dot is exactly the copied circle.

**Perspective** (`view`, `render --camera`): a disc **facing the camera**, in the plane of
the image, standing on its stop line. For the pose's axes `a` (forward), `u` (up), `r`
(right), its eye `E`, and `f = (H/2)/tan 22.5°` (`rules/camera.md`):
- `z = (P − E)·a` is the view depth of the dot's point `P` on the road, and `p` the pitch;
- the outer radius in metres is `ρ = R·(z − l·sin p)/(f + R·cos p·sin p)`, with `R = 5.5·s`
  and `l` = 0.03 m;
- the centre is `P` raised by `h = l + ρ·cos p`, and the disc is spanned by `r` and `u`.

Every point of the disc is then at the one view depth `z − h·sin p = f·ρ/R`, so it projects
to **a circle of exactly `R` pixels**, at any pitch, distance or place in the frame. Its
lowest point is `l` above the road, so the road and the markings can never cover it. At
pitch 90 it lies flat at 0.03 m, as in the orthographic frame. Tilted, it rises: its centre
stands about `R·cos²p` pixels above the stop line's point, and its lowest point just below
it. A dot behind the camera (`f·ρ/R ≤ 0.1 m`) is left out.

**Measured** (the synthetic crossing, `p1-synth`, §2.19), each dot drawn alone through the
GPU with MSAA ×4: the pixels of exactly its fill colour or black, and the box they span.

| Frame | Orthographic, `k` 0.05 | Perspective, pitch 30, 64–82 m away | Pitch 90, 72 m | Pitch 25, 474–493 m |
|---|---|---|---|---|
| 1280×720 | 24 (24 fill, 0 black) | 21–23, 5×5 to 5×6 px | 21–23, 5 or 6 each way | 21–23, 5×5 to 6×5 |
| 1920×1080 | 60 (52 fill, 8 black) | 60–64, 10×10 to 10×11 | 61–63, 10×10 | 60–63, 9 to 11 each way |
| 3840×2160 | 308 (232 fill, 76 black) | 307–313, 21×21 | 311–315, 21×21 to 22×21 | 312–318, 21×21 to 22×21 |

- **The same size at 64 m and at 490 m,** tilted or straight down, and the same as the
  orthographic dot. Computed before the GPU, every vertex projects within 7e-13 px of its
  circle, and the lowest is at 0.030000000000 m.
- **Tilted, it stands on its point:** its centre is 4.5–4.7 px above the stop line's point
  at 1080p and pitch 25–30, and within 0.1 px of it at pitch 90.
- **At 720p the outline ring is 0.67 px wide,** so no pixel of it is pure black: it shows
  as a dark rim. At 1080p it is 1 px, at 2160p 2 px.
- **In metres a perspective dot grows with its distance:** `ρ ≈ R·z/f`, which at 1080
  lines (`f` = 1,303.7 px) is 0.3 m at 72 m and 2.1 m at 490 m. A far dot is a disc some
  metres across standing over its stop line, so a box or a wall beside the line, not only
  one in front of it, can cut into it. Nothing is done about that (§2.17); gate 14 is
  worded with it.

### 2.12 Depth, ties and determinism

**Against the scene,** both cameras:

| What | Height | Against a dot |
|---|---|---|
| Road strips, junction surfaces, median noses | 0 | always under it |
| Lane dashes, centre lines; stop lines, connectors, arrows | 0.01; 0.02 m | always under it |
| **Dots** (orthographic; perspective, the lowest point) | **0.03 m** | — |
| Boxes | from 0.05 m | over it, as on the dashboard |
| Buildings, see-through stubs | their heights | over it where they stand |

- **The boxes cover a dot,** as the dashboard's vehicles do. In the orthographic frame the
  first boxes of a queue show over the dot that holds them. In perspective a box in front
  of a standing dot hides part of it, and one behind it is hidden.
- **A building covers a dot,** as it covers a box. 7 of Midtown's 188 dots have their point
  inside a building footprint, where the road passes under it (2.239 % of Midtown's
  centrelines do, vis-002 §2.5); with `--buildings`, the orthographic render shows those
  dots under the roof, as it shows the boxes there. In a flight, a dot behind a building is
  hidden, and see-through's stubs (vis-002 §2.15) clear the way to the look-at point as
  they do for the boxes. No other of Midtown's dots has more than a sliver under a
  footprint (9 touch one). *(2026-10-09, user)* OQ-4 is answered as drafted: in the
  orthographic render a dot under a roof stays hidden, as the boxes are (decision 5).
- **No depth tie across entities.** The dots are one mesh, one entity. Nothing else lies at
  0.03 m, and in perspective a dot lies in a plane facing the camera, which no other
  surface shares. (vis-001 §2.12.1: Bevy's binned pass keeps no order between entities, so a tie
  between two would be decided differently between frames.)
- **Ties between dots are inside the one mesh.** Dots overlap: 107 pairs in Midtown's
  orthographic frame (§2.9). In the orthographic frame and at pitch 90 they are coplanar,
  so their overlaps tie. The opaque pass tests depth with `GreaterEqual` (vis-001
  §2.12.1), and inside one draw the GPU keeps primitive order, so the later dot wins:
  the order of `signalized_approach_links()`, which is the dashboard's drawing order (the
  probe's synthetic crossing at `k` 3, where all four dots overlap, shows the last one's
  fill over the others). In perspective, a nearer dot covers a farther one.
- **Deterministic.** The mesh is a function of `t`, the pose and the frame size: built on
  the CPU in `f64`, cast to `f32`, in approach order, every frame. The timeline is a
  function of the network and `dt`; the engine's `HashMap`s inside `SignalState` are
  stepped junction by junction, independently, so their order changes no state, and the
  approach list is sorted. No order here comes from walking a `HashMap`.
- **Measured** (§2.19): every render with dots, twice, gives equal frames: Phase 6's seven
  cases, 8700 of 8700 and 1800 of 1800 (gate 10). Each differs from Phase 6's render of the
  same case in every frame but urban_grid's flight, where 548 of 8,700 are equal: its follow
  at `height_m` 60 between two crossings (1:31.7–1:32.7 and 2:05.0–2:22.2), with no dot in
  the frame.

### 2.13 The flag, the key, and their errors

- **On by default** (decision 2) in `render` and `view`, in both cameras, on every network
  with a signalised junction. A network with none draws nothing and reads nothing more.
- **`--no-signals`** on `render` and `view`. The name follows `--no-streets` and
  `--no-see-through`.
  - **On `render`:** nothing about signals is read or built and nothing is spawned, so it
    gives today's frames, byte for byte (gates 1 and 2).
  - **On `view`:** the dots start hidden. *(2026-10-09, round 1: the draft said nothing is
    read or built under the flag in `view` too, which left `L` nothing to show.)* The
    timeline is still built at launch, so that `L` can show them. **If it cannot be
    built** (the run is not micro, or its clock is not readable: the two errors below),
    `view --no-signals` opens as today, with no error and no timeline, and `L` does
    nothing. Without the flag, the same failure is the error below, before the window.
- **A network with a signal** is one where `signalized_approach_links()` is not empty: a
  junction with a `signal_plan` and a movement. `control: signal` without a plan is not
  one (`tests/look.rs`'s crossing), as in the engine, whose `SignalState::new` takes only
  junctions with a plan.
- **`L`** in `view` (signal *lights*; by position, `KeyCode::KeyL`, unbound today) hides
  and shows the dots, as `M` does the streets. `view --no-signals` starts with them hidden,
  and `L` shows them. `S`, the obvious letter, is `WASD`'s pan. The keyframe line does not
  carry it. In the frame order it comes after `M` (`rules/view.md`). On a network with no
  signal, or with no timeline, `L` flips its flag and nothing is drawn. *(2026-10-09,
  user)* Confirmed: the key is `L` (decision 6).
- **`--signals` is not a flag:** clap rejects it, exit 2, as it rejects `--streets`.
- **The library keeps today's default.** `Job::prepare*` build every job with signals off,
  as with streets, so every test that builds a job draws as today. `src/main.rs` turns them
  on with `Job::set_signals(true)?` unless `--no-signals` is given.

**What signals read, and their errors.** With signals on and a signalised junction in the
network, after `run::load`'s checks and the files' (`rules/inputs.md`), before the first
frame, the run's row in `results.db` (the completed row for the scenario and seed, the one
`require_completed_run` accepts; read-only and `immutable=1`) must give, **in this
order**:
1. **`mode` = `micro`** (the `runs` column). Otherwise: `error: <results.db>: the run is
   <mode>; signals are replayed only for micro runs (vis-003 OQ-3); --no-signals renders
   without signals`, with `<mode>` the column's text, or `not recorded` for a NULL.
2. **A clock:** `resolved_config` as a JSON object with a finite `sim_params.timestep`
   above 0, and a finite `duration` (the `runs` column) above 0. Otherwise:
   `error: <results.db>: the run's timestep is not readable (runs.resolved_config): <why>;
   --no-signals renders without signals`, where `<why>` names what failed (the query, a
   NULL, the JSON, `sim_params.timestep`, or `runs.duration`).

*(2026-10-09, round 1)* The mode comes first because a meso run's `resolved_config` has no
`timestep` (§2.4): checked the other way round, every real meso run would get the second
message and never the first. `inputs::run_clock` raises both, so both are tested without
a `Job` (gate 5), and through the CLI on an edited copy of a fixture (gate 15).

Each is one line, exit 1, no output file and no window, as every input error
(vis-001 §2.4). On a network with no signal neither is read. Under `render --no-signals`
neither is read, and under `view --no-signals` a failure is not an error (above). So no
run that renders or opens today fails tomorrow with that flag.

**The credit line stays as it is.** It is UI drawn over the 3D image, so where a dot lies
under the bottom-right corner, the credit line covers it, as it covers the boxes. Nothing
about the line changes.

### 2.14 The scripts that read references

Every CLI render in `scripts/` that is compared with a `ref-pin90b39292` file today passes
`--no-streets` (vis-002 §2.17.12). Each also gains **`--no-signals`**, and nothing else:
- `scripts/gates.sh`: `default`, `default2`, `camera`, `camera2` (4);
- `scripts/gates-ties.sh` gate 2: `ortho-city`, `ortho-roads` (2);
- `scripts/gates-see-through.sh` gate 2's seven and gate 7's five (12);
- `scripts/gates-parts.sh` gate 2's five and gate 9's six (11).

These 29 are every `render` line carrying `--no-streets` now. A thirtieth line with the
flag, `gates-streets.sh`'s `view --no-streets --bench 1`, is not a render and is not
edited. The other renders in scripts compare only with each other, or count frames that
differ: `gates-ties.sh` gate 9, `gates-city.sh` gate 13, `gates-credit.sh` gate 14,
`gates-streets.sh` gates 10 and 11 and `gates-look.sh` gate 10; and `gates.sh`'s
`explicit` render is checked by `ffprobe` and its stderr only. They are not edited, now
draw dots, and must still give equal pairs and still differ where they differed (gate 3).
The header comments of `gates-streets.sh` and `gates-look.sh`, which say the reference
scripts run "with --no-streets", are those phases' records and stay as they are. `gates-look.sh` writes into
`scratch/out/look/`, over Phase 6's renders, so Phase 1 copies those first (gate 2).
The `view` runs in scripts check only an exit and a JSON line.

### 2.15 A run whose signals were forced (OQ-2)

`SignalState::force_phase` sets a junction's phase and freezes it; the Python API's
`set_signal` (`crates/python/src/simulation.rs:set_signal`) calls it, and the gym
environment drives it. Such a run's states are not the plan's, and a replay would draw the
plan's.

**How the visualizer can tell: at the pin, it cannot, from the files,** and it does not
need to yet:
- **nothing records it.** `runs.resolved_config` holds only the scenario and four sim
  parameters (§2.4); the FCD has no signal column;
- **no forced run writes what the visualizer reads.** `results.db` and the FCD are written
  by `crates/output/src/runner.rs:run_to_results_db` (and its meso twin), which never calls
  `force_phase`. The Python bindings write neither (no `ResultsDbWriter` or FCD writer in
  `crates/python/src`). The dashboard server's sim loop (`crates/server/src/server.rs`) is
  a third writer of both: it never calls `force_phase` either, and its runs are micro with
  a `timestep`. So every run the visualizer can open is a fixed-time run, and the replay
  is the engine's.

**What would show it.** A forced run's FCD would put junction entries under a replayed red
that lasted their whole window: §2.7's check counts them, and the engine's own fixed-time
runs give 0 of 7,536 and 0 of 422. Phase 1 runs that check as a gate only. It is not a
check on every render: `render`'s stderr carries progress and one error line, nothing
else (vis-001 §2.4), so it could only refuse, and a refusal on an estimate is worse than
the engine's record.

**The engine request (OQ-2):** record the signal states in the run's outputs. Recommended:
a `signal_events` table in `results.db` (time, node, phase index, sub-state), written
whenever a junction's sub-state changes, forced or not; at the least, a field in
`runs.resolved_config` that says a junction was controlled from outside. For the harness
contract (vis-001 §2.7 item 6 and its OQ-4: a harness run's FCD). With states recorded,
the visualizer would draw the recorded ones and keep the replay for older runs.

### 2.16 A pin move

The look and the states change only when the user moves the pin (vis-001 §2.2.2), in a
commit of its own that re-runs the gates. Then:
- **What follows by itself:** everything from the engine's code. The states, by
  `SignalState`; which approaches get a dot, by `signalized_approach_links`; each colour,
  by `link_signal_color`; each dot's place, by `NetworkData` and `LinkGeometryIndex`. Gates
  5–8 predict their numbers again, and the fixtures' changed runs change gate 7's.
- **What does not:** the values copied from the front end and the rule copied from
  `collect_signal_state`. A pin-move phase runs `git diff <old pin> <new pin> --
  web/src/canvas/vehicleRenderer.ts web/src/canvas/colors.ts
  crates/geometry/src/frame_collect.rs` in the engine (read-only) and reads it for:
  - in `vehicleRenderer.ts`, `drawSignals`: the radius, the fill, the stroke and its
    width, and `SIGNAL_COLORS`;
  - in `colors.ts`, the dark palette's `SIGNAL_RED`, `SIGNAL_AMBER`, `SIGNAL_GREEN` and
    `VEHICLE_STROKE`;
  - in `frame_collect.rs`, `collect_signal_state`: which approaches, which `s`, which lane.

  Each change found is copied with its new line, and gates 5, 8 and 9 are predicted again.
  With none, the copy stands, and the record says so.
- *(2026-10-09, round 2)* **Nor does the shape of the run's row:** what `run_clock` reads
  (`runs.mode`, `runs.duration`, `resolved_config`'s `sim_params.timestep`) and the two
  rows written out in gate 5 and gate 15, a micro run's and a meso run's. The same phase
  adds `crates/output/src/runner.rs` to that diff and reads `run_to_results_db`'s and
  `run_meso`'s `resolved_json`; a change there changes `run_clock` and those two gates'
  rows. The new module names `90b39292`
  beside each copied value, so the diff has a place to land.

At the engine's HEAD today, those three files are not read here: the pin is what is drawn.

### 2.17 What it does not do

- **No colour on the stop lines.** vis-002 kept its stop lines per `(link, lane)` so that
  "the data item" could colour them (vis-002 §2.17.7, §2.17.13, §2.18.2, §2.18.6,
  §2.18.13). Decision 2 draws
  dots instead, as the dashboard does, and the stop lines stay white. The quads stay per
  lane; nothing uses that now. vis-002 gets a dated note saying so in Phase 1's close-out.
- **No per-lane, per-movement or protected/permitted distinction** (§2.8).
- **No dot at an unsignalised junction,** where the engine has none either; vis-002's 38
  stop lines at unsignalised approaches stay as they are.
- **No fade.** A dot keeps its pixel size at any distance, so it never thins below a pixel.
- **No cap on a far dot's size in metres** (§2.11): it is as many pixels as a near one.
- **No dot under see-through's control:** stubs uncover what they uncover, no more.
- **No pick or follow** of a dot, and no change to pick, pan or zoom.
- **Nothing in the keyframe file.** A flight's video shows dots unless `--no-signals`.

### 2.18 Build cost

**0 packages,** no Bevy feature, no `Cargo.toml` change. `SignalState` is in
`assimilator-core`, a dependency since vis-001; `rusqlite` and `serde_json` read
`resolved_config`; the disc mesh is per-vertex colour on one unlit material, as the
markings are (vis-002 §2.17.11). No engine request for the fixtures (decision 3); OQ-2 is
for runs that are not fixed-time. One new module, `src/signals.rs`, with no Bevy types.

### 2.19 Measured while drafting (2026-10-09)

A throwaway probe in gitignored `scratch/vis003p1-probe/`:
- **`repo/`** is `git archive` of `origin/main` (`cbd7570`) with:
  - a draft `src/signals.rs` (the timeline, the run clock, the dots' mesh);
  - `spawn_dots` and `set_dots` in `src/draw.rs`, `Renderer::spawn_dots`/`set_dots`,
    `Job::set_signals` and a hook in `Job::render_at`, the dots in `view`, all behind an
    environment variable (`P1=1`);
  - `src/bin/p1-probe.rs` (each fixture: the plans, the approaches and dots, the timeline
    and its cost, the link-level counts, the FCD check, the shifts, the roofs) and
    `src/bin/p1-synth.rs` (the synthetic crossing of §2.11 and gates 5 and 9).

  Built in its own `CARGO_TARGET_DIR`, sccache off: 9 m 23 s at a load of 30–128 from
  other sessions. Nothing in `src/`, `tests/` or `scripts/` was touched.
- **The engine** read from cargo's checkout at `90b3929`, read-only.
- **`synth/network.yaml`:** vis-002 gate 5's crossing with a two-phase plan (gate 5).
- **`run.sh`, `runs/`:** the CLI renders through the probe's `assimilator-video`:
  - **off** (no `P1`): Midtown's ortho and city flight with buildings, 1800 of 1800 equal
    to Phase 6's (`scratch/out/look/`, read only); Midtown's ortho with `--no-streets`,
    1800 of 1800 equal to `ref-pin90b39292-ortho-roads`;
  - **on**, twice each: Phase 6's seven cases (gate 10's list);
  - `view --bench 20` on Midtown with buildings, with and without dots.
- **`out/`**: `stats-midtown.txt`, `stats-urban_grid.txt`, `shifts.txt`, `synth*.txt`, the
  synthetic frames; **`frames/`**: stills for gate 14's wording.

**Where the probe differs from the scope:** it reads the run's clock from the first
completed `runs` row rather than the scenario's and seed's; its `run_clock` is in
`src/signals.rs`, and its `Job::set_signals` takes the `results.db` path; it has no
`--no-signals`, no `L` and no mode check; its dots are always on in `view`.

*(2026-10-09, round 1)* Measured for gate 15, with `origin/main`'s binary: a copy of
`scratch/urban_grid` whose `results.db` was edited with `sqlite3` (`PRAGMA journal_mode =
DELETE`, then `mode` set to `meso` and `resolved_config` to the meso runner's shape) reads
back so through `immutable=1`, and `render --from 10 --to 20 --speedup 1` on it exits 0
with `1920,1080,30/1,300`: today's `render` does not read `mode`. `render --signals` is
rejected by clap today.

### 2.20 The HUD (Phase 2)

Drafted 2026-10-19, the first item of §2.3's roadmap. The observable is the video itself:
every frame of a render carries, in its top-left corner, the run's clock, how many
vehicles are on the road and their mean speed.

```
# On by default, on every network, in both cameras.
assimilator-video render --project <project> --scenario baseline --seed 42 --out run.mp4

# Today's frames (Phase 1's), byte for byte.
assimilator-video render … --no-hud

# H hides and shows it; --no-hud starts with it hidden.
assimilator-video view --project <project> --scenario baseline --seed 42
```

#### 2.20.1 The user's decisions, and where it goes (decision, recorded)

Decided by the user, 2026-10-09 (§2.2, decision 1), before Phase 1 was drafted, and not
reopened here:
1. **In the top-left corner:** the sim clock, the cars on the road and their mean speed,
   from the run.
2. **On by default, with a flag to turn it off.**

The rest of this section is the draft's proposal: what each number means (§2.20.3–
§2.20.6), the units (§2.20.7, OQ-5), the text and its look (§2.20.8–§2.20.9, OQ-6), `view`
(§2.20.10), the flag, the key and the scripts (§2.20.11–§2.20.12), and the cost (§2.20.14).

Where it goes, by the methodology's §6.1:
- **Step 0:** a decision changes. A default changes: every render draws the HUD.
- **Step 1:** nothing shipped is removed or contradicted. `--no-hud` gives Phase 1's
  frames byte for byte (gates 1 and 2). No run that renders today fails tomorrow: the HUD
  reads nothing the render does not already read (§2.20.11).
- **Step 2:** vis-003 owns the subject. Decision 1 named it its Phase 2, and §2.3 holds it
  as roadmap. So it is a phase appended here, `reviewed: null`; `status` stays `accepted`.

#### 2.20.2 What the dashboard shows, and what the engine counts

Read at the pin (`90b39292`, cargo's checkout, read-only), 2026-10-19:
- **The dashboard's status bar** (`web/src/components/UnifiedStatusBar.tsx`, `SimStats`,
  l. 27–28): `Time: {simTime.toFixed(1)}s` and `Vehicles: {count}`. `simTime` and the count
  are the frame's `time` and `vehicle_count` (`web/src/hooks/simStream.ts`, l. 111–112).
  The count is `vehicles.len()` (`crates/geometry/src/frame_collect.rs:collect_frame`,
  l. 522), and a vehicle enters `vehicles` only when it has a position to draw (l. 187).
- **It shows no mean speed.** The bar's `Speed: {simStatus.speedFactor}x` (l. 95) is the
  playback speed.
- **Its speeds are m/s and km/h, never mph.** *(Not as the brief had it.)* The inspection
  panel shows a vehicle's speed in m/s to two decimals, with km/h in its tooltip
  (`web/src/components/InspectionPanel.tsx`, `fmtSpeed` and `fmtSpeedTip`, l. 191–197;
  used at l. 1205). The editor and the vehicle classes show km/h (`Speed (km/h)`,
  `Max speed (km/h)`). No file under `web/src` names mph or miles.
- **The engine's own mean speed** is the plain mean of every vehicle's speed, stopped ones
  included:
  - the run's progress tick: `avg_speed`, "mean speed of this step's vehicle snapshots
    (m/s)" (`crates/output/src/runner.rs:ProgressTick`, computed in
    `run_to_results_db_with_sink`, which `run_to_results_db` calls);
  - the run's KPI: `speed_sum` over every snapshot of every step, over their count
    (`crates/core/src/simulation.rs:Simulation::step_execute`, `Simulation::finalize`).
- **The engine's count** is the progress tick's `active`, `sim.vehicle_count()`
  (`Simulation::vehicle_count`, the world's entities). It is written to no file the
  visualizer reads.

#### 2.20.3 What the run gives: the FCD, one snapshot a second

The HUD reads only the FCD, which every render reads already (vis-001 §2.1,
`rules/inputs.md`). Measured on both fixtures (seed 42, engine `90b39292`):

| | Midtown | urban_grid |
|---|---|---|
| Snapshots; first – last time | 1,199; 1.1 – 1,199.1 s | 291; 9.1 – 299.1 s |
| Gap between successive snapshots | 1.0 s (±1e-12) at every one | 1.0 s (±1e-12) at every one |
| Vehicles; rows | 985; 202,241 | 198; 21,556 |
| A vehicle missing a row between its first and its last | 0 | 0 |
| `vehicle_class` | `car`, every row | `car`, every row |
| Rows in a snapshot | 1 – 216 | 6 – 109 |
| `speed` | 0 – 16.67 m/s | 0 – 13.89 m/s |

- **A snapshot is every vehicle of a step.** The engine collects the FCD's rows with one
  query over every vehicle entity after the step
  (`crates/core/src/simulation.rs:Simulation::step_execute`), and the runner writes them
  whenever the FCD's interval is due (`run_to_results_db`). So a snapshot's rows are the
  vehicles the engine counts at that step (§2.20.2).
- **A step with no vehicle writes no row.** The writer writes the step's rows
  (`crates/output/src/results_db.rs:ResultsDbWriter::on_step`), through the FCD writer's
  `FcdParquetWriter::write_batch`, which returns early with none. So an
  empty network is a time missing from the FCD, not a snapshot of 0. Neither fixture has
  one (the gaps above), but a run whose demand stops and starts would.
- **The FCD's interval is not recorded.** `runs.resolved_config` carries the timestep,
  not the FCD's interval (§2.4), and the runner can override it (`fcd_interval`). So the
  HUD cannot tell a missing time from the interval alone.

#### 2.20.4 "On the road": the snapshot's rows

Three candidates for the count at a frame's sim time `t`:
- **(a) The snapshot's rows:** the snapshot vis-001 shows at `t`, the latest with
  `time ≤ t + 1e-6` (`Fcd::snapshot_at`, `rules/inputs.md`), counted. Held between rows.
- **(b) The boxes drawn at `t`:** every vehicle with `t_first − 1e-6 ≤ t ≤ t_last + 1e-6`
  (`rules/motion.md`). In perspective, a box bit-identical to a later one is not drawn
  (vis-001 Phase 6), so the boxes *seen* can be fewer still. At the pin no frame of the three
  windows below has one: vis-001's 11,508 stacked pair-frames were measured before the pin
  moved to `90b39292`.
- **(c) The engine's count** (the dashboard's `Vehicles`): in no file the visualizer
  reads. At a snapshot's time it is (a)'s rows (§2.20.3). The dashboard's frame is one
  step apart from the FCD's (§2.6).

**Measured, every frame of three windows** (the probe's `out/hud-*.txt`,
`out/boxes.txt`):

| Window (frames) | Changes of (a) | Changes of (b) | Frames where (b) ≠ (a); (b) − (a) | Frames with a box bit-identical to another |
|---|---|---|---|---|
| Midtown 300–360 s at 1× (1,800) | 39 | 64 | 664; −4 to 0 | 0 |
| Midtown, the whole run at the default speedup (9,000) | 763 | 768 | 3,867; −6 to 0 | 0 |
| urban_grid, the whole run at 1× (8,700) | 84 | 87 | 1,595; −6 to 0 | 0 |

**Why (b) is lower between rows, and flickers.** A vehicle's last row is a snapshot's.
At that snapshot's time it is drawn and counted, and from the next frame it is gone:
(b) drops by the vehicles that leave at that snapshot (Midtown: at 516 of its 1,198
snapshots before the last, up to 6 at once), and rises again only at the next snapshot,
by those that enter. At 1× a frame lands on a snapshot's time once a second, so (b) shows
the higher number for one frame in 30. Midtown's window counts 64 changes of (b) to (a)'s
39 for that reason.

**The proposal: (a), the snapshot's rows,** with one rule for an empty network:
- **at `t`, the rows of the snapshot vis-001 shows at `t`** (`Fcd::snapshot_at`), held
  between rows. It is the same `k` rule as Phase 1's dots, which show `S_k` for the FCD's
  `T_k ≤ t + 1e-6` (§2.6): the count and the dots at a frame are of one step;
- **0 when no box is drawn at `t`.** That is an empty network: before the first row, after
  the last, and a time the FCD is missing because the step had no vehicle (§2.20.3). The
  vehicles of the snapshot before it all have their last row there, so none is drawn
  after it. Measured: the rule never fires inside the fixtures' windows (0 frames of the
  three above), and gate 5 makes it fire.

It is the engine's own count at that step, it changes once per row, and it never flickers.
What it does not show: between rows, a vehicle that has left the road is still counted,
for up to one FCD interval (1 s here). (b)'s "−4 to 0" is that.

**Sampled** at the times shown (the probe's `out/hud-midtown.txt`, `out/hud-urban_grid.txt`):

| `t` (s) | Snapshot | (a) rows | (b) drawn |
|---|---|---|---|
| Midtown 60 | 59.1 | 51 | 51 |
| Midtown 180 | 179.1 | 122 | 122 |
| Midtown 300 | 299.1 | 184 | 184 |
| Midtown 300.5 | 300.1 | 185 | 185 |
| Midtown 330 | 329.1 | 201 | 201 |
| Midtown 359.97 | 359.1 | 203 | **202** |
| Midtown 600 | 599.1 | 159 | 159 |
| Midtown 1,199.1 | 1,199.1 | 158 | 158 |
| urban_grid 9.1 | 9.1 | 6 | 6 |
| urban_grid 60 | 59.1 | 36 | 36 |
| urban_grid 120 | 119.1 | 76 | **75** |
| urban_grid 240 | 239.1 | 98 | **97** |
| urban_grid 299.1 | 299.1 | 107 | 107 |

**"Cars" or "vehicles".** Every row of both fixtures is a `car`, so the two words give the
same number here. The FCD has a row for every vehicle class, a bus or a lorry included,
and the dashboard says "Vehicles". The draft counts every row and says "vehicles" (OQ-6's
(d)); a per-class count is not proposed (§2.20.13).

#### 2.20.5 Mean speed: of the same rows, every one, held

Over which vehicles, measured at every snapshot (the probe's `out/mean-defs.txt`):

| | Midtown | urban_grid |
|---|---|---|
| **(i) every row's `speed`, averaged (the engine's `avg_speed`)** | **11.2 – 40.2 km/h** | **19.3 – 50.0 km/h** |
| (ii) moving rows only (`speed ≥ 0.1`) | 15.0 – 40.2 km/h; 8.7 km/h above (i) on average | 21.3 – 50.0 km/h; 6.0 above |
| (iii) the harmonic mean of the moving rows (space-mean speed) | 2.0 – 40.2 km/h | 3.2 – 50.0 km/h |
| Stopped rows (`speed < 0.1`), share of a snapshot | 28 % on average, up to 57 % | 16 %, up to 43 % |

- **(i)** falls as queues form: Midtown's 300–360 s window runs 15 – 25 km/h at the
  dashboard's rounding, while 28 % of its rows on average stand at a red light.
- **(ii)** hides them. It stays near 30 km/h whatever the queues do (Midtown 28.9 – 35.4 at
  the sampled times), which says how fast the moving cars go and nothing about the jam.
- **(iii)** is dominated by the slowest moving car: urban_grid's 3.5 km/h at 90.1 s is two
  cars creeping at 0.12 m/s among its 56 moving rows.

**Between rows, two ways:**
- **held:** the mean of the snapshot's rows, the rows the count counts;
- **interpolated:** the mean, over the boxes drawn, of each box's speed linear between its
  rows (the speed that colours the box, `rules/motion.md`).

| Window | Changes, held | Changes, interpolated (whole km/h) |
|---|---|---|
| Midtown 300–360 s at 1× | 32 | 43 |
| Midtown, the whole run | 674 | 877 |
| urban_grid, the whole run | 140 | 195 |

**The proposal: (i), held:** the mean of `speed` over the rows the count counts, so the
two numbers always describe the same vehicles at the same step, the engine's own
definition. When the count is 0 there is no speed, and the HUD shows `–` (an en dash).
Interpolated, the number changes a third more often and describes the boxes drawn, a set
that is not the count's (§2.20.4).

**Sampled** (km/h, rounded halves up; the probe's `out/mean-defs.txt`):

| Snapshot | Midtown (i) | (ii) | (iii) | | urban_grid snapshot | (i) | (ii) | (iii) |
|---|---|---|---|---|---|---|---|---|
| 59.1 | **22** | 33 | 23 | | 9.1 | **50** | 50 | 50 |
| 179.1 | **16** | 35 | 20 | | 59.1 | **29** | 41 | 34 |
| 299.1 | **17** | 31 | 13 | | 90.1 | **30** | 32 | 4 |
| 329.1 | **20** | 31 | 16 | | 179.1 | **22** | 36 | 10 |
| 599.1 | **17** | 29 | 12 | | 299.1 | **20** | 35 | 10 |

#### 2.20.6 The clock: the frame's sim time, in whole seconds

**Which time.** The frame's sim time `t`: the time its boxes are placed at and its dots
are stepped to (§2.6), from the run's 0, as the FCD, `view`'s readout, `--from`/`--to` and
the dashboard's `Time:` all count it.
- **Not from the end of the warm-up.** `warm_up` only decides which steps the KPIs count
  (`crates/core/src/simulation.rs:Simulation::accumulate_delay`); the FCD is written
  through it (Midtown's from 1.1 s, its `warm_up` 180). A clock from the warm-up would read
  −3:00 at Midtown's first row, and would need `runs.resolved_config` read, with its
  errors (§2.13), on a render without signals. Not proposed.
- **Not the snapshot's time.** The snapshot's `T_k` would make the clock stand still for a
  second and jump, and at a speedup above 1 skip seconds unevenly.

**With Phase 1's alignment.** The numbers come from the snapshot at `T_k ≤ t + 1e-6`, the
same `k` as the dots (§2.20.4). The clock shows `t` floored. With Midtown's rows at
`x.1` s, the clock turns over 0.1 s before the numbers do: at `t` 300.0 it reads `5:00`
with the 299.1 s row's numbers, and at 300.1 the 300.1 s row's.

**The format: `m:ss`, floored, `h:mm:ss` from an hour.** `⌊t + 1e-6⌋` seconds (the `1e-6`
is vis-001's snapshot tolerance), as `m:ss` under an hour (`5:00`, `19:59`) and `h:mm:ss`
from one (`1:00:00`). Before 0 (a negative `--from`) it reads `0:00`.

| Window | Changes, `m:ss` | Changes, tenths (`300.1 s`, the dashboard's `toFixed(1)`) |
|---|---|---|
| Midtown 300–360 s at 1× (60 s of video) | 59 | 600 |
| Midtown, the whole run (300 s of video, 3.99×) | 1,197 | 8,999 |
| urban_grid, the whole run (290 s of video, 1×) | 290 | 2,900 |

*(2026-10-19, round 1)* The tenths column is rounded, as `toFixed(1)` is; the draft's 599
and 2,899 were floored.

Tenths change ten times a second of video at 1×, and every frame at Midtown's default
speedup: a blur in a talk. The dashboard is a tool watched closely, and the video is
watched from a seat.

#### 2.20.7 Units (OQ-5)

The mean speed in km/h is `3.6 · v`; in mph, `3.6 · v / 1.609344`. Both rounded to a whole
unit, halves up. Midtown is in Manhattan (`map_origin` 40.765 N, 73.978 W):

| Snapshot | Midtown km/h | mph | | urban_grid km/h | mph |
|---|---|---|---|---|---|
| 59.1 / 59.1 | 22 | 14 | | 29 | 18 |
| 299.1 / 179.1 | 17 | 11 | | 22 | 14 |
| 599.1 / 299.1 | 17 | 10 | | 20 | 12 |

- **km/h** is the dashboard's and the engine's other displays' (§2.20.2). urban_grid's
  every `speed_limit` is 13.89 m/s, 50 km/h.
- **mph** is what a US audience reads, and Midtown is a US city. Its network is mostly
  limited in mph: 290 of its 533 `speed_limit` entries are 11.176 m/s, **25 mph** (New
  York City's default), and 2 are 10 mph; the rest are round in km/h (130 at 50, 64 at
  60, 43 at 30, 4 at 40). Whole mph are coarser: Midtown's 300–360 s window spans 9 – 15
  mph against 15 – 25 km/h.
- **A flag** (`--speed-unit kmh|mph`) would let each video choose. It is one more flag in
  the harness contract, and a default still has to be chosen.

The units are the user's (OQ-5). Recommended: km/h, fixed, as the dashboard. *(2026-10-19,
user)* OQ-5 is answered (a), km/h, fixed, as recommended, after the probe's stills.

#### 2.20.8 The text

**One line, `5:00 · 184 vehicles · 17 km/h`:** the clock (§2.20.6), the count with
"vehicle" or "vehicles" (one is singular), and the mean speed with its unit (`–` with
none), joined by ` · ` as the credit line joins its items (vis-002 §2.14.2). No label
words: the clock's colon and the units say what each number is.

The alternative shown in the stills (OQ-6's (c)): three lines, the clock above the count
above the speed. *(2026-10-19, user)* OQ-6's (c) and (d) are answered as drafted: one
line, and "vehicles".

**Tabular figures.** Fira Sans Medium's digits are proportional: their advances run from
450 (`1`) to 573 (`0`) units of 1,000 (the font's `hmtx`, the probe's
`out/font-widths.txt`). With them, every change of a digit would shift the words after it
by up to an eighth of an em (123 units), about 3.7 px at 1080 lines. The font has the
OpenType feature `tnum` (its `GSUB` lists it, read from the TTF in review round 1; the
probe's `font_widths.py` reads only the advances), and Bevy 0.19.1 sets features per text
(`TextFont::font_features`, `FontFeatureTag::TABULAR_FIGURES`). **The HUD sets `tnum`.**
Measured (the probe's `out/widths.txt`): at 1080 lines the fill is **387 px** wide for `1:11 · 111 vehicles · 11 km/h`,
`8:88 · 888 vehicles · 88 km/h`, `0:00 · 000 vehicles · 00 km/h` and `4:07 · 205 vehicles ·
23 km/h` alike (404 px with a fifth digit, `19:59 · 216 vehicles · 40 km/h`). Without
`tnum` the same four are 358 to 390 px, a 32 px swing.

#### 2.20.9 How it looks (OQ-6)

**Drawn as the credit line is drawn** (vis-002 §2.14.5): Bevy UI text in the headless
render, a root `Node` with `UiTargetCamera(<the image camera>)`, Fira Sans Medium from the
bytes already embedded (`draw::CREDIT_FONT`), sized to the frame. 0 packages; no new file.

**Readable over any background.** The top-left corner of a frame holds roads, boxes,
dots, roofs, the dark background or, in a flight, sky. Three treatments, each drawn by the
probe at 1080 lines over Midtown's orthographic frame, a frame of the city flight, and
urban_grid (the stills, §2.20.15):
- **(a) The credit line's outline:** a light fill, sRGB (240, 240, 240), with eight black
  copies `o = max(1, round(S/12))` px around it. Readable on a white roof and on the dark
  ground alike, and it matches the credit line in the opposite corner.
- **(b) A panel:** the same fill on a black panel at 55 % opacity, `S/2` px of padding left
  and right and `S/4` above and below, its corners rounded `0.3·S`. Reads as a caption box;
  it covers more of the frame than (a), 417 × 52 px at 1080 lines against the outline's 393 × 42 (21,630 pixels
  changed against 8,801, Midtown at 330 s).
- **(c) A shadow:** the same fill with Bevy's `TextShadow`, one black copy `o` px down and
  right. Its top and left edges have no border, which vis-002 §2.14.5 rejected for the
  credit over light roofs.

**Its size:** `S = round(min(H, 9W/16)/36)` px, the credit's rule at a larger ratio, with
`m = S` from the top and left edges:

| Frame | 1/54 (the credit's size) | **1/36** | 1/27 |
|---|---|---|---|
| 1280×720 (`view`'s default) | 13 px | **20 px** | 27 px |
| 1920×1080 | 20 px | **30 px** | 40 px |
| 3840×2160 | 40 px | **60 px** | 80 px |

The `9W/16` term keeps a portrait frame's line inside its width, as for the credit. The
HUD's line at 1/36 is 387 to 404 px wide at 1080 lines.

**The draft's proposal, to be chosen from the stills (OQ-6):** (a), the outline, at 1/36,
one line, "vehicles". *(2026-10-19, user)* OQ-6 is answered as recommended: the outline,
at 1/36 (30 px at 1080 lines), one line, "vehicles", after the probe's stills.

**Where it sits.** Top-left, `m` from both edges, over everything else in the image, in
both cameras: it is UI over the 3D image, so a flight does not move it.
- **The credit line** is bottom-right, on imported networks only (`rules/credit.md`). The
  two boxes are 965 px apart (the HUD's box ends at row 69, the credit's begins at row
  1,034, and they share no column: the HUD's ends at 420, the credit's begins at 699) at
  1080 lines on Midtown, and do not meet at any 16:9 size or in a portrait frame.
- **What it covers.** In Midtown's orthographic frame (`k` 1.6009) the HUD's box lies
  over no dot (0 of 188) and no box in any of the 300–360 s window's 1,800 frames: the
  network does not reach that corner; in urban_grid's (`k` 1.1481), no dot (0 of 36) and no box in 8,700 frames. In a flight, whatever
  the pose puts there.
- **Nothing outside its box changes:** the probe's frame with the HUD against the same
  frame without it, 8,801 pixels change in Midtown's orthographic frame at 330 s, 8,814 in the city
  flight's and 8,384 in urban_grid's at 150 s, and every one is inside the box (the panel
  21,630, 21,630 and 20,752; the shadow 6,015, 6,002 and 5,755; all inside).

**Deterministic.** The text is set before each frame, only when its string differs from
the last frame's, and the image is the UI pass's over the scene, as the credit line's.
Bevy lays out glyphs into a font atlas the first time it meets them, so the HUD lays out
every glyph it can draw (`0123456789:·– vehiclsmp/hk`) once, at spawn, before the settle
frames. *(2026-10-19, round 1)* The string is the probe's less its `\n` (there for the
three-line layout; it draws no glyph), with which gate 7's counts were measured: every
character `text` writes, and the `p` of mph, which km/h (OQ-5) leaves unused. It is kept
as measured. Measured (§2.20.15): Phase 1's seven cases rendered twice with the HUD,
every pair equal (8700 of 8700, 1800 of 1800); a frame rendered alone by a new job equals
the same frame rendered in sequence, 9 of 9 (urban_grid's frames 0, 1, 30, 299 and 300;
the city flight's 0, 29, 30 and 299 of 300–310 s), so a frame does not depend on the
glyphs an earlier one laid out.

#### 2.20.10 `view`

- **It shows there** too, top-left, at the window's logical size: `S` from the window's
  current size, re-set when the window is resized, as the dots take the window's size
  (§2.11). `view` shows what `render` will draw at the same framing.
- **Beside the readout.** The readout stays bottom-left above the slider (`rules/slider.md`,
  left 16, bottom 30), with `t` to two decimals for writing keyframes (vis-001 §2.9.4).
  The HUD's clock is the video's, in whole seconds. Both give `t`, for different jobs, at
  opposite edges of the window; neither changes the other.
- **The key is `H`** (*HUD*; by position, `KeyCode::KeyH`, unbound today: `view` uses `A B D
  E F K L M Q R S W X`, space, `Esc`, the arrows, `+` and `−`). It hides and shows the HUD,
  as `L` does the dots. In the frame order it comes after `L`. The keyframe line does not
  carry it.
- **`view --no-hud`** starts with it hidden, and `H` shows it.

#### 2.20.11 The flag, the key, and their errors

- **On by default** in `render` and `view` (decision 2), on every network, in both
  cameras.
- **`--no-hud`** on `render` and `view`. The name follows `--no-streets` and
  `--no-signals`. On `render` nothing is spawned and the frames are Phase 1's, byte for
  byte (gates 1 and 2). On `view` it starts hidden.
- **`--hud` is not a flag:** clap rejects it, exit 2.
- **No new error.** The HUD reads only the window's FCD, which `run::load` has read and
  checked, and nothing else: not `results.db`, not the network. So no run that renders
  today fails with the HUD on. A tiny frame draws it tiny: `S` is at least 1 px, and the
  probe renders urban_grid at 64×36 with it (exit 0, 60 frames; `S` is 1 px there).
- **The library keeps today's default.** `Job::prepare*` build every job with the HUD off,
  as with streets and signals, so every test that builds a job draws as today.
  `src/main.rs` turns it on with `Job::set_hud(true)` unless `--no-hud` is given.

#### 2.20.12 The scripts that read references

Every CLI render in `scripts/` that is compared with a `ref-pin90b39292` file passes
`--no-streets --no-signals` today (§2.14): those 29 lines gain **`--no-hud`**, and nothing
else (`gates.sh` 4, `gates-ties.sh` 2, `gates-see-through.sh` 12, `gates-parts.sh` 11),
with the comment above them in each of the four. One more line compares with an earlier
phase's frames: `gates-signals.sh`'s `off-` render in `case_`, against `look-p6`, gains
`--no-hud` too, and the fifth comment is `case_`'s own, on its first line (`off against
Phase 6` becomes `off (--no-signals --no-hud) against Phase 6`). The other renders compare
only with each other, count frames that differ, or check only `ffprobe` and stderr, and
still pass with the HUD drawn: `gates.sh`'s `explicit` render and its gate 4 error
renders, `gates-ties.sh` gate 9, `gates-city.sh` gate 13, `gates-credit.sh` gate 14,
`gates-streets.sh` gates 10 and 11, `gates-look.sh` gate 10, and `gates-signals.sh`'s
`signals-` pairs (gate 3). `gates-signals.sh` writes into
`scratch/out/signals/`, over Phase 1's renders, so Phase 2 copies those first (gate 2).

#### 2.20.13 What it does not do

- **No legend, no chart,** no other number: not the KPIs, not a per-class count, not the
  vehicles that have arrived.
- **No warm-up mark,** and no time of day: the run records none.
- **No interpolation** of the count or the speed between rows (§2.20.4, §2.20.5).
- **No placement option:** top-left, as decided. No colour per value (a red speed, say).
- **Nothing in the keyframe file.** A flight's video shows the HUD unless `--no-hud`.
- **No change** to the boxes, the dots, the streets, the buildings, the credit line or
  `view`'s readout.

#### 2.20.14 Build cost

**0 packages,** no Bevy feature, no `Cargo.toml` change. `bevy_text` and `bevy_ui` are on
for `view` and the credit line (vis-002 §2.14.9); `FontFeatures` is in `bevy_text` 0.19.1;
the font is embedded already. One new module, `src/hud.rs`, with no Bevy types.

Measured by the probe, at a load of 2.6 to 58 from other sessions (§2.20.15): Phase 1's seven cases with and without the HUD, within the noise of
the load but for urban_grid's default render (184.6 and 201.4 s with it, 152.7 s without,
at a falling load; its `--camera` twin 150.9 and 148.1 against 151.9 at a load of 3); and
`view --bench 20` on Midtown with buildings, two passes each way, **60.00 fps** in all four
(p99 17.05–17.12 ms). Phase 2's gates 10 and 11 record both again.

#### 2.20.15 Measured while drafting (2026-10-19)

Read-only against the engine at the pin and both fixtures (opened with `immutable=1`, the
FCD read with DuckDB), with output only under gitignored `scratch/vis003p2-probe/` and the
session's scratchpad:
- **`sql/`, `hud_stats.py`, `expected.py`:** the FCD's snapshots (`out/*-snapshots.csv`),
  the three windows' frame-by-frame counts, means and clocks (`out/hud-*.txt`), the mean's
  definitions (`out/mean-defs.txt`), the per-second series of gate 12 (`out/series.txt`),
  and gate 6's sampled strings from the FCD alone (`out/expected-*.txt`);
- **`font_widths.py`:** the font's advances (`out/font-widths.txt`); its `GSUB` features
  were read in review round 1 (§2.20.8);
- **`repo/`** is `git archive` of `origin/main` (`c814e5c`) with a draft `src/hud.rs`,
  `spawn_hud` and `set_hud_text` in `src/draw.rs`, `Renderer::set_hud`, `set_hud_text`,
  `hud_box` and `Job::set_hud`/`hud_text` with the hook in `Job::render_at`, and the HUD
  in `view`, all behind the environment variable `HUD` (`outline`, `panel`, `shadow`;
  `HUD_DIV`, `HUD_UNITS`, `HUD_LAYOUT`, `HUD_TNUM` for the options), and
  `src/bin/p2-probe.rs` (stills, the footprint, history, widths, texts, boxes). Built in
  its own `CARGO_TARGET_DIR`, sccache off: 14 m 04 s at a load of 75–114 from other
  sessions. Nothing in `src/`, `tests/` or `scripts/` was touched;
- **`run.sh`, `runs/`, `out/`:** the CLI renders and the bench (below);
- **`stills/`:** the 1080p stills for OQ-5 and OQ-6.

**Where the probe differs from the scope:** the HUD is switched by an environment
variable, not `--no-hud`; there is no `H`; `view` sizes it at launch only; its `hud_box`
is the root's rectangle grown by the offset, which for the panel is the panel itself.

**Measured with it:**
- **the text** at the 13 sampled frames of gate 6 (`out/text-*.txt`): equal to
  `expected.py`'s, from the FCD alone, 13 of 13;
- **the boxes** (`out/boxes.txt`): the counts of §2.20.4's table; no frame with a box
  bit-identical to another; no frame with no box drawn and a non-empty snapshot;
- **tabular figures** (`out/widths.txt`): §2.20.8's widths;
- **the footprint** (`out/footprint.txt`, `out/sizes.txt`): §2.20.9's, and gate 7's table;
- **history** (`out/history.txt`): 9 of 9 frames equal alone and in sequence;
- **a 64×36 render** (`out/tiny.txt`): exit 0, `64,36,30/1,60`;
- **renders** (`out/renders.txt`), Phase 1's seven cases at a 1-min load of 58 falling to
  3: with `HUD=outline`, twice each, every pair equal (8700 of 8700, 1800 of 1800), and
  0 frames equal to Phase 1's `scratch/out/signals/signals-<case>.framemd5` (read only);
  without `HUD`, once each, every frame equal to Phase 1's, 7 of 7;
- **the bench** (`out/bench.txt`): §2.20.14's;
- **the stills** (`stills/`), 1920×1080: each look, size and layout on three scenes,
  Midtown with buildings at 330 s orthographic (`*-mid-ortho.png`) and in the city flight
  (`*-mid-flight.png`), and urban_grid at 150 s (`*-ug.png`); the same frames without the
  HUD (`none-*.png`); km/h and mph (`units-*.png`); and the top-left corners side by side
  (`sheet-look.png`, `sheet-size.png`, `sheet-layout.png`, `sheet-units.png`).

  What they show, for OQ-6: the outline reads on the white roofs and on the dark ground
  alike; the panel turns a mid grey where it lies over a roof; the shadow leaves the top
  and left of each letter unbordered over the roofs, as vis-002 §2.14.5 found for the
  credit line.

### 2.21 Link colours (Phase 3)

Drafted 2026-10-20, the last item of §2.3's roadmap. The observable is the video itself:
with `--link-colours`, every road strip of the run's video is tinted by a measure of its
FCD rows over the past window ending at the frame's time, and a legend under the HUD says
what each colour means.

```
# Off by default: today's frames (Phase 2's), byte for byte.
assimilator-video render --project <project> --scenario baseline --seed 42 --out run.mp4

# On: speed as a share of the limit, over the last 60 s, per link.
assimilator-video render … --link-colours
assimilator-video render … --link-colours stopped --link-window 30 --link-lanes

# C cycles off → relative → speed → density → stopped → off.
assimilator-video view --project <project> --scenario baseline --seed 42
```

#### 2.21.1 The user's decisions, and where it goes (decision, recorded)

The user's decisions are §2.2's 8–13 (2026-10-20), and are not reopened here: one phase;
off by default, one flag with a measure (speed as a share of the limit by default, speed,
cars per lane-km, stopped cars), in `render` and `view` and both cameras; a past-only
window in seconds, default 60, refused below the run's FCD interval, the interval meaning
"right now"; per link by default and per lane by a flag; no car in the window is today's
road for the speed measures and the lowest step for the counts; `C` in `view`.

The rest of this section is the draft's proposal: what the run gives (§2.21.2), the window
(§2.21.3), each measure (§2.21.4) and its steps (§2.21.5), the palette (§2.21.6, OQ-9),
where the colour goes (§2.21.7, OQ-7), the cars (§2.21.8, OQ-8), the legend (§2.21.9,
OQ-10), depth, the fade and determinism (§2.21.10), the flags, the key and their errors
(§2.21.11), the cost (§2.21.12), and why it is one phase (§2.21.13).

Where it goes, by the methodology's §6.1:
- **Step 0:** a decision changes: a new kind of thing is drawn from the run, on request.
- **Step 1:** nothing shipped is removed or contradicted. Off is the default, and with it
  nothing is read or spawned, so every frame is Phase 2's (gates 1 and 2). No run that
  renders or opens today fails tomorrow: the new errors come only with the new flags.
- **Step 2:** vis-003 owns the subject. Decision 1 named link colours in its roadmap and
  decision 8 made them Phase 3. So it is a phase appended here, `reviewed: null`; `status`
  stays `accepted`.

#### 2.21.2 What the run gives, per link

Only the FCD, which every render reads already (vis-001 §2.1), and the network's lanes,
which `Placement` holds. Measured on both fixtures (engine `90b39292`, seed 42), read-only,
the FCD with DuckDB and the lengths from the engine's `NetworkData::link_length` through a
probe (§2.21.16):

| | Midtown | urban_grid |
|---|---|---|
| Links; lanes | 255; 533 | 48; 95 |
| Links with a row at all; from 300 s (Midtown) or 30 s (urban_grid) | 230; **230** | 25; **25** |
| At one FCD time (from then on): links with a car; cars on each; links with one car | **105.1; 1.80; 54 %** | 18.0; 4.38; 5 % |
| Lane `speed_limit`s (m/s: lanes) | 4.47: 2, 8.33: 43, 11.11: 4, **11.18: 290**, 13.89: 130, 16.67: 64 | 13.89: 95 |
| Links whose lanes have different limits | **0** | **0** |
| Rows above their lane's limit | 0 (the highest is the limit itself) | 0 |
| Rows stopped (`speed` < 0.1 m/s) | 28.4 % | 17.4 % |
| Rows inside a junction (frozen at their entry `position`, moving, on the approach) | 12,293 (6.08 %) | 1,007 (4.67 %) |
| FCD interval: the smallest gap between successive FCD times | **1.000 s** (0.9999999999991) | **1.000 s** (0.99999999999994) |
| Trimmed link lengths | 18.2 – 298.1 m, median 73.6 | — |

- **urban_grid's demand uses half its links.** 23 of its 48 links never carry a car: its
  flows run from the west and south edges to the north and east. A count measure paints
  them the lowest step all run; a speed measure leaves them grey.
- **The brief's facts, checked.** Midtown's 255 links, 230 carrying traffic after 300 s,
  ~105 with a car at a time, 1.8 cars each and 54 % with one: as stated. Its limits: as
  stated (the brief's 11.18 is 11.176, 25 mph). FCD rows every 1 s in both: as stated.
  Speed / limit in quarters with a 60 s window: ~200 links with data (200.2), a link's
  quarter changing 1.37 times a minute, shares 13 / 34 / 23 / 31 %: as stated within
  rounding (the brief's 22 %). **Not reproduced:** "at one FCD time a link's quarter
  changes ~9.7 times a minute". Counted at every FCD time from 300 s, a link with a car at
  both times changes quarter 8.5–8.8 times a minute; counting a link that gains or loses
  its only car as a change too, 12.3. The brief does not say how it counted.
- **The FCD's interval is not recorded** (§2.20.3): the runner can override it
  (`--fcd-interval`), and `runs.resolved_config` does not carry it. So it is taken from
  the file, never a constant (§2.21.3).

#### 2.21.3 The window

**Its rows.** At sim time `t` with a window of `W` seconds, every FCD row with `t − W <
time ≤ t`, each bound within vis-001's `1e-6` (`time > t − W + 1e-6` and `time ≤ t +
1e-6`), from the whole file. Its *times* are the distinct FCD times among them; `n` is
their count.
- **Past only:** no row after `t` is read (decision 10).
- **From the whole file,** not the render's window: at `--from 300` the first frame's
  window reaches back to 240 s. `Fcd::vehicles` holds every row of the file already
  (vis-001 §2.8); `render` reads nothing more.
- **Half-open,** so that a row is in exactly one of two windows that meet.

**The interval, "right now".** The run's FCD interval `I` is the smallest gap between
two successive FCD times in the whole file (1.000 s in both fixtures, as 0.99999999999909
and 0.99999999999994). The engine writes at the first step whose clock reaches the last
write's time plus its interval (`crates/output/src/runner.rs:run_to_results_db`,
`next_fcd_time = time + fcd_interval`), so the rows of a run fall a fixed whole number of
steps apart: the interval rounded up to a step (0.25 s at 0.1 s steps gives 0.3 s). A step
with no vehicle writes no row (§2.20.3) but moves the next write on, so a missing time
makes a gap of `2I`, never a smaller one; the smallest gap is the interval.
- **`W < I − 1e-6` is refused** (decision 10), with the error of §2.21.11. The `1e-6`
  lets `--link-window 1` through on both fixtures, whose `I` is a float sum a hair under 1.
- **`W = I` is "right now":** on rows written every `I` seconds, `(t − I, t]` holds
  exactly one FCD time, the one vis-001's `Fcd::snapshot_at(t)` shows: the boxes', the
  dots' step (§2.6) and the HUD's rows (§2.20.4).
- **A file with one FCD time** has no gap and no interval: any `W > 0` is taken.

**The first seconds.** Before `t − W` reaches the first FCD time, the window holds fewer
times than `W/I`, and the measure is of what there is: the speed measures average the rows
there, and the count measures average over its `n` times, not over `W/I`, so a short window
is not diluted toward 0. Before the first row, `n` = 0: no speed (today's road) and counts
of 0 (the lowest step). Measured, with `W` = 60:

| `t` | Midtown: FCD times in the window; links with a speed | urban_grid |
|---|---|---|
| the first row (1.1 / 9.1 s) | 1; 1 | 1; 6 |
| + 10 s | 11; 8 | 11; 6 |
| + 30 s | 31; 28 | 31; 9 |
| + 59 s | 60; 53 | 60; 14 |
| + 120 s | 60; 105 | 60; 22 |

urban_grid's default render begins at its first row (9.1 s), so its first frames tint the
6 links with a car and leave the rest grey; the tint fills in over the first minute.

**Backwards.** A frame's colours are a function of `t`, `W` and the measure alone,
computed from the rows each frame (§2.21.12), with nothing carried from the frame before.
So `view`'s slider, `Shift+←` and a restart at `from` (vis-001 §2.10) show at `t` what a
render shows at `t`, whichever way `t` moved; gate 9 checks a frame alone against the same
frame in sequence.

#### 2.21.4 The measures

Each row is a car at one FCD time: the FCD gives `link_id`, `lane` and `speed`. A **unit**
is a link (by default) or one lane of it (`--link-lanes`, §2.21.7); a row belongs to the
unit of its `link_id` (and `lane`). Four measures, each over the window's rows of a unit:

| Measure (`--link-colours …`) | Definition | No row in the window |
|---|---|---|
| **`relative`** (the default): speed as a share of the limit | the mean over the rows of `speed / limit`, `limit` the `speed_limit` of the row's own lane | none: today's road |
| **`speed`** | the mean over the rows of `speed` | none: today's road |
| **`density`**: cars per lane-km | rows / `n` / (`L` × lanes / 1000), `L` the link's trimmed length (`Placement::link_length`), lanes its lane count (1 per lane) | 0: the lowest step |
| **`stopped`**: stopped cars | rows with `speed` < 0.1 m/s / `n` | 0: the lowest step |

- **Weighted per row, a car-interval each.** A car on the link for 40 s counts 40 times,
  one that crossed in 5 s counts 5. The mean speed over a link's car-seconds is the
  distance they drove over the time they spent, Edie's space-mean speed, which is what a
  link's traffic flows at. Weighted per car (each car's mean, then the mean of those), a
  car that crossed at the limit in 5 s would weigh as much as one that waited 40 s at a
  red; the link would read faster than its traffic is. It is the HUD's choice too: every
  row, stopped ones included (§2.20.5).
- **`density` and `stopped` are averages over the window's times:** the cars on the unit
  at an FCD time, over `n` times. `stopped` is the mean number standing at a time, a queue
  length: 1 means one car stood there, on average, all window long. `density` divides by
  the unit's lane-km, so a long link and a short one compare.
- **The limit is the row's own lane's,** so a link whose lanes differ is never asked for
  one limit, and per lane needs no other rule. No link in either fixture has lanes that
  differ (0 of 255, 0 of 48), so per link it is the link's limit. The speed measures need
  no length.
- **A car in a junction** is on its approach: while a micro vehicle crosses a junction its
  rows keep the approach's `link_id` and its entry `position`, with `speed` live (vis-001
  OQ-2, §2.7). It counts on the approach, as the HUD counts it: 6.08 % of Midtown's rows
  and 4.67 % of urban_grid's. Junctions themselves are not coloured (§2.21.7).
- **`lane`** is the FCD's; every row's lane is below its link's lane count in both
  fixtures (0 rows otherwise). A row whose lane is not is counted on the link's last lane.
- **Vehicle classes:** every row is a `car` in both fixtures (§2.20.3); the measures count
  every row, whatever its class, as the HUD does, and the legend says "cars" as the user's
  decision 9 does.

#### 2.21.5 The steps

Four steps for every measure, so one palette serves all four (§2.21.6) and the legend has
the same shape. The edges are chosen from the values on both fixtures at every FCD time
from 300 s (Midtown) and 30 s (urban_grid), `W` = 60, per link (`out/stats-*.txt`):

| Measure | Edges (best → worst) | Midtown: share of link-times in each step | urban_grid |
|---|---|---|---|
| `relative` | **≥ 75 % · 50–75 % · 25–50 % · < 25 %** | 31 / 23 / 34 / 13 % (200 links with data at a time) | 38 / 43 / 19 / 0 % (19.9) |
| `speed` | **≥ 30 · 20–30 · 10–20 · < 10 km/h** | 35 / 24 / 31 / 10 % (200) | 62 / 35 / 3 / 0 % (19.9) |
| `density` | **< 3 · 3–6 · 6–12 · ≥ 12 cars / lane-km** | 50 / 27 / 17 / 6 % (the 230 links that carry cars) | 34 / 21 / 41 / 4 % (the 25) |
| `stopped` | **< 0.25 · 0.25–0.5 · 0.5–1 · ≥ 1 stopped cars** | 64 / 18 / 13 / 4 % (230) | 54 / 4 / 24 / 18 % (25) |

- **`relative` in quarters,** the brief's: it reads without a key ("under a quarter of the
  limit"), and both fixtures use all four but urban_grid's slowest.
- **`speed` in round km/h** (the HUD's unit, OQ-5). Midtown's link-times run 10 / 25 / 50
  / 75 / 90th percentile 2.8 / 4.2 / 6.4 / 9.8 / 11.3 m/s (10, 15, 23, 35, 41 km/h). The
  cars' five bins (`src/scene.rs:SPEED_BINS`, 2 / 5 / 9 / 13 m/s) were measured too: 4 / 31 /
  35 / 31 % with the top two merged, the lowest step nearly empty, and edges that read as
  7, 18 and 32 km/h in a legend. Not proposed.
- **The counts' edges are the fixtures' spread,** not a traffic engineer's: jam density
  is about 150 cars a lane-km (4.5 m cars, 2 m gaps), and these networks never come near
  it. Midtown's per-link density runs 25 / 50 / 75 / 90 / 99th percentile 1.1 / 3.0 / 5.7 /
  9.4 / 19.4 (max 40.6), urban_grid's 1.1 / 5.1 / 7.7 / 9.4 / 16.9 (max 22.1); stopped cars
  0 / 0 / 0.4 / 0.7 / 1.5 (max 3.4) and 0 / 0 / 0.85 / 1.4 / 1.7 (max 2.1). The first edges
  tried, 10 / 25 / 50 and 0.5 / 2 / 5, put 91 % and 83 % of Midtown's link-times in the
  lowest step: a map of one colour.
- **How often a link's step changes,** a change counted while it has a value at both
  times, per link-minute, Midtown / urban_grid, `W` = 60: `relative` 1.37 / 0.78, `speed`
  1.35 / 0.54, `density` 0.83 / 0.36, `stopped` 0.56 / 0.39. With `W` = 1 (right now)
  `relative` changes 8.47 / 4.91 times a minute; with 15 s 4.64, 30 s 3.42, 120 s 0.74,
  300 s 0.30 (Midtown). At 1× and 60 s, Midtown's 300–360 s window changes some colour in
  **59 of its 1,800 frames**, urban_grid's whole run in **63 of 8,700**: about once per
  FCD time, never between two.

**A value at an edge.** Cars close on their limit and never reach it (no row is above its
limit, and a car at a 30 km/h street's limit runs at 8.3333 m/s less a few 1e-10). A mean
speed of 29.9999999 km/h would fall in "20–30". So **the speed measures are binned on the
whole number a reader sees,** rounded halves up as the HUD rounds its speed: `relative` on
`⌊100·r + 0.5⌋` %, `speed` on `⌊3.6·v + 0.5⌋` km/h, each against whole-number edges (`p ≥ 75`
is the best step). The counts are binned on their value: `stopped` is a count over `n`,
exact against 0.25, 0.5 and 1 (a value can equal an edge, and an edge is in the step above
it); `density` is a float, binned as computed in §2.21.4's order, and no sampled value of
either fixture lies within 1.7e-3 of an edge (the closest: 5.9972, 2.9982).

The probe's first stills and its renders bin on the unrounded value; the rule above moves a
value only within half a percent or half a km/h of an edge. The gates' predictions (§4,
Phase 3) use the rule above.

#### 2.21.6 The palette (OQ-9)

Four colours, best to worst, the same for every measure. A speed measure's best step is
its fastest, a count's its lowest, so a red street is always the congested one. Two
candidates, each drawn by the probe on every scene (§2.21.16):
- **(a) traffic:** green `#1a9850`, yellow `#f2c12e`, orange `#f07f2a`, red `#d7263d`. The
  colours a map's traffic layer uses. Under simulated deuteranopia the best and the worst
  are 3.3 apart (OKLab ΔE × 100, Machado 2009 severity 1, the dataviz validator's
  `--pairs all`), where 8 is the target: for about one man in twelve, a free street and a
  jammed one look the same. Its green is near the signals' green dot (§2.10), which then
  sits on a green street with only its black ring to part them.
- **(b) blue to red:** blue `#2c7bb6`, light blue `#9ecae1`, orange `#fdae61`, red
  `#d7191c` (ColorBrewer's RdYlBu without its pale middle). Every pair at least 16.5 apart
  under protanopia and deuteranopia (18.7 with full colour vision). It shares no hue with
  the signal dots but red, and keeps red for the worst step.

Against the road and the background: every colour of either palette is opaque and parts
from the road grey `(92, 96, 104)` and the background `(18, 22, 30)`. The validator's
contrast and lightness bands are a chart's, against a chart's surface; here the tint *is*
the surface under the markings, and no text sits on it, so they are not applied.

The palette is the user's (OQ-9). **Recommended: (b),** for the colour-blind reader and
the green dots. Either is one constant in `src/links.rs`.

#### 2.21.7 Where the colour goes (OQ-7)

Two ways, each drawn by the probe on four scenes at the same frame (§2.21.16):
- **(a) The road strip tinted.** Each unit's strip, the same polyline and width as the
  road strip (`scene::strips`'s samples, at most 1 m apart, ± `total_width/2`), coloured
  by its step, at `LIFT_M` 0.005 m over the road. Markings, arrows, stop lines and dots
  stay on top (§2.21.10); junction surfaces and median noses stay grey; a unit with no
  value is not drawn, so its road is today's.
- **(b) A band beside the link's outer edge.** A ribbon `B` = 3 m wide outside the strip's
  right edge (the kerb side, right of travel), on the ground at 0.005 m; the road itself
  stays grey with every marking as today. A two-way street shows each direction's band on
  its own kerb.

What the stills show (`scratch/vis003p3-probe/stills/`, 1920×1080; `sheet-place-<scene>.png`
puts none, (a), (b) and (a) per lane side by side):
- **Midtown's overview** (`k` 1.6009, a lane 2 px): (a) reads as a map of coloured
  streets at a glance, each two-way street in two colours where its directions differ.
  (b) is a 2 px line beside each grey street, an outline more than a colour, and where a
  building meets the kerb the band is under its roof (`zoom-place-mid-ortho.png`).
- **The city flight at 330 s** (`height_m` 500): (a) whole streets in colour between the
  towers; (b) thin kerb lines, many behind buildings.
- **The orbit** (`height_m` 250): (a) the street reads as a coloured surface with its
  lane dashes and arrows on it; (b) reads as a painted kerb.
- **urban_grid** (`k` 1.1481): (a) the 6.5 m streets as coloured lines; (b) a line beside
  each, half of the grid's streets grey (the 23 links with no car).

**Per lane** (`--link-lanes`) is drawn as (a), each lane's own strip (`lane_center_offsets
()[k] ± width_k/2`): in Midtown's overview a lane is 2 px, so a three-lane avenue reads as
stripes, informative close up and busy from afar; 28 % of Midtown's lane-times (23 % of
urban_grid's) are in another step than their link's (`W` = 60). Under (b), per lane would
cut the band into one sub-band per lane, nearest lane nearest the kerb.

**Recommended: (a), the strip tinted,** per lane as each lane's strip. (b) loses at city
zoom, where the band is two pixels, and is covered by every building that meets the kerb.
The choice is the user's (OQ-7): Phase 6's lesson (a look decided on paper failed at the
visual gate) puts it to the stills.

#### 2.21.8 The cars (OQ-8)

Today a box is coloured by its own speed in five bins, red to cyan (`scene::SPEED_BINS`).
On a tinted street:
- **with their speed colours,** a red box on a red street and an orange one on an orange
  street lose their outline: in the orbit's still, the queue at the red light on the
  street tinted red is red on red (`zoom-cars-orbit.png`, top left). The speed is then
  drawn twice, once per car and once per street, in two scales;
- **neutral:** every box one light grey, sRGB `(235, 235, 235)`, shaded in perspective as
  today. The street says how fast, the boxes say where the cars are. In the overview a box
  is a white fleck on any tint (`zoom-cars-mid-ortho.png`).

**Recommended: neutral while link colours are on,** and the speed colours back when they
are off (`C` in `view`). The user's call (OQ-8), on the stills. Off, nothing changes.

#### 2.21.9 The legend (OQ-10)

§2.3's "not planned" legend was vis-001's, for the cars. This one is for the link colours,
and only while they are on: without it a viewer sees colours and no scale.
- **One line under the HUD's,** in the top-left corner: the measure's name, then four
  swatches each followed by its range, in ascending value: `speed / limit ■ < 25 % ■ 25–50
  % ■ 50–75 % ■ ≥ 75 %`; `speed ■ < 10 ■ 10–20 ■ 20–30 ■ ≥ 30 km/h`; `cars / lane-km ■ < 3
  ■ 3–6 ■ 6–12 ■ ≥ 12`; `stopped cars ■ < 0.25 ■ 0.25–0.5 ■ 0.5–1 ■ ≥ 1`.
- **Drawn as the HUD is** (§2.20.9): Fira Sans Medium at `S` (1/36 of `min(H, 9W/16)`),
  the light fill with the eight black copies; each swatch a square of `0.75·S` with an
  `o`-px black border. Its top is `m + 1.6·S` (the HUD's line box, 1.2·S, and 0.4·S
  between), or `m` under `--no-hud`; its left is `m`.
- **About 745 px wide at 1080 lines** for `relative` (x 27 to 772 in the probe's frames),
  from the top-left corner to under half the frame's width; the credit line stays bottom-right, far
  from it. In Midtown's overview it lies over the background and two street ends; in a
  flight, over whatever the pose puts there.
- **Static per render:** its text is set at spawn, so the settle frames lay out its glyphs
  (`≥`, `%`, `/` and the rest) before the first frame. In `view` it is respawned when `C`
  changes the measure.
- **Not in it:** a swatch for today's road (no car, the speed measures) and the window's
  length. A grey street reads as a street.

**Recommended: as above.** The user sees it on every still (OQ-10); `nohud-mid-ortho.png`
shows it in the HUD's place.

#### 2.21.10 Depth, the fade, and determinism

| What | Height |
|---|---|
| Road strips, junction surfaces, median noses | 0 |
| **Link colours** (either way) | **0.005 m** |
| Lane dashes, centre lines; stop lines, connectors, arrows | 0.01; 0.02 m |
| Signal dots (orthographic; perspective, the lowest point) | 0.03 m |
| Boxes | from 0.05 m |

- **Over the road, under every marking.** 0.005 m is half of vis-002's 0.01 m lift, which
  §2.17.9 measured clear of the depth test; at the view's widest zoom (4.2 km from the eye)
  a depth step is 0.5 mm, so 0.005 m is 10 steps, and 80 in the orthographic frame.
- **One mesh, one entity:** every unit's triangles, in unit order (the network's link order,
  lanes in order within a link), with a colour per vertex on one unlit white material that
  does not cull, `NoFrustumCulling`, as the markings and the dots (vis-002 §2.17.11). Where
  units meet (the lanes of one link, the two strips of a 0 m pair, strip ends at a junction's
  edge) they are in the one draw, where the GPU keeps primitive order: no tie between
  entities, and the later unit wins the same way every frame.
- **The fade stays as it is.** A marking under a pixel fades toward the road grey
  (vis-002 §2.17.10), not toward the tint under it, so on a tinted strip a faded dash is a
  grey trace. In Midtown's overview the dashes are at `α` 0 and 0.09 px wide; at 1080p
  no grey speckle is seen on the tint in the stills (`zoom-place-mid-ortho.png`); it is not
  counted. Fading toward the tint would need each marking's link, which `NetworkJson`'s
  dashes do not carry: not proposed; the user sees it at gate 12.
- **The tint needs no fade:** it is a surface the strip's width, 3.25–13 m in Midtown, 2 px
  and more in every frame here; a lane is 2 px in Midtown's overview, and two lanes of
  different steps meet in a blended pixel at MSAA ×4.
- **Deterministic.** The rows are taken in the FCD's (`time`, `vehicle_id`) order and each
  unit's sums run in that order; the units in network order; a name-to-unit map is only
  looked up, never walked (no order comes from walking a `HashMap`). The mesh is a
  function of the units' steps; it is replaced only when a step differs from the last
  frame's, and the same steps give the same mesh. Measured: renders twice equal (§2.21.16).

#### 2.21.11 The flags, the key, and their errors

- **`--link-colours [MEASURE]`** on `render` and `view`: `relative` (the default, when the
  flag is given bare), `speed`, `density` or `stopped`. One spelling, as the code's
  `Colour` and §2's prose; `--link-colors` is not an alias.
- **`--link-window <SECONDS>`**, default 60, and **`--link-lanes`**. On `render` each
  needs `--link-colours` (clap's `requires`, exit 2, as any usage error); on `view` they are
  taken alone, for `C`.
- **Off** (no `--link-colours` on `render`): nothing is read, built or spawned, the cars
  keep their speed colours, and every frame is Phase 2's (gates 1 and 2).
- **`C`** in `view` (*colours*; by position, `KeyCode::KeyC`, unbound today: `view` uses
  `A B D E F H K L M Q R S W X`, space, `Esc`, the arrows, `+` and `−`): off → `relative`
  → `speed` → `density` → `stopped` → off. `view --link-colours speed` starts at `speed`.
  In the frame order it comes after `H`; the keyframe line does not carry it.
- **`--link-colours` names the measure the user decided** (decision 9): `relative` is
  speed relative to the limit, "relative speed" as traffic reports say it. Rejected:
  `share` (a share of what), `limit` (reads as the limit itself), `speed-limit` (the same).

**The errors**, each one line, exit 1, no output file and no window (vis-001 §2.4), raised
after `run::load` has read the FCD, in this order:
1. `error: --link-window must be a positive number of seconds (got <W>)`, for `W ≤ 0` or
   not finite (`nan`, `inf`); clap rejects a word, exit 2.
2. `error: --link-window <W> s is shorter than the run's FCD interval (<I> s); --link-window
   <I> colours by each frame's own FCD row`, for `W < I − 1e-6`, with `I` printed to the
   millisecond (`1.000`).

Both name the flag. On `render` they are checked only with `--link-colours`; on `view`
at launch when `--link-colours` or `--link-window` is given. A plain `view` on a run whose
interval exceeds 60 s opens as today, and `C` then leaves the colours off (as `L` on a
network with no signal): neither fixture has one.

**No other new error:** the link colours read the FCD `run::load` has checked and the
network `Placement` holds. A unit whose link has no placed geometry draws nothing.

#### 2.21.12 Cost: per frame, from the rows

**Per frame, from the rows** (proposed): each frame sums its window's rows straight, one
pass over about `W/I` × (rows at a time) rows (Midtown: 60 × ~170). Measured, the mean
over 9,000 frame times spread over each whole run, `W` = 60 (`out/cost.txt`):

| | Midtown per link | per lane | urban_grid per link | per lane |
|---|---|---|---|---|
| Units | 255 | 533 | 48 | 95 |
| A frame's values: `relative`; `speed`; `density`; `stopped` | 47; 22; 21; 20 µs | 53; 41; 37; 41 µs | 14; 8; 8; 8 µs | 17; 12; 13; 13 µs |
| Built once: the units; the mesh's data | 13 ms; 2.8 ms | 14 ms; 4.8 ms | 1.4 ms; 0.8 ms | 1.8 ms; 2.0 ms |
| The mesh: vertices; triangles | 61,038; 60,528 | 112,438; 111,372 | 27,168; 27,072 | 53,782; 53,592 |

- **Not precomputed.** Running sums per unit over the FCD times would answer a window by
  two subtractions, but a difference of float sums is not the straight sum bit for bit,
  and gate 6 compares with a straight computation; at 50 µs a frame there is nothing to
  save. The rows are already in memory (`Fcd::vehicles`).
- **The mesh is replaced only when a step changes:** 59 of 1,800 frames of Midtown's
  window at 1×, 63 of 8,700 of urban_grid's run.
- **`view --bench 20`** on Midtown with `--buildings` (streets, see-through, signals and
  the HUD on), the probe replacing the tint mesh on **every** frame: **60.00 and 60.00 fps**
  with `relative` (p99 18.76 and 18.66 ms), 60.00 and 59.98 without (p99 18.62, 20.86), at
  a load of 7.5 falling to 4.1.
- **Render time:** the probe's renders, §2.21.16.

#### 2.21.13 One phase

Decision 8 asks the draft to measure whether one review and one build hold it. They do:
- **About Phase 1's size.** One new module with no Bevy types (`src/links.rs`: the units,
  the window, the four measures, the steps, the palette, the mesh's data), one mesh entity
  and a UI line in `src/draw.rs` and `src/render.rs` (as Phase 1's dots and Phase 2's HUD),
  the flags in `src/main.rs`, `C` in `view`, the cars' colour switch, one test file and one
  script. Phase 1 was a module, a mesh, a flag, a key, an error pair, a test file and a
  script, reviewed in two rounds and built on one branch.
- **No engine request, no new input, 0 packages.** The probe built all of it but `C`, the
  flags and the errors in about 470 lines beside its probe binary
  (`scratch/vis003p3-probe/repo/`).
- **The parts do not stand alone:** a measure without a look is not a video, and the look
  questions (OQ-7 to OQ-10) are one set of stills.

No split is proposed.

#### 2.21.14 What it does not do

- **No colour in junctions,** on median noses or on markings; no colour by movement.
- **No interpolation** between FCD times: the window's rows change once per FCD time.
- **No other measure:** no flow (cars a minute), no delay, no travel time, no per-class
  measure; no measure from `results.db`'s detectors.
- **No colour change for the signals' dots, the HUD or the credit line;** the legend is
  added, nothing else moves.
- **No option for the steps or the palette:** each measure's edges and the one palette
  are constants.
- **Nothing in the keyframe file.** A flight's video shows link colours only with the flag.
- **No meso run** is refused for link colours: a meso run's FCD has the same columns (OQ-3
  is about signals). None has been tried.

#### 2.21.15 Build cost

**0 packages,** no Bevy feature, no `Cargo.toml` change: the measures are plain Rust over
the FCD's rows, the mesh is per-vertex colour on one unlit material as the markings', and
the legend is Bevy UI text and nodes as the HUD's (`bevy_ui` is on, vis-002 §2.14.9). One
new module, `src/links.rs`, with no Bevy types. No engine request.

#### 2.21.16 Measured while drafting (2026-10-20)

Read-only against the engine at the pin and both fixtures (FCD SHA-1s `01becc86…` and
`3ad76744…`, as Phase 2's), with output only under gitignored `scratch/vis003p3-probe/`:
- **`sql/export.sql`, `stats.py`, `expected.py`:** the FCD exported by DuckDB; the facts of
  §2.21.2, every measure's distribution, steps and change rates for `W` of 1, 15, 30, 60,
  120 and 300 s, per link and per lane (`out/stats-*.txt`); each unit's value and step at
  sampled times computed straight from the rows (`expected.py`), the step counts of gate 6
  (`out/table.txt`) and the visual gate (`out/series-*.txt`); `synth.py`, gate 5's
  predictions by the same rule;
- **`repo/`** is `git archive` of `origin/main` (`8983224`) with a draft `src/links.rs`,
  the tint mesh, the legend and the neutral cars in `src/draw.rs`, `Renderer::set_links`
  and `set_link_colours`, `Job::set_links_from_env` with the hook in `Job::render_at`, the
  colours in `view` (no `C`), all behind environment variables (`LINKS` for the measure,
  `LINKS_LOOK`, `LINKS_LANE`, `LINKS_W`, `LINKS_CARS`, `LINKS_PAL`, `LINKS_LEGEND`), and
  `src/bin/p3-probe.rs` (`still`, `lengths`, `values`, `cost`, `pixels`, `history`). Built in its own
  `CARGO_TARGET_DIR`, sccache off: 6 m 11 s (the dependencies, then the crate after a
  one-line fix) at a load of 10–54 from other sessions;
- **`run.sh`, `out/`, `runs/`, `stills/`:** the steps below.

**Where the probe differs from the scope:** its first stills (all but `rec-*`), its renders
and its 12,176-value check bin the unrounded value; it was rebuilt with §2.21.5's rule for
the `rec-*` stills and gates 7 and 9's measures. It has no `--link-colours`,
`--link-window` or `--link-lanes`, no `C` and no errors; its `view` has no legend and
replaces the tint mesh every frame while benching; its band (b) is 3 m.

**Measured with it:**
- **its values against the straight computation** (`out/check.txt`): every unit with a
  value at Midtown's 300, 330, 359.97 and 1,199.1 s and urban_grid's 9.1, 60, 150 and
  299.07 s, every measure, per link and per lane: **12,176 values, every one equal to 9
  decimals, every step equal**, and the same units with no value;
- **the cost** (`out/cost.txt`) and **the bench** (`out/bench.txt`): §2.21.12;
- **the stills** (`stills/`): §2.21.6–§2.21.9;
- **the renders** (`out/renders.txt`, `runs/*.framemd5`), through the probe's
  `assimilator-video`:
  - **off** (no `LINKS`): Phase 2's seven cases, every frame equal to
    `scratch/out/hud/hud-<case>.framemd5` (read only), seven of seven (8700 of 8700, 1800
    of 1800);
  - **on**, twice each: Midtown's orthographic with buildings `relative`, the city flight
    with buildings `speed` per lane, the orbit `stopped` (60 s), urban_grid's default
    `density`, in palette (a) with the speed-coloured boxes and the unrounded steps: every
    pair equal (1800 of 1800, 8700 of 8700), and 0 frames equal to Phase 2's in each. They
    ran while another session's build took the load from 3 to 76, so their times are not
    compared (gate 10);
- **gate 7's pixels** (`out/pixels.txt`) and **gate 9's frames** alone and in sequence
  (`out/history.txt`, 9 of 9 equal), both with the rounded steps, palette (b) and neutral
  boxes;
- **the recommended look** (`stills/rec-<measure>-<scene>.png`: (a), neutral boxes,
  palette (b), the rounded steps), each measure on each scene, `sheet-rec-<scene>.png`
  (none, (a) with the speed colours, (a) neutral, the recommendation) and
  `sheet-rec-measures-<scene>.png` (the four measures), `rec-lanes-mid-ortho.png` and
  `rec-nohud-mid-ortho.png`.

## 3. Open questions

- **OQ-1** — The dot's size: the dashboard's, scaled with the frame, or larger (§2.10)?
  **RESOLVED.**
  - *(a) As drafted:* a fill of 4.5 px and a ring to 5.5 px at 1080 lines, scaled by
    `min(H, 9W/16)/1080`: the dashboard at one device pixel per CSS pixel. 17.6 m across in
    Midtown's overview; 107 overlapping pairs there.
  - *(b) Twice that:* 9 and 11 px at 1080 lines, the dashboard on a 2× display. Easier to
    read on a projector; Midtown's overview dots then cover 35 m and most of each crossing.
  - *(c) Fixed in pixels at every size:* the 1080p dot in a 4K video is half the size.
  - *Recommendation:* (a). The user sees it at gate 14; changing it changes gate 9's and
    gate 14's numbers only.
  - ~~*(design call: the user; blocks gates 9 and 14's numbers, not the design.)*~~
  - *(answered 2026-10-09, user, before review)* **(a): the dashboard's size, scaled with
    the frame.** A fill of 4.5 px and a ring to 5.5 px at 1080 lines. The probe's 1080p
    videos with dots (`runs/on-orbit.mp4`, `on-city.mp4`, `on-ortho-city.mp4`) were put to
    the user before the answer. Recorded as §2.2 decision 4 and §2.10. Gates 9 and 14 were
    predicted with (a), and their numbers do not change.
- **OQ-2** — An engine request: record the signal states in the run's outputs (§2.15).
  - *Why:* a run whose signals were forced (`set_signal`) would not replay, and nothing in
    `results.db` or the FCD says that it was. At the pin no such run writes what the
    visualizer reads, so Phase 1 does not need it.
  - *The ask:* a `signal_events` table in `results.db` (time, node, phase index,
    sub-state), written at each change; at the least, a field in `runs.resolved_config`
    marking external control. The engine side has one home there (asm-001 §10).
  - *Recommendation:* the table, raised with the harness contract (vis-001 §2.7 item 6,
    and its OQ-4 on FCD in agent runs). The visualizer would then draw the recorded states
    and keep the replay for older runs.
  - *(needs-input: engine; deferred by evidence: no producer of forced runs writes FCD or
    `results.db` at the pin. It blocks no gate of Phase 1. What forces it: the first phase,
    here or in vis-001's harness item, that draws a run not written by
    `run_to_results_db` or the dashboard server.)*
- **OQ-3** — Meso runs (§2.13).
  - *The facts:* the meso engine steps the same `SignalState` once a step
    (`crates/meso/src/engine.rs`, `advance(self.dt)` at the start of `step`), but with
    `meso.timestep`, and its vehicles take a movement's green
    (`is_movement_green`), not the link's colour. No fixture is meso, and no gate here has
    drawn a meso run at all.
  - *Recommendation:* refuse with signals on in Phase 1, with the error of §2.13 and
    `--no-signals` as the way through; take it up when a meso fixture exists.
  - *More facts (2026-10-09, round 1):* a meso run's `resolved_config` has no
    `sim_params.timestep` (§2.4), so the refusal reads `runs.mode` first (§2.13). Today's
    `render` does not read `mode` and would draw a meso run's FCD if it has the columns;
    none has been tried.
  - *(design call: the user; deferred by evidence: no meso fixture. Phase 1 builds the
    recommendation, which changes one default: a meso run that `render` or `view` takes
    today is refused tomorrow without `--no-signals` (§2.1 step 1). Gates 5 and 15 check
    the refusal and the way through; nothing else in Phase 1 waits on it.)*
  - *(answered for Phase 1, 2026-10-09, user, after the review converged)* **Refuse, as
    the spec has it.** A meso run with signals on is refused in Phase 1. The user takes
    this as deferring, so **this question stays open for meso dots,** to take up when a
    meso fixture exists. Recorded as §2.2 decision 7. No scope, gate or prediction
    changes.
- **OQ-4** — Dots under buildings in the orthographic render (§2.12). **RESOLVED.**
  - *As drafted:* a dot is part of the scene, under a roof that stands over its road, as
    a box is: 7 of Midtown's 188 with `--buildings`.
  - *The alternative:* draw dots over everything in the orthographic render. They would
    then also cover the boxes, which the dashboard does not do, and a dot would show
    through a roof the boxes under it do not.
  - *Recommendation:* as drafted.
  - ~~*(design call: the user; blocks nothing but gate 14's wording.)*~~
  - *(answered 2026-10-09, user, before review)* **As drafted.** In the orthographic render
    a dot under a roof stays hidden, as the boxes are. Recorded as §2.2 decision 5 and
    §2.12. Gate 14 was worded with it and does not change.
- **OQ-5** — The HUD's speed unit: km/h or mph (§2.20.7)? **RESOLVED.**
  - *(a) km/h, fixed:* the dashboard's unit and the engine's other displays'. Midtown's
    window reads 15 – 25 km/h.
  - *(b) mph, fixed:* what a US audience reads. Midtown is in Manhattan and its network is
    mostly limited at 25 mph; urban_grid's 50 km/h limit reads 31 mph. The window reads
    9 – 15 mph.
  - *(c) a flag,* `--speed-unit kmh|mph`, defaulting to one of them: one more flag in the
    harness contract, and still a default to choose.
  - *Recommendation:* (a), as the dashboard. The probe's stills show both on Midtown
    (`stills/units-kmh.png`, `stills/units-mph.png`).
  - ~~*(design call: the user. It blocks gate 6's predicted strings and gate 12's wording,
    not the design: either unit is one constant and one word.)*~~
  - *(answered 2026-10-19, user, before review)* **(a): km/h, fixed,** as the dashboard.
    The probe's stills (`stills/sheet-units.png`) were put to the user before the answer.
    Recorded in §2.20.7. Gates 6 and 12 were written with km/h, and do not change.
- **OQ-6** — How the HUD looks (§2.20.8, §2.20.9), chosen from the probe's stills
  (`scratch/vis003p2-probe/stills/`, 1920×1080, §2.20.15). **RESOLVED.**
  - **(a) Readable over any background:** the credit line's outline (light fill, eight
    black copies), a translucent black panel, or a one-sided shadow.
    `sheet-look.png` puts the three side by side on Midtown's orthographic frame, a frame
    of the city flight and urban_grid; each full frame is `look-<treatment>-<scene>.png`.
  - **(b) Its size:** `min(H, 9W/16)` over 54 (the credit line's size, 20 px at 1080
    lines), 36 (30 px) or 27 (40 px). `sheet-size.png`; `size-<n>-<scene>.png`.
  - **(c) One line or three:** `5:00 · 184 vehicles · 17 km/h`, or the clock, the count
    and the speed stacked. `sheet-layout.png`.
  - **(d) The words:** "vehicles" (the FCD's every row, the dashboard's word) or "cars" (the
    user's word in decision 1; the same number on both fixtures, where every row is a
    car).
  - *Recommendation:* (a) the outline, (b) 1/36, (c) one line, (d) "vehicles". Phase 6's
    lesson: a look decided on paper failed at the visual gate, so the choice is made on
    the stills, and gate 12 shows it again in motion.
  - ~~*(design call: the user. It blocks gate 7's pixel counts and box sizes, gate 6's
    strings (d) and gate 12's wording, not the design: the treatments are one function in
    `src/draw.rs`, the ratio and the words constants in `src/hud.rs`.)*~~
  - *(answered 2026-10-19, user, before review)* **As recommended: (a) the outline, (b)
    1/36 (30 px at 1080 lines), (c) one line, (d) "vehicles".** The probe's stills
    (`stills/sheet-look.png`, `sheet-size.png`, `sheet-layout.png`, `sheet-units.png`)
    were put to the user before the answer. Recorded in §2.20.8 and §2.20.9. Gates 6, 7
    and 12 were written with these answers, and do not change.
- **OQ-7** — Where the link colour goes (§2.21.7), chosen from the probe's stills
  (`scratch/vis003p3-probe/stills/`, 1920×1080, Midtown with buildings at 330 s
  orthographic and in the city flight, urban_grid at 150 s, and the orbit at 330 s).
  - **(a) The road strip tinted,** markings, arrows and dots on top, junctions grey;
    per lane, each lane's own strip.
  - **(b) A 3 m band outside the strip's kerb edge,** the road and its markings as today.
  - `sheet-place-<scene>.png` puts the scene with no colours, (a), (b) and (a) per lane
    side by side (`mid-ortho`, `mid-flight`, `ug`, `orbit`); `zoom-place-mid-ortho.png` and
    `zoom-place-ug.png` the same at twice the size, cropped.
  - *Recommendation:* (a). (b) is two pixels at city zoom and passes under every building
    that meets the kerb.
  - *(design call: the user. It blocks gate 7's pixels and gate 12's wording, not the
    design: the two are one function of `src/links.rs`.)*
- **OQ-8** — The cars while link colours are on (§2.21.8): their speed colours, or one
  neutral light grey `(235, 235, 235)`?
  - `sheet-cars-<scene>.png`: speed colours and neutral, with each palette of OQ-9;
    `zoom-cars-orbit.png` and `zoom-cars-mid-ortho.png` cropped; `sheet-rec-<scene>.png`
    none, (a) with the speed colours, (a) neutral, and the whole recommendation.
  - *Recommendation:* neutral while the colours are on. On the same-hue street a speed
    colour loses its box (the orbit's red queue on a red street), and the street already
    shows the speed.
  - *(design call: the user. It blocks gates 7 and 12's wording, not the design: one
    switch on the box colour.)*
- **OQ-9** — The palette (§2.21.6), four colours best → worst, for every measure.
  - **(a) traffic:** green, yellow, orange, red (`#1a9850`, `#f2c12e`, `#f07f2a`,
    `#d7263d`); the best and the worst 3.3 apart under deuteranopia.
  - **(b) blue to red:** blue, light blue, orange, red (`#2c7bb6`, `#9ecae1`, `#fdae61`,
    `#d7191c`); every pair at least 16.5 apart under protanopia and deuteranopia.
  - `sheet-cars-<scene>.png`'s top row is (a), its bottom row (b); every `rec-*` still is
    (b).
  - *Recommendation:* (b), for the colour-blind reader and the signals' green dots.
  - *(design call: the user. It blocks gate 7's colours and gate 12's wording.)*
- **OQ-10** — The legend (§2.21.9): one line under the HUD, top-left, at the HUD's size and
  outline, the measure's name and four swatches with their ranges; in the HUD's place under
  `--no-hud`.
  - Every still with colours shows it; `nohud-mid-ortho.png` without the HUD;
    `sheet-rec-measures-<scene>.png` shows the four measures' legends and maps in the
    recommended look (`sheet-measures-<scene>.png` in palette (a)).
  - *Recommendation:* as drawn.
  - *(design call: the user. It blocks gate 7's footprint and gate 12's wording.)*

## 4. Implementation phases

Strictly sequential. Phase 1 is the only one drafted (decision 1).

*(2026-10-19)* Phase 2, the HUD, is drafted below, after Phase 1 shipped (§2.20).

*(2026-10-20)* Phase 3, link colours, is drafted below, after Phase 2 shipped (§2.21).

### Phase 1 — Signals: the engine's signal states as dots at the stop lines, by default
*Produces the observable: yes. `render` writes the run's video with a dot at every
signalised approach, green, amber or red as the engine had it at each frame's time, in the
orthographic render and in keyframed flights. With `--no-signals` the video is
byte-identical to today's (gates 1 and 2).*

Drafted 2026-10-09; the design is §2, and the user's decisions are §2.2. Phase 1 builds on
vis-002 Phase 6 (shipped 2026-10-09) at engine `90b39292`. With `--no-signals` it changes
no output. With signals on it adds one mesh and changes nothing else drawn. OQ-1 (the
dot's size) is answered (a) and OQ-4 (dots under roofs) as drafted, and the key `L` is
confirmed (§2.2, decisions 4–6, 2026-10-09), as the scope and every gate were predicted.

- **Scope:**
  - **`src/signals.rs`** (new; no Bevy types):
    - the copied values, each with its file and line at `90b39292` in a comment (§2.10):
      `RED`, `AMBER`, `GREEN`, `OUTLINE` (sRGB), `FILL_PX` 4.5 and `OUTER_PX` 5.5 (from
      `radius = 5` and `lineWidth = 1`); and the rule of `collect_signal_state` (§2.9);
    - `LIFT_M` 0.03 and `SEGMENTS` 32 (§2.11, §2.12);
    - `Colour { Green, Amber, Red }`, from the engine's string (anything else red, as
      `SIGNAL_COLORS[s.state] ?? SIGNAL_RED`);
    - `Approach { node, link, at: [f64; 2], changes: Vec<(u32, Colour)> }`, and
      `Signals { approaches, t: Vec<f64> }`;
    - `Signals::build(&Placement, dt, duration) -> Signals`: §2.6's timeline, stepping
      `SignalState` from `new` while `time < duration − 1e-9`, as the engine's loop;
    - `Signals::for_run(&Placement, results, scenario, seed) -> Result<Option<Signals>>`:
      `None`, with nothing read, on a network with no signal (§2.13); otherwise
      `inputs::run_clock` and `build`. `Job::set_signals` and `view` both call it;
    - `Signals::colours_at(t) -> Vec<Colour>`: §2.6's rule, in approach order;
    - `scale(w, h)`, and `dots(&Signals, &[Colour], camera, w, h) -> DotsData` (positions
      in world metres, an sRGB colour per vertex, indices), where `camera` is the
      orthographic `k` or a `Pose` (§2.11). Per dot: the ring (32 inner, 32 outer vertices,
      64 triangles), then the fill (a centre and 32 rim vertices, 32 triangles), dots in
      approach order; a perspective dot with `f·ρ/R ≤ 0.1` is left out.
  - **`src/inputs.rs`:** `run_clock(results, scenario, seed) -> Result<(f64, f64)>`, the
    run's `timestep` and `duration` from the completed row for the scenario and seed (the
    one `require_completed_run` accepts), read-only and `immutable=1`. It raises both
    errors of §2.13 itself, the mode's first.
  - **`src/draw.rs`:** `spawn_dots` (one entity, one unlit white material with
    `cull_mode: None`, `NoFrustumCulling`, hidden until set) and `set_dots` (replace its
    mesh, linear vertex colours from sRGB as the markings', hidden when empty), about the
    bake origin `(fx, fy)` as every mesh.
  - **`src/render.rs`:** `Renderer::set_signals(on)` spawns or despawns the entity and
    settles again, as `set_streets`; `Renderer::set_dots(&DotsData)`.
  - **`src/lib.rs`:**
    - `Job` keeps the run's `results.db` path, scenario and seed, which it drops today:
      `prepare_inner` takes them from `inputs::RunPaths::new` on the same options
      `run::load` took. `src/run.rs` is not edited;
    - `Job` keeps an `Option<Signals>`; `Job::set_signals(on) -> Result<()>` calls
      `Signals::for_run` and tells the renderer (on), or drops the timeline and the entity
      (off); every `prepare*` builds a job with signals off;
    - `render_at(t)` sets the dots for `t` and the frame's pose (or `k`) before the boxes.
    - *(2026-10-09, round 1)* The draft's `Job::signals()`, "for the gates", is dropped:
      no gate needs a `Job`'s timeline (gates 6–8 build theirs with `run::load`,
      `inputs::run_clock` and `Signals::build`, headless).
  - **`src/view/mod.rs`, `src/view/state.rs`:**
    - `ViewOptions` gains `signals` (true unless `--no-signals`), as `streets`;
    - at launch, before the window: `Signals::for_run`, with the `results.db` path,
      scenario and seed from `inputs::RunPaths::new` on `o.load`, as `Job` takes them.
      With `signals` on, its error is `view`'s error. With `--no-signals`, an error is dropped and there is no timeline
      (§2.13);
    - `signals_shown` in the state, off at `ViewState::new`, set from `signals` at launch;
      `Pressed` gains `l`, and `L` (`KeyCode::KeyL`) flips `signals_shown` after `M`;
    - while shown and there is a timeline, `set_dots` every frame for `t`, the pose and the
      window's logical size, in `--bench` too; otherwise the entity is hidden.
  - **`src/main.rs`:** `--no-signals` on `render` and `view`, with help text; `render`
    calls `job.set_signals(true)?` unless it is given; `view` passes it to `ViewOptions`.
    No other flag or default changes.
  - **Tests.** `tests/signals.rs` (new):
    - headless, not ignored: gates 5 and 8's synthetic part, and gate 15's `L`;
    - headless, needing the fixtures, so ignored: gates 6, 7 and 8's fixture part, each
      from `run::load`, `inputs::run_clock` on the fixture's `results.db`, and
      `Signals::build`;
    - through the GPU, ignored: gate 9 (no fixture).
  - **Scripts.** `scripts/gates-signals.sh` (new, offline): gates 2 and 10, gate 15's CLI
    cases, and the renders for gate 14, into `scratch/out/signals/`. It is the first
    script to need the `sqlite3` CLI (`/usr/bin/sqlite3` on the development machine), and
    its header says so. The 29 lines of §2.14
    gain `--no-signals`, and the comment above them in each of the four scripts says so.
  - **Not edited:** every other test file and script; every source file not named above;
    `Cargo.toml` and `Cargo.lock`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid and Midtown at engine `90b39292` (FCD SHA-1s `3ad76744…` and `01becc86…`),
  baseline, seed 42, with Midtown's Phase 2 cache (`scratch/midtown/buildings.geojson`,
  SHA-256 `f241ccbd…`). Everything runs offline. The predictions come from §2.19's probe.

  **Baseline, before any change**, at `origin/main`:
  - **The ten references** `scratch/ref-pin90b39292-*.framemd5` are present with their
    SHA-256s (`cdad89d1…` camera, `dcfdd2b7…` default, `f6255241…` ortho-city, `2139c319…`
    ortho-roads, `d6641985…` flight-city1 and city-off, `020475b7…` flight-roads1,
    `d7118c3a…` city-on, `af80a068…` orbit-off, `05d7d174…` orbit-on). If one is missing or
    differs, the run stops.
  - **Phase 6's renders are copied** before anything runs, since `gates-look.sh` writes
    into `scratch/out/look/`: its fourteen `*.framemd5` and the four videos of Phase 6's
    gate 14 (`look-ortho-city.mp4`, `look-city.mp4`, `look-orbit.mp4`,
    `look-ug-camera.mp4`) go to `scratch/out/look-p6/`, with their SHA-256s recorded (each
    `framemd5` pair equal: `14e5ac82…` ug-default, `19336f38…` ug-camera, `7d605e90…` ortho-city,
    `b7d1389a…` ortho-roads, `a1e7b73a…` city, `e80c6daa…` city-roads, `6c357566…` orbit).
  - **Run** every gate script and every test file with `--include-ignored
    --test-threads=1`, and keep their output. Record spec-lint and `Cargo.lock`'s package
    count (500).

  - **What must not change:**
  1. **Off, against the references.** The 29 renders of §2.14 with `--no-signals`:
     `scripts/gates.sh` passes, its default render **8700 of 8700** against
     `ref-pin90b39292-default` and its `--camera` render **8700 of 8700** against
     `ref-pin90b39292-camera` (by hand on `scratch/out/camera.framemd5`);
     `gates-ties.sh` gate 2, `gates-see-through.sh` gate 2 and `gates-parts.sh` gate 2 each
     **1800 of 1800** against their references; the recorded lines of
     `gates-see-through.sh` gate 7 and `gates-parts.sh` gate 9 equal the baseline's.
  2. **Off, against Phase 6** (`gates-signals.sh`). Phase 6's seven cases (vis-002 Phase 6
     gate 10's list) with `--no-signals`: urban_grid's default and `--camera
     tests/flight.toml` **8700 of 8700**, and Midtown's five **1800 of 1800**, each against
     its copy in `scratch/out/look-p6/`. (The probe, without the flag: ortho with buildings
     and the city flight with buildings, 1800 of 1800.)
  3. **What Phase 1 does not touch.**
     - Every test file but `tests/signals.rs`, unedited, passes with every printed number
       equal to the baseline's.
     - The renders that compare only with each other still give equal pairs:
       `gates-ties.sh` gate 9, `gates-city.sh` gate 13 and `gates-credit.sh` gate 14,
       **1800 of 1800** each; `gates-streets.sh` gates 10 and 11 and `gates-look.sh` gate 10
       pass (equal pairs, each differing where it differed). All eight existing scripts
       pass; the ninth, `gates-signals.sh`, is gates 2, 10 and 15.
     - `git diff origin/main --stat -- src/` adds `src/signals.rs` and changes
       `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/inputs.rs`, `src/main.rs`,
       `src/view/mod.rs` and `src/view/state.rs`; `-- scripts/` adds `gates-signals.sh` and
       changes the 29 lines and four comments of §2.14 alone, read hunk by hunk; `-- tests/`
       adds `tests/signals.rs` alone.
  4. **Build cost** (§2.18). `git diff origin/main -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The replay — headless:**
  5. **A synthetic crossing with a plan** (`tests/signals.rs`, not ignored). vis-002
     gate 5's crossing (`tests/look.rs`'s YAML, written out in the test), with this plan on
     `C`, `dt` 0.1 and `duration` 100 (the probe's `scratch/vis003p1-probe/synth/
     network.yaml`):
     ```yaml
     signal_plan:
       cycle_time: 60      # not the phases' 45: the engine does not read it
       offset: 7           # nor this
       phases:
         - {id: P1, duration: 20, green_movements: [W_T, W_R, E_T, E_R], permitted_movements: [W_L, E_L], amber_time: 3, all_red_time: 2}
         - {id: P2, duration: 15, green_movements: [N_T, N_R, S_R], permitted_movements: [N_L, S_L], amber_time: 3, all_red_time: 2}
     ```
     The predictions (the probe's `out/synth.txt` for the approaches, steps and colours;
     `out/synth1080.txt` for the mesh and the perspective dot):
     - **4 approaches**, in this order, and their dots: `L_EC` (10, 2), `L_NC` (−1.75,
       13.75), `L_SC` (3.75, −13.5), `L_WC` (−12, −1.75), each within 1e-9 m. `L_EC`'s is at
       its link's end, 5 m ahead of its lane 0's own line (its 5 m delta);
     - **the changes**, by step, the same for `L_EC` and `L_WC`: green 0, amber **200**,
       red **230**, green **451**, amber 650, red 680, green **901**; for `L_NC` and
       `L_SC`: red 0, green **250**, amber **401**, red **431**, green 700, amber **851**,
       red **881**. So 40.1, 45.1, 85.1 and 90.1 s, not the plan's 40, 45, 85 and 90
       (§2.5.3); and green again at 45.1 s, not at 60 + 7;
     - **another timestep** (*2026-10-09, round 1*: so that a build which ignores the `dt`
       it is given fails): the same plan with `dt` 0.25, which is exact in binary, and
       `duration` 100 is 400 steps, and the changes fall on the plan's seconds. `L_EC` and
       `L_WC`: green 0, amber **80**, red **92**, green **180**, amber 260, red 272, green
       360; `L_NC` and `L_SC`: red 0, green **100**, amber **160**, red **172**, green 280,
       amber 340, red 352. (Not from the probe: from a re-implementation of `advance`'s
       arithmetic, which gives the 0.1 s steps above exactly.)
     - **`colours_at`** at 0, 10, 19.95, 20, 22.99, 23, 24, 25, 39.99, 40, 43, 44.99, 45,
       46, 60 and 61 s: `[G, R, R, G]` ×3, `[A, R, R, A]` ×2, `[R, R, R, R]` ×2,
       `[R, G, G, R]` ×3, `[R, A, A, R]`, `[R, R, R, R]` ×2, `[G, R, R, G]` ×3;
     - **the mesh** at `k` 0.05, 1920×1080: **388 vertices, 384 triangles** (97 and 96 a
       dot); every vertex within 1e-9 m of its circle (`4.5·k` or `5.5·k` from its dot) or
       at its centre, at height 0.03 m;
     - **a perspective dot** (pose `cx` 0, `cy` 0, `height_m` 60, yaw 30, pitch 30):
       every vertex projects (`camera::project`) within 1e-6 px of a circle of 4.5 or
       5.5 px about the centre's projection, and the lowest vertex is 0.03 m above the
       road, within 1e-9 m;
     - **`run_clock`,** on a temporary `results.db` the test writes with `rusqlite`, one
       completed row each:
       - a micro row with `timestep` 0.1 and `duration` 100: `(0.1, 100.0)`;
       - a micro row whose `resolved_config` has no `sim_params.timestep`, and one where
         it is 0: §2.13's timestep message;
       - **a row as the meso runner writes it** (`mode` `meso`; `resolved_config`
         `{"scenario": …, "mode": "meso", "sim_params": {"duration", "warm_up",
         "random_seed"}}`, with no `timestep`): §2.13's **mode** message, "the run is
         meso", and not the timestep one.

     Every number is `==` where exact (the counts, steps, colours) and within the stated
     tolerance elsewhere. **Without the feature it fails:** there are no dots, no timeline
     and no error.
  6. **The fixtures, against the engine** (`tests/signals.rs`, ignored: needs both
     fixtures):

     | | Midtown | urban_grid |
     |---|---|---|
     | Signalised junctions; phases per plan | **83**; 2 at 81, 1 at 2 | **9**; 2 at 9 |
     | Approaches = dots; none left out | **188** | **36** |
     | Steps; changes kept | **12,000; 11,708** | **3,000; 576** |
     | Run lengths (s): green; amber; red after an amber; the red an approach starts the run with | 25; 3; 2 and 32; **30, at 91 approaches** | 25; 3; 32; **30, at 18** |
     | FCD times bit-equal to a `T_k` | **1,199 of 1,199** | **291 of 291** |
     | `colours_at` against a second `SignalState` stepped alongside, at every FCD time and 0.05 and 0.5 s after it | **676,236 equal, 0 differ** | **31,428, 0** |
     | Dots at their lane-0 stop line's front midpoint (within 1 mm) | **188 of 188** | **35 of 36**; the 36th at 5.0000 m |
     | (approach, phase) pairs: green by permitted alone; green with a movement in no list | 0 of 372; **2** | 0 of 72; 0 |

     The timestep and duration come from `inputs::run_clock` on each fixture's
     `results.db` (0.1 and 1200; 0.1 and 300). A run length is the time from one entry of
     an approach's `changes` to the next, so each approach's last run, cut off by the
     run's end, is not counted; the red from step 0 is the fourth figure *(2026-10-09,
     round 1: the draft's row left it out)*. The stop line's front midpoint is §2.9's (the `Kind::Stop` quad's centroid,
     0.2 m along the heading). The probe's kept `out/stats-*.txt` predate that row's
     line: they hold "dots off lane 0's own stop line: 0" for Midtown and the one 5 m
     delta for urban_grid, which say the same.

     Also printed and recorded, not asserted: the build time of the timeline (§2.6).
  7. **The replay against the FCD** (`tests/signals.rs`, ignored), §2.7's check on each
     whole run:
     - **junction entries:** Midtown **7,536**: green 6,896, amber 570, amber then red 70,
       **red throughout 0**; urban_grid **422**: 373, 49, 0, **0**;
     - **first stopped boxes:** Midtown **2,490** episodes (48,896 rows), first row red /
       amber / green **2,301 / 27 / 162**, next row green / amber / red / none **2,406 /
       10 / 7 / 67**, **2,194** moving off 1.10 s after green; urban_grid **129** (2,555),
       **118 / 0 / 11**, **107 / 0 / 0 / 22**, **96** moving off 0.10 s after green;
     - **the control,** by §2.7's two formulas: with `S_{k+10}` read in place of `S_k`
       (the dots 1 s early), Midtown has **70** entries on red throughout; with each
       junction's offset applied (`S_{k − round(offset/dt)}`, clamped at `S_0`), urban_grid
       has **90**. (With `S_{k−10}`, the wrong sign, Midtown has 0.)
  8. **The colours and the place** (`tests/signals.rs`): each copied colour is its
     `colors.ts` hex (`==`), `Colour` maps `"green"`, `"amber"`, `"red"` and anything else
     to red; `scale` is 2/3, 1 and 2 at 1280×720, 1920×1080 and 3840×2160; and (ignored,
     the fixtures) each dot equals `Placement::place(link, max(L − stop_line_offset, 0), 0)`
     for its approach, bit for bit.
  - **The dots — through the GPU:**
  9. **The synthetic crossing, drawn** (`tests/signals.rs`, ignored, no fixture). Gate 5's
     network through `Renderer::new` with `scene::Camera { cx: 0, cy: 0, k: 0.05 }` at
     1920×1080, a pool of 1, streets on, no buildings, no credit, no box: once without
     dots, then with them at `t` = 10, 21 and 30. At each dot's centre pixel, its colour
     exactly (green, red, red, green; amber, red, red, amber; red, green, green, red).
     Over the frame: **448** pixels change at each `t`; exactly **104** of each of its two
     fill colours and **32** black (four dots of 52 and 8; the other 208 are MSAA blends at
     the rims, and no two dots overlap at this `k`). Then `Renderer::new_perspective`,
     with `t` = 10's colours and **each dot drawn alone** (a `DotsData` of that one dot),
     at three poses: gate 5's (`cx` 0, `cy` 0, `height_m` 60, yaw 30, pitch 30); the same
     with pitch 90; the same with `height_m` 400 and pitch 25. Each of the twelve: the
     pixels of its exact fill or black number **60–64**, in a box **9 to 11 px each way**,
     whatever its distance (64 to 493 m). *(2026-10-09, round 1: the draft's "10×10 or
     10×11" was the pitch-30 pose's alone; the probe's `out/synth1080.txt` has 10×10 at
     pitch 90, and 11×10, 10×10, 11×9 and 10×10 at pitch 25, as §2.11's table.)* Without
     the feature, no pixel changes. These are OQ-1's (a) numbers (answered 2026-10-09,
     decision 4).
  - **The dots — renders:**
  10. **On, deterministic** (`scripts/gates-signals.sh`). Each twice, each pair equal:
      urban_grid's default and `--camera tests/flight.toml` **8700 of 8700**; Midtown's
      `--from 300 --to 360 --speedup 1` orthographic with and without `--buildings`,
      `--camera tests/city-flight.toml` with and without `--buildings`, and `--camera
      tests/see-through-flight.toml` with `--buildings`: **1800 of 1800** each (`ffprobe`
      `1920,1080,30/1,1800`). **The look changed:** each first render differs from its
      copy in `scratch/out/look-p6/` in at least one frame, the count recorded. (The
      probe: every pair equal; against Phase 6's, Midtown's five differ in 1800 of 1800,
      urban_grid's default in 8700 of 8700 and its flight in 8,152 of 8,700.)
  11. **The copy is traceable** (by reading, recorded). `src/signals.rs` names `90b39292`
      and, beside each value of §2.16's list, its file and line; each line, read at the pin
      with `git show 90b39292:<file>`, holds that value.
  - **Recorded, with one bar:**
  12. **Render time.** Gate 10's wall times beside the baseline's `gates-look.sh` times for
      the same cases. Prediction: within the machine's noise; the timeline adds 0.26 s to a
      Midtown render and 0.02 s to urban_grid's, and the dots' mesh is 18,236 vertices a
      frame in Midtown. (The probe, at a load of 4–7: Midtown with dots 43.3–91.9 s, without
      62.7 and 75.5 s for the two cases it rendered both ways (ortho with buildings 60.9 and
      64.5 against 62.7; city flight 91.9 and 83.9 against 75.5); urban_grid 179.5–197.5 s.)
  13. **`view --bench 20`** on Midtown at the default window with `--buildings` (streets,
      see-through and signals on), and with `--no-signals`. Record the JSON and the load
      average.
      - **A `mean_fps` below 30 with signals on stops the build**, and the phase goes back
        to review.
      - Prediction: at least 30, likely 60 (vsync). The probe, at a load of 4–6,
        interleaved: with dots **60.00 and 60.00 fps**, median 16.68–16.70 ms, p99 18.1 ms;
        without, 57.30 (one 1,061 ms stall) and 59.95.
  - **The user's check:**
  14. **The user watches** gate 10's first renders with dots (`scratch/out/signals/`)
      against Phase 6's (`scratch/out/look-p6/`). Times are the video's, from 0:00; colours
      as §2.10. In Midtown's videos 0:00 is 300 s of the run, and every signal of the
      network switches at the same moments (§2.5.1): the greens turn amber at **0:25**, red
      at **0:28**, the dots that were red turn green at **0:30**, amber at **0:55** and red
      at **0:58**.
      - **Midtown, orthographic with buildings (`signals-ortho-city.mp4` against
        `look-ortho-city.mp4`).** At every crossing of the street grid, small round dots,
        about 11 px across with a thin dark rim, at the crossing's edge where the
        approaching lanes meet it: two at most crossings, a few with three or four,
        touching or overlapping. From 0:00 each crossing shows one green and one red (or
        more), but for two crossings whose dots are all green. At 0:25 every green dot in
        the frame turns amber at once, at 0:28 red, and **for two seconds every dot in the
        frame is red**, those two crossings' too (§2.5.2). At 0:30 every dot that was red
        before 0:25 turns green at once, and the two crossings' dots turn green again with
        them. The tiny boxes queue behind the red dots and pass the green ones; where they
        reach a dot they show over it. A few dots, 7 of the 188, lie under a roof, and 2
        more partly (OQ-4, answered as drafted). Nothing else differs from Phase 6's video.
      - **The city flight with buildings (`signals-city.mp4`).** From 0:00 to 0:10, high and
        straight down: as the orthographic video. Then, as the camera comes down and tilts,
        the streets grow but every dot stays the same size: at the top of the frame, far
        off, the dots of a crossing are as big as those of the crossing below the camera.
        A dot behind a tower is hidden and appears as the camera passes; a far dot, which
        is metres across there (§2.11), may be cut by a box or a wall beside it. The same switch at
        0:25, 0:28 and 0:30. From about 0:40 the camera looks across the park's dark
        ground, and few dots are in the frame.
      - **The orbit with buildings (`signals-orbit.mp4`), the crossing in the middle of the
        frame.** *(2026-10-09, round 1: the draft said four dots, two green and two red.
        The crossing is N523, 25 m from the flight's look-at point, and it has three
        signalised approaches: the one-way avenue's, and the cross street's two. The
        probe's `frames/orbit-crop-5.png` shows them.)* **Three dots,** one at each
        approach's white bar, standing a little above it. From 0:00 to 0:25, the one on
        the avenue is green and the two on the cross street red, with the cross street's
        cars waiting behind the red ones. At 0:25 the green turns amber, at 0:28 red. At
        0:30 the cross street's two dots turn green, and its first waiting box moves off
        about a second later. As the camera circles, each dot stays round and the same
        size; it is hidden where a box or a building stands in front of it, or, about 2.5 m
        across at this distance (§2.11), is cut by one beside it. The
        crossings farther off, in the top of the frame, show their dots as small as the
        near ones.
      - **urban_grid's `--camera` render (`signals-ug-camera.mp4`).** 0:00 is 9.1 s of the
        run; the switches come at **0:15.9** (amber), **0:18.9** (red), **0:20.9** (the
        others green), then every 30 s on the same pattern (0:45.9, 0:48.9, 0:50.9, …).
        Each of the nine crossings has four dots in a small ring around its centre, one on
        each approach. From 0:00, at eight crossings the dots left and right of the centre
        (the east–west street) are green and those above and below red; the centre
        crossing is the other way round. From 0:55 to 2:11, following the car at a
        `height_m` of 120 then 60: the dots of the crossing ahead stand at its bars, the
        same size as the camera comes down; between 1:31 and 1:33 and between 2:05 and
        2:22 no crossing is in the frame, and the video is Phase 6's there.
      - **In `view` on Midtown:** at launch, dots at every crossing, smaller (the 1280×720
        window: about 7 px across); playing, they switch as above; dragging the slider back
        and forth, they follow the time both ways at once; `L` hides them and shows them;
        `view --no-signals` opens without them, and `L` shows them.

      Then say whether OQ-1's and OQ-4's answers (the size, and the dots under roofs) stand
      as they look.
  - **The flag and the key** (*2026-10-09, round 1*; numbered after gate 14 so that no
    gate is renumbered; it runs with gates 5 and 10, before the user's check):
  15. **Their edges.**
      - **`L`, headless** (`tests/signals.rs`, not ignored), written as `tests/streets.rs`'s
        `gate12_m` with `l` for `m`: `signals_shown` is off at `ViewState::new`; a frame
        with `l` pressed turns it on, and another turns it off; then playing and
        following with `K` pressed in the same frame, through a right-button orbit, a
        drag and a scrub of the slider, every other field of the state equals that of a
        twin stepped without `l`.
      - **The CLI** (`scripts/gates-signals.sh`). Each `view` runs under `perl -e 'alarm
        60; exec @ARGV'`, as `gates-streets.sh` gate 11's.
        - `render … --signals`: exit 2, and no video. (So it is today: the case guards
          against the flag being added, not against the feature being absent.)
        - `view --no-signals --bench 1` on urban_grid: exit 0, and the bench JSON on the
          last line of stderr.
        - Two copies of `scratch/urban_grid` in `scratch/out/signals/`, each with its
          `results.db` edited by `sqlite3` (`PRAGMA journal_mode = DELETE;` first: the
          fixture is in WAL mode, and an `immutable=1` reader does not read a `-wal`. Its
          `-wal` is empty and `sqlite3` checkpoints on close, so the edit shows without
          the pragma too; it is kept so that nothing rests on that):
          - **`meso/`**: `UPDATE runs SET mode = 'meso', resolved_config =
            '{"scenario":"baseline","mode":"meso","sim_params":{"duration":300.0,"warm_up":30.0,"random_seed":42}}'`,
            the meso runner's shape (§2.4) less its `meso` key, which nothing here
            reads;
          - **`noclock/`**: `UPDATE runs SET resolved_config =
            '{"scenario":"baseline","sim_params":{"duration":300.0,"warm_up":30.0,"random_seed":42}}'`,
            `mode` left `micro`.
        - `render` on `meso/`: exit 1, stderr's one line is §2.13's mode error ("the run
          is meso"), and no video. `render` on `noclock/`: exit 1, the timestep error, no
          video.
        - `render --no-signals --from 10 --to 20 --speedup 1` on `meso/`: exit 0 and
          `ffprobe` `1920,1080,30/1,300` (§2.19: so it is today).
        - `view --bench 1` on `meso/`: exit 1 and the mode error. `view --no-signals
          --bench 1` on `meso/`: exit 0 and the bench JSON.

      Without the feature: `--no-signals` is itself rejected, and `render` on `meso/`
      exits 0. With the two checks in the draft's order, `render` on `meso/` gives the
      timestep error and fails the case.
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | `--no-signals`: the 29 reference renders | 8700 of 8700 / 1800 of 1800 against the ten references | 1 |
  | `--no-signals`: Phase 6's seven cases | 8700 / 1800 of 1800 against `look-p6` | 2 |
  | Other tests and scripts; diff | baseline numbers; equal pairs; the listed files | 3 |
  | Packages | 0 | 4 |
  | Synthetic: approaches, dots, change steps (at `dt` 0.1; at 0.25), colours, mesh | 4; as listed; 200/230/451…, 250/401/431…; 80/92/180…, 100/160/172…; 388 and 384 | 5 |
  | `run_clock`: a micro row; no timestep, or 0; a row as the meso runner writes it | (0.1, 100); the timestep message; the mode message | 5 |
  | Fixtures: junctions, dots, steps, changes, approaches that start red, FCD times, against `SignalState` | 83 / 9; 188 / 36; 12,000 / 3,000; 11,708 / 576; 91 / 18, for 30 s; all bit-equal; 0 differ | 6 |
  | Junction entries on red throughout; first stops; control | 0 of 7,536 / 0 of 422; 2,490 / 129 as listed; 70 and 90 | 7 |
  | Colours, `scale`, the dots' place | the copied hex; 2/3, 1, 2; bit for bit | 8 |
  | Synthetic pixels: changed, fill, black; perspective | 448; 104 + 104; 32; 60–64 px a dot, 9–11 px across | 9 |
  | Seven renders twice; against Phase 6 | equal; differ in ≥ 1 frame | 10 |
  | Copied values at their lines | each holds | 11 |
  | `view --bench 20` with signals | ≥ 30 fps | 13 |
  | `L`; `--signals`; a meso row and one with no timestep; the meso row with `--no-signals` | flips only its flag; exit 2; exit 1 with its own message; exit 0 (`render`: 300 frames) | 15 |
- **Not predicted, and so not gated:**
  - the look: OQ-1's size and OQ-4's dots under roofs as answered, the dot's colours
    beside the slow boxes', and how overlapping dots read, for the user at gate 14;
  - render times (gate 12), and `view`'s frame rate above 30 (gate 13);
  - which of two overlapping dots is on top in the orthographic frame (§2.12: the later
    in approach order; seen in the probe at `k` 3, and gated only as equal pairs, gate 10);
  - a real meso run: none exists here, and gate 15's is a micro run's files with an
    edited row; a forced run: not readable at the pin (OQ-2, OQ-3).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-003-phase-1`), one push. The commits:
    - `src/signals.rs`, `run_clock` in `src/inputs.rs`, `src/draw.rs`, `src/render.rs`,
      `src/lib.rs`, `src/main.rs`'s `render` flag, `tests/signals.rs`'s headless gates
      (5–8), and the 29 lines of §2.14 with their comments. The look changes here, since
      signals are on by default, and the reference scripts keep passing in the same
      commit: they need `render`'s flag only;
    - `view`: `src/view/mod.rs`, `src/view/state.rs` and `src/main.rs`'s `view` flag (`L`,
      the dots, `--no-signals`), and gate 15's `L` test;
    - `tests/signals.rs`'s GPU gate 9 and `scripts/gates-signals.sh` (gates 2, 10 and
      15's CLI cases, and gate 14's renders). Gates 1–4, 9–13 and 15 run after this
      commit;
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **`rules/signals.md`** (new, `max_lines: 40`): the timeline and its clock rule, the
      dots' place, look, size in both cameras, depth, the copied values with their lines,
      the flag, the key and the errors. `sources`: `src/signals.rs`, `src/inputs.rs`,
      `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/view/mod.rs`,
      `src/view/state.rs`, `src/main.rs`;
    - `rules/render.md` (60 of 60): the CLI line gains `--no-signals`, and the scene one
      line pointing to `rules/signals.md`; cut to fit, the cap not raised;
    - `rules/view.md` (60 of 60): the CLI gains `--no-signals`, the drawing `L`, the frame
      order `L` after `M`; cut to fit;
    - `rules/camera.md` (60 of 60): "What each path draws" names the dots (flat in the
      orthographic path, facing the camera in perspective); cut to fit;
    - `rules/inputs.md` (50 of 50): the checks gain the run's mode and clock, with signals
      on; cut to fit;
    - `rules/streets.md`, `rules/buildings.md`, `rules/see-through.md`, `rules/parts.md`,
      `rules/credit.md`, `rules/motion.md`, `rules/slider.md`: none needed, since nothing
      they say changes;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README:** the summary gains a sentence on vis-003's dots; the `render` and the
      `view` usage blocks each gain `[--no-signals]`; a **Signals** bullet after
      **Streets** says what is drawn, that it is the engine's own signal code replayed,
      `--no-signals`, and the two errors a run can now give with signals on (not micro;
      no readable timestep); `view`'s keys gain `L`; the gate list gains
      `scripts/gates-signals.sh` (which needs `sqlite3`) and `cargo test --release
      --test signals -- --include-ignored --test-threads=1`; the paragraph after it ("Streets are on by
      default, so the renders … pass `--no-streets`") says the same of signals and
      `--no-signals`;
    - **vis-001 §2.7 item 5:** one dated note: narrowed to vis-003 (signals, Phase 1; a
      HUD and link colours, its roadmap; the legend and chart not planned);
    - **vis-002 §2.17.7, §2.17.13, §2.18.2, §2.18.6 and §2.18.13:** a dated note each,
      where it says a signal colour will paint the stop lines: the data item draws dots at
      the stop lines, as the dashboard does, and the stop lines stay white;
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-003.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

### Phase 2 — HUD: the clock, the vehicles on the road and their mean speed, top-left, by default
*Produces the observable: yes. `render` writes the run's video with, in the top-left
corner of every frame, the frame's sim clock, the vehicles on the road at that time and
their mean speed, from the run's FCD, in the orthographic render and in keyframed flights.
With `--no-hud` the video is byte-identical to Phase 1's (gates 1 and 2).*

Drafted 2026-10-19; the design is §2.20, and the user's decisions are §2.2's decision 1 and
§2.20.1. Phase 2 builds on Phase 1 (shipped 2026-10-18) at engine `90b39292`. With
`--no-hud` it changes no output. With the HUD on it adds one UI tree in the top-left corner
and changes nothing else drawn. OQ-5 (the unit) and OQ-6 (the look) are the user's; the
scope and every gate are written with their recommendations, km/h and the outline at 1/36
on one line with "vehicles", and say what an answer changes. *(2026-10-19, user)* Both are
answered as recommended (OQ-5 (a); OQ-6 (a)–(d)), so the scope and the gates stand as
drafted.

- **Scope:**
  - **`src/hud.rs`** (new; no Bevy types):
    - `RATIO` 36, `FILL` sRGB (240, 240, 240) and `OUTLINE` (0, 0, 0) (§2.20.9);
      `size(w, h) = max(1, round(min(h, 9w/16)/RATIO))`, the margin `m = size` and the
      outline offset `max(1, round(size/12))`;
    - `numbers(&Fcd, t, drawn: bool) -> (usize, Option<f64>)`: §2.20.4's count and §2.20.5's
      mean in m/s, from `Fcd::snapshot_at(t)`; `(0, None)` when `drawn` is false or there
      is no snapshot;
    - `clock(t) -> String` (§2.20.6) and `text(t, n, mean) -> String` (§2.20.8): km/h as
      `⌊3.6·v + 0.5⌋`, `–` with no speed, "vehicle" for one;
    - `GLYPHS`: §2.20.9's string `0123456789:·– vehiclsmp/hk`, every character `text` can
      write and the `p` of mph, as the probe measured; laid out once at spawn.
  - **`src/draw.rs`:** `spawn_hud(world, camera: Option<Entity>, font, size) -> HudNodes`
    (a root at `left: m, top: m`, with `UiTargetCamera` when `camera` is given; the fill
    and eight outline copies as `spawn_credit`'s, each `TextLayout::no_wrap()` and
    `font_features` `[FontFeatureTag::TABULAR_FIGURES]`, holding `GLYPHS` until set);
    `set_hud_text` (all nine, only where the string differs) and `set_hud_size` (font size,
    margin and offset, for `view`'s resize).
  - **`src/render.rs`:** `Renderer::set_hud(on)` spawns the tree into the image camera, or
    despawns it, and settles again, as `set_streets`; `set_hud_text(&str)`;
    `hud_box() -> Option<[u32; 4]>`, the fill's laid-out rectangle grown by the offset and
    rounded outward, as `credit_box`; `hud_text() -> Option<String>`, the fill node's text.
    *(2026-10-19, round 1)* `Renderer` gains a field for the image camera's entity, which
    `build` holds today only in a local (and in `perspective` on that path), and loads
    `draw::CREDIT_FONT` itself in `set_hud`, since `spawn_credit` keeps no handle.
  - **`src/lib.rs`:** `Job::set_hud(on) -> Result<()>`; every `prepare*` builds a job with
    it off. `render_at(t)` sets the text from `hud::numbers(&self.fcd, t,
    !boxes.is_empty())` after the boxes are taken and before the frame renders.
    `Job::hud_text()` and `Job::hud_box()` read back what the last frame drew, for the
    gates.
  - **`src/view/mod.rs`, `src/view/state.rs`:**
    - `ViewOptions` gains `hud` (true unless `--no-hud`), as `signals`;
    - `hud_shown` in the state, off at `ViewState::new`, set from `hud` at launch; `Pressed`
      gains `h`, and `H` (`KeyCode::KeyH`) flips `hud_shown` after `L`;
    - the tree spawned at launch into the window, its size from the window's logical size,
      re-set when that changes; while shown, the text for `t` every frame, as `render_at`
      sets it, in `--bench` too; otherwise hidden.
  - **`src/main.rs`:** `--no-hud` on `render` and `view`, with help text; `render` calls
    `job.set_hud(true)?` unless it is given; `view` passes it to `ViewOptions`. No other
    flag or default changes.
  - **Tests.** `tests/hud.rs` (new):
    - headless, not ignored: gate 5, and gate 13's `H`;
    - headless, needing the fixtures, so ignored: gate 6's every-frame part, through
      `run::load` and `Run::boxes_at` with the frame times of `clock::{default_speedup,
      frame_count, frame_time}` as `tests/camera.rs` uses them; the run is loaded as
      `tests/signals.rs` loads its fixtures (no `Job`, which needs the GPU);
    - through the GPU, ignored, needing the fixtures: gate 6's sampled frames, gate 7 and
      gate 9. Gate 7's tabular widths set their strings on a `Renderer` the test builds
      (`Renderer::new(&strips, cam, 1, None, None)`, as `tests/signals.rs`), with
      `set_hud(true)`, `set_hud_text(s)` and `render(&[])`; no `Job` method beyond the
      scope's is added.
  - **Scripts.** `scripts/gates-hud.sh` (new, offline): gates 2 and 8, gate 13's CLI cases,
    and the renders for gate 12, into `scratch/out/hud/`. The 29 lines of §2.20.12 and
    `gates-signals.sh`'s `off-` line gain `--no-hud`, and one comment in each of the five
    scripts says so: the comment above them in the four, and `case_`'s first line in
    `gates-signals.sh` (§2.20.12).
  - **Not edited:** every other test file and script; every source file not named above;
    `Cargo.toml` and `Cargo.lock`; `assets/`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid and Midtown at engine `90b39292` (FCD SHA-1s `3ad76744…` and `01becc86…`),
  baseline, seed 42, with Midtown's buildings cache from vis-002 Phase 2
  (`scratch/midtown/buildings.geojson`, SHA-256 `f241ccbd…`). Everything runs offline. The
  predictions come from §2.20.15's probe.

  **Baseline, before any change**, at `origin/main`:
  - **The machine:** `ps` for other builds and engine runs, and the load average, recorded,
    here and again before the gate run; if another project is building or running, the run
    waits and the user is told.
  - **The ten references** `scratch/ref-pin90b39292-*.framemd5` are present with their
    SHA-256s (`cdad89d1…` camera, `dcfdd2b7…` default, `f6255241…` ortho-city, `2139c319…`
    ortho-roads, `d6641985…` flight-city1 and city-off, `020475b7…` flight-roads1,
    `d7118c3a…` city-on, `af80a068…` orbit-off, `05d7d174…` orbit-on). If one is missing or
    differs, the run stops.
  - **Phase 1's renders are copied** before anything runs, since `gates-signals.sh` writes
    into `scratch/out/signals/`: its fourteen `signals-*.framemd5` and the four videos of
    Phase 1's gate 14 (`signals-ortho-city.mp4`, `signals-city.mp4`, `signals-orbit.mp4`,
    `signals-ug-camera.mp4`) go to `scratch/out/signals-p1/` (`cp -p`, then read-only),
    with their SHA-256s recorded; each pair is equal (Phase 1's gate 10).
  - **Run** every gate script and every test file with `--include-ignored
    --test-threads=1`, and keep their output. Record spec-lint and `Cargo.lock`'s package
    count (500).

  - **What must not change:**
  1. **Off, against the references.** The 29 renders of §2.20.12 with `--no-streets
     --no-signals --no-hud`: `scripts/gates.sh` passes, its default render **8700 of 8700**
     against `ref-pin90b39292-default` and its `--camera` render **8700 of 8700** against
     `ref-pin90b39292-camera` (by hand on `scratch/out/camera.framemd5`); `gates-ties.sh`
     gate 2, `gates-see-through.sh` gate 2 and `gates-parts.sh` gate 2 each **1800 of 1800**
     against their references; the recorded lines of `gates-see-through.sh` gate 7 and
     `gates-parts.sh` gate 9 equal the baseline's.
  2. **Off, against Phase 1** (`gates-hud.sh`; the second half in `gates-signals.sh`).
     Phase 1's seven cases (its gate 10's list), signals on, with `--no-hud`: urban_grid's
     default and `--camera tests/flight.toml` **8700 of 8700**, and Midtown's five **1800 of
     1800**, each against its `signals-<case>.framemd5` in `scratch/out/signals-p1/`. And
     `gates-signals.sh`'s `off-` renders, now with `--no-hud`, against `look-p6` as before
     (its own gate 2).
     (The probe, without `HUD`: Phase 1's seven cases, 8700 of 8700 and 1800 of 1800 against `scratch/out/signals/signals-<case>.framemd5`, seven of seven.)
  3. **What Phase 2 does not touch.**
     - Every test file but `tests/hud.rs`, unedited, passes with every printed number
       equal to the baseline's.
     - The renders that compare only with each other still give equal pairs and still
       differ where they differed: `gates-ties.sh` gate 9, `gates-city.sh` gate 13 and
       `gates-credit.sh` gate 14 **1800 of 1800** each; `gates-streets.sh` gates 10 and 11,
       `gates-look.sh` gate 10 and `gates-signals.sh`'s gates 10 and 15 pass. All nine
       existing scripts pass; the tenth, `gates-hud.sh`, is gates 2, 8 and 13.
     - `git diff origin/main --stat -- src/` adds `src/hud.rs` and changes `src/draw.rs`,
       `src/render.rs`, `src/lib.rs`, `src/main.rs`, `src/view/mod.rs` and
       `src/view/state.rs`; `-- scripts/` adds `gates-hud.sh` and changes the 30 lines and
       five comments of §2.20.12 alone, read hunk by hunk; `-- tests/` adds `tests/hud.rs`
       alone.
  4. **Build cost** (§2.20.14). `git diff origin/main -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The numbers — headless:**
  5. **A synthetic run** (`tests/hud.rs`, not ignored). An `Fcd` built in the test, with
     no file, its snapshots at 1, 2, 3 and 5 s (none at 4: a step with no vehicle):

     | `time` | rows (`vehicle_id`: `speed` m/s) | count; mean |
     |---|---|---|
     | 1.0 | 1: 10.0, 2: 5.0 | 2; 7.5 m/s (27 km/h) |
     | 2.0 | 1: 0.0, 2: 5.0, 3: 2.5 | 3; 2.5 m/s (9 km/h) |
     | 3.0 | 1: 0.0, 3: 2.5 | 2; 1.25 m/s (**4.5** km/h exactly) |
     | 5.0 | 4: 13.8889 | 1; 50.00004 km/h |

     `drawn` is written in the test from each vehicle's first and last row, as the motion
     rule has it (`t_first − 1e-6 ≤ t ≤ t_last + 1e-6`). The predictions, `hud::text` at:
     - 0.5 s: `0:00 · 0 vehicles · – km/h` (before the first row);
     - 1.0: `0:01 · 2 vehicles · 27 km/h`; 1.5: the same (held);
     - 1.9999995: `0:02 · 3 vehicles · 9 km/h` (within `1e-6` of the 2.0 s row, clock
       included);
     - 2.5: `0:02 · 3 vehicles · 9 km/h`: the row's 3, while vehicle 2, whose last row is
       at 2.0, is no longer drawn (§2.20.4's (a), not (b));
     - 3.0: `0:03 · 2 vehicles · 5 km/h`: 4.5 rounds up (rounding half to even would give
       4);
     - 3.5: `0:03 · 0 vehicles · – km/h`: no box is drawn, every vehicle of the 3.0 s row
       having its last row there;
     - 4.0: `0:04 · 0 vehicles · – km/h`: the empty step, which the FCD does not hold;
     - 5.0: `0:05 · 1 vehicle · 50 km/h`.

     And `hud::clock` at 0, 0.9999995, 59.999, 60, 599.5, 3599.9999, 3600 and 36000:
     `0:00`, `0:01`, `0:59`, `1:00`, `9:59`, `59:59`, `1:00:00`, `10:00:00`. Every value
     `==`. **Without the feature it fails:** there is no `hud` module.
  6. **The fixtures, against the FCD read straight** (`tests/hud.rs`, ignored). The test
     reads each fixture's parquet itself with the `parquet` crate (the columns `time`,
     `vehicle_id`, `speed`), groups it by `time`, and computes §2.20.4–§2.20.8's text at
     every frame time, with `drawn` from each vehicle's first and last row, summing each
     time's speeds in `vehicle_id` order as `rows_of` does. It compares that with
     `hud::text` through `Run::boxes_at` and `Fcd::snapshot_at` (headless; the scope's
     Tests). *(2026-10-19, round 1)* No snapshot's mean lies within 8.5e-5 km/h of a
     half, so the order of the sum cannot change a rounding:
     - **every frame** of urban_grid's default render (**8,700**), Midtown's `--from 300 --to
       360 --speedup 1` (**1,800**) and Midtown's whole run at the default speedup
       (**9,000**): **0 differ**;
     - **sampled frames, rendered** (`Job::render_at`, then `Job::hud_text()`, the text the
       frame drew), the predictions from the probe's `out/expected-*.txt`, computed from
       the FCD alone:

       | Fixture, frame (`t`) | Text |
       |---|---|
       | Midtown window, 0 (300.0) | `5:00 · 184 vehicles · 17 km/h` |
       | 3 (300.1) | `5:00 · 185 vehicles · 16 km/h` |
       | 30 (301.0) | `5:01 · 185 vehicles · 16 km/h` |
       | 450 (315.0) | `5:15 · 196 vehicles · 24 km/h` |
       | 900 (330.0) | `5:30 · 201 vehicles · 20 km/h` |
       | 1350 (345.0) | `5:45 · 206 vehicles · 23 km/h` |
       | 1799 (359.97) | `5:59 · 203 vehicles · 18 km/h` |
       | urban_grid, 0 (9.1) | `0:09 · 6 vehicles · 50 km/h` |
       | 29 (10.07) | `0:10 · 6 vehicles · 50 km/h` |
       | 600 (29.1) | `0:29 · 18 vehicles · 42 km/h` |
       | 2700 (99.1) | `1:39 · 66 vehicles · 33 km/h` |
       | 5400 (189.1) | `3:09 · 96 vehicles · 30 km/h` |
       | 8699 (299.07) | `4:59 · 107 vehicles · 21 km/h` |

       Midtown's frame 1799 counts 203, the 359.1 s row's, while 202 boxes are drawn
       (§2.20.4); urban_grid's frame 8699 is before its last row (299.1 s) and shows the
       298.1 s row's numbers.
     - **the rule for an empty network never fires** in the three windows: 0 frames with no
       box drawn and a non-empty snapshot.
  - **The HUD — through the GPU:**
  7. **Drawn, alone** (`tests/hud.rs`, ignored: urban_grid's fixture; GPU). urban_grid's
     orthographic job (`Job::prepare_with`, no buildings; streets, see-through and signals
     on, as the probe's job had them) at 150 s, whose top-left corner holds only the
     background (no dot of 36, no box), at 1280×720, 1920×1080 and 3840×2160: the frame
     without the HUD, then with `set_hud(true)`, whose text there is `2:30 · 83 vehicles ·
     30 km/h`. "Exactly the fill" counts the frame with the HUD (the frame without has
     none); "exactly black, added" is the whole frame's count with the HUD less without
     (the frame without the HUD has a few exactly black pixels already, all outside the
     box: 4, 317 and 2,858):

     | Frame | `hud_box()` | Pixels changed, all inside it | Exactly the fill (240, 240, 240) | Exactly black, added |
     |---|---|---|---|---|
     | 1280×720 | `[18, 18, 269, 46]` | **3,915** | **340** | **941** |
     | 1920×1080 | `[27, 27, 403, 69]` | **8,384** | **1,264** | **3,087** |
     | 3840×2160 | `[55, 55, 805, 137]` | **31,453** | **8,163** | **14,795** |

     - the box's corner is `(m − o, m − o)`: `m` 20, 30 and 60 px, `o` 2, 3 and 5; its
       height is the line box `1.2·S` plus `2o`: 28, 42 and 82 px;
     - **no pixel outside the box changes;**
     - **tabular figures**, at 1920×1080: `1:11 · 111 vehicles · 11 km/h`, `8:88 · 888
       vehicles · 88 km/h`, `0:00 · 000 vehicles · 00 km/h` and `4:07 · 205 vehicles · 23
       km/h` lay out **387 px** wide each, `==`, read as `hud_box()`'s width less `2o`
       (`[27, 27, 420, 69]`: 393 − 6), through the scope's `Renderer` route (without `tnum`
       the probe measured 358, 389, 390 and 378);
     - **without the feature, no pixel changes.**

     The counts are `==`: the probe's, through the same Bevy, font and GPU. They depend on
     OQ-6's answer: another treatment, size or layout is measured again before the gate is
     run, and recorded. *(2026-10-19, user)* OQ-6 is answered as these counts were
     measured (the outline, 1/36, one line), so they stand.
  8. **On, deterministic** (`scripts/gates-hud.sh`). Phase 1's seven cases, each twice,
     each pair equal: urban_grid's default and `--camera tests/flight.toml` **8700 of
     8700**; Midtown's `--from 300 --to 360 --speedup 1` orthographic with and without
     `--buildings`, `--camera tests/city-flight.toml` with and without `--buildings`, and
     `--camera tests/see-through-flight.toml` with `--buildings`: **1800 of 1800** each
     (`ffprobe` `1920,1080,30/1,1800`). **The look changed:** each first render differs
     from its `signals-<case>.framemd5` in `scratch/out/signals-p1/` in **every frame**,
     since every frame draws the HUD. (The probe: every pair equal, 8700 of 8700 and 1800 of 1800; against Phase 1's renders, 0 frames equal in each of the seven.)
  9. **A frame does not depend on the frames before it** (`tests/hud.rs`, ignored, the
     fixtures; GPU). Each job is set as `render` sets it (`src/main.rs`): streets,
     see-through, signals and the HUD on, as the probe's were. Each of these frames,
     rendered alone by a new job, equals the same
     frame rendered in sequence from frame 0, byte for byte: urban_grid's default render,
     frames 0, 1, 30, 299 and 300; Midtown's city flight with `--buildings`, frames 0, 29,
     30 and 299 of `--from 300 --to 310 --speedup 1`. It is what `view`'s slider and
     `Shift+←` rely on, and what a render's first frame shares with every later one. (The
     probe: 9 of 9 equal, 0 pixels differ.)
  - **Recorded, with one bar:**
  10. **Render time.** Gate 8's wall times beside the same run's `--no-hud` renders of the
      same cases (gate 2), each with the load average after it. Prediction: within the
      machine's noise; the HUD sets a string of about 30 characters on nine text nodes once
      per frame, and lays it out only when it changes. (The probe: wall seconds, the 1-min load before and after each:

      | Case | HUD, twice | `--no-hud`'s probe twin (no `HUD`) |
      |---|---|---|
      | ortho-city | 53.5 (58 → 26), 54.0 (26 → 14) | 53.0 (14 → 10) |
      | ortho-roads | 36.6, 38.4 (8–10) | 43.5 (9 → 8) |
      | city | 61.6, 61.4 (8) | 65.5 (8 → 11) |
      | city-roads | 44.3, 50.9 (9–11) | 47.1 (9) |
      | orbit | 66.3, 73.7 (9 → 16) | 64.5 (16 → 11) |
      | ug-default | 184.6 (11 → 8), 201.4 (8 → 6) | 152.7 (6 → 4) |
      | ug-camera | 150.9, 148.1 (3) | 151.9 (3) |

      The one gap above the noise is urban_grid's default render, 21–32 % slower with the
      HUD, at a falling load; its `--camera` twin, at a steady load of 3, shows none.
      Gate 10 records it again beside the same run's `--no-hud` renders.)
  11. **`view --bench 20`** on Midtown at the default window with `--buildings` (streets,
      see-through, signals and the HUD on), and with `--no-hud`, in two passes fixed
      before the first begins, each the HUD on then `--no-hud`, back to back, the load
      average before and after each. Record the JSON.
      - **A `mean_fps` below 30 with the HUD on stops the build**, and the phase goes back
        to review. It is never re-run to get a pass.
      - Prediction: at least 30, likely 60 (vsync). (The probe: two passes, fixed before the first, at a 1-min load of 2.6–8.6: with the HUD 60.00 and 60.00 fps, p99 17.08 and 17.05 ms; without, 60.00 and 60.00, p99 17.12 and 17.10.)
  - **The user's check:**
  12. **The user watches** gate 8's first renders (`scratch/out/hud/`) against Phase 1's
      (`scratch/out/signals-p1/`). Times are the video's, from 0:00; the text as §2.20.8,
      in the top-left corner, light with a dark rim, 30 px tall at 1080 lines.
      - **Midtown, orthographic with buildings (`hud-ortho-city.mp4` against
        `signals-ortho-city.mp4`).** At 0:00 the corner reads `5:00 · 184 vehicles · 17
        km/h`. The clock counts up one second every second of video, to `5:59` at the end.
        The count changes about once a second, between 184 and 207. The speed climbs from
        15 at 0:02 to 25 at 0:13 while half the dots are green, stays at 23–24, then falls
        from 23 at 0:25, as every green dot turns amber and red, to 16 at 0:33, climbs
        back to 24 by 0:43 after the other dots turn green, and falls again to 18 at the
        end, with the next switch at 0:55. The digits change in place: nothing to their
        right moves when they do. The credit line, bottom-right, is unchanged. Nothing
        else differs from Phase 1's video.
      - **The city flight with buildings (`hud-city.mp4`).** The same text, the same
        numbers at the same times, in the same place, while the camera comes down and
        tilts and the roofs, streets and dark ground pass under the text.
      - **The orbit with buildings (`hud-orbit.mp4`).** The same numbers again, over the
        buildings as the camera circles.
      - **urban_grid's `--camera` render (`hud-ug-camera.mp4`).** At 0:00 the corner reads
        `0:09 · 6 vehicles · 50 km/h`. The count rises in steps of 6 every 9 seconds while
        the grid fills (12 at 0:09, 18 at 0:18), and the speed, 50 km/h until then, falls
        from 0:18 to 37 by 0:23. At the end, `4:59 · 107 vehicles · 21 km/h`.
      - **In `view` on Midtown:** at launch the corner reads `0:01 · 1 vehicle · 40 km/h`,
        smaller (the 1280×720 window: 20 px); playing, it counts as above; dragging the
        slider back and forth, the numbers follow the time both ways at once; resizing the
        window resizes it; `H` hides and shows it; `view --no-hud` opens without it, and
        `H` shows it. The readout at the bottom-left is unchanged.

      Then say whether OQ-5's and OQ-6's answers stand as they look.
  - **The flag and the key:**
  13. **Their edges.**
      - **`H`, headless** (`tests/hud.rs`, not ignored), written as `tests/signals.rs`'s
        `gate15_l` with `h` for `l`: `hud_shown` is off at `ViewState::new`; a frame with
        `h` pressed turns it on, and another turns it off; then playing and following with
        `K` pressed in the same frame, through a right-button orbit, a drag and a scrub of
        the slider, every other field of the state equals that of a twin stepped without
        `h`.
      - **The CLI** (`scripts/gates-hud.sh`). Each `view` runs under `perl -e 'alarm 60;
        exec @ARGV'`.
        - `render … --hud`: exit 2, and no video. (So it is today: the case guards against
          the flag being added, not against the feature being absent.)
        - `view --no-hud --bench 1` on urban_grid: exit 0, and the bench JSON on the last
          line of stderr.
        - `render --from 10 --to 12 --speedup 1 --width 64 --height 36` on urban_grid: exit
          0, `ffprobe` `64,36,30/1,60` (§2.20.11: a tiny frame draws a tiny HUD, no error).

      Without the feature: `--no-hud` is itself rejected, exit 2.
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | `--no-hud`: the 29 reference renders | 8700 of 8700 / 1800 of 1800 against the ten references | 1 |
  | `--no-hud`: Phase 1's seven cases; `gates-signals.sh`'s `off-` against `look-p6` | 8700 / 1800 of 1800 against `signals-p1`; as before | 2 |
  | Other tests and scripts; diff | baseline numbers; equal pairs; the listed files | 3 |
  | Packages | 0 | 4 |
  | Synthetic: nine texts (held, `1e-6`, rows not boxes, halves up, no box drawn, the empty step, one vehicle); eight clocks | as listed, `==` | 5 |
  | Fixtures: every frame's text against the FCD read straight; 13 sampled frames | 0 of 8,700 / 1,800 / 9,000 differ; as listed | 6 |
  | urban_grid's corner at 720p, 1080p, 2160p: the box; changed pixels, all in it; exact fill and black; tabular widths | as tabled, `==`; 387 px each | 7 |
  | Seven renders twice; against Phase 1 | equal; differ in every frame | 8 |
  | A frame alone against the same frame in sequence | equal, 9 of 9 frames | 9 |
  | `view --bench 20` with the HUD | ≥ 30 fps | 11 |
  | `H`; `--hud`; `view --no-hud`; a 64×36 render | flips only its flag; exit 2; exit 0; exit 0, 60 frames | 13 |
- **Not predicted, and so not gated:**
  - the look: OQ-5's unit and OQ-6's treatment, size, layout and words as answered, and
    how the text reads over each background, for the user at gate 12;
  - render times (gate 10), and `view`'s frame rate above 30 (gate 11);
  - the HUD's glyphs pixel for pixel: gate 7 counts its fill and black pixels and bounds
    them, and gate 8 checks that two renders agree.
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-003-phase-2`), one push. The commits:
    - `src/hud.rs`, `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/main.rs`'s `render`
      flag, `tests/hud.rs`'s headless gates (5, 6), and the 30 lines of §2.20.12 with their
      comments. The look changes here, since the HUD is on by default, and the reference
      scripts keep passing in the same commit: they need `render`'s flag only;
    - `view`: `src/view/mod.rs`, `src/view/state.rs` and `src/main.rs`'s `view` flag (`H`,
      the HUD, `--no-hud`), and gate 13's `H` test;
    - `tests/hud.rs`'s GPU gates (6's sampled frames, 7, 9) and `scripts/gates-hud.sh`
      (gates 2, 8 and 13's CLI cases, and gate 12's renders). Gates 1–4, 6–11 and 13 run
      after this commit;
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **`rules/hud.md`** (new, `max_lines: 40`): the numbers and their rule, the clock and
      its format, the text, the drawing (font, `tnum`, size, outline, place), the flag, the
      key, and `Job`'s accessors. `sources`: `src/hud.rs`, `src/draw.rs`, `src/render.rs`,
      `src/lib.rs`, `src/view/mod.rs`, `src/view/state.rs`, `src/main.rs`;
    - `rules/render.md` (60 of 60): the CLI line gains `--no-hud`, the scene one line
      pointing to `rules/hud.md`, and the Pipeline's per-frame bullet the HUD's text and its
      `prepare_with` bullet the HUD off by default; cut to fit, the cap not raised;
    - `rules/view.md` (60 of 60): the CLI gains `--no-hud`, the drawing `H`, the frame
      order `H` after `L`; cut to fit;
    - `rules/credit.md`: one line, the HUD in the opposite corner, drawn the same way
      (`rules/hud.md`); cut to fit;
    - `rules/signals.md`, `rules/inputs.md`, `rules/camera.md`, `rules/slider.md`,
      `rules/streets.md`, `rules/buildings.md`, `rules/see-through.md`, `rules/parts.md`,
      `rules/motion.md`: none needed, since nothing they say changes (the HUD reads no new
      input and is not part of the 3D scene);
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README:** the summary gains a sentence on the HUD; the `render` and the `view`
      usage blocks each gain `[--no-hud]`; a **HUD** bullet after **Signals** says what it
      shows and where each number comes from, and `--no-hud`; the **Signals** bullet's
      "`--no-signals` draws none, the video exactly as before" says the same now takes
      `--no-hud` too; `view`'s keys gain `H`; the
      gate list gains `scripts/gates-hud.sh` and `cargo test --release --test hud --
      --include-ignored --test-threads=1`; the paragraph on the reference renders says the
      same of `--no-hud`;
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-003.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

### Phase 3 — Link colours: each link by a measure over the past window, off by default
*Produces the observable: yes. With `--link-colours`, `render` writes the run's video with
each road strip tinted by a measure of its FCD rows over the past window ending at each
frame's time, and a legend under the HUD, in the orthographic render and in keyframed
flights. Without it the video is byte-identical to Phase 2's (gates 1 and 2).*

Drafted 2026-10-20; the design is §2.21, and the user's decisions are §2.2's 8–13. Phase 3
builds on Phase 2 (shipped 2026-10-20) at engine `90b39292`. Without `--link-colours` it
changes no output. With it, it adds one mesh and one UI line and, if OQ-8 is answered as
recommended, draws the boxes in one colour. OQ-7 to OQ-10 (where the colour goes, the
cars, the palette, the legend) are the user's; the scope and every gate are written with
their recommendations, (a) the strip tinted, neutral cars, (b) blue to red, and the legend
as drawn, and say what an answer changes.

- **Scope:**
  - **`src/links.rs`** (new; no Bevy types):
    - `Measure { Relative, Speed, Density, Stopped }`, with its CLI name (`relative`,
      `speed`, `density`, `stopped`), its edges and its legend text (§2.21.5, §2.21.9);
      `Measure::next(Option<Measure>) -> Option<Measure>` for `C`;
    - `PALETTE` (OQ-9 (b): `#2c7bb6`, `#9ecae1`, `#fdae61`, `#d7191c`, best → worst),
      `CARS` sRGB `(235, 235, 235)` (OQ-8), `STOPPED` 0.1 m/s, `LIFT_M` 0.005, `WINDOW_S` 60;
    - `Unit { link, lane: Option<u32>, length, lanes, limits, span: [f64; 2] }` and
      `Links::with_units(units, &Fcd) -> Links`, which takes every row of the file from
      `Fcd::vehicles` into (`time`, `vehicle_id`) order, its FCD times, and the interval
      `I` (§2.21.3); `Links::new(&Placement, &Fcd, per_lane)` builds the units from the
      network (link order, lanes in order) and calls it;
    - `Links::interval() -> Option<f64>` and `check_window(w, interval) -> Result<()>`,
      which raises §2.21.11's two errors, in that order;
    - `Links::values_at(t, w, m) -> Vec<Option<f64>>` (§2.21.3, §2.21.4), `step(m, v) -> u8`
      (§2.21.5's rule, 0 best), and `colours_at(t, w, m) -> Vec<Option<[u8; 3]>>`;
    - `tint(&Placement, &Links) -> TintData` (positions, each vertex's unit, each unit's
      triangles), the strips of §2.21.7 (a), and `legend(m)` (its title and four
      `(label, colour)` pairs in ascending value).
  - **`src/draw.rs`:** `spawn_tint` (one entity, hidden until set, one unlit white
    material with `cull_mode: None`, `NoFrustumCulling`), `set_tint` (replace the mesh with
    the triangles of every unit that has a colour, only when a colour differs from the last
    frame's; hidden when none has one), `despawn_tint`; `spawn_legend` (§2.21.9: a root
    under the HUD, `UiTargetCamera` when a camera is given, the outlined title and labels as
    the HUD's texts, the swatches as nodes with a background and a black border) and its
    despawn; the boxes' colour: `materials` and `boxes_mesh` take the box colour as a
    function of speed, `scene::SPEED_BINS` or `links::CARS`.
  - **`src/render.rs`:** `Renderer::set_links(Option<(TintData, legend)>)` spawns the tint
    and the legend (top `m + 1.6·S`, or `m` with no HUD) and turns the boxes neutral, or
    despawns them and restores the speed colours, then settles again;
    `set_link_colours(&[Option<[u8; 3]>])`; `legend_box() -> Option<[u32; 4]>`, as
    `hud_box`.
  - **`src/lib.rs`:** `LinkOptions { measure, window, lanes }`; `Job::set_links(Option<
    LinkOptions>) -> Result<()>`: on builds `Links::new`, runs `check_window` and tells the
    renderer; every `prepare*` builds a job with them off. `render_at(t)` sets the colours
    after the dots and before the boxes. `Job::link_colours()` reads back the last frame's.
  - **`src/view/mod.rs`, `src/view/state.rs`:**
    - `ViewOptions` gains `links: Option<Measure>`, `link_window` and `link_lanes`;
    - at launch, before the window: `Links::new`, and `check_window` when `--link-colours`
      or `--link-window` is given (its error is `view`'s); with neither, a window below
      the interval leaves `C` inert (§2.21.11);
    - `link_measure: Option<Measure>` in the state, `None` at `ViewState::new`, set from
      `links` at launch; `Pressed` gains `c`, and `C` (`KeyCode::KeyC`) sets it to
      `Measure::next` after `H`;
    - while `Some`, the colours for `t` every frame (the mesh replaced only on a change, and
      every frame of `--bench`), the boxes neutral, the legend for the measure at the
      window's size, respawned when the measure or the size changes; while `None`, hidden
      and the speed colours back.
  - **`src/main.rs`:** `--link-colours [MEASURE]` (`num_args = 0..=1`, `default_missing_value
    = "relative"`, clap's possible values), `--link-window <SECONDS>` (`f64`, default 60)
    and `--link-lanes` on `render` and `view`, with help text; on `render` the last two
    `requires = "link_colours"`; `render` calls `job.set_links(Some(..))?` when the flag is
    given. No other flag or default changes.
  - **Tests.** `tests/links.rs` (new):
    - headless, not ignored: gate 5, and gate 13's `C`;
    - headless, needing the fixtures, so ignored: gate 6, through `run::load` and
      `Links::new`, the parquet read straight with the `parquet` crate as `tests/hud.rs`
      reads it;
    - through the GPU, ignored, needing urban_grid's fixture: gate 7 (a `Renderer` the
      test builds, as `tests/signals.rs`), and gate 9 (`Job`s).
  - **Scripts.** `scripts/gates-links.sh` (new, offline): gates 2 and 8, gate 13's CLI
    cases, and gate 12's renders, into `scratch/out/links/`. No existing script line
    changes: link colours are off unless asked for.
  - **Not edited:** every other test file and script; every source file not named above;
    `Cargo.toml` and `Cargo.lock`; `assets/`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid and Midtown at engine `90b39292` (FCD SHA-1s `3ad76744…` and `01becc86…`),
  baseline, seed 42, with Midtown's buildings cache from vis-002 Phase 2
  (`scratch/midtown/buildings.geojson`, SHA-256 `f241ccbd…`). Everything runs offline. The
  predictions come from §2.21.16's probe and the FCD read straight.

  **Baseline, before any change**, at `origin/main`:
  - **The machine:** `ps` for other builds and engine runs, and the load average, recorded,
    here and again before the gate run; if another project is building or running, the run
    waits and the user is told.
  - **The ten references** `scratch/ref-pin90b39292-*.framemd5` are present with their
    SHA-256s (`cdad89d1…` camera, `dcfdd2b7…` default, `f6255241…` ortho-city, `2139c319…`
    ortho-roads, `d6641985…` flight-city1 and city-off, `020475b7…` flight-roads1,
    `d7118c3a…` city-on, `af80a068…` orbit-off, `05d7d174…` orbit-on). If one is missing or
    differs, the run stops.
  - **Phase 2's renders are copied** before anything runs, since `gates-hud.sh` writes into
    `scratch/out/hud/`: its fourteen `hud-*.framemd5` (seven cases, each twice) and the four
    videos of Phase 2's gate 12 (`hud-ortho-city.mp4`, `hud-city.mp4`, `hud-orbit.mp4`,
    `hud-ug-camera.mp4`) go to `scratch/out/hud-p2/` (`cp -p`, then read-only), with their
    SHA-256s recorded; each pair is equal (Phase 2's gate 8).
  - **Run** every gate script and every test file with `--include-ignored
    --test-threads=1`, and keep their output. Record spec-lint and `Cargo.lock`'s package
    count (500).

  - **What must not change:**
  1. **Off, against the references.** No script line changes, so the 29 renders of §2.20.12
     (`--no-streets --no-signals --no-hud`) run as they do: `scripts/gates.sh` passes, its
     default render **8700 of 8700** against `ref-pin90b39292-default` and its `--camera`
     render **8700 of 8700** against `ref-pin90b39292-camera` (by hand on
     `scratch/out/camera.framemd5`); `gates-ties.sh` gate 2, `gates-see-through.sh` gate 2
     and `gates-parts.sh` gate 2 each **1800 of 1800** against their references; the
     recorded lines of `gates-see-through.sh` gate 7 and `gates-parts.sh` gate 9 equal the
     baseline's.
  2. **Off, against Phase 2** (`gates-links.sh`). Phase 2's seven cases (its gate 8's list),
     with today's flags and no new one: urban_grid's default and `--camera tests/flight.toml`
     **8700 of 8700**, and Midtown's five (`--from 300 --to 360 --speedup 1`, orthographic
     with and without `--buildings`, `--camera tests/city-flight.toml` with and without,
     `--camera tests/see-through-flight.toml` with) **1800 of 1800**, each against its
     `hud-<case>.framemd5` in `scratch/out/hud-p2/`. (The probe, without `LINKS`: seven of
     seven, every frame equal to `scratch/out/hud/hud-<case>.framemd5`.)
  3. **What Phase 3 does not touch.**
     - Every test file but `tests/links.rs`, unedited, passes with every printed number
       equal to the baseline's.
     - All ten existing scripts pass, their logs equal to the baseline's apart from times,
       loads, fetch `"seconds"` and `--bench 1`'s JSON; `gates-ties.sh` gate 9,
       `gates-city.sh` gate 13 and `gates-credit.sh` gate 14 **1800 of 1800** each. The
       eleventh, `gates-links.sh`, is gates 2, 8 and 13.
     - `git diff origin/main --stat -- src/` adds `src/links.rs` and changes `src/draw.rs`,
       `src/render.rs`, `src/lib.rs`, `src/main.rs`, `src/view/mod.rs` and
       `src/view/state.rs`; `-- scripts/` adds `gates-links.sh` alone; `-- tests/` adds
       `tests/links.rs` alone.
  4. **Build cost** (§2.21.15). `git diff origin/main -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The measures — headless:**
  5. **A synthetic run** (`tests/links.rs`, not ignored). `Links::with_units` on four units
     and an `Fcd` built in the test, with no file:

     | Unit | Length; lanes | Limit (m/s) |
     |---|---|---|
     | A | 100 m; 2 | 10 |
     | B | 50 m; 1 | 20 |
     | C | 100 m; 1 | 8.333333333333334 (30 km/h, Midtown's) |
     | D | 100 m; 1 | 10 |

     | `time` | rows (`vehicle_id` on unit, lane: `speed` m/s) |
     |---|---|
     | 1.0 | 1 on A, 0: 10.0 · 2 on A, 1: 0.0 · 3 on B: 5.0 · 5 on C: 8.3333333330 · 6 on D: 7.4999999 |
     | 2.0 | 1 on A, 0: 5.0 · 2 on A, 1: 0.0 · 3 on B: 20.0 |
     | 3.0 | 2 on A, 1: 2.0 |
     | 5.0 | 4 on B: 0.05 |

     No row at 4.0 (a step with no vehicle): the interval is **1** (gaps 1, 1, 2). The
     predictions, each unit's steps for `relative`, `speed`, `density` and `stopped` in that
     order (0 best, 3 worst, `–` no value; `n` the window's FCD times):

     | `t`, `W` | `n` | A | B | C | D |
     |---|---|---|---|---|---|
     | 0.5, 60 (before the first row) | 0 | – – 0 0 | – – 0 0 | – – 0 0 | – – 0 0 |
     | 1, 60 | 1 | 1 2 2 3 | 2 2 3 0 | **0 0** 2 0 | **0 1** 2 0 |
     | 3, 60 | 3 | 2 2 2 2 | 1 0 3 0 | 0 0 1 0 | 0 1 1 0 |
     | 5, 60 | 4 | 2 2 2 **2** | 2 0 3 **1** | 0 0 0 0 | 0 1 0 0 |
     | 2, 1 (right now) | 1 | 2 3 2 3 | 0 0 3 0 | – – 0 0 | – – 0 0 |
     | 1.9999995, 1 | 1 | 2 3 2 3 | 0 0 3 0 | – – 0 0 | – – 0 0 |
     | 4, 1 (the missing step) | 0 | – – 0 0 | – – 0 0 | – – 0 0 | – – 0 0 |
     | 4.5, 2 | 1 | 3 3 1 0 | – – 0 0 | – – 0 0 | – – 0 0 |

     Per lane (`with_units` on A's two lanes, B, C, D), at 1, 60: A lane 0 `0 0 2 0`, A lane
     1 `3 3 2 3`; at 3, 60: A lane 0 `0 1 2 0`, A lane 1 `3 3 2 2`.
     - **What each row tests:** C's 29.9999999988 km/h and D's 74.999999 % are binned on the
       whole number shown, 30 km/h and 75 % (the bold `0`s; on the raw value they would be 1);
       at 5 s, A's 2 stopped rows over 4 times are 0.5 and B's 1 over 4 is 0.25, each on its
       edge and in the step above (bold); at 1 s A's density is 10 (2 rows, 0.2 lane-km) and
       stopped 1, on an edge; `n` counts the window's FCD times, so the missing 4.0 is not
       a time and `t` 4 has none; `1.9999995` takes the 2.0 row within `1e-6` and drops 1.0.
     - **`check_window`:** with the interval 1, `W` 0.5 and 0.999 give the shorter-than
       error naming `--link-window` and `1.000 s`; 0.9999995, 1 and 60 are taken; 0, −1,
       `NaN` and `inf` give the positive-number error; an `Fcd` with one FCD time takes 0.01.

     Every value `==`. **Without the feature it fails:** there is no `links` module.
  6. **The fixtures, against the FCD read straight** (`tests/links.rs`, ignored). The test
     reads each fixture's parquet itself (`time`, `vehicle_id`, `link_id`, `lane`,
     `speed`), the network's lengths through `Placement::link_length` and its lanes'
     limits, and computes §2.21.3–§2.21.5 at every frame time with its own loops; it
     compares each unit's step with `Links::values_at` and `step`:
     - **every frame** of Midtown's `--from 300 --to 360 --speedup 1` (**1,800**) and
       urban_grid's whole run at 1× (**8,700**), every measure, `W` = 60, per link and per
       lane: **0 differ**; and the interval, **1.000** within `1e-9` in both;
     - **the step counts** at sampled frames, `W` = 60 (no value / best / … / worst; the
       FCD read straight, `out/table.txt`):

       | Fixture, `t`; units | `relative` | `speed` | `density` | `stopped` |
       |---|---|---|---|---|
       | Midtown 300; 255 links | 67 / 60 / 40 / 62 / 26 | 67 / 70 / 39 / 60 / 19 | 0 / 143 / 59 / 42 / 11 | 0 / 175 / 34 / 37 / 9 |
       | Midtown 330 | 57 / 62 / 51 / 56 / 29 | 57 / 73 / 50 / 57 / 18 | 0 / 141 / 59 / 41 / 14 | 0 / 178 / 33 / 33 / 11 |
       | Midtown 359.97 | 60 / 57 / 51 / 61 / 26 | 60 / 64 / 55 / 60 / 16 | 0 / 134 / 65 / 44 / 12 | 0 / 169 / 43 / 33 / 10 |
       | Midtown 300; 533 lanes | 186 / 115 / 87 / 79 / 66 | 186 / 143 / 75 / 77 / 52 | 0 / 325 / 86 / 84 / 38 | 0 / 431 / 77 / 23 / 2 |
       | urban_grid 9.1; 48 links | 42 / 6 / 0 / 0 / 0 | 42 / 6 / 0 / 0 / 0 | 0 / 48 / 0 / 0 / 0 | 0 / 48 / 0 / 0 / 0 |
       | urban_grid 150 | 26 / 9 / 9 / 4 / 0 | 26 / 15 / 7 / 0 / 0 | 0 / 30 / 5 / 13 / 0 | 0 / 34 / 4 / 6 / 4 |
       | urban_grid 299.07 | 23 / 9 / 9 / 7 / 0 | 23 / 13 / 10 / 2 / 0 | 0 / 26 / 7 / 12 / 3 | 0 / 33 / 0 / 5 / 10 |
       | urban_grid 150; 95 lanes | 52 / 17 / 18 / 7 / 1 | 52 / 29 / 11 / 3 / 0 | 0 / 60 / 11 / 22 / 2 | 0 / 75 / 10 / 9 / 1 |

     (The probe's `Links::values_at`, binned on the unrounded value, equals the straight
     computation in all 12,176 values at these and four more times, §2.21.16.)
  - **The colours — through the GPU:**
  7. **Drawn** (`tests/links.rs`, ignored: urban_grid's fixture; GPU). urban_grid's strips
     through `Renderer::new` at the fit (`k` 1.1481, 1920×1080), streets on, a pool of 1,
     no buildings, no credit, no box; `set_links` with `Links::new` per link and its
     `tint`, and the legend for `relative`; the colours of `relative` at 150 s; `render(&[])`.
     First with the tint alone (`set_links` with no legend), then with the legend; no HUD,
     so the legend's top is `m`:

     | Frame | Pixels the tint changes | of them exactly a palette colour | Pixels the legend adds |
     |---|---|---|---|
     | 1280×720 | **16,246** | **11,451** | **7,116** |
     | 1920×1080 | **34,903** | **28,072** | **15,361** |
     | 3840×2160 | **137,509** | **117,697** | **57,304** |

     - **the tint stays on the road:** no pixel that is exactly the background `(18, 22,
       30)` without colours changes with the tint, at each size;
     - **at the midpoint of lane 0 of every link** (`place_lateral(link, L/2,
       lane_offset(link, 0))`, the pixel holding it): the **22** links with a value at 150 s
       show exactly their step's colour, the **26** with none exactly the road `(92, 96,
       104)`, at each size;
     - **without the feature, no pixel changes.**

     The counts are `==`: the probe's, through the same Bevy and GPU, by a `Job` whose
     renderer is this one (`Renderer::new` at the fit, no credit on urban_grid) with no box
     drawn. They depend on OQ-7's and OQ-9's answers: another place or palette is measured
     again before the gate is run, and recorded.
  - **The colours — renders:**
  8. **On, deterministic** (`scripts/gates-links.sh`). Each twice, each pair equal, every
     measure and per lane covered: urban_grid's default `--link-colours density` and
     `--camera tests/flight.toml --link-colours` **8700 of 8700**; Midtown's (`--from 300
     --to 360 --speedup 1`) orthographic `--buildings … --link-colours`, orthographic
     `--link-colours speed --link-lanes`, `--camera tests/city-flight.toml --buildings …
     --link-colours speed --link-lanes`, `--camera tests/city-flight.toml --link-colours
     stopped`, and `--camera tests/see-through-flight.toml --buildings … --link-colours
     stopped --link-window 1` **1800 of 1800** each (`ffprobe` `1920,1080,30/1,…`). **The
     look changed:** each first render differs from its case's `hud-<case>.framemd5` in
     `scratch/out/hud-p2/` in at least one frame, the count recorded. (The probe,
     four of the seven, palette (a) and the speed colours: every pair equal; against Phase
     2's, 0 frames equal in each, §2.21.16.)
  9. **A frame does not depend on the frames before it** (`tests/links.rs`, ignored, the
     fixtures; GPU). Each job set as `render` sets it with `--link-colours`. Each of these
     frames, rendered alone by a new job, equals the same frame rendered in sequence from
     frame 0, byte for byte: urban_grid's default render with `--link-colours`, frames 0, 1,
     30, 299 and 300; Midtown's city flight with `--buildings --link-colours speed
     --link-lanes`, frames 0, 29, 30 and 299 of `--from 300 --to 310 --speedup 1`. It is
     what `view`'s slider relies on (§2.21.3). (The probe: 9 of 9
     equal, 0 pixels differ.)
  - **Recorded, with one bar:**
  10. **Render time.** Gate 8's wall times beside the same run's gate 2 renders of the same
      cases, each with the load average after it. Prediction: within the machine's noise; a
      frame's values cost 8–53 µs, and the mesh is replaced about once per FCD time (59 of
      1,800 frames). (The probe's
      renders ran under another session's build, at a load of 3 to 76: Midtown off 38.8–89.6
      s at 3–20, on 74.1–185.1 s at 10–77; urban_grid off 201.2 and 212.4 s at 4–5, on 530.9
      s at 75 and 230.5 s at 10. Not comparable.)
  11. **`view --bench 20`** on Midtown at the default window with `--buildings` (streets,
      see-through, signals and the HUD on) and `--link-colours`, and without it, in two
      passes fixed before the first begins, each with colours then without, back to back,
      the load average before and after each. Record the JSON.
      - **A `mean_fps` below 30 with link colours on stops the build**, and the phase goes
        back to review. It is never re-run to get a pass.
      - Prediction: at least 30, likely 60 (vsync). (The probe, replacing the mesh every
        frame: with colours 60.00 and 60.00 fps, p99 18.76 and 18.66 ms; without, 60.00 and
        59.98, p99 18.62 and 20.86, at a 1-min load of 7.5 falling to 4.1.)
  - **The user's check:**
  12. **The user watches** gate 8's first renders (`scratch/out/links/`) against Phase 2's
      (`scratch/out/hud-p2/`). Times are the video's, from 0:00; colours by their names in
      OQ-9 (b): blue (best), light blue, orange, red (worst); "grey" is today's road.
      In every video the boxes are light grey (OQ-8), the legend sits under the HUD's line,
      the junctions stay grey, and the markings and the signals' dots lie over the tint.
      - **Midtown, orthographic with buildings, `relative` (`links-ortho-city.mp4` against
        `hud-ortho-city.mp4`).** At 0:00 the streets are coloured: of the 255 directions,
        about 60 blue, 40 light blue, 62 orange and 26 red, and 67 grey (no car in the
        last minute). The legend reads `speed / limit ■ < 25 % ■ 25–50 % ■ 50–75 % ■
        ≥ 75 %`, the swatches red to blue. Through the minute the colours hold: once a second a few streets change step
        together (about four a second over the frame), and the counts stay within 55–63 blue, 38–56 light blue, 56–69 orange, 23–29 red and
        57–69 grey. A two-way street shows each direction in its own colour. The signal
        switches at 0:25, 0:28 and 0:30 change no colour at once: a street's colour moves
        only as its last minute of cars does. Where buildings cover a street, its colour is
        under the roof, as its boxes are. Nothing else differs from Phase 2's video.
      - **The city flight with buildings, `speed` per lane (`links-city.mp4`).** From 0:00
        to 0:10, high and straight down: as the orthographic video, but each lane in its
        own colour, so a three-lane avenue reads as stripes, and the legend reads `speed ■
        < 10 ■ 10–20 ■ 20–30 ■ ≥ 30 km/h`. As the camera comes down, the stripes widen into
        lanes with their dashes and arrows on them; the colour reaches every lane's end at
        the junction's edge, and the junction is grey.
      - **The orbit with buildings, `stopped` over 1 s (`links-orbit.mp4`).** "Right now":
        a street direction is blue with no car standing on it and red with one or
        more. The crossing in
        the middle of the frame: the two approaches of the cross street, whose dots are
        red, turn red within the first 10 s as their queues stand (one from about 0:06, the
        other from about 0:10), and back to blue within 3 s of their dots turning green at
        0:30, as the queues leave. The avenue's approach, red for the first second and then
        blue, whose dot turns amber at 0:25 and red at 0:28, turns red at 0:30, when its first car stops, and stays red to the end
        as its queue grows to three. Farther streets flick between blue and red as cars
        stop and go.
      - **urban_grid's `--camera` render, `relative` (`links-ug-camera.mp4`).** 0:00 is
        9.1 s of the run: only the six streets that carry the first cars are coloured, blue;
        the rest are grey. The colour spreads street by street as cars enter, and the first
        light blue street appears at 0:30 (39.1 s of the run), when queues form at the
        lights. Half the grid stays grey all through: its 23 links never carry a car. While
        following the car (0:55 to 2:11), the streets under it are coloured and its box is
        light grey. At the end, 9 blue, 9 light blue, 7 orange and 23 grey streets, no red.
      - **In `view` on Midtown:** at launch no colours, and the boxes in their speed
        colours. `C` turns `relative` on (the boxes go light grey, the legend appears
        under the HUD), then `speed`, `density`, `stopped`, and off (the speed colours and
        no legend), the legend changing with each. Dragging the slider back and forth, the
        colours follow the time both ways at once. `view --link-colours stopped
        --link-window 1` opens with "right now" stopped cars. The readout is unchanged.

      Then say whether OQ-7's, OQ-8's, OQ-9's and OQ-10's answers stand as they look.
  - **The flags and the key:**
  13. **Their edges.**
      - **`C`, headless** (`tests/links.rs`, not ignored), written as `tests/hud.rs`'s
        `gate13_h` with `c` for `h`: `link_measure` is `None` at `ViewState::new`; five
        frames with `c` pressed give `relative`, `speed`, `density`, `stopped`, `None`; from
        `Some(Speed)` one gives `density`; then playing and following with `K` pressed in
        the same frame, through a right-button orbit, a drag and a scrub of the slider,
        every other field of the state equals that of a twin stepped without `c`.
      - **The CLI** (`scripts/gates-links.sh`). Each `view` runs under `perl -e 'alarm 60;
        exec @ARGV'`. On urban_grid, `render --from 10 --to 20 --speedup 1`:
        - `--link-colours --link-window 0.5`: exit 1, stderr's one line is
          `error: --link-window 0.5 s is shorter than the run's FCD interval (1.000 s);
          --link-window 1.000 colours by each frame's own FCD row`, and no video;
        - `--link-colours --link-window 1`: exit 0, `ffprobe` `1920,1080,30/1,300`;
        - `--link-colours --link-window 0`, `-5` and `nan`: exit 1, the positive-number
          error, no video;
        - `--link-window 30` without `--link-colours`, and `--link-lanes` without it: exit
          2, no video;
        - `--link-colours flow`: exit 2, no video;
        - `view --link-window 0.5 --bench 1`: exit 1 and the shorter-than error;
          `view --link-colours stopped --link-lanes --bench 1`: exit 0 and the bench JSON
          on the last line of stderr.

      Without the feature: `--link-colours` is itself rejected, exit 2.
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | Off: the 29 reference renders | 8700 of 8700 / 1800 of 1800 against the ten references | 1 |
  | Off: Phase 2's seven cases | 8700 / 1800 of 1800 against `hud-p2` | 2 |
  | Other tests and scripts; diff | baseline numbers; equal pairs; the listed files | 3 |
  | Packages | 0 | 4 |
  | Synthetic: steps at eight `(t, W)` and per lane; `check_window` | as tabled, `==`; refused below `I − 1e-6`, taken from it | 5 |
  | Fixtures: every frame against the FCD read straight; step counts | 0 of 1,800 / 8,700 differ; as tabled | 6 |
  | urban_grid's colours drawn at 150 s | the tint 16,246 / 34,903 / 137,509 px, none over the background; 22 midpoints exactly their colour, 26 exactly the road; the legend 7,116 / 15,361 / 57,304 px | 7 |
  | Seven renders on, twice; against Phase 2 | equal; differ in ≥ 1 frame | 8 |
  | A frame alone against the same frame in sequence | equal, 9 of 9 | 9 |
  | `view --bench 20` with link colours | ≥ 30 fps | 11 |
  | `C`; the window's errors; usage errors | cycles, only its field; exit 1 with the flag named; exit 2 | 13 |
- **Not predicted, and so not gated:**
  - the look: OQ-7 to OQ-10 as answered, how the tint sits under the markings and the dots,
    and how the legend reads over each background, for the user at gate 12;
  - render times (gate 10), and `view`'s frame rate above 30 (gate 11);
  - the legend's glyphs pixel for pixel: gate 7 bounds its box.
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-003-phase-3`), one push. The commits:
    - `src/links.rs`, `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/main.rs`'s
      `render` flags, and `tests/links.rs`'s headless gates (5, 6). Off by default, so the
      reference scripts pass unedited;
    - `view`: `src/view/mod.rs`, `src/view/state.rs` and `src/main.rs`'s `view` flags (`C`,
      the colours, the legend, the window's check), and gate 13's `C` test;
    - `tests/links.rs`'s GPU gates (7, 9) and `scripts/gates-links.sh` (gates 2, 8 and 13's
      CLI cases, and gate 12's renders). Gates 1–4, 6–11 and 13 run after this commit;
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **`rules/links.md`** (new, `max_lines: 40`): the window and the interval, the four
      measures, the steps and their rounding, the palette, the tint and its lift, the cars'
      colour, the legend, the flags, `C` and the errors. `sources`: `src/links.rs`,
      `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/view/mod.rs`, `src/view/state.rs`,
      `src/main.rs`;
    - `rules/render.md` (60 of 60): the CLI line gains `[--link-colours [m]]
      [--link-window s] [--link-lanes]`, the scene one line pointing to `rules/links.md`,
      the Pipeline's per-frame bullet the colours, and its `prepare_with` bullet link
      colours off; cut to fit, the cap not raised;
    - `rules/view.md` (60 of 60): the CLI gains the three flags, the drawing `C`, the frame
      order `C` after `H`; cut to fit;
    - `rules/camera.md` (60 of 60): "What each path draws" names the tint (flat at 0.005 m
      in both paths) and the neutral boxes; cut to fit;
    - `rules/streets.md`: one line, the tint between the road and the markings
      (`rules/links.md`); cut to fit;
    - `rules/hud.md` (40 of 40): one line, the legend under the line (`rules/links.md`);
      cut to fit;
    - `rules/signals.md`, `rules/inputs.md`, `rules/credit.md`, `rules/slider.md`,
      `rules/buildings.md`, `rules/see-through.md`, `rules/parts.md`, `rules/motion.md`:
      none needed, since nothing they say changes (the colours read no new input, and the
      dots, the credit line and the slider are untouched);
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README:** the summary gains a sentence on link colours; the `render` and the
      `view` usage blocks each gain `[--link-colours [relative|speed|density|stopped]]
      [--link-window <s>] [--link-lanes]`; a **Link colours** bullet after **HUD** says what
      each measure is, the window and its error, per lane, the legend and the neutral boxes,
      and that it is off by default; `view`'s keys gain `C`; the gate list gains
      `scripts/gates-links.sh` and `cargo test --release --test links -- --include-ignored
      --test-threads=1`;
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-003.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

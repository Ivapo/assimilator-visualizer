---
id: vis-003
title: data
note: >
  The run's data on screen, in render and view. Phase 1 draws the signals as the engine's
  dashboard does: one dot per signalised approach link at its stop line, green, amber or
  red, a fixed size on screen with a black outline, by default in both cameras;
  `--no-signals` gives today's frames and `view`'s `L` hides and shows them. The states
  are replayed with the engine's own signal code at the pin, stepped once through the
  run into a timeline, and checked against the run's FCD. A HUD and link colours are
  roadmap.
status: draft
last_updated: 2026-10-09

phases:
  - name: "Phase 1 — Signals: the engine's signal states as dots at the stop lines, by default"
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
  frames byte for byte (gates 1 and 2).
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

OQ-2 and OQ-3 stay open as drafted: deferred, and no answer was asked.

The rest of this section is the draft's proposal. It settles:
- the replay and its clock (§2.4–§2.6), and a check of it against the FCD (§2.7);
- what the link-level colour means, permitted movements included (§2.8);
- where each dot is (§2.9), how it looks (§2.10, OQ-1, answered (a)), and how it keeps its
  size in both cameras (§2.11);
- depth, ties and determinism (§2.12; OQ-4, answered as drafted);
- the flag, the key (`L`, confirmed) and their errors (§2.13), and the scripts that read
  references (§2.14);
- a run whose signals were forced (§2.15, OQ-2), and a pin move (§2.16).

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

### 2.4 What the run records: no signal state

Checked on both fixtures at engine `90b39292`, read-only:
- **`results.db`** has the run tables and none for signals. Midtown: `runs` 1 row,
  `trips` 828, every other table empty. urban_grid: `runs` 1, `trips` 91,
  `detector_intervals` 75. No table holds a phase or a colour.
- **The FCD** has `time`, `vehicle_id`, `link_id`, `lane`, `position`, `speed`,
  `acceleration`, `vehicle_class` and `vehicle_length` (vis-001 §2.1). No signal column.
- **The plans** are in the scenario's resolved network: each signalised junction's
  `signal_plan`, with `cycle_time`, `offset` and its `phases`. Each phase has a green
  `duration`, `amber_time`, `all_red_time`, `green_movements` and `permitted_movements`.
  A scenario's `apply` set of plans is merged into that network by
  `crates/config/src/project.rs:resolve_scenario`, which `render` already calls.
- **The run's clock** is in `runs.resolved_config`:
  `{"scenario": …, "sim_params": {"duration", "random_seed", "timestep", "warm_up"}}`,
  written by `crates/output/src/runner.rs:run_to_results_db`. Midtown: duration 1200,
  timestep 0.1, warm_up 180. urban_grid: 300, 0.1, 30. `runs.mode` is `micro` in both.

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
engine's own code puts none (§2.7). This repo needs nothing from the engine here. The user
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

#### 2.5.3 The switching times are the engine's steps, not the plan's seconds

The state changes only at a step, and `elapsed` carries a float sum. Stepping the engine's
code shows where that matters:
- **Both fixtures' plans** (25 + 3 + 2 at 0.1 s) switch exactly on the plan's seconds,
  through the whole run: every green lasts 250 steps, every amber 30, every red 320 (or
  20, at Midtown's two one-phase junctions).
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
- **an FCD row at time `T_k`** shows the vehicles after step `k`, under `S_k`. The
  dashboard's frame at `T_k` shows `S_k` too
  (`crates/geometry/src/frame_collect.rs:collect_frame` reads the state after the step);
- measured, **every FCD time is bit-equal to a `T_k`** summed here as the engine sums
  it: 1,199 of 1,199 in Midtown (1.0999999999999999 to 1199.100000000005) and 291 of 291 in
  urban_grid. The `warm_up` changes nothing for the signals.

**The rule: at sim time `t`, a frame shows `S_k` for the last step with `T_k ≤ t + 1e-6`**
(`S_0` before the first step), as the dashboard does between its frames. At an FCD time it
is that row's state. With both fixtures' plans the colours change exactly on the plan's
seconds: an approach turns amber at 25.0 s, not 24.9 s. The `1e-6` is vis-001's snapshot
tolerance (`rules/inputs.md`). Between steps, the state shown is up to 0.1 s behind the one
the vehicles are reacting to, which is under 3 frames at 30 fps and 1× and is the
dashboard's own lag.

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
| Build, median of 9 in one process (fastest – slowest) | **259 ms** (245–261) | **22 ms** (21.6–21.8) |
| of which `advance` alone | 18 ms | 1.1 ms |
| A frame's colours: 1,800 frames × every approach | 10.2 ms in all (5.6 µs a frame) | 2.4 ms |
| Approach-steps green / amber / red | 42.6 / 5.1 / 52.3 % | 41.7 / 5.0 / 53.3 % |

**The alternative: step a `SignalState` with the frames.** It needs no list, and a
render's frames go forward in time, so each frame would step from the last one. But `view`
goes back: the slider, `Shift+←` and a restart at `from` (vis-001 §2.10). Each step back
would build a new state and step it from 0 to `t`: 18 ms on Midtown at its end, more than
a frame at 60 fps, on every frame of a backward scrub, and more on a longer or larger run
(it grows with steps × junctions). The
timeline answers any `t` in microseconds, both ways, and `render` and `view` share it.

**Its cost is paid once,** before the first frame: a quarter of a second on Midtown, whose
renders take 48–88 s (vis-002 §2.18.15). Most of it is `link_signal_color` called for every
approach at every step. A cheaper build is possible, re-evaluating a junction's approaches
only on a step where its sub-state changed, but measured it is slower here (355 ms on
Midtown, through `junction_phase_info`, which allocates), so it is not proposed.

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

**The check bites.** The same counts with the replay shifted, or with the plans' offsets
applied as a replay from the plan would:

For each: junction entries on red throughout; first stopped boxes whose next row is under
red; the most common delay from green to moving off (with its count).

| Replay | Midtown | urban_grid |
|---|---|---|
| **The engine's (as proposed)** | **0; 7; 1.10 s (2,194)** | **0; 0; 0.10 s (96)** |
| 0.5 s early (−5 steps) | 0; 7; 0.60 s (2,194) | **6**; **96**; none twice |
| 1 s early (−10 steps) | 0; 7; 0.10 s (2,194) | **6**; **96**; none twice |
| 0.5 s late (+5 steps) | 0; 7; 1.60 s (2,194) | 0; 0; 0.60 s (96) |
| 1 s late (+10 steps) | **70**; 7; 2.10 s (2,194) | 0; 0; 1.10 s (96) |
| Half a cycle late (+300 steps) | **7,221**; **2,388**; — | **422**; **107**; — |
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
junction-facing edge, within 0.000000 m. The 36th is the approach whose lane 0 has a 5 m
stop-line delta (vis-002 §2.17.7): its dot is 5.0000 m ahead of lane 0's own line, on the
edge of the other lanes' line, as on the dashboard.

| | Midtown | urban_grid |
|---|---|---|
| Dots (approach links); none left out | **188** | **36** |
| Approaches per signalised junction | 2 at 62, 3 at 20, 4 at 1 | 4 at 9 |
| Closest two dots | 6.537 m | 17.553 m |
| Pairs of dots closer than 17.6 m (they overlap in the orthographic frame, §2.11) | **107** | **0** |

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
triangles. A 32-gon is within 0.05 px of the circle at 2160p.

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
| 1280×720 | 24 (24 fill, 0 black) | 21–23, 5×5 to 5×6 px | 21–23, 5×5 to 6×5 | 21–23, 5×5 to 6×5 |
| 1920×1080 | 60 (52 fill, 8 black) | 60–64, 10×10 to 10×11 | 61–63, 10×10 | 60–63, 9 to 11 each way |
| 3840×2160 | 308 (232 fill, 76 black) | 307–313, 21×21 | 311–315, 21×21 to 22×21 | 312–318, 21×21 to 22×21 |

- **The same size at 64 m and at 490 m,** tilted or straight down, and the same as the
  orthographic dot. Computed before the GPU, every vertex projects within 7e-13 px of its
  circle, and the lowest is at 0.030000000000 m.
- **Tilted, it stands on its point:** its centre is 4.5–4.7 px above the stop line's point
  at 1080p and pitch 25–30, and within 0.1 px of it at pitch 90.
- **At 720p the outline ring is 0.67 px wide,** so no pixel of it is pure black: it shows
  as a dark rim. At 1080p it is 1 px, at 2160p 2 px.

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
- **`--no-signals`** on `render` and `view`. With it, nothing about signals is read or
  built and nothing is spawned, so `render` gives today's frames, byte for byte (gates 1
  and 2). The name follows `--no-streets` and `--no-see-through`.
- **`L`** in `view` (signal *lights*; by position, `KeyCode::KeyL`, unbound today) hides
  and shows the dots, as `M` does the streets. `view --no-signals` starts with them hidden,
  and `L` shows them. `S`, the obvious letter, is `WASD`'s pan. The keyframe line does not
  carry it. In the frame order it comes after `M` (`rules/view.md`). *(2026-10-09, user)*
  Confirmed: the key is `L` (decision 6).
- **`--signals` is not a flag:** clap rejects it, exit 2, as it rejects `--streets`.
- **The library keeps today's default.** `Job::prepare*` build every job with signals off,
  as with streets, so every test that builds a job draws as today. `src/main.rs` turns them
  on with `Job::set_signals(true)` unless `--no-signals` is given.

**What signals read, and their errors.** With signals on and a signalised junction in the
network, after `run::load`'s checks and the files' (`rules/inputs.md`), before the first
frame, `results.db`'s run row (the one `run::load` found, read-only and `immutable=1`)
must give:
- **`resolved_config`** as a JSON object with a finite `sim_params.timestep` above 0, and
  a finite `duration` (the `runs` column) above 0. Otherwise:
  `error: <results.db>: the run's timestep is not readable (runs.resolved_config): <why>;
  --no-signals renders without signals`.
- **`mode` = `micro`.** Otherwise: `error: <results.db>: the run is <mode>; signals are
  replayed only for micro runs (vis-003 OQ-3); --no-signals renders without signals`.

Each is one line, exit 1, no output file and no window, as every input error
(vis-001 §2.4). Under `--no-signals`, or on a network with no signal, neither is read, so
no run that renders today fails tomorrow with that flag.

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

These 29 lines are the ones carrying `--no-streets` now. The other renders in scripts
compare only with each other, or count frames that differ: `gates-ties.sh` gate 9,
`gates-city.sh` gate 13, `gates-credit.sh` gate 14, `gates-streets.sh` gates 10 and 11 and
`gates-look.sh` gate 10. They are not edited, now draw dots, and must still give equal
pairs and still differ where they differed (gate 3). `gates-look.sh` writes into
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
  `crates/python/src`). So every run the visualizer can open is a fixed-time run, and the
  replay is the engine's.

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
  With none, the copy stands, and the record says so. The new module names `90b39292`
  beside each copied value, so the diff has a place to land.

At the engine's HEAD today, those three files are not read here: the pin is what is drawn.

### 2.17 What it does not do

- **No colour on the stop lines.** vis-002 kept its stop lines per `(link, lane)` so that
  "the data item" could colour them (vis-002 §2.17.7, §2.18.6, §2.18.13). Decision 2 draws
  dots instead, as the dashboard does, and the stop lines stay white. The quads stay per
  lane; nothing uses that now. vis-002 gets a dated note saying so in Phase 1's close-out.
- **No per-lane, per-movement or protected/permitted distinction** (§2.8).
- **No dot at an unsignalised junction,** where the engine has none either; vis-002's 38
  stop lines at unsignalised approaches stay as they are.
- **No fade.** A dot keeps its pixel size at any distance, so it never thins below a pixel.
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
completed `runs` row rather than the one `run::load` found; it has no `--no-signals`, no
`L` and no mode check; its dots are always on in `view`, and set every frame there.

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
    `results.db` at the pin.)*
- **OQ-3** — Meso runs (§2.13).
  - *The facts:* the meso engine steps the same `SignalState` once a step
    (`crates/meso/src/engine.rs`, `advance(self.dt)` at the start of `step`), but with
    `meso.timestep`, and its vehicles take a movement's green
    (`is_movement_green`), not the link's colour. No fixture is meso, and no gate here has
    drawn a meso run at all.
  - *Recommendation:* refuse with signals on in Phase 1, with the error of §2.13 and
    `--no-signals` as the way through; take it up when a meso fixture exists.
  - *(deferred by evidence: no meso fixture; blocks nothing in Phase 1.)*
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

## 4. Implementation phases

Strictly sequential. Phase 1 is the only one drafted (decision 1).

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
    - `Signals::colours_at(t) -> Vec<Colour>`: §2.6's rule, in approach order;
    - `scale(w, h)`, and `dots(&Signals, &[Colour], camera, w, h) -> DotsData` (positions
      in world metres, an sRGB colour per vertex, indices), where `camera` is the
      orthographic `k` or a `Pose` (§2.11). Per dot: the ring (32 inner, 32 outer vertices,
      64 triangles), then the fill (a centre and 32 rim vertices, 32 triangles), dots in
      approach order; a perspective dot with `f·ρ/R ≤ 0.1` is left out.
  - **`src/inputs.rs`:** `run_clock(results, scenario, seed) -> Result<(f64, f64, String)>`,
    the run's `timestep`, `duration` and `mode` from the row `require_completed_run`
    accepts, read-only and `immutable=1`; the errors of §2.13.
  - **`src/draw.rs`:** `spawn_dots` (one entity, one unlit white material with
    `cull_mode: None`, `NoFrustumCulling`, hidden until set) and `set_dots` (replace its
    mesh, linear vertex colours from sRGB as the markings', hidden when empty), about the
    bake origin `(fx, fy)` as every mesh.
  - **`src/render.rs`:** `Renderer::set_signals(on)` spawns or despawns the entity and
    settles again, as `set_streets`; `Renderer::set_dots(&DotsData)`.
  - **`src/lib.rs`:** `Job` keeps an `Option<Signals>`; `Job::set_signals(on)` reads the
    run clock and builds the timeline (on), or drops both (off); every `prepare*` builds a
    job with signals off; `render_at(t)` sets the dots for `t` and the frame's pose (or
    `k`) before the boxes. `Job::signals()` gives the timeline to the gates.
  - **`src/view/mod.rs`, `src/view/state.rs`:** the timeline built at launch unless the
    network has no signal; `signals_shown` (on unless `--no-signals`); `L`
    (`KeyCode::KeyL`) flips it after `M`; while shown, `set_dots` every frame for `t`, the
    pose and the window's logical size; while hidden, the entity hidden. `--bench` sets
    them every frame, as it does the markings.
  - **`src/main.rs`:** `--no-signals` on `render` and `view`, with help text; `render`
    calls `job.set_signals(true)` unless it is given. No other flag or default changes.
  - **Tests.** `tests/signals.rs` (new):
    - headless, not ignored: gates 5 and 8's synthetic part;
    - headless, needing the fixtures, so ignored: gates 6, 7 and 8's fixture part;
    - through the GPU, ignored: gate 9 (no fixture).
  - **Scripts.** `scripts/gates-signals.sh` (new, offline): gates 2 and 10, and the renders
    for gate 14, into `scratch/out/signals/`. The 29 lines of §2.14 gain `--no-signals`,
    and the comment above them in each of the four scripts says so.
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
       pass (equal pairs, each differing where it differed). All nine scripts pass.
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
     The predictions (the probe's `out/synth.txt`):
     - **4 approaches**, in this order, and their dots: `L_EC` (10, 2), `L_NC` (−1.75,
       13.75), `L_SC` (3.75, −13.5), `L_WC` (−12, −1.75), each within 1e-9 m. `L_EC`'s is at
       its link's end, 5 m ahead of its lane 0's own line (its 5 m delta);
     - **the changes**, by step, the same for `L_EC` and `L_WC`: green 0, amber **200**,
       red **230**, green **451**, amber 650, red 680, green **901**; for `L_NC` and
       `L_SC`: red 0, green **250**, amber **401**, red **431**, green 700, amber **851**,
       red **881**. So 40.1, 45.1, 85.1 and 90.1 s, not the plan's 40, 45, 85 and 90
       (§2.5.3); and green again at 45.1 s, not at 60 + 7;
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
     - **errors:** a `resolved_config` with no `timestep`, one of 0, and a `mode` of
       `meso`, each give §2.13's message (a temporary `results.db` written by the test).

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
     | Green, amber, red runs between changes (s) | 25; 3; 2 and 32 | 25; 3; 32 |
     | FCD times bit-equal to a `T_k` | **1,199 of 1,199** | **291 of 291** |
     | `colours_at` against a second `SignalState` stepped alongside, at every FCD time and 0.05 and 0.5 s after it | **676,236 equal, 0 differ** | **31,428, 0** |
     | Dots at their lane-0 stop line's front midpoint (within 1 mm) | **188 of 188** | **35 of 36**; the 36th at 5.0000 m |
     | (approach, phase) pairs: green by permitted alone; green with a movement in no list | 0 of 372; **2** | 0 of 72; 0 |

     Also printed and recorded, not asserted: the build time of the timeline (§2.6).
  7. **The replay against the FCD** (`tests/signals.rs`, ignored), §2.7's check on each
     whole run:
     - **junction entries:** Midtown **7,536**: green 6,896, amber 570, amber then red 70,
       **red throughout 0**; urban_grid **422**: 373, 49, 0, **0**;
     - **first stopped boxes:** Midtown **2,490** episodes (48,896 rows), first row red /
       amber / green **2,301 / 27 / 162**, next row green / amber / red / none **2,406 /
       10 / 7 / 67**, **2,194** moving off 1.10 s after green; urban_grid **129** (2,555),
       **118 / 0 / 11**, **107 / 0 / 0 / 22**, **96** moving off 0.10 s after green;
     - **the control:** with the replay 1 s late (+10 steps), Midtown has **70** entries on
       red throughout; with urban_grid's offsets applied, **90**.
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
     fill colours and **32** black. Then `Renderer::new_perspective` at the pose of gate 5
     (pitch 30), at pitch 90, and at `height_m` 400 and pitch 25: each dot's pixels of its
     exact fill or black **60–64**, in a box of **10×10 or 10×11** px whatever its
     distance (64 to 493 m). Without the feature, no pixel changes. These are OQ-1's (a)
     numbers (answered 2026-10-09, decision 4).
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
        more); at 0:25 every green dot in the frame turns amber at once, at 0:28 red, and
        for two seconds there is no green anywhere but at two crossings, whose dots are
        all green from 0:00 to 0:25 and from 0:30 to 0:55; at 0:30 every dot that had
        been red turns green at once. The tiny boxes queue behind the red dots and pass
        the green ones; where they reach a dot they show over it. A few dots, about one
        crossing in twelve, lie partly or wholly under a roof (OQ-4, answered as drafted).
        Nothing else differs from Phase 6's video.
      - **The city flight with buildings (`signals-city.mp4`).** From 0:00 to 0:10, high and
        straight down: as the orthographic video. Then, as the camera comes down and tilts,
        the streets grow but every dot stays the same size: at the top of the frame, far
        off, the dots of a crossing are as big as those of the crossing below the camera.
        A dot behind a tower is hidden and appears as the camera passes. The same switch at
        0:25, 0:28 and 0:30. From about 0:40 the camera looks across the park's dark
        ground, and few dots are in the frame.
      - **The orbit with buildings (`signals-orbit.mp4`), the crossing in the middle of the
        frame.** Four dots, one at each approach's white bar, on its side nearest the
        middle of the street, standing a little above the bar. From 0:00 to 0:25, the two
        on the street that runs up the frame at 0:00 are green and the two on the cross
        street red, with the cross street's cars waiting behind the red ones. At 0:25 the
        greens turn amber, at 0:28 red. At 0:30 the cross street's dots turn green, and its
        first waiting box moves off about a second later. As the camera circles, each dot
        stays round and the same size; it is hidden only where a box or a building stands
        in front of it. The crossings farther up the street, in the top of the frame, show
        their dots as small as the near ones.
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
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | `--no-signals`: the 29 reference renders | 8700 of 8700 / 1800 of 1800 against the ten references | 1 |
  | `--no-signals`: Phase 6's seven cases | 8700 / 1800 of 1800 against `look-p6` | 2 |
  | Other tests and scripts; diff | baseline numbers; equal pairs; the listed files | 3 |
  | Packages | 0 | 4 |
  | Synthetic: approaches, dots, change steps, colours, mesh | 4; as listed; 200/230/451…, 250/401/431…; 388 and 384 | 5 |
  | Fixtures: junctions, dots, steps, changes, FCD times, against `SignalState` | 83 / 9; 188 / 36; 12,000 / 3,000; 11,708 / 576; all bit-equal; 0 differ | 6 |
  | Junction entries on red throughout; first stops; control | 0 of 7,536 / 0 of 422; 2,490 / 129 as listed; 70 and 90 | 7 |
  | Colours, `scale`, the dots' place | the copied hex; 2/3, 1, 2; bit for bit | 8 |
  | Synthetic pixels: changed, fill, black; perspective | 448; 104 + 104; 32; 60–64 px a dot, 9–11 px across | 9 |
  | Seven renders twice; against Phase 6 | equal; differ in ≥ 1 frame | 10 |
  | Copied values at their lines | each holds | 11 |
  | `view --bench 20` with signals | ≥ 30 fps | 13 |
- **Not predicted, and so not gated:**
  - the look: OQ-1's size and OQ-4's dots under roofs as answered, the dot's colours
    beside the slow boxes', and how overlapping dots read, for the user at gate 14;
  - render times (gate 12), and `view`'s frame rate above 30 (gate 13);
  - a meso run and a forced run: refused, or not readable, at the pin (OQ-2, OQ-3).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-003-phase-1`), one push. The commits:
    - `src/signals.rs`, `run_clock` in `src/inputs.rs`, `src/draw.rs`, `src/render.rs`,
      `src/lib.rs`, `src/main.rs`'s flag, `tests/signals.rs`'s headless gates (5–8), and
      the 29 lines of §2.14 with their comments. The look changes here, since signals are
      on by default, and the reference scripts keep passing in the same commit;
    - `view`: `src/view/mod.rs`, `src/view/state.rs` (`L`, the dots, `--no-signals`);
    - `tests/signals.rs`'s GPU gate 9 and `scripts/gates-signals.sh` (gates 2 and 10, and
      gate 14's renders). Gates 1–4 and 9–13 run after this commit;
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
    - `rules/inputs.md` (50 of 50): the checks gain the run clock's, with signals on;
    - `rules/streets.md`, `rules/buildings.md`, `rules/see-through.md`, `rules/parts.md`,
      `rules/credit.md`, `rules/motion.md`, `rules/slider.md`: none needed, since nothing
      they say changes;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README:** the summary gains a sentence on vis-003's dots; the `render` usage
      block gains `[--no-signals]`; a **Signals** bullet after **Streets** says what is
      drawn, that it is the engine's own signal code replayed, and `--no-signals`;
      `view`'s keys gain `L`; the gate list gains `scripts/gates-signals.sh` and `cargo
      test --release --test signals -- --include-ignored --test-threads=1`;
    - **vis-001 §2.7 item 5:** one dated note: narrowed to vis-003 (signals, Phase 1; a
      HUD and link colours, its roadmap; the legend and chart not planned);
    - **vis-002 §2.17.7, §2.18.6 and §2.18.13:** a dated note each: the data item draws
      dots at the stop lines, as the dashboard does, and the stop lines stay white;
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-003.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

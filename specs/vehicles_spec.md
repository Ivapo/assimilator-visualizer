---
id: vis-004
title: vehicles
note: >
  How vehicles look in the 3D views (`render --camera` and `view`). Phase 1 draws each
  vehicle as a shape for its FCD `vehicle_class` (car, truck, bus, bike; any other name a
  plain box), its length the FCD's `vehicle_length`, with brake lights from the FCD's
  acceleration and standing still, and headlights as a look, by default; `--no-shapes`
  gives today's frames and `view`'s `V` toggles them. The orthographic render keeps
  today's boxes. A third fixture, Midtown with a class mix, is built by
  `scripts/fixture.sh mix`. Phase 2, turn signals by the visualizer's own rule, is
  roadmap only.
status: draft
last_updated: 2026-10-23

phases:
  - name: "Phase 1 — Shapes: a shape per class, brake lights and headlights, in 3D, by default"
    reviewed: null
    shipped: null
    cut: null
    by: null

extends: null
supersedes: null
superseded_by: null
related: [vis-001, vis-002, vis-003]
reference: >
  Engine: ~/dev/main/assimilator at the pin of vis-001 §2.2.2 (90b39292), read-only, from
  cargo's checkout of that rev. The FCD's columns are its writer's
  (crates/output/src/results_db.rs:FcdParquetWriter, from
  crates/core/src/output_data.rs:VehicleSnapshot); a class is its project's
  (crates/config/src/definitions.rs:VehicleClassConfig). The engine's dashboard draws
  vehicles in web/src/canvas/vehicleLayerGL.ts (the WebGL layer) and
  web/src/canvas/vehicleRenderer.ts:drawVehicles (its Canvas fallback), their widths from
  crates/geometry/src/frame_collect.rs:vehicle_dimensions, read at 90b39292 on
  2026-10-23. Taken from it: the brake rule and the brake red. Out of scope from it: its
  rounded rectangles, its speed palette, its blinkers (engine state the FCD does not
  carry; Phase 2 is this repo's own rule), its selection outline and its light palette.
---

# Vehicles

## 1. Goal

Make the vehicles of a 3D video read as traffic. Today every vehicle is a box, length ×
1.8 × 1.5 m, in its speed colour (vis-001 §2.3, §2.11.2). In Midtown's city flight a
12 m truck would be a long box like a long car, and a bus a box like a truck, and nothing
on a car says which way it faces or whether it is stopping. From Phase 1, in the
perspective views, each vehicle is drawn as a shape for its class: a car with a cabin, a
truck with a cab and a box, a bus with its windows, a bike with a rider. Its rear lights
turn bright red while the FCD says it brakes or stands, and its headlights are lit.

**The observable is vis-001's: a video file rendered from one simulation run.** Here it is
the run's keyframed flight (`render --camera`) with every vehicle drawn as its class's
shape, brake lights as the FCD has them.

```
# On by default in 3D: a flight's vehicles are shapes, with brake lights and headlights.
assimilator-video render --project <project> --scenario baseline --seed 42 \
                         --buildings <dir>/buildings.geojson --camera flight.toml --out city.mp4

# Today's frames, byte for byte.
assimilator-video render … --camera flight.toml --no-shapes

# The orthographic render keeps today's boxes, with or without the flag.
assimilator-video render --project <project> --scenario baseline --seed 42 --out run.mp4

# V shows boxes and shapes in turn; --no-shapes starts with boxes.
assimilator-video view --project <project> --scenario baseline --seed 42
```

Nothing new is read from outside the run: the class and the acceleration are FCD columns
every run at the pin writes (§2.4), and the rest is constants in this repo.

Rejected candidates for the observable:
- *A still of the shapes.* A frame of the video, not the video.
- *The `view` window with shapes.* A tool for writing flights (vis-001 §2.9), which shows
  what `render` will draw.
- *The mix fixture.* An input to the gates, as the buildings cache is (vis-002 §1).

### 1.1 Non-goals

- **No turn signals in Phase 1.** They are Phase 2 (§2.3), by this repo's own rule, since
  the engine's blinkers come from state the FCD does not carry (§2.4.3).
- **3D only** (decision 3). The orthographic render draws today's boxes, byte for byte,
  and reads no new column (§2.13).
- **No per-class data beyond the name.** The engine's class has a length and behaviour and
  no width or height (§2.4.1); this repo gives each built-in shape its width and height.
  No colour per class: the body keeps the speed colour (§2.7).
- **No shape per vehicle beyond its class and length.** No random variation, no vehicle
  models from files, no textures, no lit materials: shapes are cuboids, unlit and shaded by
  face as today's boxes (vis-001 §2.6, §2.11.2).
- **No articulation.** A truck or a bus is one rigid body, placed and turned as today's
  box is (vis-001 §2.8); a trailer does not follow its own path.
- **No night.** Headlights are a look (decision 5): no light cast on the road, no bloom,
  no time of day (the run records none).
- **No change** to motion, placement, the buildings, see-through, the streets, the
  signals, the HUD, link colours' measures, the credit line, `view`'s pick, the readout or
  the keyframe line. The overlaps of engine OQ-29 (§2.11) are drawn as they come.

## 2. Design

### 2.1 A spec of its own (decision, recorded)

Decided by the user, 2026-10-23 (§2.2, decision 1), and checked against the methodology's
§6.1:
- **Step 0:** decisions change. A new kind of thing is drawn from the run (each vehicle's
  class and its brake state, from two FCD columns nothing reads today), and a default
  changes: every 3D render draws shapes.
- **Step 1:** nothing shipped is removed or contradicted. `--no-shapes` gives today's
  frames byte for byte in 3D (gate 2), and the orthographic render is untouched (gate 1).
  The library keeps today's default (§2.13).
- **Step 2:** two specs name the subject. vis-001 owns rendering one run's files, and its
  boxes (§2.3, §2.11.2, §2.12). vis-002's roadmap (§2.13 there) carries "Vehicle shapes
  *(added 2026-10-03, agreed by the user)*: a shape per FCD `vehicle_class`, after
  vis-001's data item". vis-001 is done (7 of 7 phases shipped) and vis-002 is done (6 of
  6); vis-002's subject is the city around the network, which the vehicles are not. The
  user chose a spec of its own: how vehicles look is a line of work two phases long so far
  (shapes and lights, then turn signals), with its own source in the run (the FCD's class
  and acceleration), as vis-003 took vis-001's data item (vis-003 §2.1).

So this is a new spec, `extends: null`, `related: [vis-001, vis-002, vis-003]`. vis-002
§2.13's "Vehicle shapes" item is **taken up** here and gets one dated note pointing here,
written in Phase 1's close-out; this draft does not edit vis-002.

### 2.2 The user's decisions (decision, recorded)

Decided by the user, 2026-10-23, before drafting:
1. **A new spec, vis-004 "vehicles".** **Phase 1: shapes, brake lights, headlights.**
   **Phase 2: turn signals** by this repo's own rule: lane changes from the FCD's `lane`,
   turns from the next link's movement type; roadmap only now. If Phase 1 measures too big
   for one review and one build, the draft proposes a split as an open question and does
   not split it itself.
2. **A shape per FCD `vehicle_class` name:** built-in shapes for `car`, `truck`, `bus` and
   `bike`; any other name gets a plain shape. Every shape takes its length from the FCD's
   `vehicle_length`.
3. **3D only:** `render --camera` and `view`. The orthographic render keeps today's boxes,
   byte for byte.
4. **On by default in 3D.** An off flag gives today's frames byte-identical; a key in
   `view` toggles.
5. **Brake lights from the FCD's acceleration** (and standing still). **Headlights are a
   look only** (no data).
6. **Test data: a Midtown copy with a class mix** (trucks at least; buses and bikes if the
   engine's config makes it easy, without PT routes if those are hard), run with the
   pinned engine in scratch: a third fixture. Making it is part of Phase 1 (for example a
   `scripts/fixture.sh` mode).

The flag's and the key's names and each class's sizes were left to the draft.

The rest of this section is the draft's proposal. It settles:
- what the run gives (§2.4) and the mix fixture (§2.5);
- each shape (§2.6, OQ-1) and its colours (§2.7, OQ-2);
- the brake lights (§2.8, OQ-3 and OQ-4) and the headlights (§2.9, OQ-5);
- the far end (§2.10); depth, ties and determinism (§2.11);
- how the two columns are read (§2.12), the flag, the key and their errors (§2.13), and
  the scripts that read references (§2.14);
- the cost (§2.15), and whether one phase holds it (§2.16).

### 2.3 Roadmap (not yet phases)

- **Phase 2 — turn signals,** by this repo's own rule (decision 1): a lane change from the
  FCD's `lane` (a row whose lane differs from the vehicle's row before), a turn from the
  movement type of the junction between its link and the next one (`Placement`'s
  movements, which motion already resolves, vis-001 §2.8.2). Amber lamps at the corners
  (§2.6 leaves room for them on every built-in shape), blinking on the video's own clock.
  Not designed here. What this draft notes for it:
  - the FCD's lane changes are instants (vis-001 §2.8.4's slide is drawn over one row
    interval), so "how long before" is Phase 2's rule;
  - the engine's dashboard blinks from state the FCD does not carry (§2.4.3), so Phase 2's
    blinkers will not match the dashboard's moment for moment.

### 2.4 What the run gives

#### 2.4.1 A class is a project's id, with a length and no size

At the pin, a class is whatever the project defines
(`crates/config/src/definitions.rs:VehicleClassConfig`): an `id`, a `length` (a
`Stochastic`, a scalar or a distribution sampled per vehicle at spawn), its behaviour
(gaps, speeds, accelerations, models) and `is_public_transit`, which is display metadata
the core does not read and the FCD does not carry. **No width and no height.** The
engine's own examples:

| `id` | `length` | Where (at 90b39292) |
|---|---|---|
| `car` | 4.5 | every bundled example; Midtown |
| `truck` | 12.0 | `docs/examples/demand-tour/project.yaml` |
| `bus` | 12.0 | `docs/examples/public-transport-tour/project.yaml`, with PT routes |
| `bike` | 1.8 | `configs/bundled-examples/bike_lane/project.yaml` |

A name is free text: `Car`, `lorry` or `hgv` are classes too, and get the plain shape
(decision 2). Names are matched exactly (§2.6.6).

#### 2.4.2 The FCD's columns

The engine writes nine columns (`crates/output/src/results_db.rs:FcdParquetWriter`): `time,
vehicle_id, link_id, lane, position, speed, acceleration`, then `vehicle_class`
(`Dictionary(Int32, Utf8)`) and `vehicle_length` (the snapshot's own length, m). This repo
reads the first six and `vehicle_length` (`rules/inputs.md`); **`acceleration` and
`vehicle_class` are read nowhere today.** `acceleration` is the velocity component's at
the snapshot (`crates/core/src/simulation.rs`, `acceleration: vel.acceleration`), in
m/s², never null.

Measured on both fixtures and the mix fixture (§2.5), engine `90b39292`, seed 42, with
DuckDB, read-only (`out/brake-rules*.txt`, §2.19):

| | Midtown | urban_grid | mix (§2.5) |
|---|---|---|---|
| FCD SHA-1 | `01becc86…` | `3ad76744…` | `c8d1d569…` |
| Rows; vehicles | 202,241; 985 | 21,556; 198 | 195,709; 974 |
| Classes (vehicles) | `car` (985) | `car` (198) | `car` 732, `truck` 100, `bus` 54, `bike` 47, `van` 41 |
| `vehicle_length` | 4.5 | 4.5 | car 4.5, truck 8.01–16.23, bus 12.0, bike 1.8, van 5.5 |
| Rows with `acceleration < 0` | 60,081 | 7,898 | 56,114 |
| Rows with `speed < 0.01` | 55,536 | 3,429 | 54,882 |
| Rows with either (the dashboard's brake rule, §2.8) | **108,052 (53.4 %)** | **10,415 (48.3 %)** | **104,409 (53.3 %)** |
| `acceleration`: min; max | −6.0; 2.998 | −6.0; 2.991 | −6.0; 2.998 |

(The mix fixture's two middle rows are its classes' sums: 43,263 + 5,329 + 2,977 + 2,177 +
2,368 and 41,962 + 5,623 + 2,657 + 2,601 + 2,039.)

#### 2.4.3 What the engine's dashboard draws

Read at 90b39292, read-only:
- **A rounded rectangle** per vehicle in its speed colour, its length the vehicle's and
  its width **2.5 m for `truck` and 1.8 m for every other class**
  (`crates/geometry/src/frame_collect.rs:vehicle_dimensions`; the `12.0` length there is
  not used: the frame sends `props.length`).
- **A brake strip:** a thin red band across the rear, 4.5 % of the length deep (about
  0.2 m on a 4.5 m car), in `#ef4444`, **when `acceleration < 0` or `speed < 0.01`**
  (`vehicleLayerGL.ts`, `flags |= 1`; the Canvas fallback the same). It is not drawn when
  the vehicle's half-width is under 2 px on screen (the Canvas fallback's `hw < 2`).
- **Blinkers** at the corners, amber, from the engine's lane-change intent, a mandatory
  change, the junction movement (left, U-turn, right) and PT stops
  (`frame_collect.rs`), none of which is in the FCD (Phase 2, §2.3).
- **No headlights,** no cabin, no class shape beyond the width.

### 2.5 The mix fixture (decision 6)

**A third fixture:** the user's Midtown project at `e2de274`, its demand halved exactly as
`scripts/fixture.sh midtown` halves it (the 65 rates sum to 2,850), and then two edits,
nothing else:
- **`project.yaml`:** four classes after `car` and one named mix, `mixed`, in place of
  `vehicle_mixes: {}`:

  | Class | Share | `length` (m) | `max_speed`; accel; decel | Behaviour from |
  |---|---|---|---|---|
  | `car` | 0.75 | 4.5 (Midtown's) | Midtown's | Midtown |
  | `truck` | 0.10 | **uniform 8.0–16.5** | 25.0; 1.3; −4.0 | the engine's demand tour |
  | `bus` | 0.05 | 12.0 | 19.44; 1.5; −4.0 | its PT tour, **without PT routes** |
  | `bike` | 0.05 | 1.8 | 8.33; 1.5; −3.0 | its `bike_lane` example |
  | `van` | 0.05 | 5.5 | 33.33; 2.0; −6.0 | this draft (a name with no built-in shape) |

- **`demand.yaml`:** every flow's `class: car` becomes `mix: mixed` (65 flows).

Why these:
- **Every class of decision 2 and the fallback** in one run: `van` is a name no built-in
  shape takes, so the plain shape is drawn from real rows, not only from the synthetic
  test (gate 5).
- **A truck's length varies** (`Stochastic`'s `uniform`), so the shapes' stretch (§2.6.5)
  is seen at 8 m and 16 m in one video; the FCD carries each vehicle's sampled length.
- **A bus with no PT route** is a vehicle like any other that the engine's config makes
  easy (decision 6): `is_public_transit` is not read by the core, and a PT route would
  need stops and a schedule on Midtown's network. It drives the same flows.
- **Every Midtown lane allows every class** (533 lanes, `allowed_classes: []`), so no
  network edit is needed; the engine reads classes from `project.yaml` only (the demand
  file's own `vehicle_classes` list is not a field of `DemandConfig`, and is left as it
  is).

**Measured** (§2.19): **974 vehicles** (car 732, truck 100, bus 54, bike 47, van 41;
797 completed, 178 active at 1,200 s, 1 abandoned), 195,709 rows, the first at 1.1 s and
the last at 1,199.1 s. **FCD SHA-1 `c8d1d569423befbe9cfce9f9a94a04d75562508a`, the same in
3 of 3 runs,** each from a fresh `git archive` and edit; 3.8 MB of FCD, 5.0 MB for the
project. The runs took 635, 305 and 32 s at loads of 44 falling to 32 from another
session's build (Midtown's own run takes 26 s); the time is not gated.

**How it is built in Phase 1:** `scripts/fixture.sh mix` builds `scratch/mix/` the way the
probe's `mix-fixture.sh` did: steps 1–2 (the engine), the archive at `e2de274`, the halving
(shared with `midtown`, one function), the two edits in a Python block that asserts the
2,850 and the 65, and the run. It fetches no buildings: the network is Midtown's, so a
render reads `scratch/midtown/buildings.geojson` (vis-002's cache, read-only). It
fails if the FCD's SHA-1 is not `c8d1d569…` (a pin move re-freezes it, vis-001 §2.2.2).

### 2.6 The shapes (OQ-1)

#### 2.6.1 Cuboids, in the box's frame, drawn as today's box is

Every shape is a short list of **cuboids** ("parts"), each given in the vehicle's own
frame: `x` forward from the placed point along the heading, `y` up from the ground, `z`
to the vehicle's right, in metres. A part is the shaded unit cuboid of today
(`src/draw.rs:shaded_box_mesh`: top 1, long sides 0.55, ends 0.4, bottom 0.25) scaled and
moved into place, then turned by `90° − heading` about +Y and moved to the placed point,
lifted `rank · RANK_LIFT_3D` (0.001 m), exactly as `src/draw.rs:box_transform` places
today's box. So a shape stands where today's box stands, on the same point, at the same
heading, and the floor of every shape is 0.05 m, today's box's.

Cuboids only, unlit and shaded by face, because that is today's look (vis-001 §2.6:
"flat-shaded, flat colors, no textures, no shadows") and today's draw: one mesh of
triangles with a colour per vertex (vis-001 §2.12.2). No curve, no wheel, no rounded
corner: a wheel is under a pixel from 30 m up, and a rounded corner costs vertices for an
edge MSAA already softens.

**Lamps and glass are plates**, 0.02 m thick, laid on a face (`T` = 0.02), so that they
show from outside and never fight the face for depth (§2.11).

#### 2.6.2 The car (`car`)

Width **1.8 m** (today's box, the dashboard's), its roof at **1.5 m** (today's box's top is
1.55 m).

| Part | `x` | `y` | `z` | Paint |
|---|---|---|---|---|
| body | `−L/2 … L/2` | 0.05 … 0.85 | ±0.9 | body |
| cabin | `−0.32·L … 0.16·L` | 0.85 … 1.5 | ±0.78 | roof: body; sides and ends: glass |
| tail panel | `−L/2 − T … −L/2` | 0.47 … 0.83 | ±0.85 | dark |
| brake lamps (2) | `−L/2 − 2T … −L/2 − T` | 0.55 … 0.75 | ±(0.5 … 0.85) | brake |
| headlamps (2) | `L/2 … L/2 + T` | 0.5 … 0.65 | ±(0.5 … 0.85) | head |

The cabin sits behind the middle (its centre at `−0.08·L`), so the bonnet is longer than
the boot and the front reads from any side. A 4.5 m car's cabin is 2.16 m long.

#### 2.6.3 The truck (`truck`) and the bus (`bus`)

**Truck:** width **2.5 m** (the dashboard's `truck`), height **3.6 m** (a box truck's).

| Part | `x` | `y` | `z` | Paint |
|---|---|---|---|---|
| chassis | `−L/2 … L/2` | 0.05 … 0.95 | ±1.1 | dark |
| cab | `L/2 − c … L/2`, `c = min(0.35·L, 2.3)` | 0.95 … 3.1 | ±1.25 | body |
| box | `−L/2 … L/2 − c − g`, `g = min(0.03·L, 0.25)` | 0.95 … 3.6 | ±1.25 | body |
| windscreen | `L/2 … L/2 + T` | 1.9 … 2.9 | ±1.1 | glass |
| side windows (2) | `L/2 − c + 0.3 … L/2 − 0.15` | 1.9 … 2.8 | ±(1.25 … 1.25 + T) | glass |
| tail panel | `−L/2 − T … −L/2` | 0.97 … 1.38 | ±1.2 | dark |
| brake lamps (2) | `−L/2 − 2T … −L/2 − T` | 1.05 … 1.3 | ±(0.75 … 1.15) | brake |
| headlamps (2) | `L/2 … L/2 + T` | 1.05 … 1.3 | ±(0.75 … 1.15) | head |

The cab is lower than the box (3.1 against 3.6 m) and parted from it by a 0.25 m gap, so
the front reads from above too. One rigid body (§1.1).

**Bus:** width **2.55 m** (a city bus's), height **3.1 m**.

| Part | `x` | `y` | `z` | Paint |
|---|---|---|---|---|
| body | `−L/2 … L/2` | 0.05 … 3.1 | ±1.275 | body |
| window bands (2) | `−L/2 + 0.6 … L/2 − 0.4` | 1.25 … 2.6 | ±(1.275 … 1.275 + T) | glass |
| windscreen | `L/2 … L/2 + T` | 0.9 … 2.85 | ±1.15 | glass |
| rear window | `−L/2 − T … −L/2` | 1.8 … 2.7 | ±0.9 | glass |
| tail panel | `−L/2 − T … −L/2` | 0.42 … 0.88 | ±1.225 | dark |
| brake lamps (2) | `−L/2 − 2T … −L/2 − T` | 0.5 … 0.8 | ±(0.85 … 1.2) | brake |
| headlamps (2) | `L/2 … L/2 + T` | 0.45 … 0.7 | ±(0.85 … 1.2) | head |

The tall windscreen and the band of windows down each side are what part a bus from a
truck of the same length; a bus has no cab step.

#### 2.6.4 The bike (`bike`) and the plain shape

**Bike:** width **0.44 m** (a rider's shoulders), height **1.65 m** (a rider's head).

| Part | `x` | `y` | `z` | Paint |
|---|---|---|---|---|
| frame and wheels | `−L/2 … L/2` | 0.05 … 0.75 | ±0.04 | dark |
| rider | `−0.35 … 0.15` | 0.75 … 1.65 | ±0.22 | body |

**No lamps on a bike:** a bike's lamps are under a pixel in every frame but the closest
(0.1 m is 5 px only at `view`'s closest zoom, `k` = 0.02 m/px), and a bike has no brake
light. The rider carries the speed colour.

**The plain shape** (any other name, decision 2, and a vehicle with no class, §2.12):
**today's box**, `L × 1.8 × 1.5` m from 0.05 m, all body colour, with the car's tail
panel, brake lamps and headlamps at the car's heights. A class this repo does not know
reads as "a vehicle" and still shows which way it faces and when it brakes. Rejected: a
car (a `van` or an `hgv` drawn as a car misleads), or nothing new at all (no lights on the
very classes a project adds).

#### 2.6.5 How a shape stretches with its length

The FCD's `vehicle_length` is each snapshot's (decision 2; it varies per vehicle when the
class's length is a distribution, as the mix fixture's trucks, §2.5). Each shape takes it
as `L`:
- **car:** everything along `x` scales with `L` but the lamps' widths (the cabin is
  `0.48·L`);
- **truck:** the cab is 2.3 m from `L` = 6.57 m up and `0.35·L` below, and the box takes
  the rest less the gap (`min(0.03·L, 0.25)`), so an 8 m truck has a 5.46 m box and a
  16 m truck a 13.45 m one (the mix's trucks run 8.01–16.23 m);
- **bus:** the window bands run its length less 1 m;
- **bike:** the frame scales, the rider does not;
- **plain:** as today's box.

Any `L > 0` gives a shape; the engine's lengths are positive. No class has a length of
its own here: a 4.5 m `bus` is a short bus.

#### 2.6.6 The names

`vehicles::kind(name)`: `"car"` → car, `"truck"` → truck, `"bus"` → bus, `"bike"` →
bike, anything else → plain, compared exactly (case and spelling: the engine's ids are
exact, and `Car` is another class to it). No alias (`lorry`, `coach`, `bicycle`): a
project names its own classes, and an alias list is a guess about other people's
projects. A user who wants `hgv` drawn as a truck names the class `truck`.

#### 2.6.7 The alternative (OQ-1)

Two levels of detail, each drawn by the probe on every scene at the same frames (§2.19):
- **(a) As above** (proposed): the masses and the glass (the car's cabin glass, the
  truck's windscreen and side windows, the bus's windscreen, window bands and rear
  window).
- **(b) The masses alone:** the same parts with no glass, the car's cabin all body colour;
  2 to 4 fewer cuboids a vehicle.

**Recommended: (a).** The glass is what makes a cabin a cabin and a bus a bus at a tilt
(`sheet-oq1-shapes.png`, middle and right columns); from straight down the two are the
same (the glass is on the sides). The choice is the user's, on the stills.

### 2.7 The colours (OQ-2)

| Paint | Colour (sRGB) | Shaded by face |
|---|---|---|
| body | the speed colour (`scene::SPEED_BINS`, as today), or `links::CARS` `(235, 235, 235)` while link colours are on (vis-003 OQ-8) | yes |
| glass | `GLASS` `(34, 42, 56)`, a dark blue-grey | yes |
| dark (chassis, frame, tail panel) | `DARK` `(38, 38, 42)` | yes |
| brake lamps | `BRAKE_OFF` `(92, 22, 26)`, or `BRAKE_ON` `#ef4444` `(239, 68, 68)` while braking (§2.8) | no |
| headlamps | `HEAD_ON` `(255, 240, 196)` (§2.9) | no |

- **The speed colour stays on the body,** so a video with shapes reads speed as today's
  does. From straight down a shape is its roof, in the speed colour, and looks like
  today's box (§2.10).
- **The lights do not change with link colours:** neutral bodies and red brake lamps are
  what vis-003 OQ-8's neutral cars want (the street says how fast, the cars say where they are and
  when they stop).
- **What the stills found:** the slowest speed bin is red `(230, 57, 70)`, and a queue
  standing at a red light, the moment brake lights matter most, is all that colour. A red
  lamp on a red rear face does not show (`mix-rear-rec.png`), nor does the dashboard's
  red strip (`mix-rear-strip.png`). Two ways out, each drawn:
  - **(a) The body in the speed colour, the lamps on a dark tail panel** (proposed): a
    dark band across the rear behind the lamps, as a car's light cluster, so a lit lamp
    reads against dark on any body (`mix-rear-panel.png`);
  - **(b) The speed colour on the roof only:** sides and ends a neutral grey `(200, 202,
    206)`, the roof in the speed colour; the lamps read on the grey (`mix-rear-roof.png`),
    and from above the roofs still give the speed, but a tilted view is mostly grey sides
    (`sheet-oq2-colour.png`).

**Recommended: (a),** which keeps today's reading of speed at every pitch. The choice is
the user's (OQ-2).

### 2.8 Brake lights (OQ-3, OQ-4)

#### 2.8.1 The rule: the dashboard's, on the row at or before `t`

A vehicle's brake lights are **on at `t` when its FCD row at or before `t` has
`acceleration < 0` or `speed < 0.01`** (`vehicles::STANDING`), the dashboard's rule
(§2.4.3), and off otherwise. "The row at or before `t`" is the vehicle's latest row with
`time ≤ t + 1e-6`, as `Fcd::snapshot_at` takes a snapshot (vis-001 §2.8), and its first
row when `t` is before it (a vehicle is drawn from `t_first − 1e-6`, vis-001 §2.8.5).
- **Held between rows,** so a lamp changes only at an FCD time, never between two, and a
  frame's lamps are a function of `t` alone (gate 9).
- **The row before, not the row after.** Over a row interval, the speed change has the
  sign of the next row's acceleration slightly more often than of this row's: 93.7 % and
  92.3 % of Midtown's 146,725 intervals with a speed change (95.6 % and 93.5 % on
  urban_grid, 93.7 % and 92.3 % on the mix). The 1.4 points are not worth a light that
  comes on before the row that says so, and the held row is what the HUD and the signals
  take (vis-003 §2.20.4, §2.6).
- **`acceleration < 0` with no threshold,** as the dashboard. A threshold changes little
  (`out/brake-rules.txt`, every row of each file; flips are changes between a vehicle's
  consecutive rows):

  | Rule | Midtown: rows lit; flips per vehicle-minute; one-row runs | urban_grid | mix |
  |---|---|---|---|
  | **`a < 0` or `v < 0.01` (proposed)** | **53.4 %; 5.69; 14.2 %** | **48.3 %; 4.81; 19.8 %** | **53.3 %; 5.57; 13.9 %** |
  | `a < −0.1` or `v < 0.01` | 47.3 %; 5.36; 14.1 % | 37.7 %; 4.73; 19.2 % | 47.1 %; 5.20; 13.8 % |
  | `a < −0.5` or `v < 0.01` | 42.2 %; 4.83; 13.3 % | 29.8 %; 4.14; 16.2 % | 42.5 %; 4.66; 12.5 % |
  | `a < −1.0` or `v < 0.01` | 39.3 %; 4.36; 13.1 % | 25.9 %; 3.93; 17.9 % | 39.9 %; 4.20; 12.3 % |
  | `a < 0` or `v < 0.1` | 53.6 %; 5.68; 14.2 % | 48.4 %; 4.81; 19.8 % | 53.5 %; 5.56; 13.8 % |

  Even at −1 m/s², a light still flips 4.4 times a vehicle-minute on Midtown: the flips
  are the traffic's (stop-and-go at the lights), not noise around zero.
- **Standing still is `speed < 0.01`.** 7,565 of Midtown's 55,536 standing rows have
  `acceleration < 0` too; the other 47,971 are lit only by this half of the rule, a queue
  at a red light with its lights on, as the dashboard draws it. Link colours' 0.1 m/s
  (vis-003 §2.21.4) would light 294 more of Midtown's rows (0.15 %); not proposed: 0.01 is the
  dashboard's, and the two differ in a car creeping under 0.1 m/s, which is not standing.

#### 2.8.2 How often they flick (OQ-3)

At each FCD time of the gates' Midtown window (300–360 s, 60 times;
`out/brake-window.txt`): **199.0 vehicles drawn, 106.8 lit, and 18.5 lamps change**
(7 to 86; 86 at a signal's switch, when a queue moves off). urban_grid's whole run: 74.1
drawn, 35.8 lit, 5.9 change (0 to 26; 34 of its 291 FCD times change none). The mix: 190.5,
103.2 and 17.5. So in a video at 1×, once a second about one vehicle in eleven turns its
lamps on or off, all in the same frame, and never between.

**One-row runs.** 14.2 % of Midtown's flips (19.8 % urban_grid's, 13.9 % the mix's) are a
light that is on (or off) for one row only: a one-second blink. They are real braking:
the median acceleration of a one-row "on" is −0.43 m/s² on Midtown (p10 −2.55, p90
−0.005), −2.0 on urban_grid; 32 % of them are above −0.1 (`out/brake-blips.txt`).
- **(a) As the rule says** (proposed): every row's state, so the lights say what the FCD
  says, and gate 6 checks them against it straight.
- **(b) Debounced:** a one-row run takes its neighbours' state, a function of a vehicle's
  rows `i − 1`, `i`, `i + 1` (still of `t` alone). 23.3 % fewer flips on Midtown, 33.6 % on
  urban_grid, 22.6 % on the mix; the share lit barely moves (53.4 → 53.2 %). It hides
  real one-second braking and lights a row the rule says is off.

The choice is the user's (OQ-3), on the clips (§2.19). **Recommended: (a).**

#### 2.8.3 How they look (OQ-4)

Three looks, each drawn by the probe on the line-up and on the mix's queue at the same
frames (`sheet-oq4-brake.png`, §2.19):
- **(a) Lamps.** Two lamp plates on the rear face (§2.6's tables), `BRAKE_OFF` dark red
  `(92, 22, 26)` when off and `BRAKE_ON` `#ef4444`, the dashboard's red, when on,
  unshaded on every face so a lit lamp reads as a light. On a red body (the slowest speed
  bin, a queue at a red light) a lit lamp all but vanishes.
- **(b) The dashboard's strip.** While braking, the whole rear face and the rear 4.5 % of
  the top in `#ef4444`; while not, the same plates in the body colour, so nothing shows.
  The strongest signal (a truck's rear is a red wall), and it too vanishes on a red body.
- **(c) Lamps on a tail panel** (proposed, §2.6's tables): (a)'s lamps on a dark band
  across the rear, so a lit lamp reads against dark on any body colour.

**How far they show.** Brake lights are a close-up look. In the orbit at `height_m` 250
(`mix-orbit-*.png`) a car's rear is a few pixels: against (a), (c) changes 70 pixels of
the 1080p frame and (b) 168, and none of the three is seen without a zoom
(`zoom-orbit-panel-strip.png`). They read from about `height_m` 60 down: the line-up's
22–62 m and the close-ups' 30–40 m.

**Recommended: (c).** (b) is the dashboard's and the loudest, but turns a truck's back
into a red wall and still fails on a red body; (c) reads on every body and keeps the rear
a rear. The choice is the user's (OQ-4), on the stills and the clips.

### 2.9 Headlights (OQ-5)

A look only (decision 5): two lamp plates on the front face, as the brake lamps at the
rear (§2.6's tables), unshaded.
- **(a) Always lit** (proposed): `HEAD_ON` warm white `(255, 240, 196)`. The run has no
  time of day (§1.1), and lit headlights are what a viewer reads as "this end is the
  front", day or night (daytime running lights).
- **(b) Unlit lenses:** light grey `(170, 172, 176)`, unshaded.

Never anything beyond the vehicle: no beam on the road, no bloom (§1.1). Bikes carry none
(§2.6.4). At the line-up's 24 m the lit lamps mark every front (`lu-front-rec.png`); from
60 m up they are a pixel or less, and (a) and (b) look the same (`sheet-oq5-head.png`,
bottom row). **Recommended: (a).** The choice is the user's (OQ-5), on the stills.

### 2.10 The far end: no fade, no box

From high up a vehicle is a few pixels, and its lamps and glass are under one. Nothing
switches with distance:
- **A shape degrades into its footprint.** Seen from above it is its roof in the speed
  colour, as today's box is; the glass is on the sides and ends, the lamps on the ends. In
  the city flight's first frame (`height_m` 1,800, straight down, 300 s on the mix), the
  shapes' frame differs from today's boxes' in **455 pixels** of 2,073,600, about two a
  vehicle, from the trucks' and buses' widths (2.5 and 2.55 against 1.8 m) and the bikes'
  0.44 m (`out/far.txt`, `sheet-far.png`). In the orbit (`height_m` 250, pitch 35), 1,613.
- **No fade.** The boxes are one opaque mesh (vis-001 §2.12.2); a fade would need blending
  and the sorted pass that §2.12.4 there rejected for determinism.
- **No switch to a box** under some size (the dashboard drops its lights below 2 px of
  half-width, §2.4.3): a shape already looks like its box from far, and a switch at a
  threshold pops when a flight crosses it. A lamp under a pixel is an MSAA sample at most:
  in the orbit, every look's lamps change at most 168 pixels against another's (§2.8.3).

### 2.11 Depth, ties and determinism

- **One mesh, one draw, in `vehicle_id` order,** as vis-001 §2.12.2: each vehicle's parts in
  §2.6's table order, vehicle after vehicle, so a tie between two vehicles goes to the later
  one, the higher id, and a tie between a plate and its face to the plate, which comes
  after it. The rank lift stays 0.001 m a vehicle.
- **The identical-shape rule:** a vehicle whose placed `x`, `y`, heading and length, bit
  for bit, **and kind** equal a later vehicle's is not drawn, ranks counted first (vis-001
  §2.12.2's rule with the kind added to the key). Its brake state is not in the key: the
  shape drawn shows its own. Two stacked vehicles of different kinds are both drawn, each
  whole, and where their faces coincide the later wins, as any overlap (vis-001 §2.12.3).
  How many such stacks the mix holds is not counted here; gate 8 renders its flights
  twice.
- **Plates 0.02 m proud.** A lamp or a pane lies `T` = 0.02 m outside its face, and a
  lamp on the tail panel `2T`. At the view's widest zoom a depth step is about 0.5 mm
  (vis-003 §2.21.10), so 0.02 m is 40 steps; at the launch fit near the look-at point,
  0.16 mm (vis-001 §2.11.2), 125. The plate's back face is inside the body and never
  drawn over it.
- **Against everything else:** every vehicle is above 0.05 m, over the road (0), the tint
  (0.005), the markings (0.01–0.02) and the dots' lowest point (0.03), as today. A truck
  is 3.6 m, more than twice today's 1.55 m, so at a tilt it hides more street behind it,
  and more of a signal dot that stands behind it, as a real truck does; see-through cuts
  buildings only (vis-002 §2.15). The HUD, the legend and the credit line are UI, over
  every vehicle.
- **Engine OQ-29** (drawn bodies can overlap by up to about 1 m, a vehicle entering a
  junction mid-lane-change; 0.27 m at Midtown 5650, N442): with taller, wider shapes such
  an overlap shows as two bodies through each other in a close-up. Drawn as it comes
  (§1.1); the tie goes to the higher id, the same on every run.
- **Deterministic:** the parts are constants and `L`; the looks are a function of `t`
  and the file (§2.8.1, §2.12); the mesh is built in `f32` on the CPU in a fixed order.
  No `HashMap` is walked. Measured: four flights rendered twice each, every pair equal;
  frames alone equal the same frames in sequence (§2.19).

### 2.12 Reading the class and the acceleration

**Nothing shipped changes type.** `fcd::Row`, `fcd::Fcd`, `fcd::VehicleRows` and
`render::VehicleBox` are built field by field in eight test files (`tests/camera.rs`,
`ties.rs`, `view.rs`, `slider.rs`, `see_through.rs`, `parts.rs` build `VehicleBox`;
`tests/hud.rs` and `links.rs` build `Row`, `VehicleRows` and `Fcd`), so a new field in any
of them edits those tests, and gate 3 holds them unedited. So the two columns are read on
their own, into a table beside the run, as link colours keep their rows beside it
(vis-003 §2.21):
- **`vehicles::read(path) -> Result<Fleet>`** reads `time`, `vehicle_id`, `speed`,
  `acceleration` and `vehicle_class` from the FCD file, every row of the file (not only
  the window's: a vehicle drawn at `--from` may have its row before it), into one
  `Vehicle { vehicle_id, class: String, kind: Kind, times: Vec<f64>, brakes: Vec<bool> }`
  per vehicle, in `vehicle_id` order, each one's rows in `time` order (sorted by
  (`vehicle_id`, `time`), as `fcd::by_vehicle`). A vehicle's class is its first row's; the
  engine never changes a vehicle's class. The class names are looked up through a map,
  never walked (no order from walking a `HashMap`).
- **`Fleet::looks(t, &[VehicleBox]) -> Vec<Look>`**, one `Look { kind, brake }` per box, in
  the boxes' order, by binary search on `vehicle_id` and then on `times`. A box whose
  vehicle is not in the table (none at the pin) is `Kind::Plain`, unlit.
- **Read once, when shapes are on** (`Job::set_shapes(true)`, §2.13; `view` at launch).
  Off, the file is not read again and nothing is built.
- **Measured** (§2.19): 28.5 ms on Midtown, 26.2 ms on the mix, 2.2 ms on urban_grid, once
  per job.

**Columns that are missing or of another type.** At the pin every run writes both. Older
files exist: `scratch/derived/no_class_length.parquet` (`scripts/fixture.sh`'s pre-asm-020 copy) has no
`vehicle_class`, and a file from before the engine wrote `acceleration` would have none.
- **No `vehicle_class`:** every vehicle is the plain shape. No error: a run that renders
  today renders tomorrow, in boxes with lights.
- **No `acceleration`:** brake lights from standing still alone (`speed < 0.01`).
- **A column of another type** (say `vehicle_class` as plain `Utf8`, or `acceleration` as
  `Float32`): **`Utf8` and `LargeUtf8` are taken** for `vehicle_class` beside the
  dictionary of `Utf8` (any integer key); anything else is the "schema mismatch" error of
  `rules/inputs.md`, one line, exit 1, before the first frame: `error: schema mismatch:
  <file> column "vehicle_class" is <type>, expected Dictionary(Int32, Utf8) or Utf8`, and
  `… column "acceleration" is <type>, expected Float64`. It is raised only with shapes on
  in a 3D render or `view`, since nothing else reads them; `--no-shapes` renders the file
  as today. No engine at the pin writes either.
- **A null** in either column (the engine's are non-null) reads as missing for that row:
  no class is the plain shape, no acceleration is standing still alone.

### 2.13 The flag, the key, and their errors

- **On by default** in `render --camera` and in `view` (decision 4).
- **`--no-shapes`** on `render` and `view`: `render --camera` draws today's boxes and its
  frames are today's, byte for byte (gate 2); `view` starts with boxes. The name follows
  `--no-streets`, `--no-signals` and `--no-hud`; the lights are parts of the shapes, so the
  one flag turns all three off. Rejected: `--boxes` (says what is drawn, not what is
  turned off, and breaks the `--no-` pattern), `--no-vehicle-shapes` (long for no gain).
- **`--no-shapes` where nothing changes is accepted, with no effect:** `render` without
  `--camera`, whose orthographic frames are today's with or without it (decision 3), as
  `--no-see-through` is accepted where nothing is cut (vis-002 §2.15.6). A harness can
  then pass it on every call to keep today's output.
- **`--shapes` is not a flag:** clap rejects it, exit 2.
- **`V`** in `view` (*vehicles*; by position, `KeyCode::KeyV`, unbound today: `view` uses
  `A B C D E F H K L M Q R S W X`, space, `Esc`, the arrows, `+` and `−`; free are `G I J
  N O P T U V Y Z`) flips shapes and boxes at any moment and changes nothing else; in the
  frame order after `C`. `view --no-shapes` starts with boxes and `V` shows shapes, reading
  the table at that first press; on a file §2.12's check refuses, `V` then stays inert (as
  `L` on a network with no signal) and the window stays open. The keyframe line does not
  carry it.
- **The library keeps today's default.** `Job::prepare*` build every job with shapes off,
  as streets, signals, the HUD and link colours (vis-003 §2.20.11), so every test that
  builds a job draws as today. `src/main.rs` turns them on with `Job::set_shapes(true)`
  unless `--no-shapes` is given; on an orthographic job `set_shapes` does nothing and
  reads nothing.
- **The one new error** is §2.12's schema mismatch, raised after `run::load` (in
  `rules/inputs.md`'s order: after the link colours' window, last). No run at the pin
  meets it.

### 2.14 The scripts that read references

Every CLI render in `scripts/` that renders with `--camera` and compares its frames with
a reference, with an earlier phase's renders, or with a count the baseline recorded,
gains **`--no-shapes`**, and nothing else: **23 lines.**
- `gates.sh` **2**: `camera` and `camera2` (Phase 5 gate 10, compared by hand with
  `ref-pin90b39292-camera` and with each other);
- `gates-see-through.sh` **8**: gate 2's three `--camera` renders (`see-through-city-off`,
  `nost-flight-roads`, `default-flight-roads`, against their references) and gate 7's
  five, whose "frames differ" lines are recorded against the baseline;
- `gates-parts.sh` **10**: gate 2's four `--camera` renders and gate 9's six (recorded);
- `gates-signals.sh` **1**: `case_`'s `off-` render, against `look-p6`;
- `gates-hud.sh` **1**: `case_`'s `nohud-` render, against `signals-p1`;
- `gates-links.sh` **1**: `case_`'s `off-` render, against `hud-p2`.

The comment above each changed block names the flag, as vis-003 §2.20.12 did. The
orthographic lines of the same scripts need nothing (decision 3); a `case_` line is shared
by its orthographic cases, where the flag has no effect.

**The other 30 command lines with `--camera` (50 in all, 20 of them edited above) and every `view` line still pass with shapes drawn:**
`gates-ties.sh` gate 9 (4 renders, compared with each other), `gates-city.sh` gate 13 (3,
with each other), the `case_` pairs of `gates-signals.sh`, `gates-hud.sh`,
`gates-links.sh`, `gates-streets.sh` and `gates-look.sh` (each pair with each other, and
"differs" from an earlier phase), and the error and flag-edge renders (`gates-streets.sh`'s
`--streets`, `gates-see-through.sh` gate 10's three); every `view` line checks an exit
code, an error line or the bench's JSON, never a frame. From Phase 1 the pairs check the
shapes' determinism: `gates-ties.sh` gate 9 is Midtown's stacked frozen vehicles (vis-001
§2.12), now as shapes. Their logs equal the baseline's apart from times, loads and the
bench's JSON (gate 3), since they print equal counts, not hashes.

`gates-hud.sh` and `gates-links.sh` write into `scratch/out/hud/` and `scratch/out/links/`
over the shipped renders; neither is read by this phase's gates, which compare with
`scratch/out/hud-p2/` (read-only, vis-003 Phase 3's copy, never written by a script).

### 2.15 Cost

| | Today's box | car | truck | bus | bike | plain |
|---|---|---|---|---|---|---|
| Cuboids (as proposed, OQ-1 (a), OQ-4 (c)) | 1 | 7 | 11 | 10 | 2 | 6 |
| Vertices; triangles | 24; 12 | 168; 84 | 264; 132 | 240; 120 | 48; 24 | 144; 72 |

- **The most vehicles drawn at once** (`Motion::max_drawn`, sampled at 2,000 frame times
  over each run): Midtown **215**, the mix **211**, urban_grid **109**. At 168 vertices a
  car, Midtown's worst frame is about 36,000 vertices, a sixth of the buildings' 213,880
  (vis-002 §2.15.5), against 5,160 for today's boxes.
- **The mesh's build,** per frame, in the probe (`out/cost.txt`; looks and mesh together,
  without the tail panel): Midtown 0.98 ms, the mix 0.94 ms, urban_grid 0.43 ms, against
  43, 41 and 18 µs for today's boxes. The probe looks up its look options in the
  environment once per part; Phase 1's constants do not, so this is an upper bound.
- **The table's read** (`vehicles::read`, once): Midtown 28.5 ms, the mix 26.2 ms,
  urban_grid 2.2 ms.
- **`view --bench 20`** on Midtown with `--buildings` (streets, see-through, signals and
  the HUD on), two passes, each with shapes (`VEH=1 PANEL=1`) then without, back to back
  (`out/bench.txt`): **60.00 and 60.00 fps with shapes** (p99 17.23 and 17.07 ms, worst
  18.76 and 20.70), 60.00 and 60.00 without (p99 17.03 and 17.17, worst 18.71 and 20.76), at
  a 1-min load of 3.4 falling to 2.4. The bench plays at 2× from 140 s, so the mesh is
  rebuilt every frame.
- **Render time:** recorded, not gated (gate 10). The probe's renders at 1920×1080, the load before and
  after each from other sessions: Midtown's city flight **70 s** without shapes (7.7) and
  **107 and 98 s** with them (5.9–8.2); the orbit 67 s without; the mix's city flight 149 and
  91 s and its orbit 150 and 89 s with (25 falling to 5); urban_grid's flight 283 s without
  (8.2) and 348 and 228 s with (18 falling to 6). On Midtown at a like load, shapes cost
  about **15 ms a frame**, of which the mesh's build is 1 ms (above); the rest is Bevy's
  handling of a mesh of about 36,000 vertices replaced every frame against about 5,000.
  Not optimised in Phase 1 (gate 10 records it); the bench is the bar (gate 11).

### 2.16 One phase

Decision 1 asks the draft to measure whether one review and one build hold Phase 1. They
do:
- **About vis-003 Phase 3's size.** One new module with no Bevy types (`src/vehicles.rs`:
  the table and its read, the kinds, the brake rule, the parts), one mesh builder beside
  `boxes_mesh` in `src/draw.rs`, a switch in `src/render.rs` and `src/lib.rs`, the flag in
  `src/main.rs`, `V` in `view`, one test file, one script and one `scripts/fixture.sh`
  mode, and 23 script lines that gain a flag. vis-003 Phase 3 was a module, a mesh, a UI
  line, three flags, a key, two errors, a test file and a script, reviewed in two rounds
  and built on one branch; vis-003 Phase 1 edited 29 script lines the same way.
- **No engine request, no new input, 0 packages.** The probe built all of it but the flag,
  `V`, the missing-column fallbacks and the errors in about 460 lines beside its probe
  binary (`repo/src/vehicles.rs`, `repo/src/draw.rs`'s tail, the hooks).
- **The parts do not stand alone:** a shape without its lights is the half of decision 1
  the lights answer, and the look questions (OQ-1, OQ-2, OQ-4, OQ-5) are one set of stills.
- **The fixture is small:** a mode of `scripts/fixture.sh` (about 60 lines, two of them
  shared with `midtown`) and a 32 s engine run on a quiet machine.

No split is proposed.

### 2.17 What it does not do

- **No turn signals** (Phase 2, §2.3), no reversing lights, no hazard lights.
- **No orthographic shapes,** no brake lights in the orthographic render (decision 3).
- **No per-vehicle variety:** one shape per class, one colour rule (§2.7).
- **No articulated vehicles, no wheels that turn, no tilt in a turn** (§1.1).
- **No class-aware pick:** `view`'s pick keeps today's `length × 1.8 m` footprint within
  0.80 m (`rules/view.md`). Every point of a 2.5 m truck's body is within 0.35 m of that
  footprint, so a click on it picks it; a 0.44 m bike is picked within 1.7 m of its
  centreline, as a car is.
- **No class in the HUD, the link colours or the keyframe line;** the counts and measures
  count every row, whatever its class, as today (vis-003 §2.20, §2.21.4).
- **No PT behaviour:** a `bus` is drawn as a bus whatever its route (§2.5).

### 2.18 Build cost

**0 packages,** no Bevy feature, no `Cargo.toml` change: the columns are read with the
`arrow-array` and `parquet` crates the FCD reader uses (`DictionaryArray<Int32Type>`), the
shapes are the shaded cuboid of today in the mesh of today. One new module,
`src/vehicles.rs`, with no Bevy types. No engine request.

### 2.19 Measured while drafting (2026-10-23)

Read-only against the engine at the pin and both fixtures (FCD SHA-1s `01becc86…` and
`3ad76744…`), and the mix fixture the probe built, with output only under gitignored
`scratch/vis004p1-probe/`:
- **`mix-fixture.sh`:** §2.5's fixture, into `mix/`, `mix-2/` and `mix-3/` (each a fresh
  archive, edit and run; `out/mix-runs.txt`);
- **`sql/`, `brake_stats.py`, `brake_blips.py`, `brake_window.py`, `check_looks.py`:** the
  FCD exported by DuckDB (`tmp/<name>-rows.csv`, not kept); §2.4.2's and §2.8's counts
  (`out/brake-rules*.txt`, `out/brake-blips.txt`, `out/brake-window.txt`); the probe's
  looks against the rows read straight (`out/looks-check.txt`);
- **`repo/`** is `git archive` of `origin/main` (`ca244a0`) with a draft `src/vehicles.rs`
  (`Fleet`, `read`, `looks`, `kind_of`, `brake`, `parts`), `shapes_mesh` and `set_shapes`
  in `src/draw.rs`, `Renderer::render_looks`, the table in `Job` (read when `VEH` is set,
  drawn only with a flight) and in `view`, all behind environment variables (`VEH`;
  `SHAPES=simple`, `BRAKE=strip`, `PANEL=1`, `HEAD=off`, `COLOUR=roof`, `RULE=debounce`
  for the options), and `src/bin/p4-probe.rs` (`still`, `history`, `looks`, `cost`,
  `diff`, `lineup` with `LAMPS`). Built in its own `CARGO_TARGET_DIR`, sccache off:
  4 m 54 s at a load of 28 falling to 18 from another session's engine build. Nothing in
  `src/`, `tests/` or `scripts/` was touched;
- **`run.sh`, `measure.sh`, `sheets.sh`, `kf/`:** the stills, the measurements and the
  sheets below.

**Where the probe differs from the scope:** options by environment variable, not by flag;
no `V`, no `--no-shapes`; no fallback for a missing column, no schema error; its cost
table counts parts without the tail panel; the identical key has the kind (as proposed).

**Measured with it:**
- **looks against the FCD read straight** (`out/looks-check.txt`): every vehicle drawn at
  Midtown's 300, 330, 359.97 s, urban_grid's 9.1, 150, 299.07 s and the mix's 9.1, 150,
  300, 330, 359.97 and 1,199.1 s: **1,637 of 1,637 equal** (kind and brake);
- **a frame alone against the same frame in sequence** (`out/history.txt`): the mix's
  city flight with buildings, frames 0, 1, 29, 30 and 299 of `--from 300 --to 310
  --speedup 1`, and urban_grid's `tests/flight.toml`, frames 0, 1, 30 and 299 of `--from
  9.1 --to 19.1`: **9 of 9 equal, 0 pixels differ**, with shapes and the tail panel;
- **the lamps** (`out/lamps-*.txt`): in the line-up seen from behind (`lu-all-*`), **20 of
  20** brake lamps' centres exactly `BRAKE_ON` (10) or `BRAKE_OFF` (10); from the front
  (`allfront`), **19 of 20** headlamps exactly `HEAD_ON`;
- **the far end** (`out/far.txt`): §2.10;
- **the cost** (`out/cost.txt`): §2.15;
- **the renders** (`out/renders.txt`, `runs/*.framemd5`), through the probe's
  `assimilator-video`:
  - **off** (no `VEH`): `hud-p2`'s `city`, `orbit` and `ug-camera` cases, every frame
    equal to `scratch/out/hud-p2/hud-<case>.framemd5` (read only): 1800, 1800 and 8700 of
    each; **on, orthographic** (`VEH=1`, Midtown with buildings): 1800 of 1800 equal to
    `hud-ortho-city`;
  - **on, twice each** (`VEH=1 PANEL=1`): Midtown's city flight with buildings, the mix's
    city flight and orbit with buildings, urban_grid's `tests/flight.toml`: **every pair
    equal** (1800, 1800, 1800 and 8700 of each), and **0 frames** equal to `hud-p2`'s
    `hud-city`, `hud-orbit` (the mix against Midtown's) and `hud-ug-camera`;
- **the clips** (`runs/clip-*.mp4`, `out/clips.txt`): the mix's `--from 315 --to 345 --speedup 1 --buildings` with
  `kf/clip.toml` (gate 12's close-up, following a truck from behind), 900 frames each, 24–30 s
  each at a load of 8 falling to 3: `clip-dash.mp4` (OQ-3 (a) and OQ-4 (c), the
  recommendation), `clip-debounce.mp4` (OQ-3 (b)), `clip-lamps.mp4` (OQ-4 (a)),
  `clip-strip.mp4` (OQ-4 (b)), `clip-roof.mp4` (OQ-2 (b)) and `clip-boxes.mp4` (today);
  `clip-oq3.mp4` puts (a) and (b) side by side, and `clip-oq4.mp4` (a), (b) and (c). (a)
  and (b) of OQ-3 differ from 0:04 to 0:08 and from 0:21 to 0:24 (217 of 900 frames, by
  frame hash), where a lamp that is on or off for one second in (a) keeps its neighbours' state
  in (b);
- **the bench** (`out/bench.txt`): §2.15;
- **the stills** (`stills/`, 1920×1080): each option on the synthetic line-up seen from
  three-quarters behind (`lu-all-*`), close behind (`lu-rear-*`), the front (`lu-front-*`)
  and straight down (`lu-top-*`), and on the mix at 330 s following a truck from the front
  (`mix-close-*`) and from behind (`mix-rear-*`), in the orbit (`mix-orbit-*`), and at the
  city flight's 300 s (`mix-high-*`), and on Midtown at 330 s (`mid-close-*`); the
  suffixes `boxes` (today), `rec` (OQ-1 (a), OQ-4 (a)), `simple` (OQ-1 (b)), `strip`
  (OQ-4 (b)), `panel` (OQ-4 (c), the recommendation), `roof` (OQ-2 (b)) and `headoff`
  (OQ-5 (b)). The sheets put them side by side, half size, left to right and top to
  bottom:
  - `sheet-oq1-shapes.png`: columns today, (a), (b); rows the line-up, the close-up, the
    orbit;
  - `sheet-oq2-colour.png`: columns (a), (b); rows the line-up, the queue from behind, the
    orbit;
  - `sheet-oq4-brake.png`: columns (a), (b), (c); rows the line-up close behind, the queue;
  - `sheet-oq5-head.png`: columns (a), (b); rows the line-up's front, the close-up;
  - `sheet-far.png`: columns today, shapes; rows the 1,800 m frame, the orbit;
  - `zoom-orbit-panel-strip.png`: the orbit's middle at 2×, (c) over (b).

## 3. Open questions

The look questions (OQ-1, OQ-2, OQ-4, OQ-5) are the user's, put to the stills of §2.19, one
per option at the same frames; OQ-3 to the clips. Phase 6's lesson (vis-001 §2.12; a look
decided on paper failed at the visual gate) is why each is drawn rather than described.
Each blocks gate 7's counts and gate 12, and nothing else: the scope and every gate are
written with the recommendations, and say what another answer changes.

- **OQ-1** — The shapes' detail: (a) the masses and the glass, (b) the masses alone
  (§2.6.7)? And their sizes, as §2.6's tables give them (width × top: car 1.8 × 1.5 m, truck 2.5 ×
  3.6, bus 2.55 × 3.1, bike 0.44 × 1.65, plain 1.8 × 1.55)? *(design call: the user's, on
  `sheet-oq1-shapes.png` and `lu-*.png`; blocks gates 7 and 12.)* *Recommendation:* (a),
  and the sizes as tabled. (b) drops the glass parts from the tables: gate 5's part counts
  and gate 7's counts are measured again.
- **OQ-2** — Where the speed colour goes: (a) the whole body, as today, with the lamps on a
  dark tail panel; (b) the roof only, the sides and ends a neutral grey (§2.7)? *(design
  call: the user's, on `sheet-oq2-colour.png`; blocks gate 12.)* *Recommendation:* (a): it
  keeps today's reading of speed at every pitch, and OQ-4 (c) makes the lamps read on it.
  (b) changes one paint in `shapes_mesh` and gate 7's pixels are measured again.
- **OQ-3** — The brake rule: (a) every row's state, as the FCD says; (b) one-row runs
  debounced (§2.8.2)? *(design call: the user's, on `runs/clip-dash.mp4` against
  `runs/clip-debounce.mp4`; blocks gates 6 and 12.)* *Recommendation:* (a). 14.2 % of
  Midtown's flips are one-row runs, and they are real braking (median −0.43 m/s²). (b)
  changes `vehicles::read`'s states and gate 6's straight computation the same way.
- **OQ-4** — How a brake light looks: (a) lamps, (b) the dashboard's strip, (c) lamps on a
  dark tail panel (§2.8.3)? *(design call: the user's, on `sheet-oq4-brake.png` and the
  clips `runs/clip-lamps.mp4`, `clip-strip.mp4`, `clip-dash.mp4` (c); blocks gates 5, 7
  and 12.)* *Recommendation:* (c). The finding behind it: a red lamp, or the red strip, on
  the slowest bin's red body does not show, and that is a queue at a red light.
- **OQ-5** — Headlights: (a) always lit, warm white; (b) unlit lenses (§2.9)? *(design
  call: the user's, on `sheet-oq5-head.png`; blocks gates 7 and 12.)* *Recommendation:*
  (a): the run has no time of day, and a lit lamp marks the front.
- **OQ-6** — The names: `--no-shapes` and `V` (§2.13)? *(design call: the user's;
  blocks gate 13 and the scripts' 23 lines.)* *Recommendation:* as drafted; a rename is a
  find-and-replace in the scope and the gates.

## 4. Implementation phases

Strictly sequential; each is one plan-mode pass.

### Phase 1 — Shapes: a shape per class, brake lights and headlights, in 3D, by default
*Produces the observable: yes. `render --camera` writes the run's video with every vehicle
drawn as its class's shape, its length the FCD's, its brake lights as the FCD's
acceleration and speed have them and its headlights lit, by default. With `--no-shapes`
the video is byte-identical to today's (gate 2), and the orthographic render is
byte-identical with or without it (gate 1).*

Drafted 2026-10-23; the design is §2, and the user's decisions are §2.2's 1–6. Phase 1
builds on vis-003 Phase 3 (shipped 2026-10-23, `ca244a0`) at engine `90b39292`. OQ-1 to
OQ-6 are the user's; the scope and every gate are written with their recommendations (the
glass, the body in the speed colour, every row's state, the tail panel, lit headlights,
`--no-shapes` and `V`) and say what another answer changes.

- **Scope:**
  - **`src/vehicles.rs`** (new; no Bevy types):
    - `Kind { Car, Truck, Bus, Bike, Plain }` and `kind(name: &str) -> Kind` (§2.6.6);
    - `STANDING` 0.01 m/s and `brake(speed, acceleration) -> bool` (§2.8.1);
    - `Vehicle { vehicle_id, class: String, kind, times: Vec<f64>, brakes: Vec<bool> }`,
      `Fleet { vehicles }` in `vehicle_id` order, `Fleet::new(Vec<Vehicle>)` (sorts by id;
      the tests' constructor), `read(path) -> Result<Fleet>` (§2.12: the five columns,
      every row of the file, the missing-column fallbacks, the two schema errors),
      `Fleet::vehicle(id)` and `Fleet::looks(t, &[VehicleBox]) -> Vec<Look>`,
      `Look { kind, brake }`;
    - `Paint { Body, Cabin, Glass, Dark, Brake, Head }`, `Part { lo: [f64; 3], hi: [f64;
      3], paint }`, `T` 0.02, `FLOOR` 0.05, and `parts(kind, length) -> Vec<Part>`, §2.6's
      tables row by row in their order (OQ-1 (a), OQ-4 (c));
    - the colours of §2.7 as sRGB `[u8; 3]`: `GLASS`, `DARK`, `BRAKE_OFF`, `BRAKE_ON`,
      `HEAD_ON`.
  - **`src/draw.rs`:** `shapes_mesh(boxes, looks, (cx, cy), lift, colour) -> Option<Mesh>`
    (§2.6.1, §2.11: each vehicle's parts through its transform, the shades of
    `shaded_box_mesh` for `Body`, `Cabin`, `Glass` and `Dark`, unshaded lamps, the
    identical key with the kind) and `set_shapes`, beside `boxes_mesh` and `set_boxes`,
    which are not changed.
  - **`src/render.rs`:** `Renderer::render_looks(&boxes, Option<&[Look]>)`; `render(&boxes)`
    calls it with `None`. `Some` draws `set_shapes` on the perspective path; the
    orthographic pool ignores it.
  - **`src/lib.rs`:** `Job` keeps the run's FCD path and `shapes: Option<Fleet>`, `None`
    from every `prepare*`; `Job::set_shapes(on) -> Result<()>`: on a job with a flight,
    `true` reads the table (its error is §2.12's) and `false` drops it; on an orthographic
    job it does nothing and reads nothing. `render_at(t)` passes `Fleet::looks(t, &boxes)`
    while it is `Some`. `Job::looks_at(t) -> Option<Vec<Look>>` reads them back for gates 6
    and 9.
  - **`src/view/mod.rs`, `src/view/state.rs`:** `ViewOptions` gains `shapes: bool`; at
    launch, unless `--no-shapes`, the table is read (its error is `view`'s, before the
    window); with `--no-shapes` it is read at the first `V`, and an error then leaves `V`
    inert, as `L` on a network with no signal. The state gains `shapes_shown`, from the
    option; `Pressed` gains `v`, and `V` (`KeyCode::KeyV`) flips it after `C`. Each frame
    draws `set_shapes` with the looks while shown, `set_boxes` while not.
  - **`src/main.rs`:** `--no-shapes` on `render` and `view`, with help text; `render` calls
    `job.set_shapes(true)?` unless it is given, after `job.set_links`. No other flag or
    default changes.
  - **`src/bin/fcd-derive.rs`:** a mode `class-int` that writes a copy of an FCD file with
    `vehicle_class` as `Int32` codes (gate 13's schema error); the other modes unchanged.
  - **`tests/truck-flight.toml`** (new): the close-up of gate 12, two keyframes following
    the mix fixture's vehicle 184 (a truck), `{ t = 315.000, follow = 184, height_m =
    30.00, yaw_deg = 100.00, pitch_deg = 30.00 }` and the same at `t = 345.000`; the probe's
    `kf/clip.toml`.
  - **Tests.** `tests/vehicles.rs` (new): gate 5 and gate 13's `V` headless, not ignored;
    gate 6 headless, ignored (the three fixtures); gates 7 and 9 through the GPU, ignored
    (gate 7 builds its scene, as `tests/ties.rs`'s `synthetic`; gate 9 needs the fixtures).
  - **Scripts.**
    - `scripts/fixture.sh mix` (§2.5): `scratch/mix/`, the halving shared with `midtown`
      in one function, the two edits asserting the 2,850 and the 65, the run, and the FCD's
      SHA-1 checked against `c8d1d569…`; the usage line and the header comment gain `mix`.
    - `scripts/gates-vehicles.sh` (new, offline): gates 1's and 2's `hud-p2` cases, gate
      8, gate 13's CLI cases and gate 12's renders, into `scratch/out/vehicles/`: each of
      gate 8's renders as `vehicles-<case>.mp4` and `.framemd5` and its second as
      `vehicles-<case>-2.*` (`<case>`: `city`, `mix-city`, `mix-orbit`, `ug-camera`, and
      `truck` for gate 12's close-up, which gate 8 renders twice too, 900 of 900), and the
      close-up's boxes twin with `--no-shapes`, `vehicles-truck-boxes.mp4`.
    - The 23 lines of §2.14 gain `--no-shapes`, with their comments; nothing else in them.
  - **Not edited:** every other test file and script; every source file not named above;
    `Cargo.toml` and `Cargo.lock`; `assets/`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid, Midtown and the mix fixture at engine `90b39292` (FCD SHA-1s `3ad76744…`,
  `01becc86…` and `c8d1d569…`), baseline, seed 42, with Midtown's buildings cache
  (`scratch/midtown/buildings.geojson`, SHA-256 `f241ccbd…`) for Midtown and the mix.
  Everything runs offline. The predictions come from §2.19's probe and the FCD read
  straight.

  **Baseline, before any change**, at `origin/main`:
  - **The machine:** `ps` for other builds and engine runs, and the load average, recorded,
    here and again before the gate run; if another project is building or running, the run
    waits and the user is told.
  - **The ten references** `scratch/ref-pin90b39292-*.framemd5` are present with their
    SHA-256s (`cdad89d1…` camera, `dcfdd2b7…` default, `f6255241…` ortho-city, `2139c319…`
    ortho-roads, `d6641985…` flight-city1 and city-off, `020475b7…` flight-roads1,
    `d7118c3a…` city-on, `af80a068…` orbit-off, `05d7d174…` orbit-on), and
    `scratch/out/hud-p2/`'s fourteen `hud-*.framemd5` with theirs, recorded; each pair is
    equal. Neither is written by this phase. If one is missing or differs, the run stops.
  - **Run** every gate script and every test file with `--include-ignored
    --test-threads=1`, and keep their output. Record spec-lint and `Cargo.lock`'s package
    count (500).
  - **The mix fixture** is built with `scripts/fixture.sh mix` after its commit (gate 14).

  - **What must not change:**
  1. **The orthographic render.** The scripts' orthographic reference lines are not edited
     and pass as they do: `scripts/gates.sh` its default render **8700 of 8700** against
     `ref-pin90b39292-default`, `gates-ties.sh` gate 2, `gates-see-through.sh` gate 2's
     four and `gates-parts.sh` gate 2's one **1800 of 1800** against theirs. And
     `gates-vehicles.sh`: `hud-p2`'s three orthographic cases (urban_grid's default;
     Midtown's `--from 300 --to 360 --speedup 1` with and without `--buildings`), each
     rendered as today and with `--no-shapes`, every frame equal to its
     `hud-<case>.framemd5`: **8700, 1800 and 1800 of each, six renders.** (The probe, with
     shapes on: Midtown's with buildings, 1800 of 1800 equal to `hud-ortho-city`.)
  2. **Off in 3D.** `gates-vehicles.sh`: `hud-p2`'s four camera cases (urban_grid's
     `--camera tests/flight.toml`; Midtown's `tests/city-flight.toml` with and without
     `--buildings`, and `tests/see-through-flight.toml` with) with `--no-shapes`, every
     frame equal to its `hud-<case>.framemd5`: **8700 and 1800 of 1800.** And the 23 lines
     of §2.14: `gates.sh`'s `--camera` render **8700 of 8700** against
     `ref-pin90b39292-camera` (by hand on `scratch/out/camera.framemd5`), and `camera2`
     equal to it; `gates-see-through.sh` gate 2 and `gates-parts.sh` gate 2 **1800 of
     1800** against their references; the recorded lines of `gates-see-through.sh` gate 7
     and `gates-parts.sh` gate 9 equal to the baseline's; `gates-signals.sh`'s `off-`,
     `gates-hud.sh`'s `nohud-` and `gates-links.sh`'s `off-` renders equal to their earlier
     phases' as today. (The probe, without `VEH`: `hud-city` and `hud-orbit` 1800 of 1800, `hud-ug-camera`
     8700 of 8700.)
  3. **What Phase 1 does not touch.**
     - Every test file but `tests/vehicles.rs`, unedited, passes with every printed number
       equal to the baseline's.
     - All eleven existing scripts pass, their logs equal to the baseline's apart from
       times, loads, fetch `"seconds"` and `--bench 1`'s JSON; `gates-ties.sh` gate 9 and
       `gates-city.sh` gate 13 **1800 of 1800** each, now with shapes.
     - `git diff origin/main --stat -- src/` adds `src/vehicles.rs` and changes
       `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/main.rs`, `src/view/mod.rs`,
       `src/view/state.rs` and `src/bin/fcd-derive.rs`; `-- scripts/` adds
       `gates-vehicles.sh` and changes `fixture.sh` and the six scripts of §2.14, those six
       by the 23 lines and their comments alone; `-- tests/` adds `tests/vehicles.rs`
       and `tests/truck-flight.toml` alone.
  4. **Build cost** (§2.18). `git diff origin/main -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The looks — headless:**
  5. **Synthetic** (`tests/vehicles.rs`, not ignored), with no file but those it writes:
     - **`kind`:** `car` → Car, `truck` → Truck, `bus` → Bus, `bike` → Bike; `Car`, `van`,
       `lorry` and the empty name → Plain.
     - **`parts`**, at `L` = 1.8, 4.5, 8, 12 and 16.5 for each kind: car **7** parts,
       truck **11**, bus **10**, bike **2**, plain **6**; over every part, `x` spans exactly
       `−L/2 − 2T … L/2 + T` (bike `−L/2 … L/2`), `y` from **0.05** to the kind's height
       (car 1.5, truck 3.6, bus 3.1, bike 1.65, plain 1.55), `|z|` up to half the width
       plus `T` where glass lies on a side (truck and bus) and half the width otherwise
       (bike 0.22); the truck's box at `L` 8 is **5.46 m** long and at 16.5 **13.95 m**,
       its cab **2.3 m** at both and **1.4 m** at `L` 4.
     - **`brake`:** `(5, 0)` off; `(5, −1e-12)` on; `(0.01, 0)` off; `(0.00999, 0)` on;
       `(0, 1.5)` on (standing).
     - **`Fleet::looks`** on `Fleet::new` of vehicle 7, rows at 1, 2 and 3 s braking off,
       on and off, class `truck`: at 0.5 off (its first row), 1 off, 1.9999995 on (the 2 s
       row within `1e-6`), 2.5 on, 3 off, 10 off (its last row), kind Truck each time; a box
       with id 8, not in the table, Plain and off.
     - **`read`** on parquet files the test writes with the `parquet` crate, two vehicles
       of three rows each: with all five columns, the classes and states as written; with
       no `vehicle_class`, every vehicle Plain; with no `acceleration`, lit only where
       `speed < 0.01`; with `vehicle_class` as `Utf8`, the same as the dictionary; with it
       as `Int32`, the error `schema mismatch: <file> column "vehicle_class" is Int32,
       expected Dictionary(Int32, Utf8) or Utf8`; with `acceleration` as `Float32`, `…
       column "acceleration" is Float32, expected Float64`.

     **Without the feature it fails:** there is no `vehicles` module.
  6. **The fixtures, against the FCD read straight** (`tests/vehicles.rs`, ignored). The
     test reads each fixture's parquet itself (`time`, `vehicle_id`, `speed`,
     `acceleration`, `vehicle_class`), and at every frame time takes each drawn vehicle's
     row at or before it with its own loops, and compares the kind and the state with
     `Job::looks_at` of a job with shapes on (and a flight: `tests/flight.toml` on
     urban_grid, `tests/city-flight.toml` on the other two):
     - **every frame** of Midtown's and the mix's `--from 300 --to 360 --speedup 1`
       (**1,800** each) and urban_grid's whole run at 1× (**8,700**): **0 differ**;
     - **the counts** at sampled frames (drawn; lit; by kind), the FCD read straight
       (`out/looks-check.txt`):

       | Fixture, `t` | Drawn | Lit | car / truck / bus / bike / plain |
       |---|---|---|---|
       | Midtown 300 | 184 | 137 | 184 / 0 / 0 / 0 / 0 |
       | Midtown 330 | 201 | 132 | 201 / 0 / 0 / 0 / 0 |
       | Midtown 359.97 | 202 | 142 | 202 / 0 / 0 / 0 / 0 |
       | urban_grid 9.1 | 6 | 0 | 6 / 0 / 0 / 0 / 0 |
       | urban_grid 150 | 83 | 56 | 83 / 0 / 0 / 0 / 0 |
       | urban_grid 299.07 | 107 | 78 | 107 / 0 / 0 / 0 / 0 |
       | mix 300 | 178 | 126 | 138 / 20 / 7 / 6 / 7 |
       | mix 330 | 193 | 141 | 145 / 20 / 12 / 8 / 8 |
       | mix 359.97 | 195 | 136 | 144 / 21 / 12 / 11 / 7 |

       `359.97` and `299.07` are the last frames by index (`clock.time_of(1799)` and
       `time_of(8699)`). (The probe, at these and three more of the mix's times: 1,637 of
       1,637 equal.)
  - **The shapes — through the GPU:**
  7. **Drawn** (`tests/vehicles.rs`, ignored: GPU). The probe's line-up, built in the test:
     one strip 80 m long and 16 m wide along +x through (0, 0), sampled every 1 m;
     `Renderer::new_perspective` at `Camera::fit` of it, 1920×1080, a pool of 64, no
     buildings, no credit; twelve vehicles heading 90°, ids 1–12, a row at `y` = +4 coasting
     and a row at `y` = −4 braking, each car 4.5, truck 12, truck 8, bus 12, bike 1.8 and
     plain 5.5 m from `x` = −30 with 3 m between, speeds cycling 1, 4, 7, 11, 20, 7 m/s (the
     second row one step on); `render_looks` with their looks.
     - **Seen from behind** (pose `(0, −2)`, `height_m` 58, yaw 45, pitch 35): at the pixel
       holding each brake lamp's centre on its outer face (`camera::project`), **20 of 20**
       exactly `BRAKE_ON` (the 10 braking) or `BRAKE_OFF` (the 10 coasting).
     - **Seen from the front** (`(0, 2)`, 58, yaw 225, pitch 35): **19 of 20** headlamp
       centres exactly `HEAD_ON`; the twentieth, one of car 1's, recorded.
     - **The same scene with `render`** (boxes) differs from it, and **with no vehicle** the
       two are equal.

     The counts are `==`, the probe's through the same Bevy and GPU. They depend on OQ-1's,
     OQ-4's and OQ-5's answers: another answer is measured again before the gate is run,
     and recorded.
  - **The shapes — renders:**
  8. **On, deterministic** (`scripts/gates-vehicles.sh`). Each twice, each pair equal:
     Midtown's and the mix's `--from 300 --to 360 --speedup 1 --buildings …` with
     `tests/city-flight.toml`, the mix's with `tests/see-through-flight.toml` **1800 of
     1800**, the mix's `--from 315 --to 345 --speedup 1 --buildings … --camera
     tests/truck-flight.toml` **900 of 900**, urban_grid's `--camera tests/flight.toml` **8700 of 8700** (`ffprobe`
     `1920,1080,30/1,…`). **The look changed:** Midtown's and urban_grid's first renders
     differ from `hud-p2`'s `hud-city` and `hud-ug-camera` in **every** frame (1800 of 1800,
     8700 of 8700): a vehicle is drawn in every frame of both, and no shape is its box. (The probe,
     the four flights but the close-up: every pair equal, and 0 frames equal to `hud-p2`'s.)
  9. **A frame does not depend on the frames before it** (`tests/vehicles.rs`, ignored, the
     fixtures; GPU). Each job set as `render` sets it by default. Each of these frames,
     rendered alone by a new job, equals the same frame rendered in sequence from frame 0,
     byte for byte: the mix's city flight with `--buildings`, frames 0, 1, 29, 30 and 299 of
     `--from 300 --to 310 --speedup 1`; urban_grid's `tests/flight.toml`, frames 0, 1, 30
     and 299 of `--from 9.1 --to 19.1 --speedup 1`. At each, in sequence and alone,
     `Job::looks_at(t)` equals `Fleet::looks(t, &job.boxes_at(t))` of a `Fleet` the test
     reads with `vehicles::read`. (The probe: 9 of 9 equal, 0 pixels differ.)
  - **Recorded, with one bar:**
  10. **Render time.** Gate 8's wall times beside gate 2's renders of the same cases, each
      with the load average after it. Prediction: with shapes, Midtown's flights take about
      15 ms a frame longer, about 40 % on a 1,800-frame render; recorded, not a bar. (The
      probe: Midtown's city flight 70 s without shapes and 98–107 s with, at loads of 6 to
      8, §2.15.)
  11. **`view --bench 20`** on Midtown at the default window with `--buildings` (streets,
      see-through, signals and the HUD on), with shapes and with `--no-shapes`, in two
      passes fixed before the first begins, each with shapes then without, back to back,
      the load average before and after each. Record the JSON.
      - **A `mean_fps` below 30 with shapes stops the build**, and the phase goes back to
        review. It is never re-run to get a pass.
      - Prediction: at least 30, likely 60 (vsync). (The probe: with shapes 60.00 and 60.00 fps, p99 17.23
        and 17.07 ms; without 60.00 and 60.00, p99 17.03 and 17.17, at a load of 3.4 to
        2.4.)
  - **The user's check:**
  12. **The user watches** gate 8's first renders and the clips of `gates-vehicles.sh`
      (`scratch/out/vehicles/`) against `hud-p2`'s (`scratch/out/hud-p2/`). Times are the
      video's, from 0:00. Colours: the speed colours by their bins, red (under 2 m/s),
      orange, yellow, green, cyan (13 m/s and up); "lit" is a bright red lamp, "unlit" a
      dark red one.
      In every video with shapes, the HUD, the credit line, the streets, the signals' dots and
      the buildings are as in its twin without shapes; only the vehicles differ.
      - **The truck close-up on the mix** (`vehicles-truck.mp4`: `--from 315 --to 345
        --speedup 1 --buildings … --camera tests/truck-flight.toml`, against the same render
        with `--no-shapes`, `vehicles-truck-boxes.mp4`). The camera follows a truck from
        behind and above, 30 m up. From 0:00 to 0:17 it stands in a queue at a stop line:
        its body red, its box taller than its cab, dark glass on the cab, and at the bottom
        of its back two bright red lamps on a dark band; across the junction, at the top
        of the frame, another truck faces the camera, its windscreen dark and its two
        headlamps lit. At 0:18 the truck's lamps go dark red and it
        moves off; its body turns orange at about 0:19, yellow at about 0:21, green at about
        0:24 and cyan at about 0:29, and it crosses the junction between 0:22 and 0:23. In
        the boxes' twin it is a long red box with no lamps.
      - **The mix's orbit** (`vehicles-mix-orbit.mp4`: `tests/see-through-flight.toml` with
        `--buildings`, 250 m up). Vehicles are small but their kinds read: at 0:00, in the
        lower middle, a long green bus with a dark band of windows drives along the avenue
        toward the left; at 0:15, right of centre, two long cyan vehicles side by side head
        for the crossing; at 0:30 a red bus stands at the stop line left of the crossing in
        the middle of the frame. Brake lamps are not seen at this height (§2.8.3); the cars
        are short with darker cabins.
      - **Midtown's city flight** (`vehicles-city.mp4` against `hud-city.mp4`). From 0:00
        to about 0:20, high up, the vehicles are flecks in their speed colours, as in
        `hud-city.mp4`; the two videos differ by a pixel or two a vehicle (§2.10). From 0:30
        (500 m) to 1:00 (300 m, looking into Central Park) the cars are short shapes, no
        flicker of lamps is seen, and nothing pops as the camera comes down.
      - **urban_grid's flight** (`vehicles-ug-camera.mp4` against `hud-ug-camera.mp4`).
        While it follows a car (0:55 to 2:11), the car is a car shape seen from behind, its
        cabin glass dark and its lamps unlit; at 1:36 its lamps light as it slows, it stands
        at a red light from 1:42, and at 2:11 its lamps go out as it moves off.
      - **The line-up** (`stills/lu-*.png` of §2.19, as gate 7 draws it): each class's
        shape, a braking row with lit lamps and a coasting row with unlit ones.
      - **In `view` on the mix:** at launch the vehicles are shapes; `V` shows boxes, `V`
        again shapes; `view --no-shapes` opens with boxes. Orbiting to a low pitch behind a
        queue, the lamps read; the readout and the keyframe line are unchanged.

      Where the numbers come from: the truck's times are its FCD rows (vehicle 184, rows
      at 315.1–345.1 s; standing to 332.1 s; link change at 338.1 s), the car's urban_grid
      vehicle 1's (braking from 105.1 s, standing 111.1–119.1 s, moving at 120.1 s), the
      orbit's from the probe's render (`runs/on-mix-orbit-1.mp4`). They are descriptions for
      the user to watch against, not counts, and a miss among them is recorded, not a fail.

      Then say whether OQ-1's, OQ-2's, OQ-4's and OQ-5's answers stand as they look, and
      OQ-3's on the truck close-up.
  - **The flags, the key and the fixture:**
  13. **Their edges.**
      - **`V`, headless** (`tests/vehicles.rs`, not ignored), written as `tests/links.rs`'s
        `C` test with `v` for `c`: `shapes_shown` is `true` at `ViewState::new` and
        at a launch without `--no-shapes`, `false` with it; four frames with `v` pressed give `false`, `true`, `false`, `true`;
        then playing and following with `K` pressed in the same frame, through a
        right-button orbit, a drag and a scrub of the slider, every other field of the state
        equals that of a twin stepped without `v`.
      - **The CLI** (`scripts/gates-vehicles.sh`). Each `view` runs under `perl -e 'alarm 60;
        exec @ARGV'`. On urban_grid, `render --from 10 --to 20 --speedup 1`:
        - without `--camera`, `--no-shapes`: exit 0, and every frame equal to the same
          render without it (300 of 300);
        - `--shapes`: exit 2, no video;
        - `--camera tests/flight.toml --fcd <class-int copy>` (`fcd-derive class-int`):
          exit 1, stderr's one line the schema error naming `vehicle_class`, no video; the
          same with `--no-shapes`: exit 0, `ffprobe` `1920,1080,30/1,300`;
        - `view --bench 1` and `view --no-shapes --bench 1`: exit 0 and the bench JSON on
          the last line of stderr; `view --fcd <class-int copy> --bench 1`: exit 1 and the
          schema error; with `--no-shapes`: exit 0.

      Without the feature: `--no-shapes` is itself rejected, exit 2.
  14. **The mix fixture** (`scripts/fixture.sh mix`, then `FORCE=1` twice more): exit 0 each
      time, the FCD's SHA-1 **`c8d1d569423befbe9cfce9f9a94a04d75562508a`** each time;
      **974 vehicles** (car 732, truck 100, bus 54, bike 47, van 41), **195,709 rows**;
      `scratch/midtown/` untouched (its FCD's SHA-1 `01becc86…` before and after). (The
      probe: 3 of 3.)
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | Orthographic: the reference lines; `hud-p2`'s three, with and without the flag | equal, 8700 / 1800 of 1800 | 1 |
  | `--no-shapes` in 3D: `hud-p2`'s four; the 23 lines | equal, 8700 / 1800 of 1800; as their baselines | 2 |
  | Other tests and scripts; diff | baseline numbers; equal pairs; the listed files | 3 |
  | Packages | 0 | 4 |
  | Synthetic: kinds, parts, the rule, looks, read and its errors | as listed, `==` | 5 |
  | Fixtures: every frame's looks against the FCD read straight; counts | 0 of 1,800 / 8,700 / 1,800 differ; as tabled | 6 |
  | The line-up's lamps | 20 of 20 brake, 19 of 20 head, exact | 7 |
  | Four flights on, twice; against `hud-p2` | equal; every frame differs from `hud-p2`'s | 8 |
  | A frame alone against the same frame in sequence; the job's looks | equal, 9 of 9; equal to `Fleet::looks` | 9 |
  | `view --bench 20` with shapes | ≥ 30 fps | 11 |
  | `V`; the flag's edges; the schema error | flips only its field; as listed | 13 |
  | The mix fixture, three builds | SHA-1 `c8d1d569…` each time | 14 |
- **Not predicted, and so not gated:**
  - the look: OQ-1, OQ-2, OQ-4 and OQ-5 as answered, the shapes in motion, and how the
    lamps read at each distance, for the user at gate 12;
  - render times beyond gate 10's estimate, and `view`'s frame rate above 30 (gate 11);
  - how many stacked vehicles of different kinds a run holds (§2.11).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-004-phase-1`), one push. The commits:
    - `src/vehicles.rs`, `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/main.rs`'s
      `render` flag, and `tests/vehicles.rs`'s headless gates (5, 6). On by default in
      `render --camera`, so the same commit carries the 23 script lines of §2.14;
    - `view`: `src/view/mod.rs`, `src/view/state.rs` and `src/main.rs`'s `view` flag, and
      gate 13's `V` test;
    - `scripts/fixture.sh mix` and `src/bin/fcd-derive.rs`'s `class-int`;
    - `tests/vehicles.rs`'s GPU gates (7, 9) and `scripts/gates-vehicles.sh`. Gates 1–11,
      13 and 14 run after this commit;
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **`rules/vehicles.md`** (new, `max_lines: 50`): the table and its read, the kinds and
      the names, the parts and their sizes, the stretch, the paints, the brake rule, the
      headlamps, the identical key, `--no-shapes`, `V` and the error. `sources`:
      `src/vehicles.rs`, `src/draw.rs`, `src/render.rs`, `src/lib.rs`, `src/view/mod.rs`,
      `src/view/state.rs`, `src/main.rs`;
    - `rules/render.md` (60 of 60): the CLI line gains `[--no-shapes]`; the scene's
      vehicles bullet points to `rules/vehicles.md` for the perspective path; the
      Pipeline's `prepare_with` bullet shapes off; cut to fit, the cap not raised;
    - `rules/camera.md` (60 of 60): "What each path draws" says the perspective path draws
      shapes (`rules/vehicles.md`) and the orthographic one boxes; cut to fit;
    - `rules/view.md` (60 of 60): the CLI gains `--no-shapes`, the drawing `V`, the frame
      order `V` after `C`; cut to fit;
    - `rules/inputs.md` (50 of 50): the FCD bullet says `acceleration` and `vehicle_class`
      are read by `vehicles::read` with shapes on (`rules/vehicles.md`); the checks' order
      gains the schema check last; `sources` gains `src/vehicles.rs`; cut to fit;
    - `rules/links.md` (40 of 40): "every box `CARS`" becomes every body; cut to fit;
    - `rules/signals.md`, `rules/streets.md`, `rules/hud.md`, `rules/credit.md`,
      `rules/slider.md`, `rules/buildings.md`, `rules/see-through.md`, `rules/parts.md`,
      `rules/motion.md`: none needed, since nothing they say changes (the shapes stand
      where the boxes stood, over the same layers);
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README:** the summary gains a sentence on shapes; the `render` and `view` usage
      blocks gain `[--no-shapes]`; a **Vehicles** bullet after **Link colours** says what
      each class looks like, the plain shape, the brake rule and the headlights, that it is
      3D only and on by default; `view`'s keys gain `V`; the fixture list gains
      `scripts/fixture.sh mix`; the gate list gains `scripts/gates-vehicles.sh` and `cargo
      test --release --test vehicles -- --include-ignored --test-threads=1`;
    - **vis-002 §2.13:** one dated note on its "Vehicle shapes" item, pointing here (§2.1);
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-004.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

# agent-jam

> Multi-agent collaboration is just harmony with deadlines.

---

## Why This Exists

You have probably stared at a pull request with three approving reviews and one blocking comment and thought, "this is just music theory with more syntax highlighting." You were right. The same algebra that decides whether a chord sounds consonant or dissonant also decides whether a team ships or stalls. The set {-1, 0, +1} describes both a three-note voicing and a three-way vote. The difference is that musicians have spent centuries learning how to listen to each other; software teams usually just rebase and hope.

This crate generalizes the architecture of `ternary-jam` into domain-independent collaboration patterns. It is not a metaphor. It is an isomorphism with tests.

## The Core Idea

A musical jam session has four structural elements that survive translation into any collaborative domain: voices (agents), chords (task phases), harmony rules (consensus strategies), and tempo (work cadence). When a saxophonist plays a flat ninth over a dominant chord, the tension is *informational* — it tells the band something about where the phrase wants to resolve. When a Critic agent rejects a Builder's approve vote during the Test phase, the exact same signal is being sent: the current state wants to move.

The crate models collaboration as a discrete-time dynamical system. Each tick, every agent emits a ternary decision. The decisions are mixed through a consensus strategy, producing a single collective output. The sequence of outputs forms a trajectory through a task-progression state space. The quality of that trajectory — how much tension, how much resolution, how much novelty — is measurable. You can literally plot your team's cognitive harmony over time.

## Architecture

The design is layered like a rhythm section:

### `Trit` — The Atom

```rust
pub enum Trit { Reject = -1, Abstain = 0, Approve = 1 }
```

Everything bottoms out here. Three values, no booleans. Boolean logic forces false dichotomies; ternary logic admits uncertainty. `Trit::sum` gives you emergent consensus from a slice: positive sums approve, negative sums reject, zero abstains. This is not voting — it is vector addition in one dimension.

### `Role` and `Collaborator` — The Voices

```rust
pub enum Role { Researcher, Builder, Critic, Integrator, Explorer }
pub struct Collaborator { id: u32, role: Role, tendency: i8, actions: Vec<Trit>, position: usize }
```

Each role carries a default *tendency*: Builders lean +1, Critics lean -1, the rest sit at 0. A `Collaborator` is a state machine with an action queue. When the queue empties, the agent falls back to its tendency. This models the difference between "I have a specific opinion about this line" and "I am generally optimistic about shipping."

### `Phase` and `TaskProgression` — The Chord Changes

```rust
pub enum Phase { Research, Design, Build, Test, Refine, Ship }
pub struct TaskProgression { phases: Vec<Phase>, current: usize }
```

Phases cycle. Each phase carries an expected *tension* level: Research is calm, Design and Test are tense, Refine is where conflict resolves. The `TaskProgression` advances automatically as the `WorkSync` ticks over turns. Your project has a time signature whether you name it or not; here it is named.

### `CollabRule` — The Improv Constraint

```rust
pub enum CollabRule { Parallel, Contrary, Free, Resolve }
```

Jazz musicians use improv rules: *play what the last player played but higher* (Parallel), *play the opposite* (Contrary), *do whatever* (Free), or *bring it home* (Resolve). Each rule takes the previous agent output and a tendency, then produces the next output. `check` validates whether a proposed action respects the rule. Constraints do not limit creativity; they create it.

### `WorkSync` — The Metronome

```rust
pub struct WorkSync { turns_per_sprint: u32, current_turn: u32, ticks_per_turn: u32, tick_counter: u32 }
```

Discrete time with two nested resolutions: ticks within turns, turns within sprints. `tick` returns `true` when a new turn begins, which triggers phase advancement. You can ask for `sprint_fraction` to see how far through the current cycle you are. Time is not a `Duration` here; it is a periodic structure because collaborative work is rhythmic, not linear.

### `ConsensusMix` — The Mixer Board

```rust
pub struct ConsensusMix { weights: Vec<i8> }
```

Four strategies for combining agent outputs:

- **`weighted_vote`** — vector-weighted sum, thresholded back into a `Trit`.
- **`majority`** — winner-take-all among non-abstainers.
- **`unanimous`** — all non-abstainers must agree.
- **`veto`** — one reject overrides everything.

Different decisions demand different mixers. A code-style change gets majority. A database migration gets veto. The crate does not pretend one size fits all.

### `CognitiveHarmony` — The Diagnostic

```rust
pub struct CognitiveHarmony { agreement_count, conflict_count, novel_outputs, total_outputs, conflicts_resolved }
```

Records every tick and derives metrics: `tension`, `resolution_rate`, `novelty`, `cohesion`, `productivity`, `harmony_score`. You can instrument a running session and watch conflict spike during Design, then resolve during Refine. If your `novelty` flatlines, your agents have stopped thinking. If your `tension` stays pegged at 1.0, you have a cultural problem, not a technical one.

### `WorkSession` — The Stage

```rust
pub struct WorkSession { collaborators, progression, sync, rules, mixer, harmony, output, ticks }
```

The orchestrator. Add collaborators with rules, then call `tick` or `run`. On each tick it: advances sync, advances phase if needed, collects outputs from every collaborator (queued actions first, then rule-based fallback), records harmony metrics, mixes to consensus, and appends the result. The `output` vector is the complete history of the session — every collective decision, in order, timestamped by tick.

## Usage Examples

### A Simple Duo

```rust
use agent_jam::*;

let progression = TaskProgression::standard();
let sync = WorkSync::new(4, 4); // 4-turn sprints, 4 ticks per turn
let mixer = ConsensusMix::uniform(2);
let mut session = WorkSession::new(progression, sync, mixer);

let mut builder = Collaborator::new(0, Role::Builder);
builder.add_action(Trit::Approve);
builder.add_action(Trit::Approve);

let mut critic = Collaborator::new(1, Role::Critic);
critic.add_action(Trit::Reject); // blocks once, then tendency takes over

session.add_collaborator(builder, CollabRule::Free);
session.add_collaborator(critic, CollabRule::Contrary);

let results = session.run(16);
println!("{:?}", results);
println!("tension: {:.2}", session.harmony.tension());
```

The Critic's explicit reject will register as conflict; afterward the Contrary rule will keep generating tension against the Builder's natural +1 tendency. You will see `harmony.conflict_count` rise, then watch how the mixer resolves it.

### Weighted Consensus with a Veto Override

```rust
let mixer = ConsensusMix::new(vec![3, 1, 1]); // first agent has 3× weight
let mut session = WorkSession::new(TaskProgression::standard(), WorkSync::new(2, 2), mixer);

session.add_collaborator(Collaborator::new(0, Role::Integrator), CollabRule::Parallel);
session.add_collaborator(Collaborator::new(1, Role::Builder), CollabRule::Free);
session.add_collaborator(Collaborator::new(2, Role::Critic), CollabRule::Contrary);

let single = session.tick();
let full = session.run(10);
```

The Integrator has outsized influence but is constrained to Parallel behavior — they must build on the previous state. The Critic is free to disagree. Watch how the weighted vote handles the Integrator's direction against the Critic's opposition.

### Instrumenting a Live Session

```rust
let mut session = /* ... */;

for _ in 0..100 {
    let decision = session.tick();
    if session.harmony.tension() > 0.7 {
        eprintln!("high tension at tick {}: {:?}", session.ticks, decision);
    }
    if session.harmony.resolution_rate() < 0.5 {
        eprintln!("stuck conflicts detected");
    }
}
```

You are not just simulating agents; you are measuring the health of a collaboration in real time. The metrics are designed to be plotted.

## API Reference

### `Trit`

| Method | Signature | Description |
|--------|-----------|-------------|
| `to_i8` | `fn(self) -> i8` | Cast to signed byte. |
| `from_i8` | `fn(i8) -> Option<Self>` | Safe construction; returns `None` for values outside {-1, 0, 1}. |
| `sum` | `fn(&[Trit]) -> Trit` | Element-wise signed sum, thresholded back to `Trit`. |

### `Role`

| Method | Signature | Description |
|--------|-----------|-------------|
| `default_tendency` | `fn(self) -> i8` | Builder/Explorer → 1, Critic → -1, Researcher/Integrator → 0. |

### `Collaborator`

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn(u32, Role) -> Self` | Create with role-default tendency. |
| `with_tendency` | `fn(self, i8) -> Self` | Builder-style override, clamped to [-1, 1]. |
| `add_action` | `fn(&mut self, Trit)` | Push an explicit decision onto the queue. |
| `next_action` | `fn(&mut self) -> Trit` | Pop from queue; returns `Abstain` when empty. |
| `remaining` | `fn(&self) -> usize` | Queue depth remaining. |
| `reset` | `fn(&mut self)` | Rewind position to 0. |

### `Phase`

| Method | Signature | Description |
|--------|-----------|-------------|
| `cycle` | `fn() -> Vec<Phase>` | Standard six-phase ordering. |
| `tension` | `fn(self) -> i8` | Expected conflict level for this phase. |

### `TaskProgression`

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn(Vec<Phase>) -> Self` | Custom phase ordering. |
| `standard` | `fn() -> Self` | Default cycle. |
| `advance` | `fn(&mut self) -> Phase` | Next phase, wrapping. |
| `current_phase` | `fn(&self) -> Option<Phase>` | Current phase if any. |
| `reset` | `fn(&mut self)` | Back to phase 0. |
| `len` / `is_empty` | — | Container accessors. |

### `CollabRule`

| Method | Signature | Description |
|--------|-----------|-------------|
| `apply` | `fn(self, prev: Trit, tendency: i8) -> Trit` | Generate next action given previous output and agent tendency. |
| `check` | `fn(self, prev: Trit, next: Trit) -> bool` | Validate whether a transition respects the rule. |

### `WorkSync`

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn(turns_per_sprint, ticks_per_turn) -> Self` | Create timer. |
| `tick` | `fn(&mut self) -> bool` | Advance one tick; returns `true` on turn boundary. |
| `turn` | `fn(&self) -> u32` | Current turn index. |
| `is_sprint_start` | `fn(&self) -> bool` | At origin of cycle. |
| `sprint_fraction` | `fn(&self) -> f64` | Progress through current sprint as [0, 1). |
| `sprint_ticks` | `fn(&self) -> u32` | Total ticks in one complete sprint. |
| `reset` | `fn(&mut self)` | Zero everything. |

### `ConsensusMix`

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn(Vec<i8>) -> Self` | Custom per-agent weights. |
| `uniform` | `fn(usize) -> Self` | Equal weighting for N agents. |
| `weighted_vote` | `fn(&self, &[Trit]) -> Trit` | Weighted sum consensus. |
| `unanimous` | `fn(&self, &[Trit]) -> bool` | True if all non-abstainers agree. |
| `majority` | `fn(&self, &[Trit]) -> Trit` | Plurality winner. |
| `veto` | `fn(&self, &[Trit]) -> Trit` | Reject if any reject; else approve if all approve; else abstain. |

### `CognitiveHarmony`

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn() -> Self` | Zeroed metrics. |
| `record` | `fn(&mut self, &[Trit])` | Ingest one tick's agent outputs. |
| `resolve_conflict` | `fn(&mut self)` | Increment resolved counter. |
| `tension` | `fn(&self) -> f64` | Conflict / (agreement + conflict). |
| `resolution_rate` | `fn(&self) -> f64` | Resolved / conflict. |
| `novelty` | `fn(&self) -> f64` | Ticks with at least one Approve / total ticks. |
| `cohesion` | `fn(&self) -> f64` | Agreement / (agreement + conflict). |
| `productivity` | `fn(&self) -> f64` | Composite of agreement and novelty, normalized. |
| `harmony_score` | `fn(&self) -> i64` | Signed net agreement. |

### `WorkSession`

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn(TaskProgression, WorkSync, ConsensusMix) -> Self` | Create empty session. |
| `add_collaborator` | `fn(&mut self, Collaborator, CollabRule)` | Register agent with its rule. |
| `tick` | `fn(&mut self) -> Trit` | Advance one tick, return consensus. |
| `run` | `fn(&mut self, u32) -> Vec<Trit>` | Run N ticks, return all results. |
| `collaborator_count` | `fn(&self) -> usize` | Number of registered agents. |

## The Deeper Idea

There is a reason music cognition research and multi-agent systems keep rediscovering each other. Both are *constraint-satisfaction processes over time with multiple voices*. A chord is a consensus state; voice-leading is agent trajectory; dissonance is productive conflict; resolution is decision convergence. The ii-V-I progression is literally a deliberation protocol: establish context, introduce tension, resolve to stability.

The SuperInstance ecosystem formalizes this through the *conservation spectral framework*. In music, spectral conservation means harmonic energy is neither created nor destroyed, only transformed. In agent systems, the analogous principle is that *informational tension is conserved across the collaboration graph*. When one agent rejects, that energy does not vanish — it moves into the system, where it can either be resolved (good harmony), accumulated into avoidance cascades (bad harmony), or dissipated through abstention (no harmony at all).

`agent-jam` gives you the primitives to observe this conservation law in action. The `CognitiveHarmony` metrics are spectral measurements. A session with high `tension` and low `resolution_rate` is a system with trapped energy — it will either find a release valve or fracture. A session with zero `novelty` is a system in thermal equilibrium: technically stable, creatively dead.

Understanding your team through this lens is not poetic fluff. It is control theory. You are tuning a feedback loop. The agents are your plant, the rules are your controller, and the metrics are your sensor array. The fact that it sounds like music theory is a feature, not a bug — musicians have been tuning complex multi-agent systems since before software existed.

## Related Crates

| Crate | Relationship |
|-------|-------------|
| [`ternary-jam`](https://github.com/SuperInstance/ternary-jam) | The direct predecessor. Musical jam sessions as literal MIDI-driven coordination. `agent-jam` abstracts the same algebra into domain-independent types. |
| [`constraint-theory-core`](https://github.com/SuperInstance/constraint-theory-core) | Unified geometric constraint theory — Eisenstein lattices, deadband funnels, Laman rigidity. The mathematical foundation that proves why ternary consensus converges. |
| [`conservation-spectral-core`](https://github.com/SuperInstance/conservation-spectral-core) | Spectral analysis of tension graphs. Feed `CognitiveHarmony` output into this to detect structural anomalies in your collaboration patterns. |
| [`agent-ternary-gate`](https://github.com/SuperInstance/agent-ternary-gate) | Three-condition gating for agent firing: no surprise, no update. Use this to throttle `WorkSession` ticks when novelty drops below threshold. |
| [`avoidance-cascade`](https://github.com/SuperInstance/avoidance-cascade) | Detects and prevents the failure mode where ternary agents all retreat to Abstain. If your `harmony_score` flatlines at zero, reach for this. |

---

*Licensed under MIT. Part of the [SuperInstance](https://superinstance.ai) ecosystem.*

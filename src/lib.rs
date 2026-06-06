//! # agent-jam
//!
//! Musical jam sessions are not just for music. They are a general pattern for
//! multi-agent creative collaboration: voices become collaborators, chords become
//! tasks, harmony becomes productivity, dissonance becomes productive conflict,
//! and rhythm becomes work cadence.
//!
//! This crate generalizes the architecture of `ternary-jam` into domain-independent
//! multi-agent collaboration patterns. The same algebra that describes musical
//! harmony {-1, 0, +1} describes agent decisions {reject, abstain, approve}.

#![forbid(unsafe_code)]

/// Ternary decision value: Reject (-1), Abstain (0), Approve (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trit {
    Reject = -1,
    Abstain = 0,
    Approve = 1,
}

impl Trit {
    pub fn to_i8(self) -> i8 { self as i8 }
    pub fn from_i8(v: i8) -> Option<Self> {
        match v { -1 => Some(Trit::Reject), 0 => Some(Trit::Abstain), 1 => Some(Trit::Approve), _ => None }
    }
    pub fn sum(values: &[Trit]) -> Trit {
        let s: i32 = values.iter().map(|t| t.to_i8() as i32).sum();
        if s > 0 { Trit::Approve } else if s < 0 { Trit::Reject } else { Trit::Abstain }
    }
}

/// Agent role in the collaboration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Researcher,
    Builder,
    Critic,
    Integrator,
    Explorer,
}

impl Role {
    /// Default tendency for each role.
    pub fn default_tendency(self) -> i8 {
        match self {
            Role::Researcher => 0,   // balanced, neutral
            Role::Builder => 1,      // leans toward approve/ship
            Role::Critic => -1,      // leans toward reject/challenge
            Role::Integrator => 0,   // balanced, connecting
            Role::Explorer => 1,     // leans toward approve/novel
        }
    }
}

/// A collaborating agent — the "voice" in the jam.
#[derive(Debug, Clone)]
pub struct Collaborator {
    pub id: u32,
    pub role: Role,
    pub tendency: i8,
    pub actions: Vec<Trit>,
    pub position: usize,
}

impl Collaborator {
    pub fn new(id: u32, role: Role) -> Self {
        Self { id, role, tendency: role.default_tendency(), actions: Vec::new(), position: 0 }
    }
    pub fn with_tendency(mut self, t: i8) -> Self { self.tendency = t.clamp(-1, 1); self }
    pub fn add_action(&mut self, a: Trit) { self.actions.push(a); }
    pub fn next_action(&mut self) -> Trit {
        if self.position < self.actions.len() { let a = self.actions[self.position]; self.position += 1; a }
        else { Trit::Abstain }
    }
    pub fn remaining(&self) -> usize { self.actions.len().saturating_sub(self.position) }
    pub fn reset(&mut self) { self.position = 0; }
}

/// Work phases the team cycles through — the "chord progression".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Research,
    Design,
    Build,
    Test,
    Refine,
    Ship,
}

impl Phase {
    /// All phases in order.
    pub fn cycle() -> Vec<Phase> {
        vec![Phase::Research, Phase::Design, Phase::Build, Phase::Test, Phase::Refine, Phase::Ship]
    }
    /// Expected tension level for this phase (higher = more conflict expected).
    pub fn tension(self) -> i8 {
        match self {
            Phase::Research => 0,
            Phase::Design => 1,
            Phase::Build => 0,
            Phase::Test => 1,
            Phase::Refine => -1,
            Phase::Ship => 1,
        }
    }
}

/// A sequence of task phases.
#[derive(Debug, Clone)]
pub struct TaskProgression {
    pub phases: Vec<Phase>,
    pub current: usize,
}

impl TaskProgression {
    pub fn new(phases: Vec<Phase>) -> Self { Self { phases, current: 0 } }
    pub fn standard() -> Self { Self::new(Phase::cycle()) }
    pub fn advance(&mut self) -> Phase {
        let p = self.phases[self.current];
        self.current = (self.current + 1) % self.phases.len();
        p
    }
    pub fn current_phase(&self) -> Option<Phase> { self.phases.get(self.current).copied() }
    pub fn reset(&mut self) { self.current = 0; }
    pub fn len(&self) -> usize { self.phases.len() }
    pub fn is_empty(&self) -> bool { self.phases.is_empty() }
}

/// Constraint on how agents can contribute — the "improv rule".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollabRule {
    Parallel,  // Build on previous agent's output
    Contrary,  // Challenge/disagree with previous
    Free,      // Unconstrained
    Resolve,   // Converge toward consensus
}

impl CollabRule {
    pub fn apply(self, prev: Trit, tendency: i8) -> Trit {
        let p = prev.to_i8();
        match self {
            CollabRule::Parallel => Trit::from_i8((p + tendency.signum()).clamp(-1, 1)).unwrap_or(Trit::Abstain),
            CollabRule::Contrary => Trit::from_i8((p - tendency.signum()).clamp(-1, 1)).unwrap_or(Trit::Abstain),
            CollabRule::Free => Trit::from_i8(tendency).unwrap_or(Trit::Abstain),
            CollabRule::Resolve => {
                if p > 0 { Trit::from_i8(p - 1).unwrap_or(Trit::Abstain) }
                else if p < 0 { Trit::from_i8(p + 1).unwrap_or(Trit::Abstain) }
                else { Trit::Abstain }
            }
        }
    }
    pub fn check(self, prev: Trit, next: Trit) -> bool {
        let p = prev.to_i8(); let n = next.to_i8();
        match self {
            CollabRule::Parallel => (n - p).signum() != -p.signum() || p == 0 || n == p,
            CollabRule::Contrary => (n - p).signum() == -p.signum() || p == 0,
            CollabRule::Free => true,
            CollabRule::Resolve => n.abs() <= p.abs(),
        }
    }
}

/// Timing for the work session — "jam sync".
#[derive(Debug, Clone)]
pub struct WorkSync {
    pub turns_per_sprint: u32,
    pub current_turn: u32,
    pub ticks_per_turn: u32,
    pub tick_counter: u32,
}

impl WorkSync {
    pub fn new(turns_per_sprint: u32, ticks_per_turn: u32) -> Self {
        Self { turns_per_sprint, current_turn: 0, ticks_per_turn, tick_counter: 0 }
    }
    pub fn tick(&mut self) -> bool {
        self.tick_counter += 1;
        if self.tick_counter >= self.ticks_per_turn {
            self.tick_counter = 0;
            self.current_turn = (self.current_turn + 1) % self.turns_per_sprint;
            true
        } else { false }
    }
    pub fn turn(&self) -> u32 { self.current_turn }
    pub fn is_sprint_start(&self) -> bool { self.current_turn == 0 && self.tick_counter == 0 }
    pub fn sprint_fraction(&self) -> f64 { self.current_turn as f64 / self.turns_per_sprint.max(1) as f64 }
    pub fn sprint_ticks(&self) -> u32 { self.turns_per_sprint * self.ticks_per_turn }
    pub fn reset(&mut self) { self.current_turn = 0; self.tick_counter = 0; }
}

/// Consensus mixing strategy — how to combine agent outputs.
#[derive(Debug, Clone)]
pub struct ConsensusMix {
    pub weights: Vec<i8>,
}

impl ConsensusMix {
    pub fn new(weights: Vec<i8>) -> Self { Self { weights } }
    pub fn uniform(n: usize) -> Self { Self { weights: vec![1; n] } }

    /// Weighted vote.
    pub fn weighted_vote(&self, values: &[Trit]) -> Trit {
        let sum: i32 = values.iter().enumerate()
            .map(|(i, v)| v.to_i8() as i32 * self.weights.get(i).copied().unwrap_or(1) as i32).sum();
        if sum > 0 { Trit::Approve } else if sum < 0 { Trit::Reject } else { Trit::Abstain }
    }
    /// Unanimous agreement required.
    pub fn unanimous(&self, values: &[Trit]) -> bool {
        let non_abstain: Vec<_> = values.iter().filter(|&&v| v != Trit::Abstain).collect();
        if non_abstain.is_empty() { return true; }
        non_abstain.windows(2).all(|w| w[0] == w[1])
    }
    /// Majority rules among non-abstain votes.
    pub fn majority(&self, values: &[Trit]) -> Trit {
        let mut approve = 0i32; let mut reject = 0i32;
        for v in values { match v { Trit::Approve => approve += 1, Trit::Reject => reject += 1, _ => {} } }
        if approve > reject { Trit::Approve } else if reject > approve { Trit::Reject } else { Trit::Abstain }
    }
    /// Any reject vetoes the whole thing.
    pub fn veto(&self, values: &[Trit]) -> Trit {
        if values.iter().any(|&v| v == Trit::Reject) { Trit::Reject }
        else if values.iter().all(|&v| v == Trit::Approve) { Trit::Approve }
        else { Trit::Abstain }
    }
}

/// Cognitive harmony metrics — measuring collaboration quality.
#[derive(Debug, Clone, Default)]
pub struct CognitiveHarmony {
    pub agreement_count: u64,
    pub conflict_count: u64,
    pub novel_outputs: u64,
    pub total_outputs: u64,
    pub conflicts_resolved: u64,
}

impl CognitiveHarmony {
    pub fn new() -> Self { Self::default() }

    pub fn record(&mut self, outputs: &[Trit]) {
        self.total_outputs += 1;
        // Pairwise check
        for i in 0..outputs.len() {
            for j in (i+1)..outputs.len() {
                let a = outputs[i].to_i8(); let b = outputs[j].to_i8();
                if a != 0 && b != 0 && a != b { self.conflict_count += 1; }
                else if a != 0 && b != 0 && a == b { self.agreement_count += 1; }
            }
        }
        // Novelty: any Approve in a context of previous Abstain/Reject
        if outputs.iter().any(|&v| v == Trit::Approve) { self.novel_outputs += 1; }
    }

    pub fn resolve_conflict(&mut self) { self.conflicts_resolved += 1; }

    /// How much disagreement exists.
    pub fn tension(&self) -> f64 {
        let total = self.agreement_count + self.conflict_count;
        if total == 0 { 0.0 } else { self.conflict_count as f64 / total as f64 }
    }
    /// % of conflicts resolved.
    pub fn resolution_rate(&self) -> f64 {
        if self.conflict_count == 0 { 1.0 } else { self.conflicts_resolved as f64 / self.conflict_count as f64 }
    }
    /// How much new ground covered.
    pub fn novelty(&self) -> f64 {
        if self.total_outputs == 0 { 0.0 } else { self.novel_outputs as f64 / self.total_outputs as f64 }
    }
    /// How aligned agents are.
    pub fn cohesion(&self) -> f64 {
        let total = self.agreement_count + self.conflict_count;
        if total == 0 { 1.0 } else { self.agreement_count as f64 / total as f64 }
    }
    /// Total useful output per time.
    pub fn productivity(&self) -> f64 {
        if self.total_outputs == 0 { 0.0 } else {
            (self.agreement_count as f64 + self.novel_outputs as f64) / (self.total_outputs as f64 * 2.0)
        }
    }
    /// Overall harmony: agreement minus conflict.
    pub fn harmony_score(&self) -> i64 { self.agreement_count as i64 - self.conflict_count as i64 }
}

/// The work session — the arena where agents collaborate.
#[derive(Debug, Clone)]
pub struct WorkSession {
    pub collaborators: Vec<Collaborator>,
    pub progression: TaskProgression,
    pub sync: WorkSync,
    pub rules: Vec<CollabRule>,
    pub mixer: ConsensusMix,
    pub harmony: CognitiveHarmony,
    pub output: Vec<Trit>,
    pub ticks: u64,
}

impl WorkSession {
    pub fn new(progression: TaskProgression, sync: WorkSync, mixer: ConsensusMix) -> Self {
        Self { collaborators: Vec::new(), progression, sync, rules: Vec::new(), mixer, harmony: CognitiveHarmony::new(), output: Vec::new(), ticks: 0 }
    }
    pub fn add_collaborator(&mut self, c: Collaborator, rule: CollabRule) {
        self.collaborators.push(c); self.rules.push(rule);
    }
    pub fn tick(&mut self) -> Trit {
        let new_turn = self.sync.tick();
        if new_turn { self.progression.advance(); }
        let outputs: Vec<Trit> = self.collaborators.iter_mut().enumerate().map(|(i, c)| {
            if c.remaining() == 0 {
                let rule = self.rules.get(i).copied().unwrap_or(CollabRule::Free);
                let prev = c.actions.last().copied().unwrap_or(Trit::Abstain);
                rule.apply(prev, c.tendency)
            } else { c.next_action() }
        }).collect();
        self.harmony.record(&outputs);
        let result = self.mixer.weighted_vote(&outputs);
        self.output.push(result);
        self.ticks += 1;
        result
    }
    pub fn run(&mut self, ticks: u32) -> Vec<Trit> {
        (0..ticks).map(|_| self.tick()).collect()
    }
    pub fn collaborator_count(&self) -> usize { self.collaborators.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn trit_roundtrip() { for v in [-1i8, 0, 1] { assert_eq!(Trit::from_i8(v).unwrap().to_i8(), v); } assert!(Trit::from_i8(2).is_none()); }
    #[test] fn trit_sum() { assert_eq!(Trit::sum(&[Trit::Approve, Trit::Approve, Trit::Reject]), Trit::Approve); assert_eq!(Trit::sum(&[Trit::Reject, Trit::Reject]), Trit::Reject); assert_eq!(Trit::sum(&[Trit::Approve, Trit::Reject]), Trit::Abstain); }

    #[test] fn collaborator_queue() {
        let mut c = Collaborator::new(0, Role::Builder);
        c.add_action(Trit::Approve); c.add_action(Trit::Reject); c.add_action(Trit::Abstain);
        assert_eq!(c.next_action(), Trit::Approve); assert_eq!(c.next_action(), Trit::Reject);
        assert_eq!(c.next_action(), Trit::Abstain); assert_eq!(c.next_action(), Trit::Abstain);
    }
    #[test] fn collaborator_reset() {
        let mut c = Collaborator::new(1, Role::Critic);
        c.add_action(Trit::Reject); c.next_action(); assert_eq!(c.remaining(), 0);
        c.reset(); assert_eq!(c.remaining(), 1);
    }
    #[test] fn role_tendencies() {
        assert_eq!(Role::Builder.default_tendency(), 1);
        assert_eq!(Role::Critic.default_tendency(), -1);
        assert_eq!(Role::Researcher.default_tendency(), 0);
    }
    #[test] fn task_progression_cycles() {
        let mut tp = TaskProgression::standard();
        assert_eq!(tp.advance(), Phase::Research);
        assert_eq!(tp.advance(), Phase::Design);
        assert_eq!(tp.advance(), Phase::Build);
        for _ in 0..3 { tp.advance(); }
        assert_eq!(tp.advance(), Phase::Research); // wraps
    }
    #[test] fn task_progression_tension() {
        assert_eq!(Phase::Research.tension(), 0);
        assert_eq!(Phase::Test.tension(), 1);
        assert_eq!(Phase::Refine.tension(), -1);
    }
    #[test] fn collab_rule_free() { assert!(CollabRule::Free.check(Trit::Reject, Trit::Approve)); }
    #[test] fn collab_rule_resolve() {
        assert_eq!(CollabRule::Resolve.apply(Trit::Approve, 0), Trit::Abstain);
        assert!(CollabRule::Resolve.check(Trit::Approve, Trit::Abstain));
    }
    #[test] fn collab_rule_parallel() { assert_eq!(CollabRule::Parallel.apply(Trit::Approve, 1), Trit::Approve); }

    #[test] fn work_sync_timing() {
        let mut s = WorkSync::new(4, 4);
        assert!(s.is_sprint_start());
        assert!(!s.tick()); assert!(!s.tick()); assert!(!s.tick()); assert!(s.tick());
        assert_eq!(s.turn(), 1);
    }

    #[test] fn consensus_majority() {
        let m = ConsensusMix::uniform(3);
        assert_eq!(m.majority(&[Trit::Approve, Trit::Approve, Trit::Reject]), Trit::Approve);
        assert_eq!(m.majority(&[Trit::Abstain, Trit::Abstain]), Trit::Abstain);
    }
    #[test] fn consensus_unanimous() {
        let m = ConsensusMix::uniform(3);
        assert!(m.unanimous(&[Trit::Approve, Trit::Approve]));
        assert!(!m.unanimous(&[Trit::Approve, Trit::Reject]));
    }
    #[test] fn consensus_veto() {
        let m = ConsensusMix::uniform(3);
        assert_eq!(m.veto(&[Trit::Approve, Trit::Reject, Trit::Approve]), Trit::Reject);
        assert_eq!(m.veto(&[Trit::Approve, Trit::Approve]), Trit::Approve);
    }
    #[test] fn consensus_weighted() {
        let m = ConsensusMix::new(vec![2, 1]);
        assert_eq!(m.weighted_vote(&[Trit::Approve, Trit::Reject]), Trit::Approve); // 2*1 + 1*(-1) = 1
    }

    #[test] fn harmony_metrics() {
        let mut h = CognitiveHarmony::new();
        h.record(&[Trit::Approve, Trit::Approve]); // agreement
        h.record(&[Trit::Approve, Trit::Reject]);  // conflict
        assert!(h.cohesion() > 0.0);
        assert!(h.tension() > 0.0);
        assert!(h.novelty() > 0.0);
    }
    #[test] fn harmony_resolution() {
        let mut h = CognitiveHarmony::new();
        h.record(&[Trit::Approve, Trit::Reject]);
        h.resolve_conflict();
        assert_eq!(h.resolution_rate(), 1.0);
    }

    #[test] fn work_session_basic() {
        let p = TaskProgression::standard(); let s = WorkSync::new(4, 4);
        let m = ConsensusMix::uniform(2);
        let mut ws = WorkSession::new(p, s, m);
        let mut c1 = Collaborator::new(0, Role::Builder); c1.add_action(Trit::Approve); c1.add_action(Trit::Approve);
        let mut c2 = Collaborator::new(1, Role::Builder); c2.add_action(Trit::Approve); c2.add_action(Trit::Approve);
        ws.add_collaborator(c1, CollabRule::Free); ws.add_collaborator(c2, CollabRule::Free);
        let results = ws.run(8);
        assert_eq!(results.len(), 8);
        assert!(ws.harmony.agreement_count > 0);
    }
    #[test] fn work_session_conflict() {
        let p = TaskProgression::standard(); let s = WorkSync::new(4, 4);
        let m = ConsensusMix::uniform(2);
        let mut ws = WorkSession::new(p, s, m);
        let c1 = Collaborator::new(0, Role::Builder);
        let c2 = Collaborator::new(1, Role::Critic);
        ws.add_collaborator(c1, CollabRule::Free); ws.add_collaborator(c2, CollabRule::Contrary);
        ws.run(4);
        assert!(ws.harmony.conflict_count > 0 || ws.harmony.agreement_count > 0);
    }
}

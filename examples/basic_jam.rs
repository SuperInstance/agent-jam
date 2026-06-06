//! Basic Jam — Two agents take turns improvising over a chord progression.
//!
//! Shows how agents with different roles (Builder, Critic) produce different
//! decisions as the work session advances through phases.

use agent_jam::*;

fn main() {
    println!("🎸 ══════════════════════════════════════════════════════════");
    println!("🎸  BASIC JAM — Two Agents, One Chord Progression");
    println!("🎸 ══════════════════════════════════════════════════════════\n");

    let progression = TaskProgression::new(vec![
        Phase::Research,
        Phase::Design,
        Phase::Build,
        Phase::Test,
        Phase::Refine,
        Phase::Ship,
    ]);
    let sync = WorkSync::new(6, 2); // 6 turns per sprint, 2 ticks per turn
    let mixer = ConsensusMix::uniform(2);

    let mut session = WorkSession::new(progression, sync, mixer);

    // Agent 0: Builder — leans Approve
    let mut builder = Collaborator::new(0, Role::Builder);
    builder.add_action(Trit::Approve);
    builder.add_action(Trit::Approve);
    builder.add_action(Trit::Abstain);
    builder.add_action(Trit::Approve);
    builder.add_action(Trit::Reject);
    builder.add_action(Trit::Approve);

    // Agent 1: Critic — leans Reject, plays Contrary
    let mut critic = Collaborator::new(1, Role::Critic);
    critic.add_action(Trit::Reject);
    critic.add_action(Trit::Abstain);
    critic.add_action(Trit::Approve);
    critic.add_action(Trit::Reject);
    critic.add_action(Trit::Approve);
    critic.add_action(Trit::Approve);

    session.add_collaborator(builder, CollabRule::Parallel);
    session.add_collaborator(critic, CollabRule::Contrary);

    let phases = Phase::cycle();
    println!("Chord Progression: {:?}\n", phases.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>());

    // Run one tick at a time to show the jam evolving
    for tick in 0..12 {
        let phase_name = format!("{:?}", phases.get(tick / 2).unwrap_or(&phases[0]));
        let result = session.tick();

        let builder_state = &session.collaborators[0];
        let critic_state = &session.collaborators[1];

        println!("── Tick {:>2} | Phase: {:<10} ──", tick + 1, phase_name);
        println!("  🏗️  Builder → {:?}  (tendency: {:+})", builder_state.actions.last().unwrap_or(&Trit::Abstain), builder_state.tendency);
        println!("  🔍 Critic  → {:?}  (tendency: {:+})", critic_state.actions.last().unwrap_or(&Trit::Abstain), critic_state.tendency);
        println!("  🎯 Consensus: {:?}  | Harmony: {} | Tension: {:.2}",
            result,
            session.harmony.harmony_score(),
            session.harmony.tension()
        );
        println!();
    }

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║  SESSION SUMMARY                                  ║");
    println!("╠═══════════════════════════════════════════════════╣");
    println!("║  Total ticks:      {:>30}  ║", session.ticks);
    println!("║  Harmony score:    {:>30}  ║", session.harmony.harmony_score());
    println!("║  Agreements:       {:>30}  ║", session.harmony.agreement_count);
    println!("║  Conflicts:        {:>30}  ║", session.harmony.conflict_count);
    println!("║  Cohesion:         {:>29.1}%  ║", session.harmony.cohesion() * 100.0);
    println!("║  Novelty:          {:>29.1}%  ║", session.harmony.novelty() * 100.0);
    println!("║  Tension:          {:>29.1}%  ║", session.harmony.tension() * 100.0);
    println!("║  Productivity:     {:>29.1}%  ║", session.harmony.productivity() * 100.0);
    println!("╚═══════════════════════════════════════════════════╝");
}

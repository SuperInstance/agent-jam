//! Jam Session — 4 agents in a full jam session with multiple rounds.
//!
//! Shows how diverse roles create emergent harmony and how the session
//! converges or diverges through collaborative decision-making.

use agent_jam::*;

fn main() {
    println!("🎹 ══════════════════════════════════════════════════════════");
    println!("🎹  JAM SESSION — 4 Agents, Multiple Sprints");
    println!("🎹 ══════════════════════════════════════════════════════════\n");

    let progression = TaskProgression::standard();
    let sync = WorkSync::new(4, 3); // 4 turns/sprint, 3 ticks/turn
    let mixer = ConsensusMix::new(vec![2, 1, 2, 1]); // Builder & Integrator weighted higher

    let mut session = WorkSession::new(progression, sync, mixer);

    // Four agents with distinct roles
    let roles = vec![
        (0, Role::Researcher, CollabRule::Free, "🧪 Researcher"),
        (1, Role::Builder, CollabRule::Parallel, "🏗️  Builder"),
        (2, Role::Critic, CollabRule::Contrary, "🔍 Critic"),
        (3, Role::Integrator, CollabRule::Resolve, "🔗 Integrator"),
    ];

    for (id, role, rule, _label) in &roles {
        let c = Collaborator::new(*id, *role);
        session.add_collaborator(c, *rule);
    }

    // Run 3 full sprints
    for sprint in 1..=3 {
        println!("━━━ SPRINT {} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", sprint);

        let sprint_ticks = session.sync.sprint_ticks();
        for t in 0..sprint_ticks {
            let result = session.tick();

            if t == 0 || t == sprint_ticks / 2 || t == sprint_ticks - 1 {
                let phase = session.progression.current_phase();
                println!("  Turn {} | Phase: {:?} | Consensus: {:?} | Cohesion: {:.0}%",
                    session.sync.turn(),
                    phase.unwrap_or(Phase::Research),
                    result,
                    session.harmony.cohesion() * 100.0
                );
            }
        }

        // End-of-sprint snapshot
        println!("\n  📊 Sprint {} Summary:", sprint);
        println!("     Harmony: {} | Agreements: {} | Conflicts: {}",
            session.harmony.harmony_score(),
            session.harmony.agreement_count,
            session.harmony.conflict_count
        );
        println!("     Tension: {:.1}% | Novelty: {:.1}% | Productivity: {:.1}%\n",
            session.harmony.tension() * 100.0,
            session.harmony.novelty() * 100.0,
            session.harmony.productivity() * 100.0
        );
    }

    // Final analysis
    let total_outputs: usize = session.output.len();
    let approves = session.output.iter().filter(|&&t| t == Trit::Approve).count();
    let rejects = session.output.iter().filter(|&&t| t == Trit::Reject).count();
    let abstains = session.output.iter().filter(|&&t| t == Trit::Abstain).count();

    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  FINAL SESSION REPORT                                 ║");
    println!("╠═══════════════════════════════════════════════════════╣");
    println!("║  Total outputs:    {:>34}  ║", total_outputs);
    println!("║  Approves:         {:>34}  ║", approves);
    println!("║  Rejects:          {:>34}  ║", rejects);
    println!("║  Abstains:         {:>34}  ║", abstains);
    println!("║  Overall harmony:  {:>34}  ║", session.harmony.harmony_score());
    println!("║  Resolution rate:  {:>33.1}%  ║", session.harmony.resolution_rate() * 100.0);
    println!("╚═══════════════════════════════════════════════════════╝");

    println!("\n💡 Key insight: The Critic's Contrary rule and the Integrator's");
    println!("   Resolve rule create productive tension that the weighted vote");
    println!("   converts into actionable consensus.");
}

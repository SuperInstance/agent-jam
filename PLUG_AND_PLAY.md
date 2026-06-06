# PLUG_AND_PLAY — agent-jam

> Musical jam sessions generalized to multi-agent collaboration

## 🚀 Quick Start

```toml
[dependencies]
agent-jam = { git = "https://github.com/SuperInstance/agent-jam" }
```

```rust
use agent_jam::{WorkSession, Collaborator, Role, TaskProgression, WorkSync, ConsensusMix, CollabRule};

let mut session = WorkSession::new(
    TaskProgression::standard(),     // Research→Design→Build→Test→Refine→Ship
    WorkSync::new(4, 4),             // 4 turns/sprint, 4 ticks/turn
    ConsensusMix::uniform(2),        // Equal-weight voting
);
session.add_collaborator(Collaborator::new(0, Role::Builder), CollabRule::Free);
session.add_collaborator(Collaborator::new(1, Role::Critic), CollabRule::Contrary);

let results = session.run(8);
println!("Harmony: {}", session.harmony.harmony_score());
```

## 🎸 Music → Cognition Map

| Music | Agent | What It Does |
|-------|-------|-------------|
| Voice | Collaborator | Independent agent with a role and tendency |
| Chord progression | TaskProgression | Phases the team cycles through |
| Improv rule | CollabRule | Constraints that channel creativity |
| Jam session | WorkSession | The arena where agents produce together |
| Harmony | CognitiveHarmony | Productive output / conflict metric |

## 📚 Ecosystem

- [agent-groove](https://github.com/SuperInstance/agent-groove) — Timing and pocket states
- [agent-voice-leading](https://github.com/SuperInstance/agent-voice-leading) — Smooth state transitions
- [agent-riff](https://github.com/SuperInstance/agent-riff) — Competitive riffing mode
- [ternary-jam](https://github.com/SuperInstance/ternary-jam) — The musical original
- [flux-algebra](https://github.com/SuperInstance/flux-algebra) — Harmonic algebra (Python)

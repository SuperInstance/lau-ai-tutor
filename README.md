# lau-ai-tutor

> **Intelligence layer for PLATO** — an AI tutor that adapts to every kid.

Part of the [PLATO/LAU](https://github.com/SuperInstance) ecosystem: a framework for building educational agents that learn, teach, and evolve.

---

## What This Does

`lau-ai-tutor` is the *teaching brain* of the PLATO system. It provides:

- **Tutor personas** — configurable personalities (playful, socratic, direct, gentle) with adjustable patience, enthusiasm, and technical level
- **Learner memory** — tracks what each kid has learned, what they're struggling with, and how they learn best
- **Event-driven responses** — reacts to game-world events (conservation errors, discoveries, questions, achievements) with contextual, age-appropriate responses
- **Prompt templates** — pre-built response templates that the platform layer sends to an LLM or displays directly

**Important:** This crate does *not* call an LLM. It prepares structured prompts and responses that the platform layer delivers to the model. This keeps the tutoring logic portable and fully testable.

---

## The Key Idea

> **The tutor observes, remembers, and responds — never lectures.**

Traditional educational software tells kids what to learn. `lau-ai-tutor` watches what happens in the game world and chooses *when* and *how* to intervene:

1. A conservation law is violated → tutor explains what went wrong
2. A kid discovers something new → tutor celebrates and names the concept
3. A kid is idle for too long → tutor offers a hint
4. A kid asks a question → tutor responds with a guiding question (Socratic style)
5. A kid earns an achievement → tutor celebrates genuinely

The tutor *remembers* every interaction. Over time, it builds a profile of what the kid knows, what they struggle with, and what topics they enjoy — then uses that to suggest what to teach next.

Intervention is gated by persona attributes: a low-enthusiasm tutor won't interrupt discoveries; a low-patience tutor won't linger on crop failures.

---

## Install

```bash
cargo add lau-ai-tutor
```

Or add to `Cargo.toml`:

```toml
[dependencies]
lau-ai-tutor = "0.1"
```

Requires **Rust 2021 edition**. Dependencies: `serde`.

---

## Quick Start

### Create a Tutor

```rust
use lau_ai_tutor::*;

// Use a pre-built persona
let persona = personas::sparky(); // playful, high enthusiasm

// Or create a custom one
let persona = TutorPersona::new("Pixel", TutorStyle::Playful);

// Build the engine
let mut engine = TutorEngine::new(persona, "kid42");
```

### Process Game Events

```rust
// A conservation error occurred
let response = engine.process_event(
    TutorEvent::ConservationError(5.0),
    tick
);
if let Some(resp) = response {
    println!("Tutor says: {}", resp.text);
    // → "Hmm, the numbers went red! That means something disappeared..."
    println!("Tone: {:?}", resp.tone);
    // → Thoughtful
}

// Player asked a question
let response = engine.process_event(
    TutorEvent::QuestionAsked("Why did my bridge fall?".into()),
    tick
);

// Player earned an achievement
let response = engine.process_event(
    TutorEvent::AchievementEarned("Master Builder".into()),
    tick
);
if let Some(resp) = response {
    if let Some(action) = &resp.action {
        // Platform layer: execute TutorAction::Celebrate("Master Builder")
    }
}
```

### Check Learner Memory

```rust
let memory = engine.get_memory();

// Has the player learned gravity?
if memory.has_learned("gravity") {
    println!("They get it!");
}

// Are they struggling with thermodynamics?
if memory.is_struggling_with("thermodynamics") {
    println!("Maybe review this concept.");
}

// What should we teach next?
let available = vec![
    "gravity".into(),
    "thermodynamics".into(),
    "optics".into(),
];
if let Some(next) = memory.suggest_next_concept(&available) {
    println!("Next concept: {}", next); // prioritizes struggling + unlearned
}
```

### Use Prompt Templates Directly

```rust
// Conservation violation explanation
let msg = PromptTemplate::conservation_violation(3.14, "in the water cycle");

// Struggling with a concept (adjusts tone by attempt count)
let msg = PromptTemplate::struggling("fractions", 3); // "step back" tone
let msg = PromptTemplate::struggling("fractions", 7); // "completely reframe" tone

// Celebrate an achievement
let msg = PromptTemplate::celebration("First Steps");

// Greet a player (adapts by session count)
let msg = PromptTemplate::greeting("Alex", 0);   // first time
let msg = PromptTemplate::greeting("Alex", 5);   // returning
let msg = PromptTemplate::greeting("Alex", 50);  // long-term player
```

---

## API Reference

### `TutorStyle`
Teaching approach: `Encouraging`, `Socratic`, `Direct`, `Playful`, `Gentle`.

### `TutorPersona`
The tutor's personality.

| Field | Description |
|---|---|
| `name` | Display name (e.g. "Sparky") |
| `style` | `TutorStyle` enum |
| `patience` | 0.0 (low) → 1.0 (infinite patience) |
| `enthusiasm` | 0.0 (flat) → 1.0 (bouncing off the walls) |
| `technical_level` | 0.0 (kid-friendly) → 1.0 (full jargon) |

`new(name, style)` creates with defaults: patience=0.7, enthusiasm=0.6, technical_level=0.3.

### `LearningStyle`
How a player learns: `Visual`, `HandsOn`, `Verbal`, `Sequential`, `Exploratory`.

### `TeachingMoment`
A single meaningful teaching moment: `tick`, `concept`, `context`, `trigger` (`TriggerType`), `outcome` (`Outcome`).

### `TriggerType`
Why the moment happened: `ConservationViolation`, `AgentError`, `Discovery`, `Question`, `Milestone`, `RoomDissolution`, `FarmFailure`.

### `Outcome`
How the learner responded: `Understood`, `Confused`, `Excited`, `Frustrated`, `Curious`.

### `TutorMemory`
Everything the tutor remembers about a player.

| Method | Description |
|---|---|
| `new(player_id)` | Create blank memory (default: `Exploratory` learning style) |
| `record_moment(moment)` | Record a teaching moment; auto-updates learned/struggling sets |
| `is_struggling_with(concept)` | True if concept is in struggling set |
| `has_learned(concept)` | True if concept is in learned set |
| `suggest_next_concept(available)` | Prioritizes struggling→unlearned; returns `None` if all learned |

**Auto-tracking rules:**
- `Understood` / `Excited` → adds to learned, removes from struggling
- `Confused` / `Frustrated` → adds to struggling
- `Curious` → neither (they're exploring, not stuck)

### `TutorAction`
Platform actions the tutor can request:
- `HighlightConcept(String)` — draw attention to something
- `SuggestExperiment(String)` — suggest the player try something
- `UnlockSkill(String)` — grant a new ability
- `ShowHint(String)` — display a hint/clue
- `Celebrate(String)` — celebrate an achievement

### `Tone`
Emotional tone: `Warm`, `Excited`, `Thoughtful`, `Encouraging`, `Celebratory`.

### `TutorResponse`
A full tutor response: `text`, `action` (optional), `concept` (optional), `tone`.

| Constructor | Description |
|---|---|
| `text(text, tone)` | Simple text response |
| `with_action(text, action, tone)` | Response with a platform action |
| `about_concept(text, concept, tone)` | Response about a specific concept |

### `TutorEvent`
Game-world events the tutor can respond to:
- `ConservationError(f64)` — magnitude of violation
- `EntityCreated(String)` — player created something
- `AgentComposed(u64, u64)` — two agents combined
- `BridgeFell` — structure collapsed
- `CropFailed(String)` — farm failure (e.g. "wheat")
- `NewBiomeDiscovered(String)` — new area found
- `PlayerIdle(u64)` — ticks since last action
- `QuestionAsked(String)` — player asked something
- `AchievementEarned(String)` — player earned something

### `PromptTemplate`
Static methods for generating response text (no LLM call):

| Method | Description |
|---|---|
| `conservation_violation(error, context)` | Explains a conservation error |
| `discovery(what, concept)` | Celebrates a discovery |
| `struggling(concept, attempts)` | Adapts tone: gentle → step back → reframe |
| `celebration(achievement)` | Celebrates a milestone |
| `room_dissolution(ticks_lived)` | Marks a room reset as growth |
| `greeting(player, sessions)` | Adapts by familiarity |

### `TutorEngine`
The main orchestrator.

| Method | Description |
|---|---|
| `new(persona, player_id)` | Create engine with persona and player |
| `process_event(event, tick)` | **Main entry point** — returns `Option<TutorResponse>` |
| `generate_response(prompt)` | Create `TutorResponse` from prompt string |
| `update_memory(response, outcome)` | Record a moment and increment interaction counter |
| `get_memory()` | Reference to the player's `TutorMemory` |
| `should_intervene(event)` | Check if the tutor should respond to this event |

**Intervention rules:**
| Event | When to intervene |
|---|---|
| `ConservationError` | Always |
| `AchievementEarned` | Always |
| `BridgeFell` | Always |
| `QuestionAsked` | Always |
| `AgentComposed` | Always |
| `CropFailed` | Only if patience > 0.3 |
| `EntityCreated` / `NewBiomeDiscovered` | Only if enthusiasm > 0.3 |
| `PlayerIdle` | Only after 30+ ticks (response after 60+) |

### `personas` module
Pre-built personas: `sparky()`, `atlas()`, `luna()`, `nova()`.

| Persona | Style | Patience | Enthusiasm | Technical |
|---|---|---|---|---|
| **Sparky** | Playful | 0.6 | 0.95 | 0.2 |
| **Atlas** | Socratic | 0.9 | 0.4 | 0.8 |
| **Luna** | Gentle | 0.95 | 0.5 | 0.15 |
| **Nova** | Direct | 0.5 | 0.85 | 0.4 |

---

## How It Works

### Event → Response Pipeline

```
Game Event (TutorEvent)
        ↓
TutorEngine.should_intervene()  ← persona patience/enthusiasm gates
        ↓ (yes)
TutorEngine.process_event()
        ↓
Match event type → select PromptTemplate → generate text
        ↓
Attach Tone, TutorAction, concept
        ↓
Update TutorMemory (record moment, increment interactions)
        ↓
Return Option<TutorResponse>
```

### Memory Update Rules

When a `TeachingMoment` is recorded:

```
match outcome {
    Understood | Excited  → concepts_learned.insert(concept)
                             concepts_struggling.remove(concept)
    Confused | Frustrated → concepts_struggling.insert(concept)
    Curious               → (no change — exploring, not stuck)
}
```

### Concept Suggestion Priority

```
suggest_next_concept(available):
  1. First concept that is struggling AND not yet learned
  2. First concept that is not yet learned
  3. None (all learned)
```

### Struggling Escalation

The `struggling` template has three tiers based on attempt count:

| Attempts | Tone | Strategy |
|---|---|---|
| 1–2 | Gentle | "Let's try a different angle" |
| 3–5 | Step back | "Let's take a step back" |
| 6+ | Reframe | "Let's completely reframe" |

---

## The Math

### Memory as a State Machine

Each concept tracks the learner's state:

```
Unknown → Struggling → Learned
   ↑          │
   └──────────┘ (can relapse if confused again after being understood)
```

State transitions are driven by `Outcome`:
- `Understood`/`Excited` → **Learned** (terminal for that concept)
- `Confused`/`Frustrated` → **Struggling**
- `Curious` → stays in current state (neutral observation)

### Intervention as a Threshold Function

```
intervene(event, persona) = match event {
    ConservationError | Achievement | BridgeFell | Question | Composed => true
    CropFailed          => persona.patience > 0.3
    EntityCreated | Biome => persona.enthusiasm > 0.3
    PlayerIdle(t)       => t > 30
}
```

This is a piecewise threshold function over the persona's continuous parameters, giving smooth control over tutor behavior.

### Concept Suggestion as Greedy Priority

```
suggest(available) = argmax_{c ∈ available} priority(c)
where priority(c) = 2  if struggling(c) ∧ ¬learned(c)
                    1  if ¬learned(c)
                    0  if learned(c)
```

Returns the highest-priority unlearned concept, preferring those the learner is actively struggling with.

---

## Testing

**56 tests** covering:
- Persona construction, defaults, and cloning
- Memory: recording moments, auto-updating learned/struggling sets, concept suggestion priority
- Prompt templates: all 6 template methods with parameterized inputs
- Engine: all 9 event types processed correctly with appropriate tones, actions, and concepts
- Intervention gating: persona attributes correctly filter which events get responses
- Response constructors: text, with_action, about_concept
- All 4 pre-built personas verified
- Full serde round-trips for `TutorPersona`, `TutorMemory`, and `TutorEngine`

Run: `cargo test`

---

## License

MIT

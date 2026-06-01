# lau-ai-tutor

> Intelligence layer for PLATO — an AI tutor that adapts to every kid.

## What This Does

Intelligence layer for PLATO — an AI tutor that adapts to every kid.. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-ai-tutor
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_ai_tutor::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub enum TutorStyle 
pub struct TutorPersona 
    pub fn new(name: &str, style: TutorStyle) -> Self 
pub enum LearningStyle 
pub struct TeachingMoment 
pub enum TriggerType 
pub enum Outcome 
pub struct TutorMemory 
    pub fn new(player_id: &str) -> Self 
    pub fn record_moment(&mut self, moment: TeachingMoment) 
    pub fn is_struggling_with(&self, concept: &str) -> bool 
    pub fn has_learned(&self, concept: &str) -> bool 
    pub fn suggest_next_concept(&self, available: &[String]) -> Option<String> 
pub enum TutorAction 
pub enum Tone 
pub struct TutorResponse 
    pub fn text(text: &str, tone: Tone) -> Self 
    pub fn with_action(text: &str, action: TutorAction, tone: Tone) -> Self 
    pub fn about_concept(text: &str, concept: &str, tone: Tone) -> Self 
pub enum TutorEvent 
pub struct PromptTemplate;
    pub fn conservation_violation(error: f64, context: &str) -> String 
    pub fn discovery(what: &str, concept: &str) -> String 
    pub fn struggling(concept: &str, attempts: u32) -> String 
    pub fn celebration(achievement: &str) -> String 
    pub fn room_dissolution(ticks_lived: u64) -> String 
    pub fn greeting(player: &str, sessions: u64) -> String 
pub struct TutorEngine 
    pub fn new(persona: TutorPersona, player_id: &str) -> Self 
    pub fn process_event(&mut self, event: TutorEvent, _tick: u64) -> Option<TutorResponse> 
    pub fn generate_response(&self, prompt: &str) -> TutorResponse 
    pub fn update_memory(&mut self, response: &TutorResponse, outcome: Outcome) 
    pub fn get_memory(&self) -> &TutorMemory 
    pub fn should_intervene(&self, event: &TutorEvent) -> bool 
    pub fn sparky() -> TutorPersona 
    pub fn atlas() -> TutorPersona 
    pub fn luna() -> TutorPersona 
    pub fn nova() -> TutorPersona 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**56 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT

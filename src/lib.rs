//! # lau-ai-tutor
//!
//! The intelligence layer for **PLATO** — an AI tutor that actually talks to kids.
//! This crate provides the tutoring logic, memory, and prompt templates.
//! It does **not** call an LLM — it prepares structured prompts that the platform
//! layer sends to the model. This keeps the tutoring logic portable and testable.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Persona — the tutor's character
// ---------------------------------------------------------------------------

/// How the tutor approaches teaching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TutorStyle {
    /// Positive reinforcement, lots of praise.
    Encouraging,
    /// Answers with questions to provoke thought.
    Socratic,
    /// Straightforward, tells you what you need to know.
    Direct,
    /// Jokes, metaphors, fun analogies.
    Playful,
    /// Never pressures, always soft and supportive.
    Gentle,
}

/// A tutor's personality — who they are and how they teach.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorPersona {
    /// The tutor's display name (e.g. "Sparky").
    pub name: String,
    /// Pedagogical style.
    pub style: TutorStyle,
    /// Patience factor. 0.0 = very little patience, 1.0 = extremely patient.
    pub patience: f64,
    /// Enthusiasm factor. 0.0 = flat, 1.0 = bouncing off the walls.
    pub enthusiasm: f64,
    /// How technical the tutor's language is. 0.0 = kid-friendly, 1.0 = full jargon.
    pub technical_level: f64,
}

impl TutorPersona {
    /// Create a new tutor persona with default values for the given style.
    ///
    /// Defaults: patience=0.7, enthusiasm=0.6, technical_level=0.3
    pub fn new(name: &str, style: TutorStyle) -> Self {
        Self {
            name: name.to_string(),
            style,
            patience: 0.7,
            enthusiasm: 0.6,
            technical_level: 0.3,
        }
    }
}

// ---------------------------------------------------------------------------
// Memory — remembers the learner
// ---------------------------------------------------------------------------

/// How this player learns best.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LearningStyle {
    /// Learns through diagrams, videos, spatial layouts.
    Visual,
    /// Learns by doing — building, touching, simulating.
    HandsOn,
    /// Learns through language — reading, discussing, explaining.
    Verbal,
    /// Learns step-by-step, in order.
    Sequential,
    /// Learns by exploring, tinkering, discovering sideways.
    Exploratory,
}

/// A single meaningful teaching moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingMoment {
    /// Game tick when this moment occurred.
    pub tick: u64,
    /// The concept involved.
    pub concept: String,
    /// What was happening in the world.
    pub context: String,
    /// What triggered this moment.
    pub trigger: TriggerType,
    /// How the learner responded.
    pub outcome: Outcome,
}

/// Why a teaching moment was triggered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    /// A conservation law was violated (e.g. mass disappeared).
    ConservationViolation,
    /// The agent system made a mistake.
    AgentError,
    /// The player discovered something new.
    Discovery,
    /// The player asked a question.
    Question,
    /// The player reached a significant milestone.
    Milestone,
    /// A player-built room was dissolved.
    RoomDissolution,
    /// A farm failed (e.g. crops died).
    FarmFailure,
}

/// How the learner responded to a teaching moment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Outcome {
    /// They got it.
    Understood,
    /// They're not following.
    Confused,
    /// They're excited about it.
    Excited,
    /// They're frustrated.
    Frustrated,
    /// They're curious, asking more.
    Curious,
}

/// Everything the tutor remembers about a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorMemory {
    /// Unique identifier for the player.
    pub player_id: String,
    /// Concepts the player has successfully learned.
    pub concepts_learned: HashSet<String>,
    /// Concepts the player is currently struggling with.
    pub concepts_struggling: HashSet<String>,
    /// Log of meaningful teaching moments.
    pub teaching_moments: Vec<TeachingMoment>,
    /// Total interactions between tutor and player.
    pub total_interactions: u64,
    /// Topics the player seems to enjoy most.
    pub favorite_topics: Vec<String>,
    /// How this player learns best (inferred over time).
    pub learning_style: LearningStyle,
}

impl TutorMemory {
    /// Create a new, blank memory for a player.
    pub fn new(player_id: &str) -> Self {
        Self {
            player_id: player_id.to_string(),
            concepts_learned: HashSet::new(),
            concepts_struggling: HashSet::new(),
            teaching_moments: Vec::new(),
            total_interactions: 0,
            favorite_topics: Vec::new(),
            learning_style: LearningStyle::Exploratory,
        }
    }

    /// Record a teaching moment and update associated state.
    pub fn record_moment(&mut self, moment: TeachingMoment) {
        match moment.outcome {
            Outcome::Understood | Outcome::Excited => {
                self.concepts_learned.insert(moment.concept.clone());
                self.concepts_struggling.remove(&moment.concept);
            }
            Outcome::Confused | Outcome::Frustrated => {
                self.concepts_struggling.insert(moment.concept.clone());
            }
            Outcome::Curious => {
                // Curious players might be on the verge of learning —
                // we don't mark it as learned, but we don't mark as struggling either
            }
        }
        self.teaching_moments.push(moment);
    }

    /// True if this concept is in the struggling set.
    pub fn is_struggling_with(&self, concept: &str) -> bool {
        self.concepts_struggling.contains(concept)
    }

    /// True if this concept is in the learned set.
    pub fn has_learned(&self, concept: &str) -> bool {
        self.concepts_learned.contains(concept)
    }

    /// Suggest the next concept to teach from `available`, preferring concepts
    /// the player is struggling with (that haven't been learned yet).
    ///
    /// Returns `None` when `available` is empty or all concepts are already learned.
    pub fn suggest_next_concept(&self, available: &[String]) -> Option<String> {
        // First pass: find a concept the player is struggling with but hasn't learned
        for concept in available {
            if self.is_struggling_with(concept) && !self.has_learned(concept) {
                return Some(concept.clone());
            }
        }
        // Second pass: any unlearned concept
        for concept in available {
            if !self.has_learned(concept) {
                return Some(concept.clone());
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// TutorResponse — what the tutor says and does
// ---------------------------------------------------------------------------

/// An action the tutor wants the platform to execute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TutorAction {
    /// Highlight / ping / draw attention to a concept or element.
    HighlightConcept(String),
    /// Suggest the player try an experiment.
    SuggestExperiment(String),
    /// Unlock a skill or ability.
    UnlockSkill(String),
    /// Show a hint or clue.
    ShowHint(String),
    /// Celebrate the player's achievement.
    Celebrate(String),
}

/// The emotional tone of a tutor response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Tone {
    /// Comforting, gentle.
    Warm,
    /// High-energy, thrilling.
    Excited,
    /// Pensive, reflective.
    Thoughtful,
    /// Lifting the player up.
    Encouraging,
    /// Proud, marking a big moment.
    Celebratory,
}

/// A full response from the tutor — text, optional action, optional concept, and tone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorResponse {
    /// The message text to present (or send to the LLM as context).
    pub text: String,
    /// An optional action for the platform to execute.
    pub action: Option<TutorAction>,
    /// The concept this response is about.
    pub concept: Option<String>,
    /// The emotional tone of the response.
    pub tone: Tone,
}

impl TutorResponse {
    /// Create a simple text-only response.
    pub fn text(text: &str, tone: Tone) -> Self {
        Self {
            text: text.to_string(),
            action: None,
            concept: None,
            tone,
        }
    }

    /// Create a response with an action.
    pub fn with_action(text: &str, action: TutorAction, tone: Tone) -> Self {
        Self {
            text: text.to_string(),
            action: Some(action),
            concept: None,
            tone,
        }
    }

    /// Create a response about a specific concept.
    pub fn about_concept(text: &str, concept: &str, tone: Tone) -> Self {
        Self {
            text: text.to_string(),
            action: None,
            concept: Some(concept.to_string()),
            tone,
        }
    }
}

// ---------------------------------------------------------------------------
// TutorEvent — things that happen in the world
// ---------------------------------------------------------------------------

/// Events from the game world that the tutor may choose to respond to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorEvent {
    /// A conservation error occurred, with the deviation amount.
    ConservationError(f64),
    /// A new entity was created by the player.
    EntityCreated(String),
    /// An agent was composed from two others (env_id_1, env_id_2).
    AgentComposed(u64, u64),
    /// A bridge collapsed.
    BridgeFell,
    /// A crop failed (e.g. "wheat").
    CropFailed(String),
    /// A new biome was discovered.
    NewBiomeDiscovered(String),
    /// Player has been idle for the given number of ticks.
    PlayerIdle(u64),
    /// The player asked an explicit question.
    QuestionAsked(String),
    /// The player earned an achievement.
    AchievementEarned(String),
}

// ---------------------------------------------------------------------------
// PromptTemplate — templates for generating tutor responses
// ---------------------------------------------------------------------------

/// Pre-built prompt templates that the tutor uses to generate responses.
///
/// Each method returns a string that can be sent to an LLM or shown directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate;

impl PromptTemplate {
    /// Generate a response for a conservation violation.
    ///
    /// The `error` is the magnitude of the violation.
    pub fn conservation_violation(error: f64, context: &str) -> String {
        format!(
            "Hmm, the numbers went red! That means something disappeared from our world. \
             We lost {error:.2} units of something. {context} \
             In science, we call this a conservation violation — matter shouldn't just vanish! \
             Can you figure out where it went?"
        )
    }

    /// Generate a response for when a player discovers something.
    pub fn discovery(what: &str, concept: &str) -> String {
        format!(
            "You just figured out {what}! That's what scientists call {concept}. \
             You didn't just stumble on it — you observed, you experimented, and you found it. \
             That's the scientific method in action!"
        )
    }

    /// Generate a response when the player is struggling with a concept.
    ///
    /// Adjusts tone based on number of attempts.
    pub fn struggling(concept: &str, attempts: u32) -> String {
        if attempts <= 2 {
            format!(
                "Don't worry, {concept} takes a bit to click. Let's try a different angle. \
                 Think about it like this..."
            )
        } else if attempts <= 5 {
            format!(
                "This one's tricky! Let's take a step back. {concept} is all about \
                 the relationship between things. Here's a hint..."
            )
        } else {
            format!(
                "Alright, let's completely reframe {concept}. Forget what we tried before. \
                 Imagine instead..."
            )
        }
    }

    /// Generate a celebration message.
    pub fn celebration(achievement: &str) -> String {
        format!(
            "WOW! {achievement}! That's genuinely impressive. You should be proud — \
             this took real thinking. Take a moment to enjoy it!"
        )
    }

    /// Generate a message for when a player's room dissolves / resets.
    pub fn room_dissolution(ticks_lived: u64) -> String {
        format!(
            "Your room held together for {ticks_lived} ticks. \
             It's been perfectly adapted — but now it's time to let it go and grow \
             into something new. Every ending is a new beginning in this world."
        )
    }

    /// Generate a greeting that adapts to familiarity.
    pub fn greeting(player: &str, sessions: u64) -> String {
        if sessions == 0 {
            format!(
                "Hey there, {player}! I'm here to help you explore this world. \
                 Try things, break things, ask questions — that's how we learn. Ready?"
            )
        } else if sessions <= 10 {
            format!(
                "Welcome back, {player}! Great to see you again. \
                 Ready to pick up where we left off?"
            )
        } else {
            format!(
                "{player}! You're back! I love the curiosity you bring to this world. \
                 What shall we explore today?"
            )
        }
    }
}

// ---------------------------------------------------------------------------
// TutorEngine — the main orchestrator
// ---------------------------------------------------------------------------

/// The core engine that decides when and how the tutor responds to events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorEngine {
    /// The tutor's personality.
    pub persona: TutorPersona,
    /// What the tutor remembers about this player.
    pub memory: TutorMemory,
    /// Prompt templates for response generation.
    pub templates: PromptTemplate,
}

impl TutorEngine {
    /// Create a new engine for a player with the given persona.
    pub fn new(persona: TutorPersona, player_id: &str) -> Self {
        Self {
            persona,
            memory: TutorMemory::new(player_id),
            templates: PromptTemplate,
        }
    }

    /// Process an event from the game world, returning an optional tutor response.
    ///
    /// This is the main entry point. It decides whether to respond and generates
    /// the appropriate template-based response.
    pub fn process_event(&mut self, event: TutorEvent, _tick: u64) -> Option<TutorResponse> {
        if !self.should_intervene(&event) {
            return None;
        }

        let response = match event {
            TutorEvent::ConservationError(err) => {
                let prompt = PromptTemplate::conservation_violation(err, "in the current ecosystem");
                let mut resp = self.generate_response(&prompt);
                resp.tone = Tone::Thoughtful;
                resp.concept = Some("conservation".to_string());
                resp
            }
            TutorEvent::EntityCreated(name) => {
                let prompt = PromptTemplate::discovery(&name, "entity creation");
                let mut resp = self.generate_response(prompt.as_str());
                resp.tone = Tone::Excited;
                resp.concept = Some("creation".to_string());
                resp.action = Some(TutorAction::HighlightConcept(name));
                resp
            }
            TutorEvent::AgentComposed(a, b) => {
                let prompt = format!(
                    "You combined agent {a} and agent {b} to make something new! \
                     That's composition — combining simple parts to create complex behaviors. \
                     What does your new creation do?"
                );
                let mut resp = self.generate_response(prompt.as_str());
                resp.tone = Tone::Excited;
                resp.action = Some(TutorAction::Celebrate("agent composition".to_string()));
                resp
            }
            TutorEvent::BridgeFell => {
                let prompt = "Your bridge fell! Let's think about why. \
                              Structures need balance. Too much weight on one side, \
                              or not enough support underneath — what do you think happened?";
                let mut resp = self.generate_response(prompt);
                resp.tone = Tone::Warm;
                resp.concept = Some("structural integrity".to_string());
                resp
            }
            TutorEvent::CropFailed(name) => {
                let prompt = PromptTemplate::struggling("ecology", 3);
                let mut resp = self.generate_response(prompt.as_str());
                resp.tone = Tone::Warm;
                resp.concept = Some(name);
                resp.action = Some(TutorAction::SuggestExperiment(
                    "try different soil conditions".to_string(),
                ));
                resp
            }
            TutorEvent::NewBiomeDiscovered(biome) => {
                let prompt = PromptTemplate::discovery(&biome, "biomes and ecosystems");
                let mut resp = self.generate_response(prompt.as_str());
                resp.tone = Tone::Excited;
                resp.concept = Some("biomes".to_string());
                resp.action = Some(TutorAction::UnlockSkill(biome));
                resp
            }
            TutorEvent::PlayerIdle(ticks) => {
                if ticks > 60 {
                    let prompt = "I notice you've been quiet for a while. Need a hint? \
                         Sometimes the best thing to do is just try something — \
                         even if it doesn't work, you'll learn from it!"
                        .to_string();
                    self.generate_response(prompt.as_str())
                } else {
                    return None;
                }
            }
            TutorEvent::QuestionAsked(question) => {
                let prompt = format!(
                    "That's a great question: {question} \
                     Let's think through it together. What do you already know about this?"
                );
                let mut resp = self.generate_response(prompt.as_str());
                resp.tone = Tone::Thoughtful;
                resp
            }
            TutorEvent::AchievementEarned(name) => {
                let prompt = PromptTemplate::celebration(&name);
                let mut resp = self.generate_response(prompt.as_str());
                resp.tone = Tone::Celebratory;
                resp.action = Some(TutorAction::Celebrate(name));
                resp
            }
        };

        self.update_memory(&response, Outcome::Curious);
        Some(response)
    }

    /// Generate a [`TutorResponse`] from a prompt string.
    ///
    /// This performs template expansion (no LLM call). The platform layer
    /// can take the prompt string and send it to an actual model.
    pub fn generate_response(&self, prompt: &str) -> TutorResponse {
        TutorResponse {
            text: prompt.to_string(),
            action: None,
            concept: None,
            tone: Tone::Warm,
        }
    }

    /// Update memory based on a response and its outcome.
    ///
    /// Records a teaching moment and increments the interaction counter.
    pub fn update_memory(&mut self, response: &TutorResponse, outcome: Outcome) {
        self.memory.total_interactions += 1;

        if let Some(ref concept) = response.concept {
            let moment = TeachingMoment {
                tick: self.memory.total_interactions,
                concept: concept.clone(),
                context: response.text.clone(),
                trigger: TriggerType::Discovery,
                outcome,
            };
            self.memory.record_moment(moment);
        }
    }

    /// Get a reference to the tutor's memory.
    pub fn get_memory(&self) -> &TutorMemory {
        &self.memory
    }

    /// Decide whether the tutor should respond to a given event.
    ///
    /// Uses persona attributes (patience, enthusiasm) to modulate intervention rate.
    pub fn should_intervene(&self, event: &TutorEvent) -> bool {
        match event {
            // Conservation errors always get a response
            TutorEvent::ConservationError(_) => true,

            // Achievements always get celebrated
            TutorEvent::AchievementEarned(_) => true,

            // Bridge failures always warrant a check-in
            TutorEvent::BridgeFell => true,

            // Only intervene for crops if patience > 0.3
            TutorEvent::CropFailed(_) => self.persona.patience > 0.3,

            // Questions always deserve an answer
            TutorEvent::QuestionAsked(_) => true,

            // Discoveries are more likely to get a response with high enthusiasm
            TutorEvent::EntityCreated(_) | TutorEvent::NewBiomeDiscovered(_) => {
                self.persona.enthusiasm > 0.3
            }

            // Composition events are fun — always respond
            TutorEvent::AgentComposed(_, _) => true,

            // Only respond to idleness after some threshold
            TutorEvent::PlayerIdle(ticks) => *ticks > 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Pre-built personas
// ---------------------------------------------------------------------------

/// Pre-built tutor personas.
pub mod personas {
    use super::{TutorPersona, TutorStyle};

    /// Playful, high enthusiasm, moderate patience.
    /// Loves jokes, metaphors, and making learning feel like a game.
    pub fn sparky() -> TutorPersona {
        TutorPersona {
            name: "Sparky".to_string(),
            style: TutorStyle::Playful,
            patience: 0.6,
            enthusiasm: 0.95,
            technical_level: 0.2,
        }
    }

    /// Socratic, high patience, high technical level.
    /// Answers questions with questions. Trusts the player to reason through it.
    pub fn atlas() -> TutorPersona {
        TutorPersona {
            name: "Atlas".to_string(),
            style: TutorStyle::Socratic,
            patience: 0.9,
            enthusiasm: 0.4,
            technical_level: 0.8,
        }
    }

    /// Gentle, high patience, low technical level.
    /// Never pressures. Speaks softly and uses simple language.
    pub fn luna() -> TutorPersona {
        TutorPersona {
            name: "Luna".to_string(),
            style: TutorStyle::Gentle,
            patience: 0.95,
            enthusiasm: 0.5,
            technical_level: 0.15,
        }
    }

    /// Direct, moderate patience, high enthusiasm.
    /// Gets to the point but is always excited about it.
    pub fn nova() -> TutorPersona {
        TutorPersona {
            name: "Nova".to_string(),
            style: TutorStyle::Direct,
            patience: 0.5,
            enthusiasm: 0.85,
            technical_level: 0.4,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // TutorPersona tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_persona_new_defaults() {
        let p = TutorPersona::new("Testy", TutorStyle::Encouraging);
        assert_eq!(p.name, "Testy");
        assert_eq!(p.style, TutorStyle::Encouraging);
        assert!((p.patience - 0.7).abs() < 1e-6);
        assert!((p.enthusiasm - 0.6).abs() < 1e-6);
        assert!((p.technical_level - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_persona_fields_stored_correctly() {
        let p = TutorPersona {
            name: "Testy".to_string(),
            style: TutorStyle::Socratic,
            patience: 0.5,
            enthusiasm: 0.9,
            technical_level: 0.1,
        };
        assert_eq!(p.patience, 0.5);
        assert_eq!(p.enthusiasm, 0.9);
        assert_eq!(p.technical_level, 0.1);
    }

    #[test]
    fn test_persona_clone() {
        let p1 = TutorPersona::new("Clone", TutorStyle::Playful);
        let p2 = p1.clone();
        assert_eq!(p1.name, p2.name);
        assert_eq!(p1.style, p2.style);
    }

    // -----------------------------------------------------------------------
    // TutorMemory tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_memory_new_is_empty() {
        let m = TutorMemory::new("player1");
        assert_eq!(m.player_id, "player1");
        assert!(m.concepts_learned.is_empty());
        assert!(m.concepts_struggling.is_empty());
        assert!(m.teaching_moments.is_empty());
        assert_eq!(m.total_interactions, 0);
        assert_eq!(m.learning_style, LearningStyle::Exploratory);
    }

    #[test]
    fn test_record_moment_understood_adds_to_learned() {
        let mut m = TutorMemory::new("p1");
        let moment = TeachingMoment {
            tick: 1,
            concept: "gravity".to_string(),
            context: "player dropped an object".to_string(),
            trigger: TriggerType::Discovery,
            outcome: Outcome::Understood,
        };
        m.record_moment(moment);
        assert!(m.has_learned("gravity"));
        assert!(!m.is_struggling_with("gravity"));
        assert_eq!(m.teaching_moments.len(), 1);
    }

    #[test]
    fn test_record_moment_excited_adds_to_learned() {
        let mut m = TutorMemory::new("p1");
        let moment = TeachingMoment {
            tick: 2,
            concept: "momentum".to_string(),
            context: "player saw a collision".to_string(),
            trigger: TriggerType::Discovery,
            outcome: Outcome::Excited,
        };
        m.record_moment(moment);
        assert!(m.has_learned("momentum"));
    }

    #[test]
    fn test_record_moment_confused_adds_to_struggling() {
        let mut m = TutorMemory::new("p1");
        let moment = TeachingMoment {
            tick: 3,
            concept: "entropy".to_string(),
            context: "player tried to reverse time".to_string(),
            trigger: TriggerType::ConservationViolation,
            outcome: Outcome::Confused,
        };
        m.record_moment(moment);
        assert!(m.is_struggling_with("entropy"));
    }

    #[test]
    fn test_record_moment_frustrated_adds_to_struggling() {
        let mut m = TutorMemory::new("p1");
        let moment = TeachingMoment {
            tick: 4,
            concept: "quantum".to_string(),
            context: "player kept failing".to_string(),
            trigger: TriggerType::AgentError,
            outcome: Outcome::Frustrated,
        };
        m.record_moment(moment);
        assert!(m.is_struggling_with("quantum"));
    }

    #[test]
    fn test_record_moment_curious_does_not_mark() {
        let mut m = TutorMemory::new("p1");
        let moment = TeachingMoment {
            tick: 5,
            concept: "chemistry".to_string(),
            context: "player asked 'what if'".to_string(),
            trigger: TriggerType::Question,
            outcome: Outcome::Curious,
        };
        m.record_moment(moment);
        assert!(!m.has_learned("chemistry"));
        assert!(!m.is_struggling_with("chemistry"));
    }

    #[test]
    fn test_record_moment_removes_from_struggling_when_understood() {
        let mut m = TutorMemory::new("p1");
        // First, struggling
        m.concepts_struggling.insert("algebra".to_string());
        // Then understand it
        let moment = TeachingMoment {
            tick: 10,
            concept: "algebra".to_string(),
            context: "finally clicked".to_string(),
            trigger: TriggerType::Milestone,
            outcome: Outcome::Understood,
        };
        m.record_moment(moment);
        assert!(m.has_learned("algebra"));
        assert!(!m.is_struggling_with("algebra"));
    }

    #[test]
    fn test_suggest_next_concept_prioritizes_struggling() {
        let mut m = TutorMemory::new("p1");
        m.concepts_struggling.insert("thermodynamics".to_string());
        m.concepts_learned.insert("gravity".to_string());

        let available = vec![
            "gravity".to_string(),
            "thermodynamics".to_string(),
            "optics".to_string(),
        ];
        let suggestion = m.suggest_next_concept(&available);
        assert_eq!(suggestion, Some("thermodynamics".to_string()));
    }

    #[test]
    fn test_suggest_next_concept_falls_back_to_unlearned() {
        let m = TutorMemory::new("p1");
        let available = vec!["gravity".to_string(), "optics".to_string()];
        let suggestion = m.suggest_next_concept(&available);
        assert_eq!(suggestion, Some("gravity".to_string()));
    }

    #[test]
    fn test_suggest_next_concept_none_when_all_learned() {
        let mut m = TutorMemory::new("p1");
        m.concepts_learned.insert("gravity".to_string());
        let available = vec!["gravity".to_string()];
        assert!(m.suggest_next_concept(&available).is_none());
    }

    #[test]
    fn test_suggest_next_concept_none_when_empty() {
        let m = TutorMemory::new("p1");
        assert!(m.suggest_next_concept(&[]).is_none());
    }

    #[test]
    fn test_is_struggling_with_false_when_not_present() {
        let m = TutorMemory::new("p1");
        assert!(!m.is_struggling_with("physics"));
    }

    #[test]
    fn test_has_learned_false_when_not_present() {
        let m = TutorMemory::new("p1");
        assert!(!m.has_learned("physics"));
    }

    // -----------------------------------------------------------------------
    // PromptTemplate tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_conservation_violation_includes_error() {
        let msg = PromptTemplate::conservation_violation(std::f64::consts::PI, "water cycle");
        assert!(msg.contains("3.14"));
        assert!(msg.contains("conservation"));
        assert!(msg.contains("water cycle"));
    }

    #[test]
    fn test_discovery_includes_discovery_and_concept() {
        let msg = PromptTemplate::discovery("fire", "combustion");
        assert!(msg.contains("fire"));
        assert!(msg.contains("combustion"));
    }

    #[test]
    fn test_struggling_short_attempts() {
        let msg = PromptTemplate::struggling("fractions", 1);
        assert!(msg.contains("fractions"));
        assert!(!msg.contains("step back"));
        assert!(!msg.contains("completely reframe"));
    }

    #[test]
    fn test_struggling_medium_attempts() {
        let msg = PromptTemplate::struggling("fractions", 3);
        assert!(msg.contains("step back"));
    }

    #[test]
    fn test_struggling_many_attempts() {
        let msg = PromptTemplate::struggling("fractions", 6);
        assert!(msg.contains("completely reframe"));
    }

    #[test]
    fn test_celebration_includes_achievement() {
        let msg = PromptTemplate::celebration("Master Chemist");
        assert!(msg.contains("Master Chemist"));
    }

    #[test]
    fn test_room_dissolution_includes_ticks() {
        let msg = PromptTemplate::room_dissolution(1200);
        assert!(msg.contains("1200"));
        assert!(msg.contains("ticks"));
    }

    #[test]
    fn test_greeting_first_session() {
        let msg = PromptTemplate::greeting("Alex", 0);
        assert!(msg.contains("Alex"));
        assert!(msg.contains("Ready?"));
    }

    #[test]
    fn test_greeting_returning_session() {
        let msg = PromptTemplate::greeting("Alex", 5);
        assert!(msg.contains("Welcome back"));
        assert!(msg.contains("Alex"));
    }

    #[test]
    fn test_greeting_long_term() {
        let msg = PromptTemplate::greeting("Alex", 50);
        assert!(msg.contains("curiosity"));
        assert!(msg.contains("Alex"));
    }

    // -----------------------------------------------------------------------
    // TutorEngine tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_engine_new() {
        let persona = TutorPersona::new("Sparky", TutorStyle::Playful);
        let engine = TutorEngine::new(persona, "kid42");
        assert_eq!(engine.persona.name, "Sparky");
        assert_eq!(engine.memory.player_id, "kid42");
        assert_eq!(engine.memory.total_interactions, 0);
    }

    #[test]
    fn test_process_event_conservation_error_returns_response() {
        let persona = TutorPersona::new("Testy", TutorStyle::Direct);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::ConservationError(5.0), 1);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.text.contains("conservation"));
        assert_eq!(resp.concept, Some("conservation".to_string()));
        assert_eq!(resp.tone, Tone::Thoughtful);
    }

    #[test]
    fn test_process_event_achievement_returns_celebration() {
        let persona = TutorPersona::new("Testy", TutorStyle::Direct);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::AchievementEarned("First Steps".to_string()), 5);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert_eq!(resp.tone, Tone::Celebratory);
        assert_eq!(resp.action, Some(TutorAction::Celebrate("First Steps".to_string())));
    }

    #[test]
    fn test_process_event_bridge_fell_returns_response() {
        let persona = TutorPersona::new("Testy", TutorStyle::Gentle);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::BridgeFell, 10);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert_eq!(resp.tone, Tone::Warm);
        assert_eq!(resp.concept, Some("structural integrity".to_string()));
    }

    #[test]
    fn test_process_event_question_returns_response() {
        let persona = TutorPersona::new("Testy", TutorStyle::Socratic);
        let mut engine = TutorEngine::new(persona, "p1");
        let response =
            engine.process_event(TutorEvent::QuestionAsked("Why is the sky blue?".to_string()), 15);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.text.contains("great question"));
        assert_eq!(resp.tone, Tone::Thoughtful);
    }

    #[test]
    fn test_process_event_entity_created_highlights() {
        let persona = TutorPersona::new("Sparky", TutorStyle::Playful);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::EntityCreated("bird_01".to_string()), 20);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert_eq!(resp.tone, Tone::Excited);
        assert_eq!(resp.action, Some(TutorAction::HighlightConcept("bird_01".to_string())));
    }

    #[test]
    fn test_process_event_agent_composed_celebrates() {
        let persona = TutorPersona::new("Nova", TutorStyle::Direct);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::AgentComposed(7, 3), 25);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.text.contains("composition"));
        assert_eq!(resp.tone, Tone::Excited);
    }

    #[test]
    fn test_process_event_crop_failed_returns_response() {
        let persona = TutorPersona::new("Luna", TutorStyle::Gentle);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::CropFailed("wheat".to_string()), 30);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert_eq!(resp.tone, Tone::Warm);
        assert_eq!(resp.concept, Some("wheat".to_string()));
        assert_eq!(
            resp.action,
            Some(TutorAction::SuggestExperiment(
                "try different soil conditions".to_string()
            ))
        );
    }

    #[test]
    fn test_process_event_new_biome_unlocks_skill() {
        let persona = TutorPersona::new("Sparky", TutorStyle::Playful);
        let mut engine = TutorEngine::new(persona, "p1");
        let response =
            engine.process_event(TutorEvent::NewBiomeDiscovered("desert".to_string()), 40);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert_eq!(resp.tone, Tone::Excited);
        assert_eq!(resp.action, Some(TutorAction::UnlockSkill("desert".to_string())));
    }

    #[test]
    fn test_process_event_player_idle_short_does_not_intervene() {
        let persona = TutorPersona::new("Testy", TutorStyle::Direct);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::PlayerIdle(10), 50);
        assert!(response.is_none());
    }

    #[test]
    fn test_process_event_player_idle_long_intervenes() {
        let persona = TutorPersona::new("Testy", TutorStyle::Direct);
        let mut engine = TutorEngine::new(persona, "p1");
        let response = engine.process_event(TutorEvent::PlayerIdle(100), 50);
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.text.contains("quiet"));
    }

    // -----------------------------------------------------------------------
    // should_intervene tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_intervene_conservation_error() {
        let engine = make_test_engine();
        assert!(engine.should_intervene(&TutorEvent::ConservationError(1.0)));
    }

    #[test]
    fn test_should_intervene_bridge_fell() {
        let engine = make_test_engine();
        assert!(engine.should_intervene(&TutorEvent::BridgeFell));
    }

    #[test]
    fn test_should_intervene_question() {
        let engine = make_test_engine();
        assert!(engine.should_intervene(&TutorEvent::QuestionAsked("why?".to_string())));
    }

    #[test]
    fn test_should_intervene_achievement() {
        let engine = make_test_engine();
        assert!(engine.should_intervene(&TutorEvent::AchievementEarned("test".to_string())));
    }

    #[test]
    fn test_should_intervene_agent_composed() {
        let engine = make_test_engine();
        assert!(engine.should_intervene(&TutorEvent::AgentComposed(1, 2)));
    }

    #[test]
    fn test_should_intervene_entity_created_with_enthusiasm() {
        let mut persona = TutorPersona::new("Sparky", TutorStyle::Playful);
        persona.enthusiasm = 0.5;
        let engine = TutorEngine::new(persona, "p1");
        assert!(engine.should_intervene(&TutorEvent::EntityCreated("bird".to_string())));
    }

    #[test]
    fn test_should_intervene_low_enthusiasm_skips_entity_created() {
        let mut persona = TutorPersona::new("Boring", TutorStyle::Direct);
        persona.enthusiasm = 0.1;
        let engine = TutorEngine::new(persona, "p1");
        assert!(!engine.should_intervene(&TutorEvent::EntityCreated("thing".to_string())));
    }

    #[test]
    fn test_should_intervene_player_idle_short() {
        let engine = make_test_engine();
        assert!(!engine.should_intervene(&TutorEvent::PlayerIdle(10)));
    }

    #[test]
    fn test_should_intervene_player_idle_long() {
        let engine = make_test_engine();
        assert!(engine.should_intervene(&TutorEvent::PlayerIdle(60)));
    }

    // -----------------------------------------------------------------------
    // TutorResponse tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_response_text_constructor() {
        let resp = TutorResponse::text("Hello", Tone::Warm);
        assert_eq!(resp.text, "Hello");
        assert!(resp.action.is_none());
        assert!(resp.concept.is_none());
    }

    #[test]
    fn test_response_with_action_contains_action() {
        let resp = TutorResponse::with_action(
            "Try this!",
            TutorAction::SuggestExperiment("mix water and oil".to_string()),
            Tone::Excited,
        );
        assert_eq!(resp.text, "Try this!");
        assert_eq!(
            resp.action,
            Some(TutorAction::SuggestExperiment("mix water and oil".to_string()))
        );
    }

    #[test]
    fn test_response_about_concept_sets_concept() {
        let resp = TutorResponse::about_concept("Gravity is...", "gravity", Tone::Thoughtful);
        assert_eq!(resp.concept, Some("gravity".to_string()));
    }

    // -----------------------------------------------------------------------
    // Pre-built persona tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_sparky_persona() {
        let sparky = personas::sparky();
        assert_eq!(sparky.name, "Sparky");
        assert_eq!(sparky.style, TutorStyle::Playful);
        assert!((sparky.enthusiasm - 0.95).abs() < 1e-6);
        assert!((sparky.technical_level - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_atlas_persona() {
        let atlas = personas::atlas();
        assert_eq!(atlas.name, "Atlas");
        assert_eq!(atlas.style, TutorStyle::Socratic);
        assert!((atlas.patience - 0.9).abs() < 1e-6);
        assert!((atlas.technical_level - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_luna_persona() {
        let luna = personas::luna();
        assert_eq!(luna.name, "Luna");
        assert_eq!(luna.style, TutorStyle::Gentle);
        assert!((luna.patience - 0.95).abs() < 1e-6);
        assert!((luna.technical_level - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_nova_persona() {
        let nova = personas::nova();
        assert_eq!(nova.name, "Nova");
        assert_eq!(nova.style, TutorStyle::Direct);
        assert!((nova.patience - 0.5).abs() < 1e-6);
        assert!((nova.enthusiasm - 0.85).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // Serde round-trip tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_tutor_persona_serde_roundtrip() {
        let p = personas::sparky();
        let json = serde_json::to_string(&p).unwrap();
        let back: TutorPersona = serde_json::from_str(&json).unwrap();
        assert_eq!(p.name, back.name);
        assert_eq!(p.style, back.style);
        assert!((p.patience - back.patience).abs() < 1e-6);
    }

    #[test]
    fn test_tutor_memory_serde_roundtrip() {
        let mut m = TutorMemory::new("test_player");
        m.concepts_learned.insert("physics".to_string());
        m.total_interactions = 42;
        let json = serde_json::to_string(&m).unwrap();
        let back: TutorMemory = serde_json::from_str(&json).unwrap();
        assert_eq!(back.player_id, "test_player");
        assert!(back.has_learned("physics"));
        assert_eq!(back.total_interactions, 42);
    }

    #[test]
    fn test_tutor_engine_serde_roundtrip() {
        let engine = make_test_engine();
        let json = serde_json::to_string(&engine).unwrap();
        let back: TutorEngine = serde_json::from_str(&json).unwrap();
        assert_eq!(back.persona.name, engine.persona.name);
        assert_eq!(back.memory.player_id, engine.memory.player_id);
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_test_engine() -> TutorEngine {
        let persona = TutorPersona::new("Testy", TutorStyle::Encouraging);
        TutorEngine::new(persona, "tester")
    }
}
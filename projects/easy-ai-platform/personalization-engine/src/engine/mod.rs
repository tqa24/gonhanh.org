//! Core personalization engine logic

mod lead_scoring;
mod content_rules;
mod orchestrator;

pub use lead_scoring::LeadScoringEngine;
pub use content_rules::ContentRulesEngine;
pub use orchestrator::Orchestrator;

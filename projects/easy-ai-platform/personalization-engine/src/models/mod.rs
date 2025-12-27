//! Data models and DTOs for Personalization Engine

mod lead;
mod content;
mod recommendation;

pub use lead::{LeadScore, LeadSegment, ScoringFactors};
pub use content::{ContentRule, RuleCondition, RuleAction};
pub use recommendation::{Recommendation, RecommendationType};

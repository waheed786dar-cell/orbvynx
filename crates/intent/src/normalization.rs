//! Intent Normalization (Architecture Bible, Part 3 §8).
//!
//! "Users alag tarike se same baat likhte hain." This module maps
//! varied user phrasing onto a small set of standardized goal
//! strings, so the Planner can build reusable workflows instead of
//! branching on every possible phrasing.
//!
//! This is intentionally a simple, deterministic, rule-based
//! matcher — not an AI call. Per Part 1 §20 ("AI Philosophy"),
//! the architecture must keep functioning even with zero AI
//! providers configured. A future AI-assisted normalizer can be
//! plugged in later as an *additional*, optional layer without
//! changing this contract.

use serde::{Deserialize, Serialize};

/// A single normalization rule: if the goal text contains any of
/// `triggers` (case-insensitive), it is rewritten to `canonical`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationRule {
    pub canonical: String,
    pub triggers: Vec<String>,
}

pub struct IntentNormalizer {
    rules: Vec<NormalizationRule>,
}

impl IntentNormalizer {
    /// A small starter rule set covering common Android/Rust
    /// developer phrasing (Part 3 §8's own example).
    pub fn standard() -> Self {
        Self {
            rules: vec![
                NormalizationRule {
                    canonical: "Build Android application".into(),
                    triggers: vec![
                        "build app".into(),
                        "compile app".into(),
                        "create apk".into(),
                        "generate release build".into(),
                        "build android app".into(),
                        "build my android app".into(),
                    ],
                },
                NormalizationRule {
                    canonical: "Run test suite".into(),
                    triggers: vec![
                        "run tests".into(),
                        "test my app".into(),
                        "run the test suite".into(),
                        "execute tests".into(),
                    ],
                },
                NormalizationRule {
                    canonical: "Deploy application".into(),
                    triggers: vec![
                        "deploy app".into(),
                        "publish app".into(),
                        "release app".into(),
                        "ship it".into(),
                    ],
                },
            ],
        }
    }

    pub fn with_rule(mut self, rule: NormalizationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Returns the canonical goal string if any rule matches,
    /// otherwise `None` (meaning the original goal text should be
    /// used as-is — normalization is best-effort, never mandatory).
    pub fn normalize(&self, goal: &str) -> Option<String> {
        let lower = goal.to_lowercase();
        self.rules
            .iter()
            .find(|rule| rule.triggers.iter().any(|t| lower.contains(t.as_str())))
            .map(|rule| rule.canonical.clone())
    }
}

impl Default for IntentNormalizer {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_build_variants() {
        let normalizer = IntentNormalizer::standard();
        assert_eq!(
            normalizer.normalize("Please compile app for release"),
            Some("Build Android application".to_string())
        );
        assert_eq!(
            normalizer.normalize("create APK now"),
            Some("Build Android application".to_string())
        );
    }

    #[test]
    fn unrecognized_goal_returns_none() {
        let normalizer = IntentNormalizer::standard();
        assert_eq!(normalizer.normalize("water the plants"), None);
    }

    #[test]
    fn custom_rule_is_applied() {
        let normalizer = IntentNormalizer::standard().with_rule(NormalizationRule {
            canonical: "Clean build artifacts".into(),
            triggers: vec!["clean build".into()],
        });
        assert_eq!(
            normalizer.normalize("please clean build folder"),
            Some("Clean build artifacts".to_string())
        );
    }
}

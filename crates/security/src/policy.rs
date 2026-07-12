use crate::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub denied_capabilities: Vec<String>,
    pub max_risk_score: Option<u8>,
    pub require_explicit_grant: bool,
}

impl SecurityPolicy {
    pub fn permissive() -> Self {
        Self { name: "permissive".into(), ..Default::default() }
    }

    pub fn strict() -> Self {
        Self {
            name: "strict".into(),
            denied_capabilities: vec![],
            max_risk_score: Some(30),
            require_explicit_grant: true,
        }
    }

    pub fn check_capability(&self, capability: &str) -> SecurityResult<()> {
        if self.denied_capabilities.iter().any(|c| c == capability) {
            return Err(SecurityError::PolicyViolation(format!(
                "capability '{capability}' is denied by policy '{}'", self.name
            )));
        }
        Ok(())
    }

    pub fn check_risk(&self, risk: u8) -> SecurityResult<()> {
        if let Some(max) = self.max_risk_score {
            if risk > max {
                return Err(SecurityError::PolicyViolation(format!(
                    "risk {risk} exceeds policy '{}' max {max}", self.name
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_policy_rejects_high_risk() {
        let policy = SecurityPolicy::strict();
        assert!(policy.check_risk(50).is_err());
        assert!(policy.check_risk(10).is_ok());
    }

    #[test]
    fn denied_capability_is_rejected() {
        let mut policy = SecurityPolicy::permissive();
        policy.denied_capabilities.push("git.push".to_string());
        assert!(policy.check_capability("git.push").is_err());
        assert!(policy.check_capability("git.status").is_ok());
    }
}

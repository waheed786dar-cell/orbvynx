use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyConstraints {
    pub no_internet: bool,
    pub read_only: bool,
    pub max_risk: Option<u8>,
}

pub struct PolicyEvaluator {
    pub constraints: PolicyConstraints,
}

impl PolicyEvaluator {
    pub fn new(constraints: PolicyConstraints) -> Self {
        Self { constraints }
    }

    pub fn check_capabilities(&self, required: &[String]) -> Result<(), String> {
        if self.constraints.no_internet && required.iter().any(|c| c.contains("network") || c.contains("http")) {
            return Err("no_internet policy forbids network-dependent capabilities".into());
        }
        if self.constraints.read_only && required.iter().any(|c| c.contains("write") || c.contains("delete")) {
            return Err("read_only policy forbids write/delete capabilities".into());
        }
        Ok(())
    }

    pub fn check_risk(&self, risk: u8) -> Result<(), String> {
        if let Some(max) = self.constraints.max_risk {
            if risk > max {
                return Err(format!("risk score {risk} exceeds policy max {max}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_internet_blocks_network_capability() {
        let policy = PolicyEvaluator::new(PolicyConstraints { no_internet: true, ..Default::default() });
        assert!(policy.check_capabilities(&["network.http".to_string()]).is_err());
    }

    #[test]
    fn read_only_blocks_write_capability() {
        let policy = PolicyEvaluator::new(PolicyConstraints { read_only: true, ..Default::default() });
        assert!(policy.check_capabilities(&["filesystem.write".to_string()]).is_err());
    }

    #[test]
    fn risk_over_max_is_rejected() {
        let policy = PolicyEvaluator::new(PolicyConstraints { max_risk: Some(50), ..Default::default() });
        assert!(policy.check_risk(60).is_err());
        assert!(policy.check_risk(40).is_ok());
    }
}

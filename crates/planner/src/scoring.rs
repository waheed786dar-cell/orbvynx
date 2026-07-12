//! Risk Analysis & Cost Estimation (Architecture Bible, Part 4 §10, §11).
//!
//! Every candidate Plan gets a risk score and a cost estimate so
//! multiple plans can be compared (Part 4 §12, "Multiple Plans").
//! These are estimates only — the Executor later records *actual*
//! resource usage (Part 6 §17), which feeds back into future
//! planning via the Optimization Memory domain, without ever
//! affecting the current, already-selected plan.

use serde::{Deserialize, Serialize};

/// Risk score on a 0-100 scale (Part 4 §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RiskScore(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Medium,
    High,
}

impl RiskScore {
    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    /// Bands defined exactly as in Part 4 §10:
    /// 0-20 Safe, 21-50 Medium, 51-100 High.
    pub fn level(&self) -> RiskLevel {
        match self.0 {
            0..=20 => RiskLevel::Safe,
            21..=50 => RiskLevel::Medium,
            _ => RiskLevel::High,
        }
    }
}

/// Factors contributing to a risk score (Part 4 §10). Each factor
/// is a simple boolean signal; `compute` combines them into a
/// single score using fixed weights. This keeps risk scoring fully
/// deterministic — no AI or randomness involved, per Part 4 §17
/// ("Deterministic Planning").
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskFactors {
    pub data_loss_risk: bool,
    pub security_risk: bool,
    pub network_dependency: bool,
    pub long_execution: bool,
    pub external_apis: bool,
    pub missing_tools: bool,
    pub high_permission_level: bool,
}

impl RiskFactors {
    pub fn compute(&self) -> RiskScore {
        let mut score: u16 = 0;
        if self.data_loss_risk {
            score += 30;
        }
        if self.security_risk {
            score += 25;
        }
        if self.network_dependency {
            score += 10;
        }
        if self.long_execution {
            score += 10;
        }
        if self.external_apis {
            score += 10;
        }
        if self.missing_tools {
            score += 10;
        }
        if self.high_permission_level {
            score += 15;
        }
        RiskScore::new(score.min(100) as u8)
    }
}

/// Estimated resource cost of executing a plan (Part 4 §11). All
/// fields are estimates in the Planner's own units — the Executor's
/// `ResourceUsage` (Part 6 §17) records the real, measured numbers
/// after execution for comparison.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct CostEstimate {
    pub cpu_percent_seconds: f64,
    pub memory_mb: f64,
    pub disk_mb: f64,
    pub network_mb: f64,
    pub estimated_seconds: f64,
}

impl CostEstimate {
    pub fn combine(&self, other: &CostEstimate) -> CostEstimate {
        CostEstimate {
            cpu_percent_seconds: self.cpu_percent_seconds + other.cpu_percent_seconds,
            memory_mb: self.memory_mb.max(other.memory_mb),
            disk_mb: self.disk_mb + other.disk_mb,
            network_mb: self.network_mb + other.network_mb,
            estimated_seconds: self.estimated_seconds + other.estimated_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_bands_match_spec() {
        assert_eq!(RiskScore::new(0).level(), RiskLevel::Safe);
        assert_eq!(RiskScore::new(20).level(), RiskLevel::Safe);
        assert_eq!(RiskScore::new(21).level(), RiskLevel::Medium);
        assert_eq!(RiskScore::new(50).level(), RiskLevel::Medium);
        assert_eq!(RiskScore::new(51).level(), RiskLevel::High);
        assert_eq!(RiskScore::new(100).level(), RiskLevel::High);
    }

    #[test]
    fn risk_factors_combine_and_cap_at_100() {
        let factors = RiskFactors {
            data_loss_risk: true,
            security_risk: true,
            network_dependency: true,
            long_execution: true,
            external_apis: true,
            missing_tools: true,
            high_permission_level: true,
        };
        assert_eq!(factors.compute(), RiskScore::new(100));
    }

    #[test]
    fn cost_estimates_combine() {
        let a = CostEstimate {
            cpu_percent_seconds: 10.0,
            memory_mb: 100.0,
            disk_mb: 5.0,
            network_mb: 1.0,
            estimated_seconds: 2.0,
        };
        let b = CostEstimate {
            cpu_percent_seconds: 5.0,
            memory_mb: 200.0,
            disk_mb: 3.0,
            network_mb: 0.0,
            estimated_seconds: 1.0,
        };
        let combined = a.combine(&b);
        assert_eq!(combined.cpu_percent_seconds, 15.0);
        assert_eq!(combined.memory_mb, 200.0); // peak, not sum
        assert_eq!(combined.disk_mb, 8.0);
        assert_eq!(combined.estimated_seconds, 3.0);
    }
}

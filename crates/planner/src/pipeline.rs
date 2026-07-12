use crate::context::{CapabilityDiscovery, PlanningContext};
use crate::error::{PlannerError, PlannerResult};
use crate::plan::{Plan, PlanBuilder};
use crate::policy::PolicyEvaluator;
use crate::scoring::{CostEstimate, RiskFactors};
use orbvynx_intent::Intent;
use std::sync::Arc;

pub struct PlanningPipeline {
    pub capability_discovery: Arc<dyn CapabilityDiscovery>,
    pub policy: PolicyEvaluator,
}

impl PlanningPipeline {
    pub fn new(capability_discovery: Arc<dyn CapabilityDiscovery>, policy: PolicyEvaluator) -> Self {
        Self { capability_discovery, policy }
    }

    pub async fn plan(&self, intent: &Intent, ctx: &PlanningContext, required_capabilities: Vec<String>) -> PlannerResult<Plan> {
        let available = self.capability_discovery.available_capabilities().await;
        for cap in &required_capabilities {
            if !available.contains(cap) {
                return Err(PlannerError::NoCapabilityAvailable {
                    intent_id: intent.id(),
                    needed: cap.clone(),
                });
            }
        }

        self.policy
            .check_capabilities(&required_capabilities)
            .map_err(|reason| PlannerError::PolicyConflict { intent_id: intent.id(), reason })?;

        let mut factors = RiskFactors::default();
        if !ctx.internet_available {
            factors.network_dependency = required_capabilities.iter().any(|c| c.contains("network"));
        }
        factors.missing_tools = false;
        let risk = factors.compute();

        self.policy
            .check_risk(risk.0)
            .map_err(|reason| PlannerError::PolicyConflict { intent_id: intent.id(), reason })?;

        let cost = CostEstimate {
            cpu_percent_seconds: 5.0,
            memory_mb: 256.0,
            disk_mb: 50.0,
            network_mb: if ctx.internet_available { 10.0 } else { 0.0 },
            estimated_seconds: 30.0,
        };

        let mut builder = PlanBuilder::new(intent.id(), intent.effective_goal().to_string());
        for cap in &required_capabilities {
            builder = builder.require_capability(cap.clone()).add_step(format!("Invoke {cap}"), cap.clone());
        }

        Ok(builder.finish(cost, risk))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::StaticCapabilityDiscovery;
    use crate::policy::PolicyConstraints;
    use orbvynx_intent::IntentSource;
    use uuid::Uuid;

    fn sample_intent() -> Intent {
        Intent::new("Build my app", IntentSource::Cli, Uuid::new_v4())
    }

    #[tokio::test]
    async fn plan_succeeds_when_capability_available() {
        let discovery = Arc::new(StaticCapabilityDiscovery::new(vec!["android.build".to_string()]));
        let pipeline = PlanningPipeline::new(discovery, PolicyEvaluator::new(PolicyConstraints::default()));
        let ctx = PlanningContext { internet_available: true, ..Default::default() };

        let plan = pipeline.plan(&sample_intent(), &ctx, vec!["android.build".to_string()]).await.unwrap();
        assert_eq!(plan.required_capabilities, vec!["android.build".to_string()]);
        assert_eq!(plan.steps.len(), 1);
    }

    #[tokio::test]
    async fn plan_fails_when_capability_missing() {
        let discovery = Arc::new(StaticCapabilityDiscovery::new(vec![]));
        let pipeline = PlanningPipeline::new(discovery, PolicyEvaluator::new(PolicyConstraints::default()));
        let ctx = PlanningContext::default();

        let result = pipeline.plan(&sample_intent(), &ctx, vec!["android.build".to_string()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn plan_fails_when_policy_conflicts() {
        let discovery = Arc::new(StaticCapabilityDiscovery::new(vec!["network.http".to_string()]));
        let policy = PolicyEvaluator::new(PolicyConstraints { no_internet: true, ..Default::default() });
        let pipeline = PlanningPipeline::new(discovery, policy);
        let ctx = PlanningContext::default();

        let result = pipeline.plan(&sample_intent(), &ctx, vec!["network.http".to_string()]).await;
        assert!(result.is_err());
    }
}

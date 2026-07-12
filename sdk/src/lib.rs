pub use async_trait::async_trait;
pub use orbvynx_executor::{Capability, CapabilityInput, CapabilityOutput};
pub use serde_json::{json, Value};

use std::collections::HashMap;

pub fn param_str<'a>(input: &'a CapabilityInput, key: &str) -> Result<&'a str, String> {
    input.params.get(key).and_then(|v| v.as_str()).ok_or_else(|| format!("missing '{key}' parameter"))
}

pub fn param_bool(input: &CapabilityInput, key: &str) -> Option<bool> {
    input.params.get(key).and_then(|v| v.as_bool())
}

pub fn ok(data: Value) -> Result<CapabilityOutput, String> {
    Ok(CapabilityOutput { data })
}

pub fn input_from(pairs: Vec<(&str, Value)>) -> CapabilityInput {
    let mut params = HashMap::new();
    for (k, v) in pairs {
        params.insert(k.to_string(), v);
    }
    CapabilityInput { params }
}

#[macro_export]
macro_rules! capability {
    ($struct_name:ident, $cap_name:expr, |$input:ident| $body:block) => {
        pub struct $struct_name;

        #[$crate::async_trait]
        impl $crate::Capability for $struct_name {
            fn name(&self) -> &str {
                $cap_name
            }

            async fn invoke(&self, $input: $crate::CapabilityInput) -> Result<$crate::CapabilityOutput, String> {
                $body
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    capability!(GreetCapability, "example.greet", |input| {
        let name = param_str(&input, "name")?;
        ok(json!({ "greeting": format!("Hello, {name}!") }))
    });

    #[tokio::test]
    async fn macro_generated_capability_works() {
        let cap = GreetCapability;
        let input = input_from(vec![("name", json!("Waheed"))]);
        let output = cap.invoke(input).await.unwrap();
        assert_eq!(output.data["greeting"], "Hello, Waheed!");
    }

    #[tokio::test]
    async fn missing_param_returns_error() {
        let cap = GreetCapability;
        let input = input_from(vec![]);
        assert!(cap.invoke(input).await.is_err());
    }
}

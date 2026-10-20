use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::environment::{Environment, Variable};
use crate::services::importers::clean_name;

/// Postman environment JSON. Secret values are never written; they are exported empty.
pub fn to_postman(env: &Environment) -> Value {
    let values: Vec<Value> = env
        .variables
        .iter()
        .map(|v| {
            json!({
                "key": v.key,
                "value": if v.secret { "" } else { v.value.as_str() },
                "type": if v.secret { "secret" } else { "default" },
                "enabled": v.enabled,
            })
        })
        .collect();
    json!({ "id": env.id, "name": env.name, "values": values, "_postman_variable_scope": "environment" })
}

/// Parses a Postman environment; secret values are kept so saving can move them to the keychain.
pub fn from_postman(json: &str) -> AppResult<Environment> {
    let root: Value = serde_json::from_str(json)
        .map_err(|_| AppError::Validation("File bukan JSON yang valid".into()))?;
    let values = root
        .get("values")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::Validation(
                "Format tidak dikenali. Ekspor environment dari Postman terlebih dahulu".into(),
            )
        })?;
    let variables = values
        .iter()
        .filter_map(|item| {
            let key = item.get("key")?.as_str()?.trim().to_string();
            (!key.is_empty()).then(|| Variable {
                key,
                value: item
                    .get("value")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                enabled: item.get("enabled").and_then(Value::as_bool).unwrap_or(true),
                secret: item.get("type").and_then(Value::as_str) == Some("secret"),
                has_value: false,
            })
        })
        .collect();
    let name = clean_name(
        root.get("name").and_then(Value::as_str).unwrap_or_default(),
        "Environment impor",
    );
    Ok(Environment {
        id: Uuid::now_v7(),
        name,
        variables,
        is_active: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_without_leaking_secrets() {
        let env = Environment {
            id: Uuid::now_v7(),
            name: "Prod".into(),
            variables: vec![
                Variable {
                    key: "base".into(),
                    value: "https://api".into(),
                    enabled: true,
                    secret: false,
                    has_value: false,
                },
                Variable {
                    key: "token".into(),
                    value: "s3cret".into(),
                    enabled: false,
                    secret: true,
                    has_value: true,
                },
            ],
            is_active: true,
        };
        let json = to_postman(&env).to_string();
        assert!(!json.contains("s3cret"));

        let back = from_postman(&json).unwrap();
        assert_eq!(back.name, "Prod");
        assert_eq!(back.variables[0].value, "https://api");
        assert!(
            back.variables[1].secret
                && !back.variables[1].enabled
                && back.variables[1].value.is_empty()
        );
        assert!(!back.is_active);
    }

    #[test]
    fn rejects_collections_and_invalid_json() {
        assert!(from_postman(r#"{"info":{"name":"x"},"item":[]}"#).is_err());
        assert!(from_postman("nope").is_err());
    }
}

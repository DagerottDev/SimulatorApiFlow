use core_model::{AppError, proxy_rules::{ProxyRule, PROXY_RULE_SCHEMA_VERSION}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RulePreviewInput {
    pub method: String,
    pub host: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RulePreview {
    pub matched: bool,
    pub rule_id: String,
}

pub fn preview_proxy_rule(rule: ProxyRule, input: RulePreviewInput) -> Result<RulePreview, AppError> {
    if rule.schema_version != PROXY_RULE_SCHEMA_VERSION {
        return Err(AppError::new("proxy_rule_version_unsupported", "Unsupported proxy rule version.", true));
    }
    if [rule.matcher.host.value.len(), rule.matcher.path.value.len()].into_iter().any(|length| length > 256) {
        return Err(AppError::new("proxy_rule_pattern_too_long", "Rule patterns must be 256 bytes or shorter.", true));
    }
    if input.method.len() > 32 || input.host.len() > 255 || input.path.len() > 4096 {
        return Err(AppError::new("proxy_rule_preview_too_large", "Preview request fields exceed the supported size.", true));
    }
    let matched = rule.matcher.matches(&input.method, &input.host, &input.path)
        .map_err(|error| AppError::new("proxy_rule_pattern_invalid", error.to_string(), true))?;
    Ok(RulePreview { matched, rule_id: rule.id })
}

use core_model::AppError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{io::Write, process::{Command, Stdio}};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Lease {
    pub service_id: String,
    pub location_id: String,
    // Opaque backup, private journal only; never expose via browser snapshots or exports.
    pub original: String,
    pub had_configuration: bool,
}

impl std::fmt::Debug for Lease {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("MacProxyLease").field("service_id", &self.service_id).field("location_id", &self.location_id).finish_non_exhaustive()
    }
}

fn invoke_blocking(input: Value) -> Result<Value, AppError> {
    if !cfg!(target_os = "macos") {
        return Err(AppError::new("unsupported_ios_platform", "Automatic Simulator setup requires macOS and Xcode.", true));
    }
    let mut child = Command::new("/usr/bin/xcrun")
        .args(["swift", "-e", include_str!("mac_proxy.swift")])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().map_err(|error| AppError::new("mac_proxy_helper_unavailable", error.to_string(), true))?;
    let payload = serde_json::to_vec(&input).map_err(|error| AppError::new("mac_proxy_arguments_invalid", error.to_string(), true))?;
    child.stdin.take().unwrap().write_all(&payload).map_err(|error| AppError::new("mac_proxy_helper_failed", error.to_string(), true))?;
    let result = child.wait_with_output().map_err(|error| AppError::new("mac_proxy_helper_failed", error.to_string(), true))?;
    if !result.status.success() {
        return Err(AppError::new("mac_proxy_setup_failed", String::from_utf8_lossy(&result.stderr).trim().to_string(), true));
    }
    serde_json::from_slice(&result.stdout).map_err(|error| AppError::new("mac_proxy_helper_failed", error.to_string(), true))
}

async fn invoke(input: Value) -> Result<Value, AppError> {
    tokio::task::spawn_blocking(move || invoke_blocking(input)).await
        .map_err(|error| AppError::new("mac_proxy_helper_failed", error.to_string(), true))?
}

pub(crate) async fn list() -> Result<Value, AppError> {
    if !cfg!(target_os = "macos") { return Ok(json!([])); }
    invoke(json!({"action": "list"})).await
}
pub(crate) async fn prepare(service_id: &str) -> Result<Lease, AppError> {
    if service_id.is_empty() || service_id.len() > 128 || service_id.chars().any(char::is_control) {
        return Err(AppError::new("invalid_arguments", "Choose a valid Mac network service.", true));
    }
    serde_json::from_value(invoke(json!({"action": "prepare", "serviceId": service_id})).await?)
        .map_err(|error| AppError::new("mac_proxy_helper_failed", error.to_string(), true))
}
pub(crate) async fn set_enabled(lease: &Lease, enabled: bool) -> Result<(), AppError> {
    invoke(json!({"action": if enabled { "enable" } else { "disable" }, "lease": lease})).await?;
    Ok(())
}

pub(crate) async fn can_release(lease: &Lease) -> Result<bool, AppError> {
    Ok(invoke(json!({"action": "canRelease", "lease": lease})).await?["safe"].as_bool() == Some(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_lease_roundtrips_without_exposing_original_in_debug() {
        let lease = Lease {
            service_id: "selected-service".into(),
            location_id: "original-location".into(),
            original: "private-opaque-proxy-configuration".into(),
            had_configuration: false,
        };
        let debug = format!("{lease:?}");
        assert!(debug.contains("selected-service"));
        assert!(debug.contains("original-location"));
        assert!(!debug.contains(&lease.original));
        assert!(!debug.contains("had_configuration"));

        let encoded = serde_json::to_value(&lease).unwrap();
        assert_eq!(encoded["serviceId"], "selected-service");
        assert_eq!(encoded["locationId"], "original-location");
        assert_eq!(encoded["hadConfiguration"], false);
        let restored: Lease = serde_json::from_value(encoded).unwrap();
        assert_eq!(restored.service_id, lease.service_id);
        assert_eq!(restored.location_id, lease.location_id);
        assert_eq!(restored.original, lease.original);
        assert!(!restored.had_configuration);
    }
}

mod ai_commands;
mod atomic_file;
mod breakpoint_commands;
mod compare_commands;
mod fixture_commands;
mod inspect;
mod interchange_commands;
mod mock_commands;
mod proxy_rule_commands;
mod network_commands;
mod protocol_commands;
mod search_index;
pub use inspect::ingest_capture_event as ingest_capture_event_for_storage;
pub use protocol_commands::{inspect_bytes as inspect_protocol_bytes, ProtocolInspection};
mod rule_server;
mod replay_commands;
mod sdk_commands;
mod settings_commands;
mod sidecar_commands;
mod sharing_transport;
mod sharing_commands;
mod workspace_commands;
mod desktop_discovery;

use ai_storage::AiDatabase;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use capture_core::{CaptureConfig, CaptureEngine, CaptureHandle};
use capture_mitm::MitmDumpEngine;
use core_model::{
    AppError, CaptureMode, CaptureModeKind, CaptureSession, CaptureTarget, CaptureTargetKind,
    ConnectionDiagnostic, Device, DevicePlatform, FlowSummary, SCHEMA_VERSION, SessionStatus,
};
use device_android::AndroidDeviceProvider;
use device_ios::IosDeviceProvider;
use sdk_protocol::SDK_INGESTION_PORT;
use sdk_storage::SdkDatabase;
use sdk_transport::SdkIngestionServer;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::ops::Deref;
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex as StdMutex},
    time::{SystemTime, UNIX_EPOCH},
};
use storage::{BodyStore, Database};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::{JoinHandle, JoinSet},
    time::{Duration, sleep},
};
use url::Url;

const DEFAULT_CAPTURE_PORT: u16 = 8181;
const DEVICE_CAPTURE_PORT: u16 = 8183;
const DEVICE_SDK_PORT: u16 = 8184;

#[derive(Clone, Copy)]
pub struct State<'a, T>(&'a T);
impl<T> Deref for State<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

const ANDROID_HOST_ALIAS: &str = "10.0.2.2";

struct AppState {
    database: Database,
    sdk_database: SdkDatabase,
    ai_database: AiDatabase,
    body_store: BodyStore,
    capture_engine: Arc<MitmDumpEngine>,
    active_connection: Mutex<Option<ActiveConnection>>,
    connection_operation: Mutex<()>,
    shutting_down: std::sync::atomic::AtomicBool,
    rollback_path: PathBuf,
    proxy_rule_diagnostics: Arc<StdMutex<VecDeque<ProxyRuleDiagnostic>>>,
    tls_diagnostics: Arc<StdMutex<RecentTlsDiagnostics>>,
    #[cfg(unix)]
    rule_socket_path: PathBuf,
    #[cfg(windows)]
    rule_loopback_listener: StdMutex<Option<rule_server::LoopbackListener>>,
    rule_server_task: StdMutex<Option<JoinHandle<()>>>,
}

#[derive(Debug, Clone)]
struct ActiveConnection {
    handle: CaptureHandle,
    device_id: Option<String>,
    target: CaptureTarget,
    strategy: String,
    previous_android_proxy: Option<String>,
    mac_proxy_lease: Option<mac_proxy::Lease>,
    routing_enabled: bool,
    lan_guard: Option<LanGuard>,
    sdk_guard: Option<LanGuard>,
}

#[derive(Debug, Clone)]
struct LanGuard {
    host: String,
    port: u16,
    task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[derive(Debug, Clone, Serialize)]
struct ProxyRuleDiagnostic {
    code: String,
    message: String,
}

#[derive(Debug, Default)]
struct RecentTlsDiagnostics {
    session_id: Option<String>,
    client_failed: bool,
    server_failed: bool,
}

impl RecentTlsDiagnostics {
    fn begin_session(&mut self, session_id: &str) {
        *self = Self { session_id: Some(session_id.into()), ..Self::default() };
    }

    fn record(&mut self, session_id: &str, code: &str) {
        if self.session_id.as_deref() != Some(session_id) {
            return;
        }
        match code {
            "tls_client_handshake_failed" => self.client_failed = true,
            "tls_server_handshake_failed" => self.server_failed = true,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MacProcess {
    pid: u32,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LanInterface {
    name: String,
    address: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceDiscoveryPayload {
    devices: Vec<Device>,
    diagnostics: Vec<ConnectionDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionSnapshot {
    connected: bool,
    capture_running: bool,
    session_id: Option<String>,
    device_id: Option<String>,
    strategy: Option<String>,
    proxy_host: Option<String>,
    proxy_port: Option<u16>,
    capture_target: Option<CaptureTarget>,
    routing_enabled: Option<bool>,
    mac_network_service: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectDeviceResult {
    connection: ConnectionSnapshot,
    diagnostics: Vec<ConnectionDiagnostic>,
    pairing_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RollbackJournal {
    schema_version: u16,
    device_id: String,
    platform: DevicePlatform,
    session_id: String,
    previous_android_proxy: Option<String>,
    ios_ca_installed: bool,
    #[serde(default)]
    mac_proxy_lease: Option<mac_proxy::Lease>,
}

fn health(state: State<'_, AppState>) -> String {
    format!("Rust core ready · {}", state.database.path().display())
}

fn list_devices() -> DeviceDiscoveryPayload {
    let mut devices = Vec::new();
    let mut diagnostics = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let ios_provider = IosDeviceProvider;
        if ios_provider.is_available() {
            match ios_provider.list_devices() {
                Ok(mut discovered) => devices.append(&mut discovered),
                Err(error) => diagnostics.push(ConnectionDiagnostic {
                    code: error.code,
                    title: "iOS Simulator discovery failed".into(),
                    message: error.message,
                    recoverable: error.recoverable,
                    suggested_action: Some(
                        "Open Xcode and ensure Command Line Tools and Simulator runtimes are installed."
                            .into(),
                    ),
                }),
            }
        } else {
            diagnostics.push(ConnectionDiagnostic {
                code: "ios_tool_unavailable".into(),
                title: "iOS tools unavailable".into(),
                message: "xcrun/simctl could not be found on this machine.".into(),
                recoverable: true,
                suggested_action: Some(
                    "Install Xcode and select its Command Line Tools before connecting an iOS Simulator."
                        .into(),
                ),
            });
        }
    }

    let android_provider = AndroidDeviceProvider;
    if android_provider.is_available() {
        match android_provider.list_devices() {
            Ok(mut discovered) => devices.append(&mut discovered),
            Err(error) => diagnostics.push(ConnectionDiagnostic {
                code: error.code,
                title: "Android Emulator discovery failed".into(),
                message: error.message,
                recoverable: error.recoverable,
                suggested_action: Some(
                    "Start ADB from Android Platform Tools and ensure the emulator is visible in `adb devices`."
                        .into(),
                ),
            }),
        }
    } else {
        diagnostics.push(ConnectionDiagnostic {
            code: "android_tool_unavailable".into(),
            title: "Android tools unavailable".into(),
            message: "adb could not be found on this machine.".into(),
            recoverable: true,
            suggested_action: Some(
                "Install Android Platform Tools or make the Android SDK platform-tools directory available on PATH."
                    .into(),
            ),
        });
    }

    DeviceDiscoveryPayload {
        devices,
        diagnostics,
    }
}

fn list_mac_processes() -> Result<Vec<MacProcess>, AppError> {
    if cfg!(target_os = "macos") { desktop_discovery::processes() } else { Ok(Vec::new()) }
}
fn list_desktop_processes() -> Result<Vec<MacProcess>, AppError> { desktop_discovery::processes() }
fn list_lan_interfaces() -> Result<Vec<LanInterface>, AppError> { desktop_discovery::interfaces() }

fn capture_platform_info() -> serde_json::Value {
    serde_json::json!({"os": std::env::consts::OS, "iosSimulator": cfg!(target_os = "macos"),
        "localCapture": cfg!(any(target_os = "macos", target_os = "windows", target_os = "linux")),
        "guidance": if cfg!(target_os = "linux") { "Linux local capture needs kernel 6.8+, outbound traffic, and permission to load mitmproxy's eBPF helper. WSL is unsupported; run from a terminal for any native privilege prompt." }
        else if cfg!(target_os = "windows") { "Windows local capture may require native administrator permission. Trust the development CA only for the clients you control." }
        else { "macOS may prompt for local capture permission. Local capture observes outbound traffic." }})
}

fn new_pairing_token() -> Result<String, AppError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| AppError::new("pairing_token_failed", error.to_string(), true))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

impl LanGuard {
    async fn start(interface_ip: Ipv4Addr, paired_ip: Ipv4Addr, port: u16, upstream_port: u16) -> Result<Self, AppError> {
        let listener = TcpListener::bind((interface_ip, port))
            .await
            .map_err(|error| AppError::new("lan_proxy_bind_failed", error.to_string(), true))?;
        let port = listener.local_addr().map_err(|error| AppError::new("lan_proxy_bind_failed", error.to_string(), true))?.port();
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                let Ok((mut client, peer)) = listener.accept().await else {
                    break;
                };
                while connections.try_join_next().is_some() {}
                if peer.ip() != IpAddr::V4(paired_ip) || connections.len() >= 256 {
                    continue;
                }
                connections.spawn(async move {
                    if let Ok(mut upstream) =
                        TcpStream::connect((Ipv4Addr::LOCALHOST, upstream_port)).await
                    {
                        let _ = copy_bidirectional(&mut client, &mut upstream).await;
                    }
                });
            }
        });
        Ok(Self {
            host: interface_ip.to_string(),
            port,
            task: Arc::new(Mutex::new(Some(task))),
        })
    }

    async fn stop(&self) {
        if let Some(task) = self.task.lock().await.take() {
            task.abort();
            let _ = task.await;
        }
    }
}

fn list_flows(state: State<'_, AppState>) -> Result<Vec<FlowSummary>, AppError> {
    state
        .database
        .list_flows(5_000)
        .map_err(|error| AppError::storage(error.to_string()))
}

fn list_sessions(state: State<'_, AppState>) -> Result<Vec<CaptureSession>, AppError> {
    state
        .database
        .list_sessions(100)
        .map_err(|error| AppError::storage(error.to_string()))
}

async fn current_connection(state: State<'_, AppState>) -> Result<ConnectionSnapshot, AppError> {
    let active = state.active_connection.lock().await;
    match active.as_ref() {
        Some(active) => {
            let mut snapshot = connection_snapshot(active);
            snapshot.capture_running = state.capture_engine.is_running(&active.handle).await;
            Ok(snapshot)
        }
        None => Ok(disconnected_snapshot()),
    }
}

async fn connect_capture_target(
    target: CaptureTarget,
    session_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<ConnectDeviceResult, AppError> {
    if target.schema_version != SCHEMA_VERSION {
        return Err(AppError::new(
            "unsupported_capture_target_version",
            "Unsupported capture target version.",
            true,
        ));
    }
    match &target.kind {
        CaptureTargetKind::IosSimulator { device_id } if device_id.starts_with("ios:") => {
            return connect_device(device_id.clone(), session_name, state).await;
        }
        CaptureTargetKind::AndroidEmulator { device_id } if device_id.starts_with("android:") => {
            return connect_device(device_id.clone(), session_name, state).await;
        }
        CaptureTargetKind::IosSimulator { .. } | CaptureTargetKind::AndroidEmulator { .. } => {
            return Err(AppError::new(
                "invalid_capture_target",
                "The selected device type and ID do not match.",
                true,
            ));
        }
        _ => {}
    }
    if !cfg!(any(target_os = "macos", target_os = "windows", target_os = "linux")) {
        return Err(AppError::new(
            "unsupported_capture_platform",
            "Capture targets require macOS, Windows, or Linux.",
            true,
        ));
    }
    let _operation = state.connection_operation.lock().await;
    if state.shutting_down.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::new("service_shutting_down", "The service is shutting down. Finish routing recovery before restarting.", true));
    }
    if state.active_connection.lock().await.is_some() {
        return Err(AppError::new(
            "connection_already_active",
            "Disconnect the active capture session first.",
            true,
        ));
    }
    if state.rollback_path.exists() {
        return Err(AppError::new(
            "pending_connection_rollback",
            "Recover the previous connection before starting another capture.",
            true,
        ));
    }

    let listen_port = match &target.kind {
        CaptureTargetKind::ProxyListener { listen_port, .. } if (1024..=65535).contains(listen_port) => *listen_port,
        CaptureTargetKind::ProxyListener { .. } => return Err(AppError::new("proxy_listener_port_invalid", "Choose a listener port from 1024 to 65535.", true)),
        _ => DEFAULT_CAPTURE_PORT,
    };
    let (mode, strategy, paired_ip, interface_ip) = match &target.kind {
        CaptureTargetKind::MacAll => (CaptureModeKind::LocalAll, "desktop_local_all", None, None),
        CaptureTargetKind::MacProcess { pid, name } => {
            let exists = list_desktop_processes()?
                .iter()
                .any(|process| process.pid == *pid && process.name == *name);
            if !exists {
                return Err(AppError::new(
                    "capture_process_missing",
                    "The selected process is no longer running. Refresh targets and choose it again.",
                    true,
                ));
            }
            (
                CaptureModeKind::LocalProcess { pid: *pid },
                "desktop_local_process",
                None,
                None,
            )
        }
        CaptureTargetKind::PhysicalIos { address, interface }
        | CaptureTargetKind::PhysicalAndroid { address, interface } => {
            let paired_ip = address.parse::<Ipv4Addr>().map_err(|_| {
                AppError::new(
                    "invalid_pair_address",
                    "Enter the physical device's IPv4 address.",
                    true,
                )
            })?;
            if !paired_ip.is_private() {
                return Err(AppError::new(
                    "invalid_pair_address",
                    "The paired device must have a private LAN IPv4 address.",
                    true,
                ));
            }
            let interface_ip = list_lan_interfaces()?
                .into_iter()
                .find(|item| &item.name == interface)
                .ok_or_else(|| {
                    AppError::new(
                        "invalid_lan_interface",
                        "Select an active private LAN interface.",
                        true,
                    )
                })?
                .address
                .parse::<Ipv4Addr>()
                .map_err(|_| {
                    AppError::new(
                        "invalid_lan_interface",
                        "The selected LAN interface has no IPv4 address.",
                        true,
                    )
                })?;
            if paired_ip == interface_ip {
                return Err(AppError::new(
                    "invalid_pair_address",
                    "The paired address must belong to the device, not this Mac.",
                    true,
                ));
            }
            (
                CaptureModeKind::RegularProxy,
                "physical_lan_proxy",
                Some(paired_ip),
                Some(interface_ip),
            )
        }
        CaptureTargetKind::ProxyListener { mode, .. } => {
            validate_proxy_listener_mode(mode)?;
            (mode.clone(), "manual_proxy_listener", None, None)
        }
        _ => unreachable!(),
    };
    let timestamp = now_epoch_millis()?;
    let session_id = format!("session-{timestamp}");
    let mode = CaptureMode {
        schema_version: SCHEMA_VERSION,
        kind: mode,
    };
    if let Ok(mut diagnostics) = state.tls_diagnostics.lock() {
        diagnostics.begin_session(&session_id);
    }
    let handle = state
        .capture_engine
        .start(CaptureConfig {
            session_id: session_id.clone(),
            listen_host: "127.0.0.1".into(),
            listen_port,
            mode: mode.clone(),
        })
        .await
        .map_err(capture_error_to_app_error)?;

    let connection_result: Result<ConnectDeviceResult, AppError> = async {
        let certificate = if matches!(&mode.kind, CaptureModeKind::DnsProxy) { None } else { Some(wait_for_certificate(&state.capture_engine.certificate_path()).await?) };
        let (sdk_guard, pairing_token) = match (interface_ip, paired_ip) {
            (Some(interface_ip), Some(paired_ip)) => {
                let token = new_pairing_token()?;
                let server = Arc::new(SdkIngestionServer::paired_lan(interface_ip, DEVICE_SDK_PORT, paired_ip, token.clone()));
                let listener = server.bind().await.map_err(|error| AppError::new(error.code, error.message, true))?;
                let task = sdk_commands::spawn_sdk_ingestion(state.sdk_database.clone(), server, Some(listener));
                (Some(LanGuard { host: interface_ip.to_string(), port: DEVICE_SDK_PORT, task: Arc::new(Mutex::new(Some(task))) }), Some(token))
            }
            _ => (None, None),
        };
        let lan_guard = match (interface_ip, paired_ip) {
            (Some(interface_ip), Some(paired_ip)) => match LanGuard::start(interface_ip, paired_ip, DEVICE_CAPTURE_PORT, DEFAULT_CAPTURE_PORT).await {
                Ok(guard) => Some(guard),
                Err(error) => {
                    if let Some(guard) = &sdk_guard { guard.stop().await; }
                    return Err(error);
                }
            },
            _ => None,
        };
        let session = CaptureSession {
            schema_version: SCHEMA_VERSION,
            id: session_id.clone(),
            name: sanitize_session_name(session_name.unwrap_or_else(|| format!("Capture {timestamp}"))),
            status: SessionStatus::Active,
            started_at: timestamp,
            ended_at: None,
            device_id: None,
            app_id: None,
            connection_strategy: Some(strategy.into()),
            capture_engine: Some("mitmdump".into()),
            notes: None,
            capture_target: Some(target.clone()),
            capture_mode: Some(mode),
        };
        if let Err(error) = state.database.create_session(&session) {
            if let Some(guard) = &lan_guard { guard.stop().await; }
            if let Some(guard) = &sdk_guard { guard.stop().await; }
            return Err(AppError::storage(error.to_string()));
        }
        let active = ActiveConnection {
            handle: handle.clone(),
            device_id: None,
            target: target.clone(),
            strategy: strategy.into(),
            previous_android_proxy: None,
            mac_proxy_lease: None,
            routing_enabled: false,
            lan_guard,
            sdk_guard,
        };
        let mut diagnostics = Vec::new();
        if active.lan_guard.is_some() {
            diagnostics.push(ConnectionDiagnostic {
                code: "physical_device_pairing".into(),
                title: "Set the device's manual proxy".into(),
                message: format!("Configure this development device to use {}:{}. Only the paired address can connect while this session is active.", active.lan_guard.as_ref().unwrap().host, active.lan_guard.as_ref().unwrap().port),
                recoverable: true,
                suggested_action: Some(format!("Install the capture CA from mitm.it while using this proxy, then enable full trust on iOS or development CA trust in the Android app. Configure SDK telemetry separately at http://{}:{DEVICE_SDK_PORT} with the session pairing token. Disable the device proxy when done; Mobile API Studio did not change its settings.", active.lan_guard.as_ref().unwrap().host)),
            });
        } else if matches!(&target.kind, CaptureTargetKind::ProxyListener { .. }) {
            diagnostics.push(ConnectionDiagnostic {
                code: "manual_proxy_listener".into(),
                title: "Manual listener is active".into(),
                message: format!("The selected listener mode is configured on 127.0.0.1:{listen_port}. Configure only the development client you control to use it."),
                recoverable: true,
                suggested_action: Some("Disconnect this session to stop the listener. DNS mode requires the client to send DNS queries to this port; SOCKS5 requires a SOCKS5 client setting.".into()),
            });
        } else {
            diagnostics.push(ConnectionDiagnostic {
                code: "desktop_local_capture_trust".into(),
                title: "Trust the development CA for HTTPS".into(),
                message: format!("Local capture is active. For HTTPS inspection, trust the certificate at {} for this development computer.", certificate.as_ref().expect("non-DNS capture has a certificate").display()),
                recoverable: true,
                suggested_action: Some("Native capture may require OS permission. Linux requires kernel 6.8+ and an authorized eBPF helper; WSL is unsupported. Certificate-pinned apps need their own debug configuration.".into()),
            });
        }
        let snapshot = connection_snapshot(&active);
        *state.active_connection.lock().await = Some(active);
        Ok(ConnectDeviceResult { connection: snapshot, diagnostics, pairing_token })
    }.await;
    if connection_result.is_err() {
        state
            .capture_engine
            .stop(handle)
            .await
            .map_err(capture_error_to_app_error)?;
    }
    connection_result
}

fn validate_proxy_listener_mode(mode: &CaptureModeKind) -> Result<(), AppError> {
    match mode {
        CaptureModeKind::ReverseProxy { url } | CaptureModeKind::UpstreamProxy { url } => {
            let parsed = Url::parse(url).map_err(|error| AppError::new("proxy_listener_url_invalid", error.to_string(), true))?;
            let reverse_h3 = matches!(mode, CaptureModeKind::ReverseProxy { .. }) && parsed.scheme() == "http3";
            if url.len() > 2048 || url.chars().any(char::is_control) || !(matches!(parsed.scheme(), "http" | "https") || reverse_h3) || parsed.host_str().is_none()
                || !parsed.username().is_empty() || parsed.password().is_some() || !matches!(parsed.path(), "" | "/") || parsed.query().is_some() || parsed.fragment().is_some() {
                return Err(AppError::new("proxy_listener_url_invalid", "Use a credential-free host URL without path, query, or fragment: HTTP(S), or http3:// for reverse capture.", true));
            }
            Ok(())
        }
        CaptureModeKind::Socks5 | CaptureModeKind::DnsProxy => Ok(()),
        _ => Err(AppError::new("proxy_listener_mode_invalid", "Choose reverse, upstream, SOCKS5, or DNS listener mode.", true)),
    }
}

async fn connect_device(
    device_id: String,
    session_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<ConnectDeviceResult, AppError> {
    connect_device_with_proxy(device_id, session_name, None, state).await
}

async fn connect_device_with_proxy(
    device_id: String,
    session_name: Option<String>,
    network_service: Option<String>,
    state: State<'_, AppState>,
) -> Result<ConnectDeviceResult, AppError> {
    let _operation = state.connection_operation.lock().await;
    if state.shutting_down.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::new("service_shutting_down", "The service is shutting down. Finish routing recovery before restarting.", true));
    }
    if state.active_connection.lock().await.is_some() {
        return Err(AppError::new(
            "connection_already_active",
            "Disconnect the active capture session before connecting another runtime.",
            true,
        ));
    }
    if state.rollback_path.exists() {
        return Err(AppError::new(
            "pending_connection_rollback",
            "A previous connection left device changes pending. Recover them before starting another capture.",
            true,
        ));
    }

    let timestamp = now_epoch_millis()?;
    let session_id = format!("session-{timestamp}");
    let is_android = device_id.starts_with("android:");
    let is_ios = device_id.starts_with("ios:");
    if is_ios && !cfg!(target_os = "macos") {
        return Err(AppError::new("unsupported_ios_platform", "iOS Simulator capture requires macOS and Xcode.", true));
    }
    if !is_android && !is_ios {
        return Err(AppError::new(
            "unsupported_device",
            "Only discovered iOS Simulators and Android Emulators can be connected.",
            true,
        ));
    }
    if !list_devices().devices.iter().any(|device| {
        device.id == device_id
            && (device.state.eq_ignore_ascii_case("booted") || device.state == "device")
    }) {
        return Err(AppError::new(
            "capture_device_unavailable",
            "The selected Simulator or Emulator is no longer available. Refresh devices and try again.",
            true,
        ));
    }

    let mac_proxy_lease = match network_service {
        Some(service) if is_ios => Some(mac_proxy::prepare(&service).await?),
        Some(_) => return Err(AppError::new("unsupported_proxy_target", "Automatic Mac routing is only available for an iOS Simulator.", true)),
        None => None,
    };
    // Android's 10.0.2.2 alias reaches the host loopback interface.
    let listen_host = "127.0.0.1";
    let strategy = if is_android {
        "android_adb_global_proxy"
    } else if mac_proxy_lease.is_some() {
        "ios_mac_network_proxy"
    } else {
        "ios_manual_proxy"
    };
    if let Ok(mut diagnostics) = state.tls_diagnostics.lock() {
        diagnostics.begin_session(&session_id);
    }
    let handle = state
        .capture_engine
        .start(CaptureConfig {
            session_id: session_id.clone(),
            listen_host: listen_host.into(),
            listen_port: DEFAULT_CAPTURE_PORT,
            mode: CaptureMode {
                schema_version: SCHEMA_VERSION,
                kind: CaptureModeKind::RegularProxy,
            },
        })
        .await
        .map_err(capture_error_to_app_error)?;

    let mut previous_android_proxy = None;
    let mut proxy_apply_attempted = false;
    let connection_result: Result<ConnectDeviceResult, AppError> = async {
        let session = CaptureSession {
            schema_version: SCHEMA_VERSION,
            id: session_id.clone(),
            name: sanitize_session_name(
                session_name.unwrap_or_else(|| format!("Capture {timestamp}")),
            ),
            status: SessionStatus::Active,
            started_at: timestamp,
            ended_at: None,
            device_id: Some(device_id.clone()),
            app_id: None,
            connection_strategy: Some(strategy.into()),
            capture_engine: Some("mitmdump".into()),
            notes: None,
            capture_target: Some(CaptureTarget {
                schema_version: SCHEMA_VERSION,
                kind: if is_android {
                    CaptureTargetKind::AndroidEmulator { device_id: device_id.clone() }
                } else {
                    CaptureTargetKind::IosSimulator { device_id: device_id.clone() }
                },
            }),
            capture_mode: Some(CaptureMode { schema_version: SCHEMA_VERSION, kind: CaptureModeKind::RegularProxy }),
        };
        state
            .database
            .create_session(&session)
            .map_err(|error| AppError::storage(error.to_string()))?;

        let mut diagnostics = Vec::new();
        if is_android {
            let provider = AndroidDeviceProvider;
            previous_android_proxy = provider
                .get_http_proxy(&device_id)
                .map_err(device_error_to_app_error)?;
            let journal = RollbackJournal {
                schema_version: SCHEMA_VERSION,
                device_id: device_id.clone(),
                platform: DevicePlatform::Android,
                session_id: session_id.clone(),
                previous_android_proxy: previous_android_proxy.clone(),
                ios_ca_installed: false,
                mac_proxy_lease: None,
            };
            save_rollback_journal(&state.rollback_path, &journal)?;
            provider
                .set_http_proxy(&device_id, ANDROID_HOST_ALIAS, DEFAULT_CAPTURE_PORT)
                .map_err(device_error_to_app_error)?;
            diagnostics.push(ConnectionDiagnostic {
                code: "android_ca_trust_guided".into(),
                title: "HTTPS trust may require app configuration".into(),
                message: "The emulator is routed through Mobile API Studio. HTTPS interception also requires the app to trust the mitmproxy CA.".into(),
                recoverable: true,
                suggested_action: Some(
                    "For development builds, trust user-added CAs with Android network security configuration, or install the CA manually from mitm.it. Certificate-pinned apps require an app-side debug path rather than proxy bypassing."
                        .into(),
                ),
            });
        } else {
            let certificate = wait_for_certificate(&state.capture_engine.certificate_path()).await?;
            let journal = RollbackJournal {
                schema_version: SCHEMA_VERSION,
                device_id: device_id.clone(),
                platform: DevicePlatform::Ios,
                session_id: session_id.clone(),
                previous_android_proxy: None,
                ios_ca_installed: true,
                mac_proxy_lease: mac_proxy_lease.clone(),
            };
            save_rollback_journal(&state.rollback_path, &journal)?;
            IosDeviceProvider
                .install_root_ca(&device_id, &certificate)
                .map_err(device_error_to_app_error)?;
            if let Some(lease) = &mac_proxy_lease {
                proxy_apply_attempted = true;
                mac_proxy::set_enabled(lease, true).await?;
                diagnostics.push(ConnectionDiagnostic {
                    code: "ios_automatic_setup_complete".into(),
                    title: "Simulator setup complete".into(),
                    message: "The capture CA is installed and the selected Mac network service routes HTTP/HTTPS through Mobile API Studio. Other proxy-aware Mac apps may also use this route.".into(),
                    recoverable: true,
                    suggested_action: Some("Disable routing to restore the previous network settings while keeping capture open. Certificate-pinned apps still need their own debug configuration.".into()),
                });
            } else {
                diagnostics.push(ConnectionDiagnostic {
                    code: "ios_proxy_manual_configuration".into(),
                    title: "Simulator certificate installed; routing is manual".into(),
                    message: "The root CA is installed. Route your app through 127.0.0.1:8181, or use the automatic Mac network routing option.".into(),
                    recoverable: true,
                    suggested_action: Some("If HTTPS reports a trust error, check Certificate Trust Settings in the Simulator. Pinned apps need an app-side debug configuration.".into()),
                });
            }
        }

        let active = ActiveConnection {
            handle: handle.clone(),
            device_id: Some(device_id.clone()),
            target: session.capture_target.clone().expect("new capture session has a target"),
            strategy: strategy.into(),
            previous_android_proxy: previous_android_proxy.clone(),
            mac_proxy_lease: mac_proxy_lease.clone(),
            routing_enabled: mac_proxy_lease.is_some(),
            lan_guard: None,
            sdk_guard: None,
        };
        let snapshot = connection_snapshot(&active);
        *state.active_connection.lock().await = Some(active);
        Ok(ConnectDeviceResult {
            connection: snapshot,
            diagnostics,
            pairing_token: None,
        })
    }
    .await;

    if let Err(original_error) = connection_result {
        if let Some(lease) = &mac_proxy_lease {
            if proxy_apply_attempted {
                let restored = if mac_proxy::can_release(lease).await.unwrap_or(false) { Ok(()) } else { mac_proxy::set_enabled(lease, false).await };
                if let Err(recovery) = restored {
                    // Keep a usable listener and connection: never strand routing on a closed proxy.
                    *state.active_connection.lock().await = Some(ActiveConnection {
                        handle: handle.clone(), device_id: Some(device_id.clone()),
                        target: CaptureTarget { schema_version: SCHEMA_VERSION, kind: CaptureTargetKind::IosSimulator { device_id: device_id.clone() } },
                        strategy: strategy.into(), previous_android_proxy: None,
                        mac_proxy_lease: mac_proxy_lease.clone(), routing_enabled: true,
                        lan_guard: None, sdk_guard: None,
                    });
                    return Err(AppError::new("connection_rollback_failed", format!("Setup failed: {}. Routing recovery failed: {}. Capture remains running; use Disable routing or Disconnect to retry.", original_error.message, recovery.message), true));
                }
            }
        }
        let rollback_error = if is_android && state.rollback_path.exists() {
            restore_android_proxy(&AndroidDeviceProvider, &device_id, previous_android_proxy.as_deref()).err()
        } else { None };
        if let Some(error) = &rollback_error {
            *state.active_connection.lock().await = Some(ActiveConnection {
                handle: handle.clone(), device_id: Some(device_id.clone()),
                target: CaptureTarget { schema_version: SCHEMA_VERSION, kind: CaptureTargetKind::AndroidEmulator { device_id: device_id.clone() } },
                strategy: strategy.into(), previous_android_proxy: previous_android_proxy.clone(),
                mac_proxy_lease: None, routing_enabled: false, lan_guard: None, sdk_guard: None,
            });
            return Err(AppError::new("connection_rollback_failed", format!("Setup failed: {}. Proxy recovery failed: {}. Capture remains running; disconnect to retry.", original_error.message, error.message), true));
        }
        let stop_error = state.capture_engine.stop(handle.clone()).await.err();
        if let Some(stop_error) = stop_error {
            *state.active_connection.lock().await = Some(ActiveConnection {
                handle: handle.clone(), device_id: Some(device_id.clone()),
                target: CaptureTarget { schema_version: SCHEMA_VERSION, kind: if is_android { CaptureTargetKind::AndroidEmulator { device_id: device_id.clone() } } else { CaptureTargetKind::IosSimulator { device_id: device_id.clone() } } },
                strategy: strategy.into(), previous_android_proxy: previous_android_proxy.clone(),
                mac_proxy_lease: mac_proxy_lease.clone(), routing_enabled: false,
                lan_guard: None, sdk_guard: None,
            });
            return Err(AppError::new(
                "capture_cleanup_failed",
                format!(
                    "Connection failed: {}. Capture cleanup also failed: {}",
                    original_error.message, stop_error.message
                ),
                true,
            ));
        }
        clear_rollback_journal(&state.rollback_path)?;
        state.database.complete_session(&session_id, &now_epoch_millis()?).map_err(|error| AppError::storage(error.to_string()))?;
        return Err(original_error);
    }
    connection_result
}

async fn set_ios_routing(enabled: bool, state: State<'_, AppState>) -> Result<ConnectionSnapshot, AppError> {
    let _operation = state.connection_operation.lock().await;
    let mut active = state.active_connection.lock().await.clone().ok_or_else(|| AppError::new("connection_not_active", "Start automatic Simulator capture first.", true))?;
    let lease = active.mac_proxy_lease.as_ref().ok_or_else(|| AppError::new("manual_proxy_connection", "This capture uses manual routing.", true))?;
    if enabled && !state.capture_engine.is_running(&active.handle).await {
        return Err(AppError::new("capture_not_running", "The capture listener stopped. Disconnect before starting another session.", true));
    }
    mac_proxy::set_enabled(lease, enabled).await?;
    active.routing_enabled = enabled;
    let snapshot = connection_snapshot(&active);
    *state.active_connection.lock().await = Some(active);
    Ok(snapshot)
}

async fn disconnect_device(state: State<'_, AppState>) -> Result<ConnectionSnapshot, AppError> {
    let _operation = state.connection_operation.lock().await;
    let active = state.active_connection.lock().await.clone();
    let Some(active) = active else {
        return Ok(disconnected_snapshot());
    };
    if let Some(lease) = &active.mac_proxy_lease {
        if let Err(error) = mac_proxy::set_enabled(lease, false).await {
            if !mac_proxy::can_release(lease).await.unwrap_or(false) { return Err(error); }
            // External settings no longer route to this listener; preserve them and disconnect safely.
        }
    }
    if let Some(device_id) = active
        .device_id
        .as_deref()
        .filter(|id| id.starts_with("android:"))
    {
        restore_android_proxy(
            &AndroidDeviceProvider,
            device_id,
            active.previous_android_proxy.as_deref(),
        )?;
    }
    if let Some(guard) = &active.lan_guard {
        guard.stop().await;
    }
    if let Some(guard) = &active.sdk_guard {
        guard.stop().await;
    }
    state
        .capture_engine
        .stop(active.handle.clone())
        .await
        .map_err(capture_error_to_app_error)?;
    let ended_at = now_epoch_millis()?;
    state
        .database
        .complete_session(&active.handle.session_id, &ended_at)
        .map_err(|error| AppError::storage(error.to_string()))?;
    clear_rollback_journal(&state.rollback_path)?;
    *state.active_connection.lock().await = None;
    Ok(disconnected_snapshot())
}

async fn pending_rollback(state: State<'_, AppState>) -> Result<Option<serde_json::Value>, AppError> {
    let _operation = state.connection_operation.lock().await;
    if state.active_connection.lock().await.is_some() {
        return Ok(None);
    }
    Ok(load_rollback_journal(&state.rollback_path)?.map(|journal| serde_json::json!({
        "schemaVersion": journal.schema_version, "deviceId": journal.device_id,
        "platform": journal.platform, "sessionId": journal.session_id,
        "previousAndroidProxy": journal.previous_android_proxy, "iosCaInstalled": journal.ios_ca_installed,
        "macNetworkService": journal.mac_proxy_lease.as_ref().map(|lease| &lease.service_id),
    })))
}

async fn recover_pending_rollback(
    state: State<'_, AppState>,
) -> Result<Vec<ConnectionDiagnostic>, AppError> {
    let _operation = state.connection_operation.lock().await;
    if state.active_connection.lock().await.is_some() {
        return Err(AppError::new(
            "rollback_capture_active",
            "Disconnect the active capture before recovering an interrupted connection.",
            true,
        ));
    }
    let Some(journal) = load_rollback_journal(&state.rollback_path)? else {
        return Ok(Vec::new());
    };
    let mut diagnostics = Vec::new();
    if let Some(lease) = &journal.mac_proxy_lease {
        if let Err(error) = mac_proxy::set_enabled(lease, false).await {
            if !mac_proxy::can_release(lease).await.unwrap_or(false) { return Err(error); }
        }
        diagnostics.push(ConnectionDiagnostic { code: "mac_proxy_restored".into(), title: "Mac routing recovery complete".into(), message: "Original proxy settings were restored, or safe external changes were preserved.".into(), recoverable: true, suggested_action: None });
    }

    match journal.platform {
        DevicePlatform::Android => {
            restore_android_proxy(
                &AndroidDeviceProvider,
                &journal.device_id,
                journal.previous_android_proxy.as_deref(),
            )?;
            diagnostics.push(ConnectionDiagnostic {
                code: "android_proxy_restored".into(),
                title: "Android proxy restored".into(),
                message: "The emulator proxy setting from the interrupted capture session was restored.".into(),
                recoverable: true,
                suggested_action: None,
            });
        }
        DevicePlatform::Ios => diagnostics.push(ConnectionDiagnostic {
            code: "ios_ca_left_installed".into(),
            title: "Simulator CA remains installed".into(),
            message: "Mobile API Studio does not reset the Simulator keychain automatically because that could delete unrelated developer credentials.".into(),
            recoverable: true,
            suggested_action: Some(
                "You may leave the locally generated CA installed, disable its full-trust toggle, or remove it manually if desired."
                    .into(),
            ),
        }),
    }
    clear_rollback_journal(&state.rollback_path)?;
    Ok(diagnostics)
}

fn connection_snapshot(active: &ActiveConnection) -> ConnectionSnapshot {
    ConnectionSnapshot {
        connected: true,
        capture_running: true,
        session_id: Some(active.handle.session_id.clone()),
        device_id: active.device_id.clone(),
        strategy: Some(active.strategy.clone()),
        proxy_host: active
            .lan_guard
            .as_ref()
            .map(|guard| guard.host.clone())
            .or_else(|| match &active.target.kind {
                CaptureTargetKind::AndroidEmulator { .. } => Some(ANDROID_HOST_ALIAS.into()),
                CaptureTargetKind::IosSimulator { .. } => Some("127.0.0.1".into()),
                CaptureTargetKind::ProxyListener { .. } => Some("127.0.0.1".into()),
                _ => None,
            }),
        proxy_port: match active.target.kind {
            CaptureTargetKind::PhysicalIos { .. } | CaptureTargetKind::PhysicalAndroid { .. } => {
                active.lan_guard.as_ref().map(|guard| guard.port)
            }
            CaptureTargetKind::IosSimulator { .. } | CaptureTargetKind::AndroidEmulator { .. } => {
                Some(active.handle.listen_port)
            }
            CaptureTargetKind::ProxyListener { .. } => Some(active.handle.listen_port),
            _ => None,
        },
        capture_target: Some(active.target.clone()),
        routing_enabled: active.mac_proxy_lease.as_ref().map(|_| active.routing_enabled),
        mac_network_service: active.mac_proxy_lease.as_ref().map(|lease| lease.service_id.clone()),
    }
}

fn disconnected_snapshot() -> ConnectionSnapshot {
    ConnectionSnapshot {
        connected: false,
        capture_running: false,
        session_id: None,
        device_id: None,
        strategy: None,
        proxy_host: None,
        proxy_port: None,
        capture_target: None,
        routing_enabled: None,
        mac_network_service: None,
    }
}

fn restore_android_proxy(
    provider: &AndroidDeviceProvider,
    device_id: &str,
    previous: Option<&str>,
) -> Result<(), AppError> {
    match previous {
        Some(proxy) => {
            let (host, port) = proxy.rsplit_once(':').ok_or_else(|| {
                AppError::new(
                    "android_proxy_restore_invalid",
                    format!("Unable to parse previous Android proxy value: {proxy}"),
                    true,
                )
            })?;
            let port = port.parse::<u16>().map_err(|error| {
                AppError::new("android_proxy_restore_invalid", error.to_string(), true)
            })?;
            provider
                .set_http_proxy(device_id, host, port)
                .map_err(device_error_to_app_error)
        }
        None => provider
            .clear_http_proxy(device_id)
            .map_err(device_error_to_app_error),
    }
}

async fn wait_for_certificate(path: &Path) -> Result<PathBuf, AppError> {
    for _ in 0..100 {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(AppError::new(
        "capture_ca_not_ready",
        format!("Capture CA was not created at {}", path.display()),
        true,
    ))
}

fn save_rollback_journal(path: &Path, journal: &RollbackJournal) -> Result<(), AppError> {
    let bytes = serde_json::to_vec_pretty(journal)
        .map_err(|error| AppError::new("rollback_serialize_failed", error.to_string(), true))?;
    atomic_file::write_private(path, &bytes)
        .map_err(|error| AppError::new("rollback_write_failed", format!("{error:?}"), true))
}

fn load_rollback_journal(path: &Path) -> Result<Option<RollbackJournal>, AppError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(path)
        .map_err(|error| AppError::new("rollback_read_failed", error.to_string(), true))?;
    let journal = serde_json::from_slice(&bytes)
        .map_err(|error| AppError::new("rollback_parse_failed", error.to_string(), true))?;
    Ok(Some(journal))
}

fn clear_rollback_journal(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| AppError::new("rollback_clear_failed", error.to_string(), true))?;
    }
    Ok(())
}

fn spawn_capture_ingestion(database: Database, body_store: BodyStore, engine: Arc<MitmDumpEngine>, diagnostics: Arc<StdMutex<VecDeque<ProxyRuleDiagnostic>>>, tls_diagnostics: Arc<StdMutex<RecentTlsDiagnostics>>) {
    let mut receiver = engine.subscribe();
    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    if let capture_core::CaptureEvent::TransportDiagnostic { session_id, code, .. } = &event {
                        if let Ok(mut recent) = tls_diagnostics.lock() {
                            recent.record(session_id, code);
                        }
                    }
                    if let capture_core::CaptureEvent::EngineFailed { code, .. } = &event {
                        if code.starts_with("proxy_rule") || code == "script_hook_failed" {
                            let safe_code = code.chars().filter(|character| character.is_ascii_alphanumeric() || *character == '_').take(80).collect::<String>();
                            if let Ok(mut queue) = diagnostics.lock() {
                                queue.push_back(ProxyRuleDiagnostic { code: safe_code, message: "Proxy rule execution failed; the affected flow was stopped.".into() });
                                if queue.len() > 20 { queue.pop_front(); }
                            }
                        }
                    }
                    let _ = inspect::ingest_capture_event(&database, &body_store, event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn initialize_state(app_data_dir: PathBuf, addon_path: PathBuf) -> Result<AppState, String> {
    fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
    let database =
        Database::open(app_data_dir.join("app.db")).map_err(|error| error.to_string())?;
    let sdk_database = SdkDatabase::open(database.path()).map_err(|error| error.to_string())?;
    let ai_database = AiDatabase::open(database.path()).map_err(|error| error.to_string())?;
    let body_store =
        BodyStore::new(app_data_dir.join("bodies")).map_err(|error| error.to_string())?;
    let capture_executable = sidecar_commands::configured_capture_executable(&database)?;
    #[cfg(unix)]
    let rule_socket_path = rule_server::socket_path()?;
    #[cfg(windows)]
    let rule_loopback_listener = rule_server::loopback_listener()?;
    let mut capture_engine = MitmDumpEngine::new(addon_path, app_data_dir.join("mitmproxy"))
        .with_executable(capture_executable);
    #[cfg(unix)]
    { capture_engine = capture_engine.with_rule_socket(&rule_socket_path); }
    #[cfg(windows)]
    { capture_engine = capture_engine.with_rule_loopback(rule_loopback_listener.port, rule_loopback_listener.token.clone()); }
    let capture_engine = Arc::new(capture_engine);
    Ok(AppState {
        database,
        sdk_database,
        ai_database,
        body_store,
        capture_engine,
        active_connection: Mutex::new(None),
        connection_operation: Mutex::new(()),
        shutting_down: std::sync::atomic::AtomicBool::new(false),
        rollback_path: app_data_dir.join("connection-rollback.json"),
        proxy_rule_diagnostics: Arc::new(StdMutex::new(VecDeque::new())),
        tls_diagnostics: Arc::new(StdMutex::new(RecentTlsDiagnostics::default())),
        #[cfg(unix)]
        rule_socket_path,
        #[cfg(windows)]
        rule_loopback_listener: StdMutex::new(Some(rule_loopback_listener)),
        rule_server_task: StdMutex::new(None),
    })
}

fn resolve_addon_path() -> Result<PathBuf, String> {
    let path = std::env::var_os("MAS_ADDON_PATH").map(PathBuf::from).unwrap_or_else(||
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/mitm-addon/mas_bridge.py"));
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("Capture addon not found at {}", path.display()))
    }
}

fn capture_error_to_app_error(error: capture_core::CaptureError) -> AppError {
    AppError::new(error.code, error.message, error.recoverable)
}
fn device_error_to_app_error(error: impl IntoDeviceError) -> AppError {
    let error = error.into_parts();
    AppError::new(error.0, error.1, error.2)
}
trait IntoDeviceError {
    fn into_parts(self) -> (String, String, bool);
}
impl IntoDeviceError for device_android::DeviceError {
    fn into_parts(self) -> (String, String, bool) {
        (self.code, self.message, self.recoverable)
    }
}
impl IntoDeviceError for device_ios::DeviceError {
    fn into_parts(self) -> (String, String, bool) {
        (self.code, self.message, self.recoverable)
    }
}

fn now_epoch_millis() -> Result<String, AppError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::initialization(error.to_string()))?;
    Ok(duration.as_millis().to_string())
}

fn sanitize_session_name(name: String) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        "Untitled Session".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

#[derive(Clone)]
pub struct CoreService {
    state: Arc<AppState>,
}

impl CoreService {
    pub fn start(app_data_dir: PathBuf) -> Result<Self, String> {
        let state = initialize_state(app_data_dir, resolve_addon_path()?)?;
        #[cfg(unix)]
        {
            let task = rule_server::start(state.database.clone(), &state.rule_socket_path)?;
            *state.rule_server_task.lock().map_err(|error| error.to_string())? = Some(task);
        }
        #[cfg(windows)]
        {
            let listener = state.rule_loopback_listener.lock().map_err(|_| "Rule transport unavailable")?.take().ok_or("Rule transport already started")?;
            let task = rule_server::start_loopback(state.database.clone(), listener)?;
            *state.rule_server_task.lock().map_err(|_| "Rule transport unavailable")? = Some(task);
        }
        spawn_capture_ingestion(
            state.database.clone(),
            state.body_store.clone(),
            state.capture_engine.clone(),
            state.proxy_rule_diagnostics.clone(),
            state.tls_diagnostics.clone(),
        );
        let sdk_server = Arc::new(SdkIngestionServer::localhost(SDK_INGESTION_PORT));
        sdk_commands::spawn_sdk_ingestion(state.sdk_database.clone(), sdk_server, None);
        Ok(Self {
            state: Arc::new(state),
        })
    }

    pub async fn invoke(
        &self,
        command: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        if matches!(command, "setup_ios_simulator" | "set_ios_routing" | "connect_device" | "connect_capture_target" | "disconnect_device" | "recover_pending_rollback") {
            // A browser reload may cancel its request while native authorization is open.
            // Keep the entire transaction, lock and cleanup alive until it completes.
            let state = self.state.clone();
            let command = command.to_owned();
            return tokio::spawn(async move { dispatch::invoke(&command, args, &state).await }).await
                .map_err(|error| AppError::new("connection_operation_failed", error.to_string(), true))?;
        }
        dispatch::invoke(command, args, &self.state).await
    }

    pub async fn shutdown(&self) -> Result<(), AppError> {
        self.state.shutting_down.store(true, std::sync::atomic::Ordering::SeqCst);
        let service = self.clone();
        tokio::spawn(async move { service.finish_shutdown().await }).await
            .map_err(|error| AppError::new("connection_shutdown_failed", error.to_string(), true))?
    }

    async fn finish_shutdown(&self) -> Result<(), AppError> {
        disconnect_device(State(&self.state)).await?;
        if let Ok(mut guard) = self.state.rule_server_task.lock() {
            if let Some(task) = guard.take() { task.abort(); }
        }
        #[cfg(unix)]
        {
            let _ = fs::remove_file(&self.state.rule_socket_path);
            if let Some(directory) = self.state.rule_socket_path.parent() { let _ = fs::remove_dir(directory); }
        }
        Ok(())
    }
}

mod dispatch;
mod mac_proxy;

#[cfg(test)]
mod certificate_wait_tests {
    use super::*;

    #[test]
    fn accepts_ca_created_after_cold_start_delay() {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
            .block_on(async {
                let suffix = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let directory = std::env::temp_dir()
                    .join(format!("mas-ca-wait-{}-{suffix}", std::process::id()));
                fs::create_dir_all(&directory).unwrap();
                let certificate = directory.join("mitmproxy-ca-cert.pem");
                let delayed_certificate = certificate.clone();
                let writer = tokio::spawn(async move {
                    sleep(Duration::from_millis(2_500)).await;
                    fs::write(delayed_certificate, b"test ca").unwrap();
                });

                let found = wait_for_certificate(&certificate).await.unwrap();
                writer.await.unwrap();
                fs::remove_dir_all(directory).unwrap();
                assert_eq!(found, certificate);
            });
    }
}

#[cfg(test)]
mod lan_guard_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn paired_guard_forwards_and_disconnect_closes_listener() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let upstream = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
            let upstream_port = upstream.local_addr().unwrap().port();
            let server = tokio::spawn(async move {
                let (mut stream, _) = upstream.accept().await.unwrap();
                let mut request = [0_u8; 4];
                stream.read_exact(&mut request).await.unwrap();
                assert_eq!(&request, b"ping");
                stream.write_all(b"pong").await.unwrap();
            });
            let guard = LanGuard::start(Ipv4Addr::LOCALHOST, Ipv4Addr::LOCALHOST, 0, upstream_port).await.unwrap();
            let mut client = TcpStream::connect((Ipv4Addr::LOCALHOST, guard.port)).await.unwrap();
            client.write_all(b"ping").await.unwrap();
            let mut response = [0_u8; 4];
            client.read_exact(&mut response).await.unwrap();
            assert_eq!(&response, b"pong");
            server.await.unwrap();
            guard.stop().await;
            assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, guard.port)).await.is_err());

            let upstream = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
            let guard = LanGuard::start(Ipv4Addr::LOCALHOST, Ipv4Addr::new(127, 0, 0, 2), 0, upstream.local_addr().unwrap().port()).await.unwrap();
            let _unpaired = TcpStream::connect((Ipv4Addr::LOCALHOST, guard.port)).await.unwrap();
            assert!(tokio::time::timeout(Duration::from_millis(100), upstream.accept()).await.is_err());
            guard.stop().await;
        });
    }
}

#[cfg(test)]
mod ios_setup_journal_checks {
    use super::*;
    #[test]
    fn legacy_journal_loads_and_browser_recovery_omits_private_backup() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let legacy: RollbackJournal = serde_json::from_value(serde_json::json!({"schemaVersion": SCHEMA_VERSION, "deviceId": "ios:test", "platform": "ios", "sessionId": "fixture", "previousAndroidProxy": null, "iosCaInstalled": true})).unwrap();
            assert!(legacy.mac_proxy_lease.is_none());
            let dir = std::env::temp_dir().join(format!("mas-ios-journal-{}-{}", std::process::id(), now_epoch_millis().unwrap()));
            let state = initialize_state(dir.clone(), resolve_addon_path().unwrap()).unwrap();
            let mut journal = legacy;
            journal.mac_proxy_lease = Some(mac_proxy::Lease { service_id: "fixture-service".into(), location_id: "fixture-location".into(), original: "PRIVATE-OPAQUE-SNAPSHOT".into(), had_configuration: false });
            save_rollback_journal(&state.rollback_path, &journal).unwrap();
            assert_eq!(load_rollback_journal(&state.rollback_path).unwrap().unwrap().mac_proxy_lease.unwrap().original, "PRIVATE-OPAQUE-SNAPSHOT");
            let view = pending_rollback(State(&state)).await.unwrap().unwrap();
            assert_eq!(view["macNetworkService"], "fixture-service");
            assert!(!view.to_string().contains("PRIVATE-OPAQUE-SNAPSHOT"));
            assert!(view.get("macProxyLease").is_none());
            state.shutting_down.store(true, std::sync::atomic::Ordering::SeqCst);
            let blocked = connect_device("ios:fixture".into(), None, State(&state)).await.unwrap_err();
            assert_eq!(blocked.code, "service_shutting_down");
            drop(state); fs::remove_dir_all(dir).unwrap();
        });
    }
    #[test]
    fn cancelled_browser_request_still_finishes_routing_recovery() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let dir = std::env::temp_dir().join(format!("mas-ios-cancellation-{}-{}", std::process::id(), now_epoch_millis().unwrap()));
            let core = CoreService { state: Arc::new(initialize_state(dir.clone(), resolve_addon_path().unwrap()).unwrap()) };
            let journal: RollbackJournal = serde_json::from_value(serde_json::json!({"schemaVersion": SCHEMA_VERSION, "deviceId": "ios:test", "platform": "ios", "sessionId": "fixture", "previousAndroidProxy": null, "iosCaInstalled": true})).unwrap();
            save_rollback_journal(&core.state.rollback_path, &journal).unwrap();
            let held = core.state.connection_operation.lock().await;
            let caller_core = core.clone();
            let caller = tokio::spawn(async move { caller_core.invoke("recover_pending_rollback", serde_json::json!({})).await });
            tokio::task::yield_now().await;
            caller.abort();
            assert!(caller.await.unwrap_err().is_cancelled());
            drop(held);
            tokio::time::timeout(Duration::from_secs(2), async {
                while core.state.rollback_path.exists() { tokio::task::yield_now().await; }
            }).await.expect("Recovery must survive cancellation of its browser caller");
            drop(core); fs::remove_dir_all(dir).unwrap();
        });
    }

}

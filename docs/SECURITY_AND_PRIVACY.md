# Security and Privacy

> **Status:** this document reflects security/privacy boundaries implemented through v0.5 and the eight expansion milestones. Formal security testing and release review are still part of the deferred owner-led validation/release stage.

Mobile API Studio can handle authentication headers, cookies, test/customer payloads, local proxy configuration, a development CA, SDK metadata, and optional external AI requests. These boundaries are therefore part of product architecture, not optional cleanup.

## 1. Local-first default

By default, these remain on the developer machine:

- capture sessions and flow metadata;
- request/response bodies;
- local CA material;
- saved requests and environments;
- mock rules/fixtures;
- SDK client/context/log/network events;
- deterministic comparison results;
- AI result history.

Data leaves the machine only through an explicit user action such as normal request Replay to a target server, workspace export, an optional AI send after preview, or an explicitly reviewed sharing upload/team publication.

## 2. Capture CA policy

The current capture path uses mitmproxy CA material inside Mobile API Studio's local capture data directory.

Required policy:

- private CA material stays local;
- it is never included in normal export/diagnostic flows;
- the UI explains why HTTPS interception needs local trust;
- CA installation is for developer-controlled test runtimes;
- the product does not implement silent certificate-pinning bypass.

One-click iOS Simulator setup installs the development root certificate using `simctl keychain add-root-cert` in the selected booted Simulator. It does not install a trusted CA in the Mac system keychain. If a particular runtime reports a trust failure, check Simulator Certificate Trust Settings; this is troubleshooting rather than a mandatory step for every setup.

## 3. Proxy/device mutation and rollback

Before changing a supported mutable proxy setting, Mobile API Studio records enough prior state to restore it.

Current behavior:

- Android Emulator proxy state is read before mutation and restored on disconnect/recovery.
- A rollback journal survives abnormal shutdown for supported mutations. On macOS it is written atomically with private permissions before changing network settings; the saved proxy dictionary never enters browser responses or workspace exports.
- iOS Simulator automatic routing is an explicit Connect option. It sets HTTP/HTTPS proxies on the selected current-route Mac network service, so other proxy-aware Mac apps can be affected. It preserves the original configuration and Network Location; active proxies, PAC, autodiscovery and proxy authentication block setup.
- Disable routing and disconnect restore the saved settings. macOS authorization may be required. External changes are preserved; recovery refuses to overwrite an unexpected configuration or a different Network Location.
- A failed restoration keeps the capture listener and local UI running. Successful restoration requires readable effective routing that no longer points at the capture proxy before listener shutdown. After an abrupt kill, use the pending recovery action when restarting. The manual app-scoped alternative remains available.

Never assume the pre-existing state was “no proxy.”

## 4. Secret headers

Sensitive values are identified case-insensitively. Core/default examples include:

```text
Authorization
Proxy-Authorization
Cookie
Set-Cookie
X-API-Key
X-Auth-Token
```

Additional sensitive names can be treated as secrets by higher-level redaction settings.

Default-safe behaviors include:

- redacted cURL export;
- secret-aware comparison output;
- workspace export without credential-store values;
- AI context sanitization before any provider call.

## 5. Internal SDK correlation metadata

The optional app-aware SDK uses the development-only header:

```text
X-Mobile-API-Studio-Request-Id
```

The proxy records this value for local correlation and removes the header before forwarding the request to the real backend.

The same internal header is omitted from:

- normal cURL export;
- Replay requests;
- AI context.

It is product-local debugging metadata, not part of the application's API contract.

## 6. Body handling

Bodies can contain credentials or sensitive test/customer data.

Implemented/design rules:

- bodies are stored locally in a content-addressed store;
- large/binary content is loaded on demand rather than continuously pushed to the webview;
- truncation/binary/content-type metadata is retained;
- debugger/application logs should not contain raw captured bodies by default;
- exported/AI contexts are bounded rather than unbounded body dumps.

Future retention controls such as per-host “do not store body” can be added independently of the body-store architecture.

## 7. Environment and provider secrets

Secret values use the OS credential-store abstraction rather than ordinary SQLite values.

Secure-store usage (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux) includes:

- secret environment variables;
- OpenAI/BYOK API key.

Workspace export includes metadata indicating a secret must be re-entered but does not export the credential-store secret value/reference as usable credentials.

## 8. App-aware SDK boundary

SDKs are optional and disabled/pass-through unless explicitly enabled by the developer.

### iOS

SDK telemetry uses an ephemeral URLSession configured to avoid recursive app instrumentation/proxy routing.

### Android

SDK telemetry uses a local direct transport rather than the application's intercepted OkHttp path.

The SDK does not create a remote cloud telemetry dependency. Desktop ingestion is local to the development host.

## 9. AI boundary

AI analysis is explicitly opt-in and sits after deterministic local analysis.

Current flow:

```text
selected comparison/flow
 -> deterministic local evidence
 -> sensitive-header redaction
 -> internal-header omission
 -> configurable JSON/query secret-key redaction
 -> form-encoded body secret-key redaction
 -> body/string/context limits
 -> exact context preview
 -> SHA-256 context fingerprint
 -> explicit user send action
 -> fingerprint re-check
 -> AI provider
```

The current OpenAI provider uses the Responses API with `store: false`.

The user's provider key is loaded from the OS secure store and is not persisted in SQLite or workspace exports.

AI is never required for capture, Replay, mocks, SDK context, or deterministic comparison.

## 10. Context-preview guarantee

The external-send command recomputes the sanitized context and checks that its SHA-256 fingerprint matches the preview fingerprint supplied by the UI.

If the evidence changes after preview, the external request is rejected and the user must preview again.

This prevents a stale preview from authorizing materially different context.

## 11. Certificate pinning policy

Mobile API Studio does **not** implement pinning bypass as a product feature.

When a client rejects the interception CA:

- diagnose likely trust/pinning behavior;
- recommend debug networking configuration for applications the developer controls;
- use the optional app-aware SDK for additional context where appropriate;
- do not inject generic bypass hooks into third-party or production apps.

## 12. Listener exposure

### Browser UI and control API

The source-built service binds to `127.0.0.1:8180` by default. `--port` changes only this listener. API commands use a process-lifetime random token in a request header, not a URL. Host and Origin checks reject unexpected web origins, and responses carry restrictive browser headers. Browser import/export uses local file selection and downloads; the existing bundle redaction rules still apply. Other processes running as the same host user remain within the local trust boundary.

### Capture

Android Emulator host routing can require the capture proxy to be reachable from the emulator through its host alias. This is a broader binding than loopback and must be treated as a development-host exposure.

### SDK telemetry

The desktop SDK ingestion listener binds to host loopback. Android Emulator reaches that loopback through `10.0.2.2`; it is not intended as a LAN telemetry service. The listener accepts the documented loopback Host values and rejects requests carrying an Origin header, so a web page cannot submit SDK events through a cross-origin browser request. Native SDK clients do not set Origin.

Control/SDK interfaces should not become unauthenticated general LAN APIs.

## 13. Import/export

Workspace bundles are versioned and designed to remain portable without carrying credential-store secrets.

Security requirements:

- exclude provider/environment secret values;
- redact sensitive headers;
- do not include CA private key material;
- keep import paths/data scoped to the application store;
- refuse destructive replacement while incompatible live state such as an active capture would make it unsafe.

## 14. Local AI history

AI output is stored locally for developer convenience together with metadata such as:

- task/target;
- provider;
- model;
- sanitized-context fingerprint;
- timestamp/result.

The provider API key is not stored with that history.

## 15. Diagnostic bundles and logs

Any future diagnostic bundle should be designed around metadata rather than raw traffic.

May include:

- app version;
- host OS/runtime versions;
- capture-engine version;
- sanitized device metadata;
- typed error codes;
- redacted configuration;
- application/sidecar logs.

Must exclude by default:

- CA private key;
- Authorization/Cookie/API-key values;
- raw captured bodies;
- environment/provider secrets.

## 16. Validation/release threat-model checklist

The following are not claimed as formally tested yet and should be explicitly reviewed during final validation/public-release work:

- local CA private-key file permissions;
- wider capture-port exposure on the development host;
- malicious HTML/JSON rendering behavior in the desktop webview;
- untrusted import bundle handling/path traversal;
- body-reference file access boundaries;
- malformed/unbounded sidecar events;
- breakpoint auto-continue/recovery paths;
- proxy rollback after abnormal termination;
- trusted-CA cleanup/uninstall guidance;
- SDK accidental release enablement;
- AI redaction across representative real payloads;
- Keychain secret deletion/update behavior.

See [FINAL_VALIDATION.md](FINAL_VALIDATION.md) for the owner-led validation stage.

## 17. Expanded capture, scripting and sharing

Desktop/process capture requires explicit scope and platform permissions; paired LAN capture binds only the selected private interface and device address. Pairing secrets and CA private material are excluded from ordinary exports. The browser command API remains loopback-only. Physical device trust/proxy changes are manual; Mac process setup and physical acceptance were excluded from this run.

Script hooks run in disposable bounded QuickJS processes without host APIs, not an OS sandbox. Failures stop the affected flow. Imported scripts remain disabled. General exports omit script source; explicit script exports require review because source can contain user-entered secrets. Local CLI/MCP enforces peer identity and an allowlist and does not expose arbitrary commands, AI calls or credential-store access.

Sharing is opt-in: sign-in reads identity only, and uploads transmit the exact reviewed selected HAR bytes with bounded expiry/revocation. Query/body inclusion is explicit. Known secrets and internal headers are filtered, but arbitrary secrets in free-form bodies cannot be detected reliably. Active share links are bearer secrets. Revocation clears accessible artifact bytes; SQLite/backups may retain prior pages, so it is not secure erasure. Remote hosting needs owner-provided TLS and token-safe logging. Manual team publish uses revision compare-and-swap; imports create disabled rules with fresh IDs and exclude scripts, local-file actions and credential/CA material. See the [sharing service contract](../apps/sharing-server/README.md).

Later focused security/runtime checks are in [PLATFORM_ACCEPTANCE.md](PLATFORM_ACCEPTANCE.md). The historical checklist above remains a broader release-review checklist; bounded passes do not constitute a formal security audit or platform certification.

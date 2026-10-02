# Roadmap checkpoint — 2026-10-02

The full eight-milestone roadmap remains authorized. Separate milestone worktrees, stacked draft PRs, disposable data and loopback ports preserve the original macOS release checkout and its validation record. No host-wide proxy settings were changed. The owner explicitly authorized continuing to the five-hour usage limit of 100% for this run, overriding the normal 80% reserve.

| Milestone | Reviewable source | Verification record |
| --- | --- | --- |
| 1 Capture targets | Draft #19 | CAPTURE_TARGETS.md |
| 2 Proxy rules | Draft #20 | PROXY_RULES.md |
| 3 Protocol inspection | Draft #21 | PROTOCOL_INSPECTION.md |
| 4 Network conditions | Draft #22 | NETWORK_CONDITIONS.md |
| 5 Compose/interchange | Draft #23 | COMPOSE_INTERCHANGE.md |
| 6 Scripting/automation | Draft #24 | SCRIPTING_AUTOMATION.md and SCRIPTING_CHECKPOINT.md |
| 7 Hosted sharing/team workspace | Draft #25 | SHARING_WORKSPACE.md |
| 8 Platform support | Draft #26 | PLATFORM_SUPPORT.md |

Milestones 6–8 are integrated in the platform branch. Final macOS integration passed 24 app-core checks, 10 local-service checks, worker limit checks, frontend production build, Windows CLI mocked DLL/peer-auth checks, actual relocated bundle assets and CLI discovery, workspace exclusivity, graceful shutdown and idle forced-stop restart. The final portable bundle passed real request/response/WebSocket script hooks and the measured network-profile regression (baseline 0.093s; 64 KiB/s up/down plus 200ms: 1.220s; disable-all released a waiting request in 0.350s). Each command uses disposable data and synthetic loopback traffic. Native Charles/Proxyman imports remain conditional on documented versions and representative fixtures; unsupported formats recommend HAR.

Sharing is self-hosted and opt-in; no external deployment or real traffic upload occurred. Its HTTP/security checks and combined native preview/upload/TTL/revocation/role/CAS/atomic pull regression passed before the milestone-7 checkpoint and again against the final milestone-8 package.

The subsequent owner-authorized acceptance run passed Linux ARM64 build/package/runtime checks in Docker, unlocked/unavailable Secret Service cases, and the complete Windows GNU cross-build. Computer use passed bounded Mac UI workflows and showed the completed iOS acceptance app. Simulator HTTPS and explicit reverse HTTP passed with manual SDK enrichment, internal correlation header removal, disabled SDK pass-through and disconnect. Automatic URLProtocol instrumentation also passed through the explicit reverse HTTP endpoint. An observed missing cancellation event was fixed in `stopLoading`; the automatic success/HTTP-500/cancel/refusal/runtime-disable matrix passed with exactly one terminal event per enabled request. The Android SDK/sample debug build passed after an observed one-line overload ambiguity fix. Rust 1.85 failed the locked dependency graph; Rust 1.88 passed and is now the declared minimum. See [PLATFORM_ACCEPTANCE.md](PLATFORM_ACCEPTANCE.md) for exact checks and limits.

Remaining acceptance gates:

1. Physical iOS/Android and Android HTTPS app-trust acceptance. No connected physical Apple/ADB device was found. Local Android API 37 ARM emulator HTTP/SDK checks, denied-permission silence, exact nonempty proxy restoration and interrupted-session journal recovery passed after the observed sample permission fix. The official image passed complete size/checksum validation. Cloud Linux x86-64 userspace passed; its subsequent Android attempts lost terminal connectivity before device setup, so no Cloud Android runtime result is claimed.
2. Native Windows runtime/credential storage and Linux host eBPF capture. Cross-compilation and Linux userspace Docker checks do not establish those results.
3. Selected-process Mac capture: the owned synthetic client timed out during listener startup; a bounded direct mitmdump probe reported that macos-redirector exited. The Mitmproxy Network Extension is waiting for user approval; the owner declined further Mac setup, and it was not enabled.
4. Default Simulator routing and automatic URLProtocol integration through the regular proxy. Explicit per-app HTTP routing to a reserved non-loopback hostname now passed with a fixture-scoped ATS exception and manual SDK enrichment. Loopback HTTP bypassed the proxy variants and a LAN fixture timed out. Manual HTTPS and automatic reverse HTTP passed; custom-session limitations remain documented.
5. The remaining full device/recovery/protocol/SDK/AI and UI acceptance matrix. Bounded local checks close only the cases recorded, not all owner-led validation.
6. External sharing TLS/deployment and signed platform release acceptance, if the owner elects to deploy or distribute builds.

Implementation Phases 0–5 remain historically implementation-complete without claiming they were formally tested. Source changes and partial local checks do not close the roadmap's release gates. Use latest verified Luna/Sol subagents, bounded validated Jev choices, and relevant deterministic checks. Preserve this checkpoint and the worktrees if the allowance is exhausted.

The final synthetic reverse-proxy crash check exposed a surviving mitmproxy listener after service SIGKILL. The capture engine now passes its owner PID to the addon, which monitors Unix reparenting or a retained Windows process handle and shuts down when the owner exits. Windows creation-time checks reject recycled startup PIDs and allow intermediate console launchers. After the fix, actual macOS SIGKILL closed the owned listener; restart, reconnect, synthetic traffic and disconnect passed with exact-child cleanup. `scripts/check-capture-parent-watchdog.py --bundle /path/to/bundle` records this regression; `scripts/check-parent-watchdog-windows.py` checks identity/lifetime branches with mocked APIs. Windows runtime, active-device rollback and the full recovery matrix remain unverified.

Final native contract regressions also passed: exact AI form-secret preview redaction (1), same-origin and secret-bearing cross-origin replay redirect behavior (2), and authenticated paired LAN SDK ingestion (1). Workspace-core compiled with no unit cases; no workspace test count is claimed. These checks supplement the verified source and do not replace owner-led device/UI/platform acceptance.

Native iOS WebSocket acceptance subsequently exposed a shared addon/parser event-tag mismatch. Explicit message/close tags now match the emitted wire names. The focused regression failed before the fix; all three capture-mitm checks and the actual Simulator rerun passed afterward. Eight persisted text/binary/empty messages, payloads, direction/sequence, close metadata, SDK enrichment, upstream correlation removal and disconnect passed. Evidence is retained with the other iOS acceptance artifacts.

Subsequent local Android runtime/recovery acceptance and retained native iOS WebSocket history checks passed. Android API 37 local-network permission handling and shared SDK session attribution were fixed from observed failures. Saved WebSocket payloads/close metadata/SDK enrichment survived native restart; export/merge import and imported restart passed. Credential-free network-none Linux rejected an AI preview invalidated by a redaction-policy change. See PLATFORM_ACCEPTANCE.md for limits and artifacts.

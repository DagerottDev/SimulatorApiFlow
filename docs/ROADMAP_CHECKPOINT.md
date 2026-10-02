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

Remaining acceptance gates:

1. Physical iOS/Android HTTP(S), selected-process capture and existing Simulator/Emulator workflows. The available Apple development entries were simulated and shut down; no connected physical device, booted Simulator or ADB environment was found.
2. Native Windows and Linux runtime, credential-store sessions, Android integration, packaging and the full capture/replay/rule/import/export/recovery matrix. Isolated target checks passed for relevant platform modules; full builds require missing C toolchains/SDKs and the Linux D-Bus sysroot. Docker is not running. No platform release is claimed.
3. Browser interaction and UI acceptance. Production builds and native/API checks passed; browser automation was blocked by tool security policy.
4. Active-capture interrupted-start/forced-stop recovery and the remaining owner-led protocol, SDK-correlation and AI-redaction matrix. Idle service restart is verified and does not close active-device rollback gates.
5. External hosting/TLS and release acceptance, if the owner elects to deploy sharing or distribute a platform build.

Implementation Phases 0–5 remain historically implementation-complete without claiming they were formally tested. Source changes and partial local checks do not close the roadmap's release gates. Use latest verified Luna/Sol subagents, bounded validated Jev choices, and relevant deterministic checks. Preserve this checkpoint and the worktrees if the allowance is exhausted.

The final synthetic reverse-proxy crash check exposed a surviving mitmproxy listener after service SIGKILL. The capture engine now passes its owner PID to the addon, which monitors Unix reparenting or a retained Windows process handle and shuts down when the owner exits. Windows creation-time checks reject recycled startup PIDs and allow intermediate console launchers. After the fix, actual macOS SIGKILL closed the owned listener; restart, reconnect, synthetic traffic and disconnect passed with exact-child cleanup. `scripts/check-capture-parent-watchdog.py --bundle /path/to/bundle` records this regression; `scripts/check-parent-watchdog-windows.py` checks identity/lifetime branches with mocked APIs. Windows runtime, active-device rollback and the full recovery matrix remain unverified.

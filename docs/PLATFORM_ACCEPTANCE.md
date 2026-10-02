# Platform acceptance — 2026-10-02

These checks were authorized after the eight implementation milestones. They do not change the historical record that Phases 0–5 were implemented without automated tests or CI gates. All requests and credential values below were synthetic; all data directories were disposable. No host-wide proxy/PAC settings, host CA trust, pinning, credential providers, or external sharing deployment were changed.

Desktop checks used baseline checkout `fd1ffa6`; the Android build additionally includes the interceptor completion fix described below; the native Mac bundle was built from `b98610b` (the later commit only records validation). Mac testing used macOS 27.0/26A428 on ARM64, Rust 1.98.1, Xcode 27.0 and mitmproxy 12.2.3. The final manifest also passed a locked offline native package check.

## Linux ARM64

Docker Desktop 29.7.2 ran Debian Bookworm/glibc 2.36 on Linux 7.0.12-linuxkit/aarch64. Build dependencies came from official Rust/Node images and standard registries. Runtime containers used `--network none` and loopback fixtures.

- Node 22, pnpm 10.15.0 frozen install and production UI build passed.
- Rust 1.90 release builds passed for the local service, QuickJS worker and sharing server; portable packaging passed.
- Release checks passed: 23 app-core cases and 10 local-service cases. Selected secret-store/sharing crates contained no unit cases; no extra count is claimed.
- Relocated assets, CLI authentication/discovery, workspace locks, graceful/forced restart, active-capture SIGKILL cleanup/reconnect, proxy/TLS rules, script hooks, network conditions, HAR/redaction/import/export and sharing role/TTL/revocation/CAS checks passed through the existing runnable scripts.
- Secret Service without a session bus reported unavailable and rejected writes without plaintext fallback. A disposable unlocked GNOME Keyring session passed synthetic set/get/delete.
- The locked dependency graph rejected Rust 1.85.1. `cargo +1.88.0 check --locked` passed for app-core, secret-store, service, worker and sharing server; the workspace minimum is now 1.88.

This establishes Linux ARM64 userspace behavior in Docker. Linux host local-process/eBPF/helper permissions, Android and a platform release remain unverified. `ps` and `ip` are discovery prerequisites; minimal containers need iproute2.

## Windows build

An official Rust 1.90 Bookworm container plus Debian MinGW-w64 completed:

```sh
cargo build --locked --release --target x86_64-pc-windows-gnu \
  -p mobile-api-studio-server \
  -p mobile-api-studio-script-worker \
  -p mobile-api-studio-sharing-server
```

All three outputs were verified as PE32+ x86-64 executables with standard Windows system DLL imports. They were not launched on Windows. This does not validate named pipes, Credential Manager, Windows capture/UI, packaging or release. The sharing server still intentionally fails closed on non-Unix hosts; Windows clients can use a verified HTTPS deployment.

## iOS Simulator

A disposable SwiftUI app using the repository's real Swift SDK built with Xcode 27.0 and ran on the existing iPhone 17 Pro Max/iOS 26.5 Simulator. Computer use through Device Hub showed its completed acceptance screen.

- SDK-disabled HTTPS requests retained their headers/configuration and produced no SDK correlation.
- Manually instrumented HTTPS through the regular capture proxy returned 200; captured flows joined the SDK client/context and the correlation header was removed before the synthetic TLS origin.
- Explicit reverse HTTP through URLSession returned 200, captured the app requests and joined manual SDK events with the internal header removed.
- Automatic URLProtocol instrumentation also passed through the explicit reverse HTTP endpoint: no manual instrument/complete calls, one SDK-enriched flow, no forwarded internal header, and a 200 response.
- A subsequent automatic lifecycle matrix passed disabled-before/after requests, successful and HTTP 500 responses, cancellation and connection refusal. Each enabled request produced exactly one started and terminal SDK event; terminal duration/status/error were correct, SDK telemetry was absent from captured app flows, and all forwarded fixture requests lacked the internal correlation header.
- This matrix exposed a cancellation defect: URLProtocol's weak completion callback could lose the instance after `stopLoading`, leaving the SDK request open. `stopLoading` now completes the request with cancellation before cancelling the underlying task. The shared SDK completion removes the in-flight record once, so a later callback produces no duplicate terminal event. The same Simulator check failed before the fix and passed after it.
- Computer use of the persisted iOS Traffic Inspector showed app/device, screen, feature, Swift source/function and nearby context; the fixture response confirmed the internal header was absent upstream.
- All cases verified disconnect and cleared the pending rollback journal.

The synthetic origin used verified upstream TLS with a temporary fixture CA. The capture target installed the isolated development CA into this Simulator; no Mac host CA trust was installed, and no whole-Simulator keychain reset was used. The Simulator development CA remains part of this test setup.

Regular-proxy HTTP also passed with manual SDK instrumentation and explicit per-session HTTP proxy configuration. The app requested `http://mas-acceptance.test`; a proxy-only map routed that reserved hostname to the synthetic loopback origin, without host DNS changes. A plaintext exception was limited to that hostname in the disposable unsigned app's ATS policy. Both disabled/enabled requests returned 200 and were captured; the enabled request joined SDK context and forwarded no internal header. The initial attempt was rejected by ATS before traffic; after the scoped fixture exception the check passed.

Loopback HTTP bypassed the tested per-session proxy configurations, and a private LAN fixture timed out. Automatic integration through the regular Simulator proxy and physical iOS acceptance remain open; explicit reverse HTTP and per-app manual routing do not establish default/global Simulator routing. The existing custom-session limitation in SDK_INTEGRATION.md is unchanged.

## Mac computer use

The portable native service used a separate temporary workspace. Codex's in-app browser exercised:

- Start/disconnect a loopback reverse listener; inspect captured protocol, headers, JSON body, timing and provenance.
- Save/enable a 120 ms network profile, observe a 0.159 s fixture request, and disable all profiles.
- Compose/send a request, preview a cURL import, save a collection/request and run a bounded two-request repeat; all responses were 200 and repeats were recorded individually.
- Save a fixture-scoped response JavaScript hook, observe its added header in the real response and inspector, create/edit a mock and observe status 201/source mock; disable the hook and mock afterward.
- Compare the two captured fixture sessions: one normalized endpoint, two matched/changed calls, one status change from 200 to 201, and the script-added header were visible in the comparison.
- Prepare the exact local AI comparison context and fingerprint without a configured key; the send action remained disabled and no provider request was made. This synthetic fixture had no sensitive fields, so the preview reported zero redactions; native form-secret checks separately exercised redaction.
- Sign in to a disposable loopback sharing service, review one selected redacted HAR, explicitly upload it, and sign out. Sign-in started no capture/upload. No real traffic or external service was involved.

Selected-process capture did not pass: a single owned waiting Python client timed out during native listener startup. A direct bounded diagnostic reported `macos-redirector exited with status None`. Mitmproxy local mode placed its Redirector app in `/Applications`; a read-only native status check found its Network Extension activated but waiting for user approval, with no enabled flag. Computer use could not access System Settings. The owner declined further Mac setup, so the extension was not enabled and selected-process acceptance remains deferred. The client and diagnostic processes were stopped.

## Android source build

A disposable copy of the real Android SDK/sample workspace built with native macOS Gradle 9.5.1, JDK 21, API 37.0 and Build Tools 36.0.0. `clean assembleDebug` passed all 65 actionable tasks (61 executed, 4 up-to-date), including both Kotlin compilations, the library AAR and sample APK. Task counts are build work, not a test count.

The first build found an ambiguous error completion call: both `complete` overloads supplied a default second argument. The interceptor now explicitly passes `statusCode = null` on error; success handling and error rethrow are preserved. Compilation failed before this one-line fix and passed afterward. Linux ARM64 Docker reached Kotlin compilation but its x86-64 AAPT2 could not run; the successful packaging check used native macOS AAPT2 instead.

The temporary SDK reused existing accepted license records; no new terms were accepted and the installed host SDK was unchanged. Elevated `adb devices -l` returned no attached devices. Emulator 37.2.12 installed only into the temporary SDK and its unsandboxed version/Hypervisor checks passed; a restricted run had misleadingly reported a NEON error. The API 37 ARM64 image transfer failed under Java and LibreSSL. The single bounded Python/OpenSSL transfer also failed with a TLS record-layer error at 128 MiB. Partials were removed; no image or AVD is available. No Android runtime result is claimed.

## Evidence and remaining work

Raw logs, fixture sources, screenshots and unsigned bundles were retained locally under `/private/tmp/mas-linux-acceptance`, `/private/tmp/mas-windows-build`, `/private/tmp/mas-ios-acceptance`, `/private/tmp/mas-mac-ui-acceptance`, and `/private/tmp/mas-android-acceptance`. No generated binaries were committed. Test-owned containers were removed without touching other Docker workloads.

Codex Cloud initially had no environments. The owner authorized configuration, and [Mobile API Studio platform acceptance](https://chatgpt.com/codex/cloud/settings/environment/6abfabca6dc081918707893723ac7c0e) was created for this repository with Node 22, Rust 1.89, no secrets and agent internet access off after setup. Its deterministic setup selects `codex/platform-support`, builds/packages Linux userspace, runs the existing bounded checks and reports Android SDK/KVM capability without spawning a coding agent. The first test found no `origin` remote; the setup now fetches the verified public repository URL. The retried test reached native Linux x86-64 Rust compilation; full remote results remain pending.

Android source build passed; connected-device/runtime acceptance remains open. See ROADMAP_CHECKPOINT.md for the remaining acceptance gates; none of these bounded checks establishes a signed/notarized release or the full owner-led matrix.

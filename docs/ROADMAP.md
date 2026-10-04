# Roadmap and Phase Gates

> **Overall status:** implementation Phases 0–5 are complete and merged to `main`. The roadmap is now an implementation record. Formal testing/CI/benchmarking/final validation were intentionally not used as phase gates and remain owner-led follow-up work.

## Phase 0 — Foundation — ✅ merged

**Outcome:** prove the internal desktop/Rust/storage/capture pipeline.

Implemented:

- [x] Tauri 2 + React + TypeScript shell
- [x] Cargo/pnpm workspace
- [x] core model crate
- [x] SQLite migrations
- [x] content-addressed body storage
- [x] fake/local capture event path
- [x] `CaptureEngine` abstraction
- [ ] CI/test suite — deferred by project rule

## Phase 1 — v0.1 Capture / Inspect / Replay — ✅ merged

Implemented:

### Device and connection

- [x] iOS Simulator discovery
- [x] Android Emulator discovery
- [x] platform capability/error handling
- [x] connection coordinator + rollback journal
- [x] CA/proxy connection strategies
- [x] typed connection diagnostics

### Capture and UI

- [x] mitmdump adapter/sidecar bridge
- [x] persistent Traffic timeline
- [x] request/response Inspector
- [x] body rendering foundations
- [x] host/method/status filtering

### Productivity

- [x] safe redacted cURL export
- [x] Replay draft/editor
- [x] native Rust replay execution
- [x] replay result persistence

## Phase 2 — v0.2 Daily Debugger — ✅ merged

Implemented:

- [x] persistent named sessions
- [x] cross-session search/filtering
- [x] endpoint normalization
- [x] saved request collections
- [x] environment variables
- [x] OS-secure secret variables
- [x] versioned export/import
- [x] Connection Doctor
- [x] setup/onboarding state
- [x] configurable/managed capture executable path

## Phase 3 — v0.3 Mocking — ✅ merged in PR #11

Implemented:

- [x] persistent mock-rule engine
- [x] create mock from captured flow
- [x] status/header/body/JSON overrides
- [x] response mutation
- [x] latency
- [x] timeout/drop
- [x] reusable fixtures
- [x] request breakpoint
- [x] response breakpoint
- [x] bounded breakpoint timeout/cancel handling
- [x] mocked-flow classification
- [x] “Disable all mocks” safety action

## Phase 4 — v0.4 App-Aware SDK — ✅ merged in PR #13

### iOS

- [x] Swift Package
- [x] URLSession/`URLProtocol` integration
- [x] manual/custom-client instrumentation
- [x] context/log events
- [x] source metadata helpers
- [x] disabled/no-op behavior until explicitly enabled
- [x] sample iOS app

### Android

- [x] Kotlin core
- [x] OkHttp interceptor
- [x] manual/custom-client instrumentation
- [x] context/log events
- [x] source metadata helpers
- [x] disabled/pass-through behavior
- [x] sample Android app

### Desktop

- [x] versioned local SDK protocol/transport
- [x] app/device handshake and client registry
- [x] SDK event persistence
- [x] proxy↔SDK request correlation
- [x] correlation-header stripping before upstream delivery
- [x] session app attribution
- [x] SDK health/status workspace
- [x] Traffic/Inspector app context and logs
- [x] app-context search/filtering

## Phase 5 — v0.5 Compare + AI — ✅ merged in PR #15

### Deterministic comparison

- [x] session pairing
- [x] endpoint normalization/matching
- [x] repeated-call alignment
- [x] request method/query/header/body diff
- [x] response status/header/body diff
- [x] timing and size diff
- [x] missing/extra call detection
- [x] JSON shape/type drift
- [x] SDK app/screen/feature/source context comparison

### Deterministic diagnostics

- [x] duplicate/retry detection
- [x] slowest-request ranking
- [x] error clustering
- [x] waterfall overlap/sequential groups
- [x] evidence-count comparison summary

### Desktop Compare UI

- [x] Compare workspace route
- [x] baseline/candidate session selector
- [x] endpoint + occurrence drill-down
- [x] request/response/timing inspector
- [x] missing/extra/drift indicators
- [x] deterministic diagnostics panel
- [x] SDK context difference display

### Optional AI

- [x] provider-neutral interface
- [x] OpenAI Responses API provider
- [x] BYOK OS-secure storage
- [x] provider/model/redaction settings
- [x] deterministic redaction pipeline
- [x] exact external-context preview
- [x] SHA-256 fingerprint gate between preview and send
- [x] session-diff explanation
- [x] selected-flow diagnosis
- [x] local AI history with provider/model/context fingerprint
- [x] OpenAI `store: false`
- [x] AI remains optional and never blocks local deterministic comparison

---

# Current stage — Owner-led final validation

The product implementation plan is finished. The next stage is deliberately separate from the phase gates.

Validation is owned independently by the repository owner and may include whichever manual/automated approach they choose later.

Documentation for that stage: [FINAL_VALIDATION.md](FINAL_VALIDATION.md).

The repository does **not** currently claim:

- a passing automated test suite;
- CI certification;
- benchmark certification;
- compatibility across every Xcode/iOS/Android runtime;
- release/security audit completion.

Defects found during validation should become focused validation/fix issues rather than reopening completed implementation phases.

---

# Expansion roadmap — integrated on main

The subsequent eight-milestone expansion is integrated through PRs #19–26:

| Milestone | Delivered capability | Documentation |
| --- | --- | --- |
| 1 — #19 | Desktop/process and paired physical-device capture targets | [Capture targets](CAPTURE_TARGETS.md) |
| 2 — #20 | Ordered proxy rules and manual listener modes | [Proxy rules](PROXY_RULES.md) |
| 3 — #21 | Protocol/body inspection and persistent WebSocket history | [Protocol inspection](PROTOCOL_INSPECTION.md) |
| 4 — #22 | Scoped network conditions and recovery controls | [Network conditions](NETWORK_CONDITIONS.md) |
| 5 — #23 | Compose, bounded repeats and open-format interchange | [Compose/interchange](COMPOSE_INTERCHANGE.md) |
| 6 — #24 | Bounded JavaScript hooks and private CLI/MCP automation | [Scripting/automation](SCRIPTING_AUTOMATION.md) |
| 7 — #25 | Reviewed sharing and manual team workspace sync | [Sharing](SHARING_WORKSPACE.md) |
| 8 — #26 | Windows/Linux service portability and relocatable packaging | [Platform support](PLATFORM_SUPPORT.md) |

This expansion does not change the original Phases 0–5 implementation/testing history. Later focused checks and runtime cases are recorded in [PLATFORM_ACCEPTANCE.md](PLATFORM_ACCEPTANCE.md); the [completion review](ROADMAP_COMPLETION.md) distinguishes passed cases from owner-excluded checks and conditional adapters. Start with the [usage guide](USAGE.md).

# Deferred product scope

- deeper OpenAPI workflows;
- plugin marketplace;
- production APM integration;
- public signed installer, release/update infrastructure and release certification.

Implementation of an additional target or protocol is not proof of compatibility with every host, device or client. Physical devices, Mac process setup, native Windows/Linux capture targets and the final rendered gRPC/Doctor check remain unverified after owner exclusion from this acceptance run.

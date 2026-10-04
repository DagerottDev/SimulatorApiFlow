<a id="readme-top"></a>

[![Stars][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Apache-2.0 License][license-shield]][license-url]

<div align="center">
  <img src="apps/desktop/src-tauri/icons/128x128.png" alt="Mobile API Studio logo" width="80" height="80">
  <h1>Mobile API Studio</h1>
  <p>A local-first API debugger for capturing, inspecting, replaying, mocking, and comparing development traffic in a browser workspace.</p>
  <p>
    <a href="docs/USAGE.md">Usage guide</a>
    &middot;
    <a href="#getting-started">Build from source</a>
    &middot;
    <a href="https://github.com/DagerottDev/mobile-api-studio/issues/new?template=bug_report.md">Report a bug</a>
    &middot;
    <a href="https://github.com/DagerottDev/mobile-api-studio/issues/new?template=feature_request.md">Request a feature</a>
  </p>
</div>

<details>
  <summary>Table of Contents</summary>
  <ul>
    <li><a href="#about-the-project">About The Project</a>
      <ul><li><a href="#built-with">Built With</a></li></ul>
    </li>
    <li><a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
        <li><a href="#portable-bundle">Portable Bundle</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a>
      <ul>
        <li><a href="#how-capture-works">How Capture Works</a></li>
        <li><a href="#local-automation">Local Automation</a></li>
      </ul>
    </li>
    <li><a href="#security-and-data">Security and Data</a></li>
    <li><a href="#documentation">Documentation</a></li>
    <li><a href="#development">Development</a></li>
    <li><a href="#roadmap-and-validation">Roadmap and Validation</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#support-the-project">Support the Project</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ul>
</details>

## About The Project

Mobile API Studio helps mobile developers inspect what their app sends, reproduce a failing request, test response changes, and compare behavior between sessions. Start with an iOS Simulator or Android Emulator; additional capture targets depend on the host and capture mode.

<a id="what-you-can-do"></a>

| Workflow | Implemented capabilities |
| --- | --- |
| Capture and inspect | HTTP(S), HTTP/2 and TLS details, trailers, persistent WebSocket history, GraphQL and body viewers, and bounded Protobuf decoding with your descriptor. |
| Compose and replay | Edit captured or saved requests, compose from scratch, record bounded repeats, and preview cURL, HAR, Postman v2.1 JSON or CSV interchange. |
| Change live traffic | Ordered proxy rules, local/remote response mapping, header/body rewrites, cookie/cache blocking, breakpoints, and bounded JavaScript hooks. |
| Simulate network conditions | Global, app, host or endpoint latency, jitter, bandwidth, offline and request-failure profiles, with a visible disable-all control. |
| Understand and compare | Optional Swift/Kotlin SDK context and session comparisons for calls, payloads, schema, timing, errors and retries. |
| Automate locally | A Python CLI and MCP stdio bridge through a private local control endpoint. |
| Share deliberately | Reviewed redacted HAR uploads to an optional self-hosted service, expiring/revocable links, and manual team rule/fixture publishing and pulling. |
| Explain with AI | Local redaction and an exact evidence preview before explicitly sending to an optional provider. |

**Source status:** All eight expansion milestones are integrated on `main` through PRs #19–26. Recorded bounded Simulator/Emulator HTTP(S), protocol, proxy recovery and network-profile checks passed, alongside Linux container/Cloud userspace checks and Windows cross-compilation. Mac process capture, physical devices, native Windows/Linux capture and the final rendered gRPC/Connection Doctor check were owner-excluded and remain unverified. See the [completion review](docs/ROADMAP_COMPLETION.md) and [acceptance record](docs/PLATFORM_ACCEPTANCE.md). No signed installer or certified public release is offered.

### Built With

- **Rust and Axum** — local service, capture coordination, replay and comparison.
- **React, TypeScript and Vite** — browser workspace.
- **SQLite** — local sessions, metadata and workspace persistence.
- **mitmproxy / mitmdump** — capture engine with the repository's Python bridge.
- **Swift Package and Kotlin/OkHttp** — optional development SDKs.
- **QuickJS** — isolated JavaScript rule workers.

The localhost service is the current entry point. The earlier Tauri source remains for historical parity comparison. See the [architecture](docs/ARCHITECTURE.md).

<p align="right"><a href="#readme-top">Back to top</a></p>

## Getting Started

Build from source or create a portable bundle for your host. No prebuilt release is currently published. macOS has recorded bounded mobile runtime checks; Linux userspace builds and Windows GNU cross-compilation have narrower evidence. Consult the [platform support matrix](docs/PLATFORM_SUPPORT.md) before choosing a host.

### Prerequisites

| Requirement | Version or purpose |
| --- | --- |
| Rust | 1.88 or newer. |
| Node.js | 20.19+ in the Node 20 line, or 22.12+; npm is also used by the launchers. |
| pnpm | 10.15.0, as declared in `package.json`. |
| Python and mitmproxy | Python 3 and `mitmdump`, installed separately for capture. |
| macOS build tools | Xcode Command Line Tools; full Xcode and a Simulator runtime for iOS work. |
| Linux build tools | C build tools, `pkg-config` and `libdbus-1` development headers. |
| Android tooling | Android SDK Platform Tools and an Android Emulator for Android work. |

iOS Simulator workflows require macOS. Windows native runtime and Linux host capture remain unverified; additional mode-specific permissions are documented in [Platform support](docs/PLATFORM_SUPPORT.md).

<a id="build-from-source"></a>

### Installation

Clone the repository, then install its locked JavaScript dependencies:

```sh
git clone https://github.com/DagerottDev/mobile-api-studio.git
cd mobile-api-studio
pnpm install --frozen-lockfile
```

On macOS or Linux, start the workspace from the repository root:

```sh
./scripts/run-local.sh
```

On Windows PowerShell:

```powershell
.\scripts\run-local.ps1
```

Both launchers build the React UI and QuickJS worker, then compile and start the Rust service. Open the printed URL if the browser does not open automatically; the default is **http://127.0.0.1:8180**. Capture and SDK ingestion use `8181` and `8182`.

To use another UI port, a separate data directory, and open the browser yourself:

```sh
./scripts/run-local.sh --port 8191 --data-dir /absolute/path/to/separate-workspace --no-open
```

The PowerShell launcher accepts the same flags. Changing `--port` changes only the UI port. Run one service per workspace and keep the earlier desktop app closed when using the same data directory. Stop with **Ctrl+C** to end capture and restore supported Android or automatic Simulator proxy settings. Source launchers need the checkout assets to remain in place.

If `mitmdump` is not on `PATH`, the service can discover a standard Homebrew install, or you can set its absolute path in **Settings**. No AI provider or sharing service is required for local debugging.

### Portable Bundle

After installing the prerequisites and dependencies, choose an empty output directory:

```sh
python3 scripts/package-local.py /absolute/path/to/empty-bundle
/absolute/path/to/empty-bundle/start.sh
```

Windows uses the bundle's `start.ps1`. The bundle includes the service, worker, UI, capture addon and CLI, and can run outside the checkout. Python, `mitmdump` and device tools remain separate host prerequisites. Packaging does not produce a signed installer. See [Platform support](docs/PLATFORM_SUPPORT.md).

<p align="right"><a href="#readme-top">Back to top</a></p>

## Usage

### How Capture Works

1. Boot an iOS Simulator or Android Emulator and run your development app.
2. Open **Connect** and select the runtime. For iOS automatic routing, select the current Mac network service and choose **Set up Simulator & start capture**; approve macOS network authorization if prompted. Android uses **Start capture** with its CA guidance.
3. Generate a request in your app. Open **Traffic**, select the session, and inspect its headers, body, timing, TLS and errors.
4. Open **Replay** to edit and resend a captured request to a server you control. Use **Mocks**, **Network** and **Compare** to change responses, simulate conditions and compare sessions.
5. For automatic iOS capture, **Disable routing** restores prior Mac proxy settings while keeping the session open; **Enable routing** resumes routing. **Disconnect** restores supported settings and ends capture. Remove manually configured proxies when ending a manual workflow.

Automatic Simulator routing temporarily changes HTTP/HTTPS proxies on the selected **Mac network service** and can also route other proxy-aware Mac apps. The CA is installed only in the selected Simulator. Existing active proxies/PAC block automatic setup. Uncheck **Set up Mac network routing automatically** for a manual app-scoped workflow. The SDK adds context; it does not automatically route every custom URLSession through a regular proxy. See [Simulator setup](docs/IOS_SIMULATOR_SETUP.md) for trust, scope and recovery.

Android HTTPS requires development CA trust; prefer app-scoped debug trust. Android API 37 also needs the appropriate local-network permission. Apps with certificate pinning need their own debug configuration; Mobile API Studio does not bypass pinning. Optional [SDK integration](docs/SDK_INTEGRATION.md) adds app, screen, feature and source context.

Open **Settings → Connection Doctor** for prerequisite, routing and TLS diagnostics. HTTP/3 is limited to supported local/reverse modes and has no Replay support. WebSocket replay and ping/pong payload inspection are unavailable; Protobuf decoding requires your descriptor and supports bounded uncompressed messages. See [Protocol inspection](docs/PROTOCOL_INSPECTION.md) for details, including engine-specific trailer limits.

### Local Automation

With the service running, use another terminal from the repository root:

```sh
printf '{}' | python3 scripts/mas-cli.py health
printf '{}' | python3 scripts/mas-cli.py list_sessions
```

The CLI uses a private Unix socket on macOS/Linux or a current-user named pipe on Windows. MCP clients can run `python3 /absolute/path/to/repository/scripts/mas-cli.py --mcp`. Arguments come from stdin; custom socket paths, supported commands and JavaScript hook examples are in [Scripting and automation](docs/SCRIPTING_AUTOMATION.md) and the [usage guide](docs/USAGE.md).

<p align="right"><a href="#readme-top">Back to top</a></p>

## Security and Data

Traffic and workspace data stay on your computer by default. Replay sends requests to your chosen target; sharing and AI require explicit preview and send. Signing in starts no capture or background synchronization. Redaction covers known secrets, but bodies and scripts can contain application secrets that need your review.

The browser service binds only to `127.0.0.1`, checks Host and Origin, and requires a process-lifetime command token held in browser memory, outside URLs and logs. Optional paired LAN capture listeners have separate controls; they do not expose the browser API.

Default workspace locations:

| Host | Data directory |
| --- | --- |
| macOS | `~/Library/Application Support/dev.mobileapistudio.desktop` |
| Windows | `%LOCALAPPDATA%\dev.mobileapistudio.desktop` |
| Linux | `$XDG_DATA_HOME/dev.mobileapistudio.desktop`, or `~/.local/share/dev.mobileapistudio.desktop` when unset or relative. |

Back up `app.db` with the service stopped before schema migrations. **Settings** imports a selected JSON workspace file and exports a redacted bundle. After a forced stop, use **Connect**'s pending rollback recovery control before another capture; recovery preserves unexpected external proxy changes. CA private material stays local and is omitted from normal exports.

Read [Security and privacy](docs/SECURITY_AND_PRIVACY.md) for implemented boundaries and pending security/release review. Report vulnerabilities through [SECURITY.md](SECURITY.md).

<p align="right"><a href="#readme-top">Back to top</a></p>

## Documentation

| Guide | Covers |
| --- | --- |
| [Usage](docs/USAGE.md) | End-to-end workflows, separate workspaces, CLI examples and opt-in sharing. |
| [Simulator setup](docs/IOS_SIMULATOR_SETUP.md) / [Capture targets](docs/CAPTURE_TARGETS.md) | Routing, CA trust, recovery and capability-dependent target modes. |
| [Proxy rules](docs/PROXY_RULES.md) / [Network conditions](docs/NETWORK_CONDITIONS.md) | Rule ordering, listeners, breakpoints and scoped impairment profiles. |
| [Protocol inspection](docs/PROTOCOL_INSPECTION.md) | HTTP versions, WebSocket history, body viewers and descriptor decoding. |
| [Compose and interchange](docs/COMPOSE_INTERCHANGE.md) | Request editing, bounded repeats and reviewed open-format import/export. |
| [SDK integration](docs/SDK_INTEGRATION.md) | Swift/Kotlin setup, local ingestion and request correlation. |
| [Scripting and automation](docs/SCRIPTING_AUTOMATION.md) | Isolated hooks, private CLI and MCP contracts. |
| [Sharing workspace](docs/SHARING_WORKSPACE.md) / [Sharing server](apps/sharing-server/README.md) | Self-hosted service setup, reviewed uploads, roles and manual sync. |
| [AI privacy and providers](docs/AI_PRIVACY_AND_PROVIDERS.md) | Provider setup, redaction and exact-preview send boundaries. |
| [Architecture](docs/ARCHITECTURE.md) / [Platform support](docs/PLATFORM_SUPPORT.md) | Component boundaries, host prerequisites, packaging and evidence limits. |
| [macOS release preparation](docs/MACOS_RELEASE.md) | Source distribution and remaining owner-led release gates. |

<p align="right"><a href="#readme-top">Back to top</a></p>

## Development

After installing dependencies, these contributor commands are available from the repository root:

```sh
pnpm typecheck
pnpm build
cargo check --workspace --locked
cargo test -p mobile-api-studio-server --locked
cargo test -p app-core --locked
```

These are local checks; the repository has no CI gate. Match checks to the change and record what you actually verified. The launcher also builds the separate script worker. Focused runtime scripts in `scripts/` have their own fixture and environment requirements; consult the [local validation record](docs/LOCALHOST_VALIDATION.md) and [platform acceptance record](docs/PLATFORM_ACCEPTANCE.md) before running them.

<p align="right"><a href="#readme-top">Back to top</a></p>

## Roadmap and Validation

The [roadmap](docs/ROADMAP.md) records completed implementation Phases 0–5 and the eight integrated expansion milestones. Phases 0–5 were merged without automated tests or CI as phase gates. Later migration and expansion work has focused deterministic checks and bounded runtime/browser evidence; it does not retroactively validate those implementation phases.

The owner controls the separate [final validation](docs/FINAL_VALIDATION.md) and release decision. Physical-device, Mac process, native Windows/Linux capture and the final rendered gRPC/Connection Doctor checks remain unverified after exclusion from the acceptance run. Cross-builds and userspace containers do not substitute for native device acceptance.

Documented deferred scope includes deeper OpenAPI workflows, a plugin marketplace, production APM integration and signed release/update infrastructure. See the [completion review](docs/ROADMAP_COMPLETION.md) for conditional features and the completed acceptance scope.

<p align="right"><a href="#readme-top">Back to top</a></p>

<a id="for-contributors"></a>

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) for focused branches, pull request expectations and validation policy. Use [issues][issues-url] to report reproducible defects or discuss features, and include the platform and checks performed in your pull request. Keep credentials, captured private traffic and certificate keys out of issues and contributions.

<p align="right"><a href="#readme-top">Back to top</a></p>

## Support the Project

[![Animated Buy me a coffee card linking to DagerottDev's support page][support-card]][coffee-url]

- [Buy Me a Coffee][coffee-url] — international support.
- [Buy DagerottDev a Chai on Bondin](https://bondin.io/dagerottdev) — support from India.

Support is optional. Contributions, bug reports and documentation improvements are welcome too.

<p align="right"><a href="#readme-top">Back to top</a></p>

## License

Mobile API Studio is licensed under the [Apache License 2.0](LICENSE). Third-party dependencies retain their own licenses.

<p align="right"><a href="#readme-top">Back to top</a></p>

## Acknowledgments

- [mitmproxy](https://mitmproxy.org/) for the capture engine.
- README layout inspired by [Best-README-Template](https://github.com/othneildrew/Best-README-Template).

<p align="right"><a href="#readme-top">Back to top</a></p>

[stars-shield]: https://img.shields.io/github/stars/DagerottDev/mobile-api-studio?style=for-the-badge
[stars-url]: https://github.com/DagerottDev/mobile-api-studio/stargazers
[issues-shield]: https://img.shields.io/github/issues/DagerottDev/mobile-api-studio?style=for-the-badge
[issues-url]: https://github.com/DagerottDev/mobile-api-studio/issues
[license-shield]: https://img.shields.io/github/license/DagerottDev/mobile-api-studio?style=for-the-badge
[license-url]: LICENSE
[support-card]: .github/assets/buy-me-a-coffee.gif
[coffee-url]: https://buymeacoffee.com/dagerottdev

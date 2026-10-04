# Mobile API Studio

**A local-first API debugger with a browser workspace.** Capture HTTP(S), inspect protocols and bodies, compose or replay requests, change responses and network conditions, and compare sessions. Start with an iOS Simulator or Android Emulator; additional capture targets depend on host capabilities.

[Build from source](#build-from-source) · [Usage guide](docs/USAGE.md) · [Contribute](CONTRIBUTING.md) · [Support the project](#support-the-project)

> **Source status:** The eight expansion milestones are integrated on `main` through PRs #19–26. Bounded iOS Simulator and Android Emulator HTTP(S), protocol, proxy recovery, and network-profile checks passed, alongside Linux container/Cloud userspace checks and Windows cross-compilation. Mac process capture, physical devices, native Windows/Linux capture, and the final rendered gRPC/Connection Doctor check were excluded by the owner and remain unverified. See the [completion review](docs/ROADMAP_COMPLETION.md) and [acceptance record](docs/PLATFORM_ACCEPTANCE.md). No signed installer or certified public release is offered.

## What you can do

| Workflow | In the app |
| --- | --- |
| Capture and inspect | Connect runtimes or supported manual listeners; inspect HTTP/2, TLS, trailers, WebSocket history, GraphQL and body formats; decode bounded Protobuf with your descriptor. |
| Compose and replay | Edit captured or saved requests, compose from scratch, record bounded repeats, and preview cURL, HAR, Postman v2.1 JSON or CSV interchange. |
| Change live traffic | Order proxy rules, map local/remote responses, rewrite headers/bodies, block cookies/cache, pause at breakpoints, or use bounded JavaScript hooks. |
| Simulate network conditions | Apply global, app, host or endpoint latency, jitter, bandwidth, offline and request-failure profiles; disable all from any page. |
| Understand and compare | Add optional Swift/Kotlin SDK context, then compare calls, payloads, schema, timing, errors and retries between sessions. |
| Automate locally | Use the private Python CLI or MCP stdio bridge without exposing the browser API on the LAN. |
| Share deliberately | Preview selected redacted HAR uploads to an optional self-hosted service; expire/revoke links and manually publish/pull team rules and fixtures. |
| Explain with AI | Preview locally redacted evidence before explicitly sending it to an optional provider. |

Traffic and workspace data stay on your computer by default. Sharing and AI require an explicit preview and send; signing in starts no capture or synchronization. See [Security and privacy](docs/SECURITY_AND_PRIVACY.md) for the implemented boundaries and the validation still pending.

## Build from source

Source builds and local checks passed on **macOS**, **Linux ARM64 in Docker**, and **Linux x86-64 userspace in Codex Cloud**. Windows GNU cross-compiles; native Windows runtime remains unverified. See the [platform support matrix](docs/PLATFORM_SUPPORT.md).

Install:

- Rust 1.88 or newer, plus Xcode Command Line Tools on macOS or C build tools, `pkg-config`, and `libdbus-1` development headers on Linux;
- Node.js 20.19+ or 22.12+ and pnpm 10.15.0;
- Python 3 and `mitmdump` from mitmproxy for capture;
- Xcode and an iOS Simulator runtime for iOS work, or Android SDK Platform Tools and an Android Emulator for Android work.

Then, from the repository root:

```sh
pnpm install --frozen-lockfile
./scripts/run-local.sh
```

The command builds the React UI and JavaScript worker, starts the Rust service, and opens the local URL. Pass `--port 8190` to use another UI port. Capture and SDK ingestion remain on `8181` and `8182`. Stop the service with Ctrl+C so it can end capture and restore supported Android or automatic Simulator proxy settings. Keep the earlier desktop app closed while using the same data directory. Windows uses `./scripts/run-local.ps1` in PowerShell. See the [usage guide](docs/USAGE.md) for isolated data directories, portable bundles, and CLI examples.

Workspace data stays in the operating system data directory listed in the [platform support matrix](docs/PLATFORM_SUPPORT.md). Back up `app.db` before any future schema migration. Browser import uses a selected JSON file; export downloads a redacted workspace bundle. The service listens only on `127.0.0.1`, checks Host and Origin, and requires a process-lifetime token for commands. The token is held in browser memory, outside URLs and logs.

`mitmdump` is installed separately. The service uses the Python bridge in this repository and can discover a standard Homebrew install or use the absolute path set in Settings. Read [platform support](docs/PLATFORM_SUPPORT.md) for host requirements and [macOS release preparation](docs/MACOS_RELEASE.md) for the remaining validation gates.

## How capture works

1. Boot an iOS Simulator or Android Emulator and open **Connect**.
2. For iOS, select the booted Simulator and choose **Set up Simulator & start capture**. This installs its capture certificate and enables HTTP/HTTPS routing through the selected Mac network service. Approve macOS network authorization if prompted. Android uses **Start capture** with its own CA guidance.
3. Generate traffic, then use **Traffic**, **Replay**, **Mocks**, **Network**, and **Compare**.
4. Choose **Disable routing** to restore the previous Mac proxy settings while keeping the session open; **Enable routing** turns capture routing back on. **Disconnect** restores settings and ends capture.

Automatic Simulator routing can also affect proxy-aware Mac apps on that network service. The CA is installed only in the selected Simulator. Existing active proxies/PAC are preserved and block automatic setup; uncheck automatic routing to use a manual app-scoped workflow. Android Emulator proxy changes are journaled for rollback. See [Simulator setup](docs/IOS_SIMULATOR_SETUP.md) for scope, recovery and validation. Apps with certificate pinning need their own debug configuration; Mobile API Studio does not bypass pinning. Optional [iOS and Android SDKs](docs/SDK_INTEGRATION.md) add app context without requiring production instrumentation.

## For contributors

The current app uses Axum, React, TypeScript, Rust, SQLite, and mitmproxy. The iOS SDK is a Swift Package; the Android SDK is a Kotlin library with an OkHttp interceptor. The old Tauri source remains for historical parity comparison; the localhost service is the current entry point. Browse the [architecture](docs/ARCHITECTURE.md), [roadmap](docs/ROADMAP.md), and [contribution guide](CONTRIBUTING.md) to find a starting point. Report vulnerabilities through the [security policy](SECURITY.md).

Implementation Phases 0–5 were merged without automated tests or CI as phase gates. The later localhost migration and expansion add focused deterministic checks and bounded runtime/browser acceptance records; no CI gate has been added. The repository owner controls the separate [final validation](docs/FINAL_VALIDATION.md); please describe what you actually verified in a pull request.

## Support the project

[![Animated Buy me a coffee card linking to DagerottDev's support page](.github/assets/buy-me-a-coffee.gif)](https://buymeacoffee.com/dagerottdev)

- [Buy Me a Coffee](https://buymeacoffee.com/dagerottdev) — international support.
- [Buy DagerottDev a Chai on Bondin](https://bondin.io/dagerottdev) — support from India.

Support is optional. Contributions, bug reports, and documentation improvements are welcome too.

## License

Mobile API Studio is licensed under the [Apache License 2.0](LICENSE). Third-party dependencies retain their own licenses.

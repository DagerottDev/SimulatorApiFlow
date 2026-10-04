# Using Mobile API Studio

## Start the workspace

Install the prerequisites in the [README](../README.md#build-from-source), then run from the repository root:

```sh
pnpm install --frozen-lockfile
./scripts/run-local.sh
```

On Windows PowerShell, use `.\scripts\run-local.ps1` instead. The launcher builds the UI and JavaScript worker before starting the local service. The default browser address is `http://127.0.0.1:8180`. Capture uses `8181`, and SDK ingestion uses `8182`. If the browser does not open, use the printed address. In Codex, open it in the in-app browser if that address is permitted.

To choose another UI port or a separate workspace:

```sh
./scripts/run-local.sh --port 8191 --data-dir /absolute/path/to/separate-workspace --no-open
```

Changing `--port` changes only the UI port. Run one service per workspace; keep the legacy desktop app closed when using the same data. Stop with Ctrl+C to disconnect and restore supported Android proxy settings. Back up `app.db` before migrations with the service stopped. Default data locations and platform limits are in [Platform support](PLATFORM_SUPPORT.md).

## Capture your development app

1. Boot a Simulator in Xcode or an Android Emulator, and run your development app.
2. Open **Connect**, select the runtime, and follow the readiness and proxy/certificate guidance before connecting.
3. Generate requests in your app. Open **Traffic**, select the capture session, and inspect a request's headers, body, timing, TLS and errors.
4. Disconnect when finished. Android's prior proxy is restored; remove any manually configured proxy when ending a manual workflow.

Simulator routing is manual. For an app you own, explicitly route its test URLSession through the proxy or use an appropriate reverse listener. The optional SDK adds context; it does not route every custom URLSession through a regular proxy automatically. Android API 37 local-network access also needs the appropriate application permission; a denial remains an error. HTTPS needs development CA trust in the test app/runtime. For Android, prefer app-scoped debug trust. Certificate pinning requires your app's debug configuration; there is no pinning bypass.

The [SDK integration guide](SDK_INTEGRATION.md) explains Swift/Kotlin setup, local ingestion addresses, and correlation. Enable instrumentation only in development. App-scoped network conditions need SDK attribution already available when the request starts; later Inspector context does not prove it was available then.

**Connect** also exposes capability-dependent desktop/process capture, explicitly paired physical-device LAN listeners, and manual reverse/upstream/SOCKS/DNS modes. These require their own routing, permissions and trust. They do not silently change the host-wide proxy. See [Capture targets](CAPTURE_TARGETS.md) and [Proxy rules](PROXY_RULES.md). Physical devices and Mac process setup were excluded from this acceptance run.

## Inspect protocols and troubleshoot

In **Traffic**, use the Inspector's appropriate body view for JSON, XML, forms, multipart, raster images or GraphQL. Raw text remains available. WebSocket history retains message direction and sequence. For gRPC/Protobuf, select your binary `FileDescriptorSet` and full message name to decode supported uncompressed messages; compressed or unsupported data stays raw with a diagnostic.

HTTP/2 and trailers depend on the capture mode and engine. HTTP/3 is limited to supported local/reverse modes and QUIC v1; it has no Replay support. WebSocket replay and ping/pong payload inspection are unavailable. The tested mitmproxy 12.2.3 does not support HTTP/1 chunked response trailers. See [Protocol inspection](PROTOCOL_INSPECTION.md).

Open **Settings → Connection Doctor** for readiness, routing and TLS-failure diagnostics. Doctor diagnoses trust and pinning; it does not automatically repair trust. Native descriptor decoding and Doctor runtime checks passed; the final rendered verification was excluded after a saved browser permission block.

## Compose, replay and exchange requests

Open **Replay**, choose a captured/saved source or compose a blank request. Edit the method, complete URL (including query), headers and raw/JSON/form/multipart/binary body, then send to a server you control. Create a collection in **Workspace** before using **Save draft**.

**Bounded repeat** records individual results: count 1–100, concurrency 1–4, interval up to 60 seconds, a four-minute start schedule and five-minute run limit. Already sent requests can affect the server; a deadline does not prove a pending request was never sent.

Preview cURL, HAR 1.2, Postman v2.1 JSON or CSV in Replay before explicitly loading/saving/sending requests or importing traffic. Preview a selected source export before downloading it. Known secret headers are redacted, but captured bodies can contain application secrets. Native Charles/Proxyman adapters are not offered without documented versions and representative fixtures. Full workspace backup/restore and script bundle import live in **Settings**. See [Compose and interchange](COMPOSE_INTERCHANGE.md).

## Change responses and network conditions

Open **Mocks** for both ordered proxy rules and response mocks. Create a disabled rule, set a narrow method/host/path match, choose its behavior, save, preview the match, then enable it. Lower numerical priority runs first. Rewrites can change headers/bodies; terminal rules can block or map a response. Rule audits show the applied operations. A breakpoint requires an explicit decision in the breakpoint controls. Choose a file through the map-local picker rather than granting arbitrary filesystem access.

In **Network**, create a profile, select global/app/host/endpoint scope, configure latency, jitter, rates, offline or request-failure percentage, save and enable it. One profile wins: endpoint, host, app, global, followed by priority and stable ordering. The active banner's **Disable all** restores ordinary behavior. Profiles simulate buffered HTTP request conditions; failure percentage is not packet loss, and DNS, encrypted passthrough and ongoing WebSocket messages are outside those conditions. See [Network conditions](NETWORK_CONDITIONS.md).

**Compare** selects a baseline and candidate session to inspect missing calls, payload/schema changes, timing and app context. **AI** is optional: configure a provider, preview the redacted evidence, then explicitly send the unchanged preview.

## Automate locally

With the service running, use another terminal:

```sh
printf '{}' | python3 scripts/mas-cli.py health
printf '{}' | python3 scripts/mas-cli.py list_sessions
```

For a custom macOS/Linux data directory:

```sh
printf '{}' | python3 scripts/mas-cli.py --socket /absolute/path/to/separate-workspace/control/socket health
```

For an MCP client, configure a stdio process running `python3 /absolute/path/to/repository/scripts/mas-cli.py --mcp`; add `--socket` for a custom Unix endpoint. Windows uses a current-user named pipe. CLI JSON arguments come from stdin. The allowlist covers capture/session/search, rules, profiles and selected export; it does not grant arbitrary commands, secret access, AI calls or direct Replay.

For JavaScript hooks, use **Mocks → Proxy rule → JavaScript hook** and define a synchronous `transform(event)` returning the complete edited event. Each invocation uses a disposable bounded QuickJS worker without host APIs. Failures stop the matching flow. Imported scripts stay disabled; source needs explicit export review. Contract and examples: [Scripting and automation](SCRIPTING_AUTOMATION.md).

## Share only a reviewed selection

Sharing is optional and self-hosted. To run a local sharing service on a supported Unix host, use a second terminal and a new private directory:

```sh
mkdir -m 700 /absolute/path/to/private-sharing-data
cargo run -p mobile-api-studio-sharing-server -- --data-dir /absolute/path/to/private-sharing-data
```

Its default address is `http://127.0.0.1:8190`, separate from the app's UI port. Bootstrap creates a private `owner-access-token` file. Enter that token privately in **Sharing**, follow the service's token removal/rotation guidance, and keep it out of screenshots and shell history. Remote deployment requires your own HTTPS reverse proxy; none is configured by these commands.

Select flows, choose whether to include query/body data, prepare the redacted preview, review it, then upload with an expiry. Anyone with the active link can download its HAR; revoke the link when it is no longer needed. Signing in alone transfers no capture or workspace data.

For team definitions, load the remote revision and select local rules/fixtures, preview, then publish. Publishing replaces the shared selection with revision checks. Pulling is a separate reviewed import with fresh IDs and disabled rules. Roles restrict editing; there is no background sync. Details and limits: [Sharing workspace](SHARING_WORKSPACE.md) and [server setup](../apps/sharing-server/README.md).

## Build a portable local bundle

After installing dependencies, choose an empty output directory:

```sh
python3 scripts/package-local.py /absolute/path/to/empty-bundle
/absolute/path/to/empty-bundle/start.sh
```

Windows uses the bundle's `start.ps1`. The bundle includes binaries, UI, addon and CLI, and can run outside the checkout. Python, mitmdump and device tools are separate prerequisites. It is not a signed installer. See [Platform support](PLATFORM_SUPPORT.md) and the [acceptance record](PLATFORM_ACCEPTANCE.md) for what was actually verified.

# One-click iOS Simulator setup

## Use it

1. Start SimulatorApiFlow with `./scripts/run-local.sh` and boot a Simulator in Xcode.
2. Open **Connect**, select the Simulator, and leave **Set up Mac network routing automatically** selected.
3. Keep the current-route Mac network service selected, then click **Set up Simulator & start capture**.
4. Approve macOS's standard network authorization prompt if required. Enter any administrator credentials directly into macOS; SimulatorApiFlow does not collect them.
5. Generate requests in your development app and inspect them in **Traffic**.

Setup starts capture, waits for the capture CA, installs it in the selected Simulator with `simctl keychain add-root-cert`, and applies HTTP/HTTPS routing through `127.0.0.1:8181`. Xcode and a usable booted runtime are required.

## Turn routing off

**Disable routing** restores the original Mac proxy configuration and keeps the capture session open. **Enable routing** applies it again. **Disconnect** restores routing before ending the session. Normal Ctrl+C shutdown also restores supported settings; if restoration fails, the service stays available so you can retry.

The capture certificate remains installed in the development Simulator when routing is off. Disable does not delete certificates or saved traffic.

## Scope and recovery

This uses the selected Mac network service, so other proxy-aware Mac apps may also send traffic through capture. The Mac's CA trust is unchanged; those apps may reject intercepted HTTPS. Custom transports, apps that ignore system proxies, and certificate-pinned apps need their own development configuration.

An existing active proxy, PAC/autodiscovery or proxy authentication prevents automatic setup. Use manual app-scoped routing instead by unchecking the automatic option. The SDK adds context; it does not force every custom transport to use the proxy.

Before changing settings, a private local journal saves the original proxy configuration and Network Location. Unexpected external changes are preserved. If recovery reports a changed Network Location, return to the original location and retry. After an abrupt termination, restart with the same data directory and use **Recover** before starting capture. Keep the service running until routing recovery succeeds.

## Validation — 2026-10-04

The bounded macOS/iOS 26.5 follow-up passed on a disposable iPhone 17 Pro Max Simulator:

| Check | Result |
| --- | --- |
| One-click setup | Capture CA installed; selected Mac HTTP/HTTPS routing verified through native effective settings. |
| Default URLSession HTTPS | Three synthetic requests returned HTTP 200 with routing on/off/on. The first and third were captured; the disabled request was absent from capture. No per-session proxy override, SDK instrumentation or custom trust delegate was used. |
| Disable and disconnect | Original proxy dictionary and configuration presence were restored exactly. Disable kept the session open; disconnect stopped capture. |
| Cleanup | Owned service stopped, pending journal cleared and disposable Simulator deleted; the existing Simulator was preserved. |
| Deterministic checks | 30 app-core and 10 local-server checks passed, including private journal storage/redaction, legacy journal compatibility and caller-cancellation recovery. Native helper self-check and TypeScript/Vite production build passed. |

The first attempt timed out during authorization and left original settings unchanged; its journal was recovered on the successful retry. Native authorization now runs outside the async worker, and the complete transaction survives browser caller cancellation.

This verifies default URLSession HTTPS and the selected network service on this host/runtime. It does not establish every custom transport, pinned app, runtime or physical device. Rendered Connect controls were not independently exercised; the UI typecheck and production build passed. Earlier roadmap checks and their exclusions remain in [Platform acceptance](PLATFORM_ACCEPTANCE.md).

# Platform support

Mobile API Studio is a source-built localhost service with a browser UI. It does not currently publish packaged installers. Source launchers use the checkout assets. The portable bundle includes the UI, addon, and QuickJS worker and resolves them from its own directory; it can run outside the checkout.

| Host | Local service and browser UI | Secure environment storage | iOS Simulator | Android Emulator | Local CLI/MCP |
| --- | --- | --- | --- | --- | --- |
| macOS | Source build, localhost checks, and portable bundle exercised; owner-led device and release checks remain open | Keychain | Available with Xcode; owner-led checks remain open | Requires Android Platform Tools and an emulator | Unix control socket |
| Windows | Portability code is present; native build and runtime are unverified | Windows Credential Manager | Not available | Intended through Android Platform Tools; unverified | Current-user named pipe |
| Linux | Portability code is present; native build and runtime are unverified | Secret Service over D-Bus; the user session must provide an unlocked Secret Service | Not available | Intended through Android Platform Tools; unverified | Unix control socket |

“Portability code is present” describes source changes only; it is not a claim that a Windows or Linux build, credential-store session, capture, or device workflow has passed validation. The local CLI uses a private Unix-domain socket on macOS/Linux and a current-user named pipe on Windows. iOS Simulator workflows remain macOS-only. The older Tauri desktop bundle is historical and is not the distribution path for this service.

## Build and run

Install Rust compatible with the workspace `rust-version`, Node.js 20.19+ or 22.12+, pnpm 10.15.0, mitmproxy's `mitmdump`, and Python 3. On Linux, building the Secret Service backend also needs `pkg-config` and the `libdbus-1` development package. For Android work, also install Android Platform Tools and an Android Emulator. iOS Simulator work additionally needs Xcode and an installed Simulator runtime on macOS.

On macOS and Linux:

```sh
pnpm install --frozen-lockfile
./scripts/run-local.sh
```

On Windows PowerShell:

```powershell
pnpm install --frozen-lockfile
.\scripts\run-local.ps1
```

Both launchers build the browser UI and the QuickJS worker, then start the Rust service. Pass `--port 8190` or `--no-open` after the launcher name to change the UI port or suppress opening the browser. The service listens on `127.0.0.1`; capture and SDK ingestion use ports `8181` and `8182`.

The default data directories are:

- macOS: `~/Library/Application Support/dev.mobileapistudio.desktop`
- Windows: `%LOCALAPPDATA%\dev.mobileapistudio.desktop`
- Linux: `$XDG_DATA_HOME/dev.mobileapistudio.desktop`, or `~/.local/share/dev.mobileapistudio.desktop` when `XDG_DATA_HOME` is unset or relative

Back up `app.db` before any schema migration. Stop the service normally so it can end capture and restore an Android Emulator proxy. After a forced stop, use the pending rollback recovery control before starting another capture.

## Portable bundle

After installing the source-build prerequisites, create a bundle for the current host with:

```sh
python3 scripts/package-local.py /path/to/empty/mobile-api-studio
```

The bundle contains the service and QuickJS worker, built UI, mitmproxy addon, Python CLI, and launchers. It can be moved as a directory; the launchers resolve the UI and addon from the bundle root, and the worker is discovered beside the service. mitmproxy, Python, and Android Platform Tools remain host prerequisites. No native installer is produced.

## Local capture and automation

The Connect view discovers local processes and IPv4 interfaces using fixed, read-only OS commands. macOS uses `ps` and `ipconfig`, Windows uses PowerShell, and Linux uses `ps` and `ip`. Discovery failures remain visible without hiding mobile targets. Existing `mac_all`/`mac_process` capture target records retain their canonical wire values; `desktop_all`/`desktop_process` are accepted aliases.

Local process capture uses mitmproxy's native local mode. macOS and Windows can capture desktop traffic; Linux local mode captures outgoing connections, requires its privileged helper and kernel 6.8 or newer, and is unsupported under WSL. The UI gives platform-specific permission guidance. See [mitmproxy modes](https://docs.mitmproxy.org/stable/concepts/modes/). None of these changes alter host-wide proxy settings.

The Windows CLI derives a per-user named-pipe endpoint and checks the server process's token user before sending command arguments, including explicit endpoint overrides. The server restricts its pipe to the current user, rejects remote clients, verifies client identity, and bounds connections, request/response bytes and time. Unix control endpoints retain current-user ownership and 0700/0600 permissions. Windows mitmproxy rule lookup uses a separate authenticated loopback-only, bounded read-only transport; it exposes neither browser commands nor SDK secrets.

## Verification record

Host verification on macOS: 24 app-core checks, 10 service checks, QuickJS worker limits, frontend production build, and Windows CLI mocked API/peer-auth checks passed during integration. The final relocated bundle passed exact UI asset delivery, CLI discovery, workspace exclusivity, graceful shutdown, idle forced-stop restart, live HTTP/WebSocket script hooks and measured network-condition regressions. Windows native pipe/platform and credential adapter modules and the capture engine passed isolated target compilation checks. Linux platform code passed an isolated target check. Full Windows service compilation remains blocked by the missing Windows C toolchain/SDK; full Linux compilation remains blocked by the missing Linux C/D-Bus sysroot. These partial checks do not prove runtime compatibility.

Run `python3 scripts/check-platform-bundle.py /path/to/bundle` to check relocated assets, CLI discovery, workspace exclusivity, normal shutdown and idle forced-stop restart with disposable data. Active-capture crash recovery, platform credential-store sessions, physical devices, Simulator/Emulator workflows, browser interaction and full Windows/Linux capture/replay/rule/import/export recovery matrices remain owner-led acceptance gates. No native installer or externally published release is claimed.

The final synthetic reverse-proxy crash check exposed a surviving mitmproxy listener after service SIGKILL. The capture engine now passes its owner PID to the addon, which monitors Unix reparenting or a retained Windows process handle and shuts down when the owner exits. Windows creation-time checks reject recycled startup PIDs and allow intermediate console launchers. After the fix, actual macOS SIGKILL closed the owned listener; restart, reconnect, synthetic traffic and disconnect passed with exact-child cleanup. `scripts/check-capture-parent-watchdog.py --bundle /path/to/bundle` records this regression; `scripts/check-parent-watchdog-windows.py` checks identity/lifetime branches with mocked APIs. Windows runtime, active-device rollback and the full recovery matrix remain unverified.

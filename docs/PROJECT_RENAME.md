# SimulatorApiFlow rename

Mobile API Studio is now **SimulatorApiFlow**. The project remains the same local-first, open-source API debugger for iOS Simulators and Android Emulators. This rename does not change its validation or release status.

The canonical repository is [DagerottDev/SimulatorApiFlow](https://github.com/DagerottDev/SimulatorApiFlow). Existing GitHub repository links redirect to the renamed repository.

## Update source integrations

- Rust packages and binaries now use `simulator-api-flow`, `simulator-api-flow-server`, `simulator-api-flow-script-worker`, and `simulator-api-flow-sharing-server`.
- The JavaScript workspace is `simulator-api-flow`; the frontend package is `@simulator-api-flow/desktop`.
- The CLI is `scripts/saf-cli.py`; `scripts/mas-cli.py` remains a compatibility entry point. MCP discovery advertises `simulator_api_flow` and still accepts the legacy `mobile_api_studio` tool name.
- The capture addon is `sidecars/mitm-addon/saf_bridge.py`. Update an explicit `MAS_ADDON_PATH` override if it points into this source checkout.
- Swift integrations use `sdks/ios/SimulatorApiFlow`, `import SimulatorApiFlow`, and the `SimulatorApiFlow` types.
- Android integrations use `sdks/android/simulator-api-flow`, Gradle module `:simulator-api-flow`, package `dev.simulatorapiflow.sdk`, and the `SimulatorApiFlow` types.
- Sample apps and their Xcode project use the new name. Existing apps must update SDK source paths, imports, and type names before rebuilding.

## Compatibility identifiers

The following identifiers deliberately retain their original spelling so the rename does not hide saved data, orphan secrets, invalidate exports, or change security-sensitive protocol handling:

- The application/data identity `dev.mobileapistudio.desktop`, including the Tauri identifier and existing platform data directories.
- The secure-store service `dev.mobileapistudio.environment`.
- The Windows control-pipe prefix `mobile-api-studio-control-`.
- The SDK correlation header `X-Mobile-API-Studio-Request-Id`, its redaction/filtering rules, and pairing/authentication headers such as `X-MAS-Token` and `X-MAS-Pairing-Token`.
- Existing `MAS_*` environment variables, capture event/metadata keys, and `.mas.json` workspace files.

No database, secret, certificate, or proxy-setting migration is required. Historical evidence paths and generated build artifacts retain their original names.

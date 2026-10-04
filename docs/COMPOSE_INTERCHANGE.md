# Compose and interchange — milestone 5

> **Integrated on main:** This milestone is part of PRs #19–26. For current usage see [USAGE.md](USAGE.md); later acceptance and owner exclusions are in [ROADMAP_COMPLETION.md](ROADMAP_COMPLETION.md). Verification/pending lists below retain their original checkpoint scope.

This branch is based on the network-conditions milestone. Source checks passed. Browser, native/device and release acceptance remain owner-led.

The request editor can start blank, edit a captured/saved request, replace headers and URL/query, edit raw/JSON/form/multipart text or a binary/base64 body, and save a draft into an existing collection. Editing the complete URL edits query parameters. Multipart field helpers handle text fields; arbitrary multipart payloads can be supplied as a complete binary body with an explicit boundary/content type.

Repeats accept 1–100 requests, concurrency 1–4, intervals up to 60 seconds and a start schedule of at most four minutes. The run stops after five minutes. Already sent requests can have effects; a request interrupted at the deadline may have reached the target. Completed requests, including transport failures, are individually stored. Pending deadline outcomes must not be treated as confirmed unsent requests.

Bodies are limited to 2 MiB, headers to 100, and URL metadata is bounded. Execution requires HTTP(S) without embedded credentials or a fragment. Existing environment and secret interpolation remain in the backend. An explicit Content-Type header takes precedence over body metadata.

Interchange supports: cURL, HAR 1.2, explicit Postman v2.1 JSON and CSV. Imports are previewed before an explicit send/save/traffic-import action. Export previews include only selected flow/saved-request IDs. Known secret headers are redacted, but application bodies may contain sensitive data and need review before downloading. Native Charles/Proxyman formats remain conditional on documented versions and representative fixtures; do not invent support.

## Current verification

- Frontend TypeScript and production build passed.
- All 18 app-core checks passed, including real loopback binary request repeats and transport-failure persistence.
- Selected HAR/CSV binary roundtrips, export header redaction, SDK metadata omission, read-only malformed previews, cURL shell/file rejection and nested Postman parsing passed.
- Existing replay redirect checks passed (2).
- Workspace compilation passed; existing unused-code warnings remain.
- Browser interaction was unverified at this source-check checkpoint because security policy blocked the localhost preview. Later owner-authorized checks passed bounded Compose/send/repeat/cURL-preview and captured/saved/imported Replay loading workflows; see [PLATFORM_ACCEPTANCE.md](PLATFORM_ACCEPTANCE.md). This does not establish every interchange format or UI control. Current browser permission blocks must still be respected.
- Native, physical-device and release acceptance remain owner-led.

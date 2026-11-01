# Rust/Tauri desktop migration

## Decision

OpenWork Desktop uses **Rust + Tauri 2** for its native shell. The React/Vite interface remains shared with the web app. The OpenWork server and OpenCode engine remain separate processes for now; this is a native-shell migration, not a rewrite of the application and inference stack in Rust.

Tauri uses the operating system's WebView instead of bundling Electron's Chromium runtime. This should reduce the desktop shell's install footprint and baseline memory on supported systems, but no size or memory claim is considered proven until the release builds are measured on the same machines.

## What this branch changes

- Restores the previous Rust/Tauri host and its workspace, engine, server, deep-link, updater, file, and window commands.
- Makes `pnpm dev` and the desktop package's normal `build` target use Tauri.
- Keeps Electron commands under explicit `dev:electron`, `build:electron`, and `package:electron` names as a temporary rollback path.
- Detects Tauri as a desktop runtime in the shared app.
- Routes the current desktop IPC facade through the restored Rust bridge for the commands that have a compatible implementation.
- Builds the current `openwork-server` CLI binary before Tauri dev/build, then copies that binary into the Tauri sidecar directory. This is needed because the current Electron host embeds the server library in-process, while the restored Rust host launches the supported CLI sidecar.

## Not complete: do not ship as a drop-in replacement yet

The restored Rust host predates the current Electron desktop API. Static comparison found only about 30 overlapping command methods in the old bridge versus roughly 90 in the current command contract. Methods that have no Rust implementation fail explicitly instead of returning fabricated success.

The following areas still need parity work and end-to-end coverage:

- Built-in browser tabs, web contents, browser automation and tab ownership.
- PTY/terminal sessions and lifecycle events.
- Computer-use control, dictation, OS notifications, keep-awake and application integration.
- Current connection/organization policy hooks, newer recovery and desktop-control commands.
- Orchestrator/sandbox lifecycle. The historical Tauri shell expects an orchestrator sidecar that is no longer built by the current workspace.
- Packaging, signing, deep-link/updater behavior and data migration on macOS, Windows and Linux.

The UI still contains some Electron-specific bridge checks. Tauri detection is added, but not every direct Electron reference has been replaced yet. The migration should not be considered complete until those references are either moved behind a host-neutral API or deliberately implemented in the Rust host.

## Build and development

- `pnpm dev` — start the Tauri desktop app.
- `pnpm --filter @openwork/desktop dev:electron` — temporary Electron fallback.
- `pnpm --filter @openwork/desktop build:tauri` — build the Tauri desktop app.
- `pnpm --filter openwork-server build:bin` — build the current server CLI sidecar used by Tauri.

Tauri development/build requires Rust/Cargo, the Tauri v2 system prerequisites (including WebKitGTK development packages on Linux), and Bun for the OpenWork server binary. See the Tauri documentation for platform-specific prerequisites.

## Performance validation before removing Electron

Compare equivalent clean builds on the same OS and hardware. Record:

1. Installed application size, excluding user data.
2. Cold-start time until the first usable workspace.
3. Idle resident memory after startup and after opening a representative chat.
4. CPU and memory while streaming a response and while the built-in browser is open.
5. Wake-up time after suspend and workspace switching.

Keep the same model, server, workspace and UI workload for both builds. Do not infer a win from installer size alone; the WebView and server still contribute to runtime memory.

## Exit criteria for deleting Electron

Delete the Electron runtime and its dependencies only after all current desktop IPC methods and direct native bridge usage have a Tauri implementation, the migration and existing user data are verified, and packaged builds pass on macOS, Windows and Linux. Then remove the Electron scripts, dependencies, tests, and Electron-only files in a dedicated cleanup change.

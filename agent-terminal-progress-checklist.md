# Agent Terminal Progress Checklist

Last updated: 2026-03-04

## 0) Priority Order (User-Directed)

1. [ ] Native workspace tab UI (visual tab strip is implemented; tab switching behavior still pending)
2. [ ] Visible split pane UI rendering + focus/geometry reconciliation
3. [ ] `wgpu` backend completion + webview texture composition
4. [x] Notification plumbing and acknowledgements (kept minimal, mostly done)
5. [x] Windows build validation path

## 1) Progress Summary

- [x] Foundation for an agent-oriented control plane is implemented.
- [x] v2 IPC transport and method routing are implemented.
- [x] Runtime identity model (`window:N`, `workspace:N`, `surface:N`) is implemented.
- [x] Notification store with target dedupe and unread tracking is implemented.
- [x] Split semantics model (`left/right/up/down`) and `surface.split` API are implemented.
- [x] Webview surface registry APIs (`webview.open/list/navigate/close`) are implemented.
- [x] PTY environment identity injection is implemented.
- [x] Agent UI status banner in message bar is implemented (workspace tab summary + recent notifications).
- [x] Native top tab strip lane is rendered in-window (always visible, workspace summary).
- [x] Agent notification/ack messages are implemented with target-aware replacement.
- [x] Window-title Agent summary is implemented (unread + workspace tab indicators).
- [x] Window urgency now tracks unread state and auto-clears when notifications are read.
- [x] Closing `[Agent #...]` message-bar notifications auto-acknowledges latest unread notification in that workspace.
- [x] Build validation (`cargo fmt`, `cargo check`) passes.
- [x] Windows target checks pass for `x86_64-pc-windows-msvc` and `x86_64-pc-windows-gnu`.
- [ ] Real split pane UI rendering is not implemented yet.
- [ ] Real embedded webview rendering (Servo + wgpu texture composition) is not implemented yet.
- [ ] Full `wgpu` render backend implementation is not implemented yet.

## 2) Checklist Against Plan

### Phase A: Data Model and Identity

- [x] Add runtime ID helpers (`window/workspace/surface` refs).
- [x] Add system tree snapshot model and serialization.
- [x] Add handle resolver (`resolve.handle`).
- [x] Add v2 IPC request/response envelope.

### Phase B: Split Engine Reliability

- [x] Add split direction semantics and orientation mapping.
- [x] Add `surface.split` API to mutate workspace split tree state.
- [ ] Hook split tree into live UI pane layout.
- [ ] Add focus reconcile loop after split mutations.
- [ ] Add geometry reconcile loop after split mutations.

### Phase C: Notification State Machine

- [x] Add notification target model `(workspace_id, surface_id?)`.
- [x] Add per-target dedupe behavior.
- [x] Add unread counting by workspace.
- [x] Add `notification.create/list/mark_read` APIs.
- [x] Add in-app Agent notification banner updates with replacement semantics.
- [ ] Add focus-aware suppression policy.
- [x] Add macOS OS notification center fan-out (`osascript`).

### Phase D: Agent Integration Layer

- [x] Inject `ALACRITTY_WORKSPACE_ID` into shell env.
- [x] Inject `ALACRITTY_SURFACE_ID` into shell env.
- [x] Inject `ALACRITTY_SOCKET` into shell env when available.
- [ ] Session mapping store (`session_id -> workspace/surface`) is not implemented.
- [ ] Transcript-based completion summarization is not implemented.

### Phase E: Socket Focus Policy and Control Plane

- [x] Add v2 method dispatch over existing socket transport.
- [x] Add `identify` and `system.tree`.
- [x] Implement `workspace.switch` focus intent method.
- [x] Keep explicit gate for unimplemented focus-intent methods (`surface.focus`, `notification.open`).
- [x] Keep legacy IPC messages (`create-window`, `config`, `get-config`) intact.

### Phase F: `wgpu` + Webview Path

- [x] Add backend selection abstraction (`Auto`, `Wgpu`, `Gl`).
- [x] Add runtime fallback behavior when `Wgpu` is requested but unavailable.
- [x] Add webview surface store and control-plane methods.
- [ ] Implement actual `wgpu` terminal renderer.
- [ ] Implement Servo offscreen embedder runtime.
- [ ] Composite webview textures with terminal content in GPU pipeline.

### Phase G: Regression and Validation

- [x] `cargo fmt` run.
- [x] `cargo check` run and passing.
- [x] `cargo check -p alacritty --target x86_64-pc-windows-msvc` run and passing.
- [x] `cargo check -p alacritty --target x86_64-pc-windows-gnu` run and passing.
- [x] Added unit tests for new runtime helper modules.
- [ ] Add integration tests for v2 control plane methods.
- [ ] Add split churn/multi-window routing regression tests.

## 3) Build Into a Terminal Binary

From repo root:

```bash
# Debug build
cargo build -p alacritty

# Release build
cargo build -p alacritty --release
```

Run the built terminal:

```bash
# Debug binary
./target/debug/alacritty

# Release binary
./target/release/alacritty
```

Optional `wgpu` preference test (currently falls back to OpenGL):

```toml
# ~/.config/alacritty/alacritty.toml
[debug]
render_backend = "Wgpu"
```

Expected today: startup succeeds and logs that it is falling back to OpenGL backend.

## 4) Feature Test Commands

Use one daemon instance with a known socket:

```bash
SOCK=/tmp/alacritty-agent.sock
./target/release/alacritty --daemon --socket "$SOCK"
./target/release/alacritty msg --socket "$SOCK" create-window
```

### 4.1 Identify and System Tree

```bash
./target/release/alacritty msg --socket "$SOCK" v2 --method identify
./target/release/alacritty msg --socket "$SOCK" v2 --method system.tree
./target/release/alacritty msg --socket "$SOCK" v2 --method workspace.switch --params '{"workspace_id":"workspace:1"}'
```

Expected:

- `identify` returns JSON with pid, version, socket, and backend preference.
- `system.tree` returns windows/workspaces/surfaces with stable refs.
- `workspace.switch` focuses the target workspace (supports exact refs and short refs).

### 4.2 Notification APIs

```bash
./target/release/alacritty msg --socket "$SOCK" v2 --method notification.create --params '{"workspace_id":"workspace:1","title":"Agent done","body":"Review output"}'
./target/release/alacritty msg --socket "$SOCK" v2 --method notification.list
./target/release/alacritty msg --socket "$SOCK" v2 --method notification.mark_read --params '{"id":0}'
```

Expected:

- Create inserts or replaces by target and shows `[Agent #N | workspace:X]` in the message bar.
- Window title updates with an `Agent[...]` suffix summarizing unread count and tab indicators.
- List shows notification records.
- Mark-read flips `read=true` and shows an acknowledgement message in the target workspace.
- Clicking `[X]` on the `[Agent #N ...]` message also acknowledges the latest unread item for that workspace.

### 4.3 Split Tree API (State-Level)

```bash
./target/release/alacritty msg --socket "$SOCK" v2 --method surface.split --params '{"workspace_id":"workspace:1","target_surface_id":"surface:1","direction":"right"}'
./target/release/alacritty msg --socket "$SOCK" v2 --method system.tree
```

Expected:

- `surface.split` returns updated surface references.
- `system.tree` reflects updated surface IDs in workspace model.
- No visible pane UI split yet (state-level implementation only).

### 4.4 Webview Surface APIs (State-Level)

```bash
./target/release/alacritty msg --socket "$SOCK" v2 --method webview.open --params '{"workspace_id":"workspace:1","url":"https://alacritty.org"}'
./target/release/alacritty msg --socket "$SOCK" v2 --method webview.list
```

Copy the returned `id` (for example `surface:10000000`), then:

```bash
./target/release/alacritty msg --socket "$SOCK" v2 --method webview.navigate --params '{"id":"surface:10000000","url":"https://example.com"}'
./target/release/alacritty msg --socket "$SOCK" v2 --method webview.close --params '{"id":"surface:10000000"}'
```

Expected:

- Open/list/navigate/close work through JSON responses.
- No embedded rendered browser pane yet (registry/control-plane implementation only).

## 5) Current Known Gaps

- Split operations currently mutate runtime model, not actual rendered pane layout.
- Webview methods currently manage logical surfaces only.
- `render_backend = "Wgpu"` is currently selection/fallback plumbing, not full `wgpu` rendering.
- Focus-intent APIs (`surface.focus`, `notification.open`) are intentionally blocked as pending.
- The Agent UI is currently message-bar based; there is no native sidebar/tab strip UI yet.

## 6) Windows Build and Validation

Install Rust Windows targets:

```bash
make windows-targets-install
```

Validate compile health for Windows targets:

```bash
make check-windows
```

Build on a native Windows machine:

```bash
make build-windows-host
```

CI/release coverage:

- `.github/workflows/ci.yml` runs on `windows-latest`
- `.github/workflows/release.yml` builds Windows portable `.exe` and `.msi`

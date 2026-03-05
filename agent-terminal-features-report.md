# Agent-Terminal Feature Analysis Report (cmux)

This report captures the core architecture and implementation details behind cmux’s “terminal for agents” model, with emphasis on:

- Vertical tab + split-based multitasking
- Agent completion/attention notifications
- Socket/CLI-driven automation for parallel agent workflows
- Edge cases and reliability patterns needed for reimplementation in another terminal

## 1) Product Model: "Terminal for Agents"

cmux is built as a native macOS terminal centered on **parallel agent sessions**. The key primitives are:

- **Workspaces (vertical tabs in sidebar)** for top-level task separation
- **Panes (split layout)** inside each workspace for concurrent terminals/browser surfaces
- **Surfaces** (terminal or browser) as executable/interactive units
- **Notification routing** to the exact workspace/surface needing attention
- **Scriptable control plane** (CLI + Unix socket API)

Primary product framing appears in:

- [README.md](/Users/sridatta.bandreddi/Desktop/code/cmux/README.md)

## 2) Core Architecture

Model hierarchy:

- `Window -> Workspace -> Pane -> Surface`

Core implementation surfaces:

- Workspace/split lifecycle: [Sources/Workspace.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/Workspace.swift)
- Workspace registry/focus flow: [Sources/TabManager.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TabManager.swift)
- Socket/CLI methods and policy: [Sources/TerminalController.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalController.swift)
- App-level routing/open/fallback logic: [Sources/AppDelegate.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/AppDelegate.swift)

## 3) Vertical Tabs + Split Multitasking

### 3.1 Workspace tabs (vertical sidebar)

The sidebar is the workspace list (vertical tabs), not just cosmetic navigation:

- Shows per-workspace unread count, latest notification text, git/PR/cwd/ports metadata
- Supports workspace reorder, move across windows, pinning

Key files:

- [Sources/ContentView.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/ContentView.swift)
- [Sources/TabManager.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TabManager.swift)

### 3.2 Split direction semantics

Split direction is explicit and deterministic:

- `left/right -> horizontal split`
- `up/down -> vertical split`
- `insertFirst` controls which side receives new pane (left/top vs right/bottom)

Key locations:

- `SplitDirection.orientation` and `insertFirst`: [Sources/TabManager.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TabManager.swift)
- Direction parsing: [Sources/TerminalController.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalController.swift)
- Split creation APIs: [Sources/Workspace.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/Workspace.swift)

### 3.3 Programmatic vs UI split behavior

cmux distinguishes:

- **Programmatic splits** (`newTerminalSplit`, `newBrowserSplit`) that provide their own panel mapping
- **UI splits** where delegate callbacks may need to auto-create/repair content

Critical reliability behavior:

- Pre-generate split tab IDs and mappings before Bonsplit mutates layout, to avoid transient empty flashes
- Suppress old view first-responder side effects during split reparenting
- Run focus and geometry reconciliation passes after split churn

Key implementation:

- `newTerminalSplit`, `newBrowserSplit`: [Sources/Workspace.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/Workspace.swift)
- `scheduleFocusReconcile`, `reconcileTerminalGeometryPass`: [Sources/Workspace.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/Workspace.swift)
- `didSplitPane` placeholder repair logic: [Sources/Workspace.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/Workspace.swift)

## 4) Notification System (Agent Attention + Completion)

### 4.1 Notification sources

Notifications can enter from:

- Terminal OSC actions (desktop notification actions)
- Socket/CLI methods (`notification.create`, `notify_target`, etc.)
- Agent hooks (`cmux claude-hook ...`, `cmux notify`)

Key files:

- [Sources/GhosttyTerminalView.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/GhosttyTerminalView.swift)
- [Sources/TerminalController.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalController.swift)
- [CLI/cmux.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/CLI/cmux.swift)

### 4.2 Store model + dedupe policy

`TerminalNotificationStore` is the central state machine.

Core behavior:

- Stores `tabId` (workspace), optional `surfaceId`, title/subtitle/body, createdAt, read state
- **Dedupes by target** by removing prior notification for same `(tabId, surfaceId)` before inserting new
- Maintains unread indexes for fast badge/ring checks

Key file:

- [Sources/TerminalNotificationStore.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalNotificationStore.swift)

### 4.3 Suppression rules

Notification is suppressed if:

- app is focused **and**
- target tab/panel is currently focused

This avoids alerting when user is already looking at the panel.

Important detail:

- App focus checks only count main terminal windows for suppression, not arbitrary auxiliary windows.

Key function:

- `AppFocusState.isAppFocused()`: [Sources/TerminalNotificationStore.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalNotificationStore.swift)

### 4.4 Visual + system surfaces

Each notification fans out to:

- **Pane ring** (blue ring)
- **Sidebar unread badge/count**
- **Notifications panel list**
- **macOS UNUserNotificationCenter** notification

Key files:

- Ring/flash: [Sources/GhosttyTerminalView.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/GhosttyTerminalView.swift)
- Sidebar unread + latest text: [Sources/ContentView.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/ContentView.swift)
- Notification list panel: [Sources/NotificationsPage.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/NotificationsPage.swift)

### 4.5 Open / jump behavior

User actions (click banner, click panel row, jump-to-latest-unread) resolve target context and focus it.

Flow:

1. Resolve window context owning `tabId`
2. If missing context, use fallback routing
3. Focus target workspace/surface
4. Trigger focus flash
5. Mark notification read when focus is confirmed

Key functions:

- `jumpToLatestUnread`
- `openNotification`
- `openNotificationInContext`
- `openNotificationFallback`
- `markReadIfFocused`

All in:

- [Sources/AppDelegate.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/AppDelegate.swift)

## 5) Agent Hooking + Completion Context

### 5.1 Shell/environment surface identity

Each terminal surface injects env vars for deterministic targeting:

- `CMUX_WORKSPACE_ID`
- `CMUX_SURFACE_ID`
- compatibility keys `CMUX_TAB_ID`, `CMUX_PANEL_ID`
- `CMUX_SOCKET_PATH`

Source:

- [Sources/GhosttyTerminalView.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/GhosttyTerminalView.swift)

### 5.2 Claude wrapper + hook injection

Wrapper script behavior:

- Pass-through when outside cmux
- Inside cmux, injects Claude hook commands (`session-start`, `notification`, `stop`, `prompt-submit`)
- Injects/generated session IDs unless user already passes resume/session flags

Source:

- [Resources/bin/claude](/Users/sridatta.bandreddi/Desktop/code/cmux/Resources/bin/claude)

### 5.3 Session mapping store

`ClaudeHookSessionStore` maps `session_id -> workspaceId/surfaceId + context`, persisted with lockfile semantics.

Features:

- Upsert/lookup/consume mapping
- Fallback resolution by workspace/surface when session_id absent
- 7-day stale session pruning

Source:

- [CLI/cmux.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/CLI/cmux.swift)

### 5.4 Completion summarization

On `stop`, cmux attempts enriched completion messages:

- Uses transcript JSONL if available (last assistant message)
- Falls back to stored session context (`cwd`, last message)
- Routes notification to mapped workspace/surface target

Source:

- `summarizeClaudeHookStop`, `summarizeClaudeHookNotification` in [CLI/cmux.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/CLI/cmux.swift)

## 6) Socket/CLI Multitasking Design

### 6.1 Focus policy: no accidental focus steal

Socket commands execute under a policy gate:

- Only explicit focus-intent methods may mutate in-app focus/activation
- Non-focus methods must preserve user focus context

This is key for background agent orchestration.

Key implementation:

- `focusIntentV1Commands`
- `focusIntentV2Methods`
- `withSocketCommandPolicy`

in:

- [Sources/TerminalController.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalController.swift)

Audit/design doc:

- [docs/socket-focus-steal-audit.todo.md](/Users/sridatta.bandreddi/Desktop/code/cmux/docs/socket-focus-steal-audit.todo.md)

### 6.2 Stable machine-facing references

v2 API provides stable short refs (`window:1`, `workspace:2`, etc.) in addition to UUIDs, reducing friction for agent callers.

Key functions:

- `v2EnsureHandleRef`
- `v2Identify`
- `v2SystemTree`

in:

- [Sources/TerminalController.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalController.swift)

### 6.3 Browser + terminal mixed multitasking

`browser.openSplit` strategy:

- Reuse right sibling pane when suitable, otherwise create right split
- Supports external-open rules if configured

Source:

- `v2BrowserOpenSplit` in [Sources/TerminalController.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalController.swift)

## 7) Performance/Threading Patterns to Copy

### 7.1 Off-main expensive notification center cleanup

UNUserNotificationCenter removal operations can block; cmux dispatches delivered/pending removal off-main.

Source:

- [Sources/TerminalNotificationStore.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalNotificationStore.swift)

### 7.2 Telemetry fast path dedupe off-main

For high-frequency shell telemetry (pwd/metadata):

- Validate/normalize off-main
- Drop duplicates off-main
- Schedule minimal main-thread mutation only if changed

Key items:

- `SocketFastPathState.shouldPublishDirectory`
- `explicitSocketScope`
- `reportPwd`

in:

- [Sources/TerminalController.swift](/Users/sridatta.bandreddi/Desktop/code/cmux/Sources/TerminalController.swift)

## 8) Edge Cases and Failure Modes (Must-Have)

### 8.1 Notification edge cases

- Notification storm on same surface: replace prior target notification
- Suppression false positives when app active but not truly focused in main terminal window
- mark-read race after focus switch: use delayed confirmation (`markReadIfFocused`)
- stale delivered/pending OS notifications after in-app read/clear
- clear on workspace close and per-surface close

### 8.2 Split/layout edge cases

- Nested split can transiently collapse/disappear sibling panes
- Drag-to-split of single tab may create placeholder-only pane; needs repair to real terminal
- Rapid split/close cycles can produce blank terminal panes
- Reparenting can trigger incorrect focus callbacks and divergence
- Post-layout geometry desync (AppKit bounds vs Ghostty surface size)

### 8.3 Multi-window routing edge cases

- Notification open when owning window context not yet registered
- Workspace moved across windows must preserve surface identity and focus routing
- Jump-to-unread during startup/context lag

### 8.4 Agent-hook edge cases

- Missing/ambiguous session_id mapping
- stale mapping file entries
- transcript unavailable for stop summary fallback
- user-provided session/resume flags must not conflict with wrapper injection

## 9) Test Coverage Patterns Worth Replicating

Notification behavior regressions:

- [tests/test_notifications.py](/Users/sridatta.bandreddi/Desktop/code/cmux/tests/test_notifications.py)

Split stability regressions:

- [tests/test_tab_dragging.py](/Users/sridatta.bandreddi/Desktop/code/cmux/tests/test_tab_dragging.py)
- [tests/test_nested_split_no_arranged_subview_underflow.py](/Users/sridatta.bandreddi/Desktop/code/cmux/tests/test_nested_split_no_arranged_subview_underflow.py)
- [tests/test_nested_split_does_not_disappear.py](/Users/sridatta.bandreddi/Desktop/code/cmux/tests/test_nested_split_does_not_disappear.py)

Multi-window identity stability:

- [tests_v2/test_windows_api.py](/Users/sridatta.bandreddi/Desktop/code/cmux/tests_v2/test_windows_api.py)

Agent hook mapping + completion notification behavior:

- [tests/test_claude_hook_session_mapping.py](/Users/sridatta.bandreddi/Desktop/code/cmux/tests/test_claude_hook_session_mapping.py)

## 10) Reimplementation Blueprint for Another Terminal

### Phase 1: Data model and identity

Implement canonical IDs and graph:

- Window, workspace, pane, surface IDs
- Stable API refs + UUIDs
- Query APIs: identify + full system tree

### Phase 2: Split engine reliability

Implement:

- Direction semantics (`left/right/up/down`)
- Programmatic/UI split distinction
- Focus reconcile and geometry reconcile loops
- Placeholder repair on drag-to-split anomalies

### Phase 3: Notification state machine

Implement:

- Per-target dedupe (`workspaceId + surfaceId`)
- Focus-aware suppression
- Read/unread indexes
- OS notification fan-out + cleanup paths
- Click/jump routing with fallback context resolution

### Phase 4: Agent integration layer

Implement:

- Surface/workspace env injection
- Hook wrapper(s) for target agent CLIs
- Session mapping store with expiry
- Completion summarization from transcript + fallback context

### Phase 5: Socket focus policy + threading

Implement:

- Focus-intent allowlist policy for commands
- Off-main parsing/dedupe for telemetry
- Minimal main-thread UI mutation

### Phase 6: Regression harness

Port tests for:

- Notification suppression/read logic
- Split churn and nested split stability
- Multi-window workspace move identity stability
- Hook/session notification routing

## 11) Key Design Principles Extracted

1. **Identity-first routing**: every notification and command targets explicit workspace/surface identities.
2. **Attention should be contextual**: include meaningful body/subtitle, not generic "agent waiting" noise.
3. **No hidden focus side effects**: automation must not disrupt active user flow.
4. **UI resilience under churn**: split/reparent operations need explicit repair/reconcile mechanisms.
5. **Fast path discipline**: dedupe/coalesce noisy signals before touching main-thread UI state.
6. **State + visuals are coupled**: unread model must consistently drive ring, badge, panel, and OS notifications.

## 12) Additional Compatibility Note

cmux currently carries a Ghostty fork patch for kitty OSC 99 parsing, so terminals reimplementing this model should ensure robust OSC notification compatibility (99/777 + chunking behavior).

Reference:

- [docs/ghostty-fork.md](/Users/sridatta.bandreddi/Desktop/code/cmux/docs/ghostty-fork.md)


# Tacoshell — State Management

## 1. Overview

All application state is managed through five independent Zustand stores. Stores communicate via subscriptions — no store directly imports another. The most complex piece of state is the tab tree, which models arbitrarily nested split panes as a binary tree.

---

## 2. Store Inventory

| Store | File | Responsibility |
|-------|------|----------------|
| `useAuthStore` | `stores/useAuthStore.ts` | GitHub OAuth state, user profile, token |
| `useVaultStore` | `stores/useVaultStore.ts` | Decrypted vault contents, sync status |
| `useConnectionStore` | `stores/useConnectionStore.ts` | Active connections, state machine per session |
| `useTabStore` | `stores/useTabStore.ts` | Tab tree, split configuration, active tab tracking |
| `useSettingsStore` | `stores/useSettingsStore.ts` | App preferences, theme, keybindings |

---

## 3. Tab Tree — Data Model

The tab workspace is modeled as a binary tree where every node is either a `SplitNode` or a `PaneNode`.

### 3.1 Types

```typescript
type Direction = 'horizontal' | 'vertical'

type TabType = 'terminal' | 'sftp' | 'k8s' | 'welcome'

interface Tab {
  id: string              // uuid-v4
  connectionId: string | null
  type: TabType
  title: string
  isDirty: boolean        // unsaved changes indicator
}

interface PaneNode {
  type: 'pane'
  id: string
  tabs: Tab[]
  activeTabId: string | null
}

interface SplitNode {
  type: 'split'
  id: string
  direction: Direction
  ratio: number           // 0.0–1.0, position of the divider
  first: TabNode
  second: TabNode
}

type TabNode = PaneNode | SplitNode
```

### 3.2 Example Tree

A workspace with one horizontal split, and the right side further split vertically:

```
SplitNode (horizontal, ratio: 0.5)
├── PaneNode [tab: "server-01 (SSH)"]
└── SplitNode (vertical, ratio: 0.6)
    ├── PaneNode [tab: "server-02 (SSH)", tab: "server-03 (SFTP)"]
    └── PaneNode [tab: "prod-cluster (k8s)"]
```

---

## 4. Tab Tree — Operations

### 4.1 Split Pane

Replaces a `PaneNode` with a `SplitNode` containing the original pane and a new empty pane.

```typescript
splitPane(paneId: string, direction: Direction): void
```

Before:
```
PaneNode [tab: "server-01"]
```

After `splitPane(paneId, 'horizontal')`:
```
SplitNode (horizontal, ratio: 0.5)
├── PaneNode [tab: "server-01"]  ← original pane, preserved
└── PaneNode []                  ← new empty pane, becomes active
```

### 4.2 Close Pane

When a pane is closed (last tab removed), the sibling pane takes the parent's position.

```typescript
closePane(paneId: string): void
```

Before:
```
SplitNode (horizontal)
├── PaneNode [tab: "server-01"]  ← closed
└── PaneNode [tab: "server-02"]
```

After `closePane(leftPaneId)`:
```
PaneNode [tab: "server-02"]  ← sibling promoted to root
```

### 4.3 Move Tab Between Panes

Removes a tab from its current pane and appends it to a target pane.

```typescript
moveTab(tabId: string, fromPaneId: string, toPaneId: string): void
```

If `fromPaneId` becomes empty after the move, `closePane` is called automatically.

### 4.4 Resize Split

Adjusts the divider ratio on a `SplitNode`.

```typescript
resizeSplit(splitId: string, ratio: number): void
```

### 4.5 Add Tab

Adds a new tab to a specific pane (or the active pane if no pane specified).

```typescript
addTab(tab: Omit<Tab, 'id'>, paneId?: string): string  // returns new tab id
```

### 4.6 Close Tab

Removes a tab from its pane. If the pane becomes empty, closes the pane.

```typescript
closeTab(tabId: string): void
```

---

## 5. Connection State Machine

Each active connection follows a strict state machine managed in `useConnectionStore`:

```
IDLE
  │
  │ connect()
  ▼
CONNECTING
  │
  ├─ success ──► AUTHENTICATING
  │                    │
  │                    ├─ success ──► CONNECTED ──► DISCONNECTING ──► DISCONNECTED
  │                    │                  │                │
  │                    └─ failure ──► ERROR    │           └── IDLE (can reconnect)
  │                                        │
  └─ failure ──► ERROR                    timeout/drop
                    │                        │
                    └─ reconnect() ──► RECONNECTING
                                            │
                                            └── (back to CONNECTING)
```

### State Shape

```typescript
type ConnectionStatus =
  | 'idle'
  | 'connecting'
  | 'authenticating'
  | 'connected'
  | 'reconnecting'
  | 'disconnecting'
  | 'disconnected'
  | 'error'

interface ActiveConnection {
  id: string                   // uuid-v4, the session ID
  profileId: string            // reference to vault profile
  status: ConnectionStatus
  error: string | null
  connectedAt: Date | null
  reconnectAttempts: number
}
```

---

## 6. Store Communication Pattern

Stores never directly import each other. They communicate via subscriptions set up in a `StoreOrchestrator` (initialized once at app startup):

```typescript
// src/stores/orchestrator.ts

export function initStoreOrchestration() {
  // When a connection disconnects, mark its tab title as disconnected
  useConnectionStore.subscribe(
    (state) => state.connections,
    (connections) => {
      const { updateTabTitle } = useTabStore.getState()
      for (const conn of Object.values(connections)) {
        if (conn.status === 'disconnected') {
          updateTabTitle(conn.id, (title) => `[disconnected] ${title}`)
        }
      }
    }
  )

  // When vault sync completes, refresh the sidebar profile list
  useVaultStore.subscribe(
    (state) => state.lastSyncAt,
    () => {
      // sidebar reacts to vault store directly — no explicit signal needed
    }
  )
}
```

---

## 7. State Persistence

| Store | Persisted | Where | Encrypted |
|-------|-----------|-------|-----------|
| `useAuthStore` | OAuth token only | OS Keychain / Keystore | Yes (OS-managed) |
| `useVaultStore` | Full vault | GitHub private repo (remote), IndexedDB/app_data (local cache) | Yes (AES-256-GCM) |
| `useConnectionStore` | Nothing | In-memory only | N/A |
| `useTabStore` | Layout (tree structure, no tab content) | Local app settings file | No |
| `useSettingsStore` | All preferences | Local app settings file | No |

Tab layout is persisted so the workspace is restored between sessions. Connection sessions themselves are not persisted — connections must be re-established on app launch.

---

## 8. React Integration

Components access stores using Zustand's selector pattern to minimize re-renders:

```typescript
// Only re-renders when the active tab changes, not on any store update
const activeTab = useTabStore((s) => {
  const pane = findPaneById(s.root, s.activePaneId)
  return pane?.tabs.find((t) => t.id === pane.activeTabId) ?? null
})
```

Heavy operations (tree traversals, vault decryption) are memoized with `useMemo` or computed outside React using Zustand's `getState()`.

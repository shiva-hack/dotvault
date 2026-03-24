# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

dotvault is a **Tauri 2.x desktop app** that discovers, catalogs, compares, and encrypts `.env` files across all your projects. It scans root directories, finds projects, parses their env files, and stores everything in an encrypted local SQLite vault.

## Commands

```bash
npm install                # Install frontend dependencies
npm run tauri dev          # Start Tauri development (requires Rust toolchain)
npm run tauri build        # Production build
npm run dev                # Start Vite dev server only (frontend)
npm run build              # Build frontend only
```

### Prerequisites

- **Rust** (stable, via rustup)
- **Node.js** ≥ 18
- **Tauri CLI**: `cargo install tauri-cli` (or use `npx tauri`)
- **System deps** (Linux): `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev`

## Tech Stack

- **Shell**: Tauri 2.x (Rust backend, webview frontend)
- **Backend**: Rust with rusqlite, aes-gcm, argon2, walkdir, notify
- **Frontend**: React 19 + TypeScript + Tailwind CSS v4 + React Router v7
- **Storage**: SQLite (via rusqlite with bundled feature)
- **Encryption**: AES-256-GCM + Argon2id key derivation

## Architecture

### Rust Backend (`src-tauri/src/`)

```
src-tauri/src/
├── main.rs           # Entry point
├── lib.rs            # Tauri app setup, state management
├── commands.rs       # All #[tauri::command] handlers
├── db/
│   ├── mod.rs        # Database struct, all CRUD operations
│   └── schema.rs     # SQLite CREATE TABLE statements
├── parser/
│   └── mod.rs        # .env file parser + tier detection + ecosystem detection
├── scanner/
│   └── mod.rs        # Recursive filesystem scanner, project discovery
├── crypto/
│   └── mod.rs        # AES-256-GCM encryption, Argon2id key derivation, vault state
├── watcher/
│   └── mod.rs        # File system watcher (notify crate)
└── search/
    └── mod.rs        # Search filters + fuzzy matching
```

### React Frontend (`src/`)

```
src/
├── main.tsx          # React entry point
├── App.tsx           # Root component: routing, vault state, keyboard shortcuts
├── index.css         # Tailwind v4 @theme tokens (design system)
├── lib/
│   └── tauri.ts      # Type definitions + invoke wrappers for all Tauri commands
├── hooks/
│   └── useVault.ts   # Vault state management hook
├── components/
│   ├── Sidebar.tsx   # Navigation sidebar with roots/projects tree
│   ├── Header.tsx    # Top bar with search trigger + scan button
│   ├── SearchPalette.tsx  # Cmd+K search palette
│   └── TierBadge.tsx # Color-coded environment tier badge
└── pages/
    ├── SetupScreen.tsx      # First-time master password creation
    ├── UnlockScreen.tsx     # Vault unlock screen
    ├── ProjectsPage.tsx     # Overview dashboard with all projects
    ├── ProjectDetailPage.tsx # Single project: env hierarchy tree + file table
    ├── EnvFilePage.tsx      # Env file variables with masked/reveal values
    ├── ComparisonPage.tsx   # Cross-environment comparison matrix
    └── SettingsPage.tsx     # Vault settings, password change, export
```

### Design System

Uses Tailwind v4's `@theme` directive. Key tokens:
- `bg-bg` (#09090B) — main background
- `bg-surface` (#18181B) — cards/panels
- `bg-surface-2` (#27272A) — hover states
- `text-accent` (#6366F1) — indigo accent
- `text-green/yellow/red` — status colors
- `font-ui` — Inter/system sans-serif
- `font-code` — JetBrains Mono

### Tauri Command Surface

All frontend-to-backend calls go through `src/lib/tauri.ts` which wraps `invoke()` with typed APIs. Commands are registered in `lib.rs` via `generate_handler![]`.

### Key Design Decisions

- **Values encrypted, keys plaintext**: Variable names are stored unencrypted for instant search and comparison. Only secret values are AES-256-GCM encrypted.
- **Vault state in OnceLock**: The encryption key lives in a process-global `OnceLock<Mutex<VaultState>>` so it's accessible across commands without being part of Tauri's managed state serialization.
- **Stable project IDs**: Projects are identified by UUID, matched by (root_path, name) on re-scan to avoid duplication.
- **Non-blocking scan**: Scanning runs in Tauri's command thread pool. The file watcher uses a separate thread.

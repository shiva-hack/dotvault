# dotvault

> One vault for every secret, across every project, every environment.

dotvault is a lightweight desktop app that discovers, catalogs, compares, and encrypts every `.env` file across all your projects. Point it at your project directories and it recursively finds every environment file — grouped by project, parsed by environment tier, and encrypted at rest with a single master password.

Built with [Tauri 2](https://v2.tauri.app/), Rust, React, and TypeScript.

---

## Features

- **Multi-root scanning** — Register one or more root directories. dotvault recursively discovers all projects (Node.js, Rust, Python, Go, Ruby, PHP, .NET) and their `.env*` files.
- **Dot-depth hierarchy** — Automatically parses `.env.development`, `.env.production.local`, etc. into a structured tree showing environment layering at a glance.
- **Cross-environment comparison** — Matrix view showing which variables exist (or are missing) across environments. Color-coded: green = everywhere, yellow = partial, red = single environment only.
- **Encryption at rest** — All secret values are encrypted with AES-256-GCM. The encryption key is derived from your master password via Argon2id. Variable *names* stay in plaintext for instant search.
- **Global search** — `Cmd/Ctrl+K` opens a search palette. Fuzzy-match across variable names, projects, and environment tiers.
- **Live file watching** — Detects when `.env` files are added, changed, or removed and refreshes automatically.
- **Emergency export** — Decrypt and export all stored env files back to disk as plaintext whenever you need them.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Shell | Tauri 2.x |
| Backend | Rust (rusqlite, aes-gcm, argon2, walkdir, notify) |
| Frontend | React 19 + TypeScript + Tailwind CSS v4 |
| Storage | SQLite (local, embedded) |
| Encryption | AES-256-GCM + Argon2id |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 18
- Tauri system dependencies:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev`
  - **Windows**: [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/), WebView2

### Install & Run

```bash
# Clone the repo
git clone https://github.com/your-username/dotvault.git
cd dotvault

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build for Production

```bash
npm run tauri build
```

The compiled app will be in `src-tauri/target/release/bundle/`.

## Usage

1. **Create a master password** — On first launch, set a strong password. This encrypts all your secret values. It's never stored anywhere.
2. **Add a root directory** — Click "Add Root" and pick a folder (e.g. `~/projects`). dotvault scans it recursively.
3. **Browse your projects** — The sidebar shows every discovered project grouped by root. Click a project to see its environment files.
4. **View variables** — Click any `.env` file to see its key-value pairs. Values are masked by default — click "Reveal" to decrypt.
5. **Compare environments** — From a project detail page, click "Compare Environments" to see a matrix of which variables exist in which files.
6. **Search everything** — Press `Cmd+K` (or `Ctrl+K`) to search across all variables, projects, and tiers.

## Project Structure

```
dotvault/
├── src/                    # React frontend
│   ├── components/         # Sidebar, Header, SearchPalette, TierBadge
│   ├── pages/              # Setup, Unlock, Projects, ProjectDetail, EnvFile, Comparison, Settings
│   ├── hooks/              # useVault state management
│   └── lib/                # Tauri API bridge with TypeScript types
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands.rs     # Tauri command handlers
│       ├── db/             # SQLite schema and CRUD
│       ├── parser/         # .env file parser + tier detection
│       ├── scanner/        # Filesystem scanner + project discovery
│       ├── crypto/         # AES-256-GCM + Argon2id encryption
│       ├── watcher/        # File system watcher (notify)
│       └── search/         # Search filters + fuzzy matching
├── package.json
├── vite.config.ts
└── src-tauri/
    ├── Cargo.toml
    └── tauri.conf.json
```

## Security

- Secret **values** are encrypted with AES-256-GCM before being stored in the local SQLite database.
- The encryption key is derived from your master password using **Argon2id** (memory-hard KDF).
- The master password is **never stored** — only the Argon2 salt and a verification hash.
- Variable **names** are stored in plaintext to enable instant search and comparison without decryption.
- The vault auto-locks after inactivity, clearing the encryption key from memory.

For more details, see [SECURITY.md](SECURITY.md).

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to get started.

## License

This project is licensed under the [MIT License](LICENSE).

---

Made by [Shivam / Tagmark Studio](https://tagmark.studio)

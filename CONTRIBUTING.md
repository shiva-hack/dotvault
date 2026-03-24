# Contributing to dotvault

Thanks for your interest in contributing! This document covers the development setup and guidelines for contributing to dotvault.

## Development Setup

### Prerequisites

1. **Rust** (stable) — install via [rustup](https://rustup.rs/)
2. **Node.js** >= 18 — install via [nvm](https://github.com/nvm-sh/nvm) or [nodejs.org](https://nodejs.org/)
3. **Tauri system dependencies**:
   - macOS: `xcode-select --install`
   - Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev`
   - Fedora: `sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel`
   - Windows: [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) + WebView2

### Getting Started

```bash
# Fork and clone the repo
git clone https://github.com/YOUR_USERNAME/dotvault.git
cd dotvault

# Install frontend dependencies
npm install

# Start development
npm run tauri dev
```

This launches both the Vite dev server (frontend hot reload) and the Rust backend. Changes to React code will hot-reload; changes to Rust code will trigger a recompile.

### Running Tests

```bash
# Rust tests (parser, tier detection, etc.)
cd src-tauri && cargo test

# Type-check the frontend
npm run build
```

## Project Architecture

The app has two halves:

**Rust backend** (`src-tauri/src/`) handles filesystem scanning, .env parsing, encryption/decryption, SQLite storage, and file watching. All business logic lives here.

**React frontend** (`src/`) is a thin UI layer that calls Rust commands via Tauri's `invoke()` bridge. It handles routing, state, and rendering.

The `src/lib/tauri.ts` file is the single point of contact between frontend and backend — it contains typed wrappers for every Tauri command.

## How to Contribute

### Reporting Bugs

Open an issue with:
- Steps to reproduce
- Expected vs. actual behavior
- OS and version
- Relevant error messages or logs

### Suggesting Features

Open an issue tagged `enhancement` describing:
- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered

### Pull Requests

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Add or update tests if applicable
4. Run `cargo test` and `npm run build` to verify everything compiles
5. Write a clear PR description explaining what changed and why
6. Submit the PR

### Code Style

- **Rust**: Follow standard Rust conventions. Run `cargo fmt` before committing. Run `cargo clippy` and address warnings.
- **TypeScript/React**: Use the existing patterns in the codebase. Functional components with hooks, named exports.
- **CSS**: Use Tailwind utility classes. Custom design tokens are defined in `src/index.css` via `@theme`.
- **Commits**: Write clear, concise commit messages. Use imperative mood ("Add feature" not "Added feature").

### Areas Where Help is Appreciated

- Windows and Linux testing
- Accessibility improvements
- Internationalization (i18n)
- Additional .env format edge cases
- Performance optimization for large codebases (10,000+ projects)
- CI/CD pipeline setup

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold a welcoming, inclusive environment.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

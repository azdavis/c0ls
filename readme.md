# c0ls

A language server for C0.

See [architecture.md](docs/architecture.md) for more information.

## Setup

### Install the dependencies

- `rustc` + `cargo` https://rustup.rs
- `node` + `npm` https://nodejs.org
- POSIX `sh` (pre-installed on most Unix-like systems)

### Get the repo

Probably with a `git clone` or a ZIP download.

### Build

1. In the repo root, run `cargo build`.
2. In `extensions/vscode`, run `npm install && npm run build`.

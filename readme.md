# c0ls

A language server for the C0 programming language.

C0 is a C-like language used in Carnegie Mellon University's introductory
imperative programming class.

This language server and accompanying VSCode extension provides features like:

- Basic syntax highlighting and bracket matching
- Inline errors (parse errors, type errors, etc)
- Jump-to-definition for variables, structs, functions, typedefs
- Hover for info: expression type, function signature, etc

See [architecture.md](docs/architecture.md) for more information.

## Note

A more full-featured and well-supported alternative is [available][1].

## Setup

### Install the dependencies

- `rustc` + `cargo` https://rustup.rs
- `node` + `npm` https://nodejs.org

### Get the repo

Probably with a `git clone` or a ZIP download.

### Build

1. In the repo root, run `cargo build`.
2. In `extensions/vscode`, run `npm install && npm run build`.

[1]: https://github.com/CalLavicka/c0-vscode-extension

### Try it in VScode

1. Open the project root in VSCode.
2. Go to the Run tab (play button with bug).
3. Select 'extension' at the top.
4. Press the green play button.
5. When a new VSCode window opens with "Extension Development Host" in the
   title, open a new directory in that window.
6. Put some C0 files in the directory to try out the language server.

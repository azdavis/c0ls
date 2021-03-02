# Architecture

c0ls is a [language server][lang-server] for the [C0][c0] programming language.
It takes in as input a set of C0 files, and produces as output rich semantic
information about those files.

It answers questions like:

- What are the errors in this file? (syntax errors, type errors, etc)
- What is the type of this expression?
- Where is the definition of this variable/function/struct/typedef?

## Code map

c0ls is primarily written in Rust, and is built with Rust's package manager,
Cargo.

The project makes use of a Cargo feature called 'workspaces', which allow
splitting up a project into distinct 'crates' (packages) that can then depend on
each other. The bulk of the code in the project is thus in the `crates/`
subdirectory, but some code lives elsewhere as well.

### `crates/syntax`

Types for working with C0 syntax trees: tokens, expressions, statements, etc.

Most of the code here is generated.

These types preserve location information, which is helpful for things like
knowing where to put the error message or knowing where something is defined.

### `crates/syntax-gen`

Generates most of the code for the `syntax` crate, with the help of a C0
[ungrammar][].

### `crates/lex`

The lexer, which takes in a string of C0 code and produces a flat list of
tokens.

The lexer also handles parsing `#use` pragmas, since the syntax of library
literals would be tricky to handle elsewhere.

### `crates/event-parse`

A generic framework for writing event-based parsers. Such parsers are ones that
take as input a flat list of tokens, and produce as output a flat list of
events. Events describe how to build a structured syntax tree from the flat list
of tokens:

- Start a tree node
- Consume some tokens
- Finish the node

This also lets us handle trivia (whitespace, comments) in one place rather than
all over the parser.

### `crates/parse`

Parses a list of tokens into a list of events, then combines the events and the
tokens to produce a [rowan][] syntax tree.

### `crates/hir`

High-level Intermediate Representation of C0 code.

All information about source locations, delimiters, etc is gone. Certain
constructs like `a->b` are also gone, since they can be represented by the
combination of more primitive constructs: `(*a).b`.

### `crates/lower`

Lowers a syntax tree into a HIR tree.

Also produces a mapping between HIR nodes and their corresponding syntax tree
nodes.

### `crates/uses`

Resolves `#use` pragmas.

For instance, this resolves `#use "foo.h0"` to actually point at `foo.h0`, if it
exists (and errors if it doesn't).

### `crates/std-lib`

The standard C0 libraries, i.e. what you get when you e.g. `#use <conio>`.

### `crates/statics`

Performs static analysis on HIR: checks for type errors, undefined variables,
etc.

### `crates/analysis`

Provides the `Db` type, which takes in C0 files and updates to those files, and
allows answering queries about those files.

### `crates/c0ls`

A language server, which communicates via LSP over stdout, feeds the parsed
queries to an `analysis::Db`, and replies with its responses.

### `crates/fmt`

An experimental C0 code formatter. Throws away all comments, so currently nigh
unusable.

### `crates/uri-db`

A database of URIs. Allows us to turn a URI (heap-allocated, expensive to pass
around) into a cheap, integer-sized ID, and also convert that ID back into a
URI.

### `crates/identifier-case`

Conversions between various identifier cases, like `snake_case` and
`PascalCase`.

### `crates/text-pos`

Allows translating between byte indices and line-and-character positions in a
string.

### `crates/topo-sort`

Generic topological sorting. We feed in the `#use`s of all the files to this to
know what order to process the files in.

### `crates/unwrap-or`

Utility crate providing the macro form of `Option::unwrap_or`.

### `.cargo`

Configuration for Cargo, notably defining the `cargo xtask` shortcut.

### `.github`

Configuration for GitHub, notably the CI via GitHub Actions.

### `.vscode`

Configuration for VSCode, useful if you are using VSCode to develop this
project.

### `docs`

Developer-oriented documentation for the project.

### `extensions/vscode`

The VSCode language client extension that communicates with the Rust language
server.

### `xtask`

Miscellaneous repo tasks. Run `cargo xtask` for a list.

## Cross-cutting concerns

### Signaling errors

c0ls often operates on code that the user is in the middle of writing. This
means we can't just terminate the process when we e.g. see a syntax error.

Indeed, each of the various 'phases' (lexing, parsing, lowering, resolving uses,
running statics) always return an output _in addition_ to any errors they
encounter, as opposed to the usual _either_ an output _or_ error(s) approach of
`Result<T, E>`.

In this way, the 'actual' output returned when there are errors may not be fully
accurate, but it at least gives us _something_ to proceed to the next phase with
to try to build up an approximate view of the code.

### Incremental recalculation

c0ls needs to recalculate its view of the world every time the input files
change, and the input files change very rapidly when the user is editing them.
This means we need to be efficiently apply incremental updates to rebuild the
`Db`.

Right now, we do that in only a few ways:

- We ask for, and handle, incremental text document updates from the language
  server client, as opposed to full-text changes. For example, if we have a
  large document an add a single character, this is the difference between being
  sent an update for "the following characters were added at this position" and
  "here is the entire new contents of the file".
- When a file's content changes, we only need to re-(lex, parse, lower, resolve)
  that file.

It is an eventual goal for c0ls to be far more incremental than it is right now.
Some ways we could do that are:

- Only re-run the statics on the transitive closure of files that `#use` changed
  files.
- Stop re-running analysis and use the old result if we notice that the inputs
  haven't changed. (This is possible because the massive majority of code is
  written as pure functions with no side effects.)

These things would be given to us 'for free' if we used something like
[salsa][], but we don't right now. This means we're probably only fast enough
for tiny projects.

[c0]: https://www.cs.cmu.edu/~fp/courses/15122-f10/misc/c0-reference.pdf
[lang-server]: https://microsoft.github.io/language-server-protocol/
[rowan]: https://github.com/rust-analyzer/rowan
[salsa]: https://github.com/salsa-rs/salsa
[ungrammar]: https://github.com/rust-analyzer/ungrammar

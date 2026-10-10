# Run `cargo install just`. Then simply run `just` to get a list of executable recipes.

# Lists all available commands.
default:
  just --list

# Run `cargo sort` on the workspace.
sort:
  cargo sort . -w -g

# Run `cargo fmt` for the workspace members. `--all` would also reformat path dependencies such as `../leptonic`.
fmt:
  cargo fmt

# Format the `view!` macros of crudkit-leptos and the full-stack example with leptosfmt.
leptosfmt:
  cargo install leptosfmt
  leptosfmt ./crudkit-leptos/src ./examples/full-stack/src

# Run `cargo update`, updating dependencies to the latest non-breaking version.
update:
  cargo update

# Run `cargo check` for the workspace.
check:
  cargo check

# Run `cargo test` for the workspace, then crudkit-leptos' markup tests, which render HTML with its `ssr` feature.
test:
  cargo test
  cargo test -p crudkit-leptos --features ssr --lib atoms::tests

# Run `cargo upgrades`, checking if new crate versions including potentially breaking changes are available.
upgrades: # "-" prefix allows for non-zero status codes!
  -cargo upgrades

# Run `cargo upgrade`, automatically bumping all dependencies to their latest versions.
upgrade: # "-" prefix allows for non-zero status codes!
  -cargo upgrade

# Run `cargo clippy --tests -- -Dclippy::all -Dclippy::pedantic` for the workspace.
clippy: # "-" prefix allows for non-zero status codes!
  -cargo clippy --tests -- -Dclippy::all -Dclippy::pedantic

# Run the full-stack example (`examples/full-stack`) with `cargo leptos serve`. Serves http://127.0.0.1:3000.
example:
  cd examples/full-stack && cargo leptos serve

# Build the full-stack example (SSR server and hydrate WASM bundle) without running it.
example-check:
  cd examples/full-stack && cargo leptos build

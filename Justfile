# Run `cargo install just`. Then simply run `just` to get a list of executable recipes.

# Lists all available commands.
default:
  just --list

# Run `cargo sort` for every workspace.
sort:
  cargo sort ./crudkit-core -w -g
  cargo sort ./crudkit-rs -w -g
  cargo sort ./crudkit-sea-orm -w -g
  cargo sort ./crudkit-web -w -g
  cargo sort ./crudkit-leptos -w -g

# Run `cargo fmt` for every workspace.
fmt:
  cargo fmt --all --manifest-path ./crudkit-core/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-rs/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-sea-orm/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-web/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-leptos/Cargo.toml

leptosfmt:
  cargo install leptosfmt
  leptosfmt ./crudkit-leptos/crates/crudkit-leptos/*

# Run `cargo update` for every workspace, updating the dependencies of all crates to the latest non-breaking version. Rewrites Cargo.lock files.
update:
  cargo update --manifest-path ./crudkit-core/Cargo.toml
  cargo update --manifest-path ./crudkit-rs/Cargo.toml
  cargo update --manifest-path ./crudkit-sea-orm/Cargo.toml
  cargo update --manifest-path ./crudkit-web/Cargo.toml
  cargo update --manifest-path ./crudkit-leptos/Cargo.toml

# Run `cargo check` for every workspace.
check:
  cargo check --manifest-path ./crudkit-core/Cargo.toml
  cargo check --manifest-path ./crudkit-rs/Cargo.toml
  cargo check --manifest-path ./crudkit-sea-orm/Cargo.toml
  cargo check --manifest-path ./crudkit-web/Cargo.toml
  cargo check --manifest-path ./crudkit-leptos/Cargo.toml

# Run `cargo test` for every workspace.
test:
  cargo test --manifest-path ./crudkit-core/Cargo.toml
  cargo test --manifest-path ./crudkit-rs/Cargo.toml
  cargo test --manifest-path ./crudkit-sea-orm/Cargo.toml
  cargo test --manifest-path ./crudkit-web/Cargo.toml
  cargo test --manifest-path ./crudkit-leptos/Cargo.toml

# Run `cargo upgrades` for every workspace, checking if new crate versions including potentially breaking changes are available.
upgrades: # "-" prefixes allow for non-zero status codes!
  -cargo upgrades --manifest-path ./crudkit-core/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-rs/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-sea-orm/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-web/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-leptos/Cargo.toml

# Run `cargo upgrade` for every workspace, automatically bumping all dependencies to their latest versions
upgrade: # "-" prefixes allow for non-zero status codes!
  -cargo upgrade --manifest-path ./crudkit-core/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-rs/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-sea-orm/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-web/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-leptos/Cargo.toml

# Run `cargo clippy --tests -- -Dclippy::all -Dclippy::pedantic` for every workspace.
clippy: # "-" prefixes allow for non-zero status codes!
  -cargo clippy --tests --manifest-path ./crudkit-core/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-rs/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-sea-orm/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-web/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-leptos/Cargo.toml -- -Dclippy::all -Dclippy::pedantic

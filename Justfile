# Run `cargo install just`. Then simply run `just` to get a list of executable recipes.

# Lists all available commands.
default:
  just --list

# Run `cargo sort` for every crate.
sort:
  cargo sort ./crudkit-condition -w -g
  cargo sort ./crudkit-id -w -g
  cargo sort ./crudkit-leptos -w -g
  cargo sort ./crudkit-rs -w -g
  cargo sort ./crudkit-shared -w -g
  cargo sort ./crudkit-validation -w -g
  cargo sort ./crudkit-websocket -w -g

# Run `cargo fmt` for every crate.
fmt:
  cargo fmt --all --manifest-path ./crudkit-condition/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-id/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-leptos/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-rs/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-shared/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-validation/Cargo.toml
  cargo fmt --all --manifest-path ./crudkit-websocket/Cargo.toml

leptosfmt:
  cargo install leptosfmt
  leptosfmt ./crudkit-leptos/crates/crudkit-leptos/*

# Run `cargo update` for every crate, updating the dependencies of all crates to the latest non-breaking version. Rewrites Cargo.lock files.
update:
  cargo update --manifest-path ./crudkit-condition/Cargo.toml
  cargo update --manifest-path ./crudkit-id/Cargo.toml
  cargo update --manifest-path ./crudkit-leptos/Cargo.toml
  cargo update --manifest-path ./crudkit-rs/Cargo.toml
  cargo update --manifest-path ./crudkit-shared/Cargo.toml
  cargo update --manifest-path ./crudkit-validation/Cargo.toml
  cargo update --manifest-path ./crudkit-websocket/Cargo.toml

# Run `cargo check` for every crate.
check:
  cargo check --manifest-path ./crudkit-condition/Cargo.toml
  cargo check --manifest-path ./crudkit-id/Cargo.toml
  cargo check --manifest-path ./crudkit-leptos/Cargo.toml
  cargo check --manifest-path ./crudkit-rs/Cargo.toml
  cargo check --manifest-path ./crudkit-shared/Cargo.toml
  cargo check --manifest-path ./crudkit-validation/Cargo.toml
  cargo check --manifest-path ./crudkit-websocket/Cargo.toml

# Run `cargo test` for every crate.
test:
  cargo test --manifest-path ./crudkit-condition/Cargo.toml
  cargo test --manifest-path ./crudkit-id/Cargo.toml
  cargo test --manifest-path ./crudkit-leptos/Cargo.toml
  cargo test --manifest-path ./crudkit-rs/Cargo.toml
  cargo test --manifest-path ./crudkit-shared/Cargo.toml
  cargo test --manifest-path ./crudkit-validation/Cargo.toml
  cargo test --manifest-path ./crudkit-websocket/Cargo.toml

# Run `cargo upgrades` for every crate, checking if new crate versions including potentially breaking changes are available.
upgrades: # "-" prefixes allow for non-zero status codes!
  -cargo upgrades --manifest-path ./crudkit-condition/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-id/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-leptos/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-rs/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-shared/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-validation/Cargo.toml
  -cargo upgrades --manifest-path ./crudkit-websocket/Cargo.toml

# Run `cargo upgrade` for every crate, automatically bumping all dependencies to their latest versions
upgrade: # "-" prefixes allow for non-zero status codes!
  -cargo upgrade --manifest-path ./crudkit-condition/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-id/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-leptos/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-rs/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-shared/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-validation/Cargo.toml
  -cargo upgrade --manifest-path ./crudkit-websocket/Cargo.toml

# Run `cargo clippy --tests -- -Dclippy::all -Dclippy::pedantic` for every crate.
clippy: # "-" prefixes allow for non-zero status codes!
  -cargo clippy --tests --manifest-path ./crudkit-condition/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-id/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-leptos/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-rs/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-shared/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-validation/Cargo.toml -- -Dclippy::all -Dclippy::pedantic
  -cargo clippy --tests --manifest-path ./crudkit-websocket/Cargo.toml -- -Dclippy::all -Dclippy::pedantic

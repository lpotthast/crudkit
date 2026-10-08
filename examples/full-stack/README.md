# CrudKit full-stack example

A single [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) binary that serves

- the crudkit-rs CRUD REST API under `/api` (`POST /api/{clubs,people}/crud/{read-count,read-one,read-many,...}`),
  backed by SeaORM and SQLite, and
- a Leptos 0.8 SSR + hydrate frontend mounting crudkit-leptos `CrudInstance`s.

It demonstrates two resources, `clubs` and `people`, with SQL read views exposing `has_validation_errors`. Editing a
club shows the club's people as a nested `CrudInstance` (through a custom update-field renderer and
`CrudParentConfig`). People also have a standalone page. People with a gender other than `male`, `female` or `diverse`
are flagged by a validator.

CrudKit ships no prebuilt UI and no styles: applications build their screens from CrudKit's atoms. The example is
"Keeper", a registry for a Quidditch league, and `src/ui` holds its styled CRUD components in a parchment-and-ink
design system, styled by `style/ui.scss` with fonts from Google Fonts. They pass their own `ui-*` classes to the atoms,
style the atoms' default classes and data attributes where they render atoms unchanged, lay out forms in sections of
their own with `CrudField`, and add markup the atoms do not cover (empty state, page summary) through CrudKit's hooks.
Icons are Lucide icons rendered with `leptos_icons`. Clubs also have a resource action ("Reload") that reports through
a `CrudNotification`.

Notifications go to a Leptonic toast queue (`CrudNotifier::toasts`), shown as the app's own toasts.

There is no authentication (`NoAuth` + `OpenAuthPolicy`) and no collaboration transport (a no-op
`CollaborationService`).

## Running

Requires `cargo-leptos` (`cargo install --locked cargo-leptos`) and the `wasm32-unknown-unknown` target
(`rustup target add wasm32-unknown-unknown`). `wasm-bindgen` in `Cargo.lock` must match the `wasm-bindgen` CLI that
cargo-leptos uses. If they differ, cargo-leptos prints the command that fixes it.

```bash
cd examples/full-stack
cargo leptos watch
```

Or, from the repository root, `just example`. Then open <http://127.0.0.1:3000>.

On start, the server creates `data.sqlite` in this directory (override with `DATABASE_URL`, e.g.
`sqlite::memory:`), applies all migrations and seeds dummy data into an empty database: six Quidditch teams and 27
characters from the Harry Potter books. Delete the file to start over.

## Layout

```text
src/
├── main.rs        # SSR entry point: axum server serving the Leptos app and the CRUD API.
├── lib.rs         # Hydrate entry point.
├── app.rs         # HTML shell and routes.
├── layout.rs      # The app's frame, `CrudInstanceMgr`, and English texts around the routed page.
├── api.rs         # API base URL and HTTP executor used by CrudKit instances.
├── models/        # Frontend (crudkit-web) models shared by SSR and the WASM client.
├── resources/     # Instance configurations.
├── ui/            # Styled CRUD components: shell, instance, views, inputs, and dialogs.
├── pages/         # Pages for clubs (with nested people) and people.
└── server/        # `ssr` only: SeaORM entities, migrations, CrudKit resources, API routes, validators.
style/main.scss    # Page reset; imports `ui.scss`.
```

The example uses Leptonic's `hooks` branch, through crudkit-leptos and directly for Leptonic's own atoms, and expects
it checked out at `../leptonic` next to this repository.

## SQLite notes

crudkit-sea-orm's read-view migration helpers (`crudkit_sea_orm::migrations::crud_read_view`) emit PostgreSQL SQL.
This example therefore creates its read views with an equivalent SQLite statement (see `src/server/migrations.rs`).
Everything else (repository, queries, unified validation storage) works unchanged on SQLite.

## Build flags

`.cargo/config.toml` sets `--cfg=web_sys_unstable_apis`, which Leptonic requires.

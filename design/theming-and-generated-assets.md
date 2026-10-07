# Theming and Generated Assets

CrudKit keeps its component SCSS in `crudkit-leptos-theme` and copies it into a consuming application's style tree
during the `crudkit-leptos` build script. The copied directory is generated output.

## Source and Output Ownership

The authoritative theme source is `crudkit-leptos-theme/scss`. It contains component selectors, shared helpers, a
builder, and light/dark variable definitions.

`crudkit_leptos_theme::generate(path)` owns `path`:

1. write every file of the embedded SCSS tree, skipping files whose content is unchanged;
2. write `crudkit-themes.scss`, which imports the builder, light theme, and dark theme;
3. remove every other file and directory below `path`.

Files are replaced in place rather than by deleting the directory, so the server and client builds of one
application can generate concurrently. The target path must still be disposable. Applications must not put
handwritten styles in the generated `crudkit` directory or edit copied files; the next generation replaces them.

Application-owned overrides belong outside that directory and should target the public classes or CSS variables
after importing the generated theme.

## Build-Time Discovery

The `crudkit-leptos` build script finds the consuming workspace root from Cargo's target directory, or from the
`CRUDKIT_APP_DIR` environment variable when the target directory lives elsewhere. It then reads
`[package.metadata.crudkit]` or `[workspace.metadata.crudkit]`.

If no CrudKit metadata exists, generation is skipped. When metadata exists, `style-dir` must be declared. Output is
written to:

```text
{workspace root}/{style-dir}/crudkit
```

Documentation builds skip generation. Cargo.toml and Cargo.lock are registered as explicit rerun inputs; normal
dependency rebuilding covers changes to the embedded theme crate.

## Theme Contract

The builder imports CrudKit component styles. Colors and surfaces are `--crudkit-*` custom properties, defined for
`:root` and `[data-theme="light"]` and overridden beneath `[data-theme="dark"]`. The theme targets only CrudKit's DOM
contract: `crudkit-*` classes rendered by `crudkit_leptos::components` and the `data-*` state attributes rendered by
CrudKit's atoms and Leptonic's atoms. It neither imports Leptonic's theme nor depends on Leptonic's classes or
variables, and applications may omit it entirely.

The Rust component crate owns semantic class names and DOM structure. The theme crate owns the default presentation
for those names. Applications own final compilation, importing the generated entry point, selecting `data-theme`,
and adding non-generated overrides.

Changing a component class or a styled `data-*` attribute is therefore a coordinated Rust-and-SCSS contract change,
not a local style cleanup.

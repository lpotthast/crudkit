# Theming and Generated Assets

CrudKit keeps its component SCSS in `crudkit-leptos-theme` and copies it into a consuming Leptonic application's
style tree during the `crudkit-leptos` build script. The copied directory is generated output.

## Source and Output Ownership

The authoritative theme source is `crudkit-leptos-theme/scss`. It contains component selectors, shared helpers, a
builder, and light/dark variable definitions.

`crudkit_leptos_theme::generate(path)` is deliberately destructive:

1. remove `path` recursively when it exists;
2. recreate it;
3. extract the embedded SCSS tree;
4. write `crudkit-themes.scss`, which imports the builder, light theme, and dark theme.

The target path must therefore be disposable. Applications must not put handwritten styles in the generated
`crudkit` directory or edit copied files; the next generation replaces them.

Application-owned overrides belong outside that directory and should target the public classes or CSS variables
after importing the generated theme.

## Build-Time Discovery

The `crudkit-leptos` build script finds the consuming workspace root from Cargo's target directory, then reads
`[package.metadata.leptonic]` or `[workspace.metadata.leptonic]`.

If no Leptonic metadata exists, generation is skipped. When metadata exists, both `style-dir` and `js-dir` must be
declared, although CrudKit currently uses only `style-dir`. Output is written to:

```text
{workspace root}/{style-dir}/crudkit
```

Documentation builds skip generation. Cargo.toml and Cargo.lock are registered as explicit rerun inputs; normal
dependency rebuilding covers changes to the embedded theme crate.

## Theme Contract

The builder imports CrudKit component styles. Light and dark variants scope CSS variables beneath
`[data-theme="light"]` and `[data-theme="dark"]`. Some rules intentionally override Leptonic variables because CrudKit
fields are rendered with Leptonic controls.

The Rust component crate owns semantic class names and DOM structure. The theme crate owns the default presentation
for those names. Applications own final compilation, importing the generated entry point, selecting `data-theme`,
and adding non-generated overrides.

Changing a component class or expected Leptonic selector is therefore a coordinated Rust-and-SCSS contract change,
not a local style cleanup.

//! Generates the theme of the `components` feature into the consuming application.

#[cfg(feature = "components")]
#[path = "build/theme.rs"]
mod theme;

#[cfg(feature = "components")]
fn main() -> anyhow::Result<()> {
    theme::generate()
}

#[cfg(not(feature = "components"))]
fn main() {}

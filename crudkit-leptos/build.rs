use anyhow::{Context, Result};
use cargo_toml::{Manifest, Value};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

static ENABLE_LOGGING: LazyLock<bool> = LazyLock::new(|| {
    option_env!("CRUDKIT_BUILD_ENABLE_LOGGING")
        .and_then(|v| str::parse::<bool>(v).ok())
        .unwrap_or(false)
});

static MIN_LOG_LEVEL: LazyLock<Level> = LazyLock::new(|| {
    option_env!("CRUDKIT_BUILD_MIN_LOG_LEVEL")
        .and_then(|v| str::parse::<Level>(v).ok())
        .unwrap_or(Level::Debug)
});

#[derive(Debug)]
struct CrudkitMetadata {
    relative_style_dir: String,
}

pub fn main() -> Result<()> {
    // Do nothing when building documentation.
    if cfg!(doc) {
        return Ok(());
    }

    println!("cargo:rerun-if-env-changed=CRUDKIT_APP_DIR");
    // `CRUDKIT_APP_DIR` is required when the target directory does not live inside the application directory.
    let root_dir = if let Some(app_dir) = std::env::var_os("CRUDKIT_APP_DIR") {
        PathBuf::from(app_dir)
    } else {
        let out_dir = get_out_dir().context("Could not find 'out_dir'.")?;
        let target_dir = get_cargo_target_dir(out_dir).context("Could not find 'target_dir'.")?;
        target_dir
            .parent()
            .context("Expected 'target_dir' to have a parent.")?
            .to_owned()
    };

    log(Level::Debug, format!("root_dir is: {}", root_dir.display()));

    let cargo_lock_path = root_dir.join("Cargo.lock");
    let cargo_toml_path = root_dir.join("Cargo.toml");
    assert!(
        cargo_toml_path.exists(),
        //.expect("Can't check existence of file Cargo.toml"),
        "Unable to find '{}'",
        cargo_toml_path.display()
    );

    let Some(metadata) = read_crudkit_metadata(&cargo_toml_path)? else {
        return Ok(());
    };

    println!("cargo:rerun-if-changed={}", cargo_lock_path.display());
    println!("cargo:rerun-if-changed={}", cargo_toml_path.display());

    let style_dir = root_dir.join(&metadata.relative_style_dir);

    let theme_dir = style_dir.join("crudkit");
    crudkit_leptos_theme::generate(&theme_dir).context("Could not generate the CrudKit theme.")?;
    log(
        Level::Info,
        format!("theme written to {}", theme_dir.display()),
    );

    Ok(())
}

/// Parses the application's `Cargo.toml`. Returns `None` if it does not configure the theme.
fn read_crudkit_metadata(cargo_toml_path: &PathBuf) -> Result<Option<CrudkitMetadata>> {
    let cargo_toml: Manifest<Value> = Manifest::from_path_with_metadata(cargo_toml_path)
        .with_context(|| {
            format!(
                "Could not parse Cargo.toml at '{}'",
                cargo_toml_path.display()
            )
        })?;

    log(
        Level::Debug,
        format!("Processing '{}'", cargo_toml_path.display()),
    );

    let Some(metadata) = cargo_toml
        .package
        .as_ref()
        .and_then(|pkg| pkg.metadata.as_ref())
        .and_then(|metadata| metadata.get("crudkit"))
        .or_else(|| {
            cargo_toml
                .workspace
                .as_ref()?
                .metadata
                .as_ref()?
                .get("crudkit")
        })
    else {
        let declares_leptonic_metadata = cargo_toml
            .package
            .as_ref()
            .and_then(|pkg| pkg.metadata.as_ref())
            .is_some_and(|metadata| metadata.get("leptonic").is_some());
        if declares_leptonic_metadata {
            // Earlier versions generated the theme from Leptonic's metadata.
            println!(
                "cargo:warning=CrudKit no longer reads `[package.metadata.leptonic]`. Declare \
                 `[package.metadata.crudkit] style-dir = \"..\"` to generate CrudKit's theme."
            );
        }
        log(
            Level::Debug,
            "Skipping theme generation. The application declares no 'crudkit' metadata.",
        );
        return Ok(None);
    };

    let relative_style_dir = metadata
        .as_table()
        .context("CrudKit metadata was not of type 'table'.")?
        .get("style-dir")
        .context("CrudKit's 'style-dir' metadata was not declared.")?
        .as_str()
        .context("CrudKit's 'style-dir' metadata was not of type 'string'.")?
        .to_owned();

    log(
        Level::Debug,
        format!("relative_style_dir is: {relative_style_dir:?}"),
    );

    Ok(Some(CrudkitMetadata { relative_style_dir }))
}

fn get_out_dir() -> Result<PathBuf> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    log(Level::Debug, format!("out_dir is: {}", out_dir.display()));
    Ok(out_dir)
}

// Credits @ssrlive (source: https://github.com/rust-lang/cargo/issues/9661)
fn get_cargo_target_dir(out_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let mut target_dir = None;
    let mut sub_path = out_dir.as_ref();
    while let Some(parent) = sub_path.parent() {
        if parent.ends_with("target") {
            target_dir = Some(parent);
            break;
        }
        sub_path = parent;
    }
    let target_dir = target_dir.with_context(|| {
        format!(
            "Could not find `target` dir in parents of {}",
            out_dir.as_ref().display()
        )
    })?;
    Ok(target_dir.to_path_buf())
}

fn log(level: Level, msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    if *ENABLE_LOGGING && level >= *MIN_LOG_LEVEL {
        println!("cargo:warning=[{level}] {msg}");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" | "Debug" | "DEBUG" => Ok(Self::Debug),
            "info" | "Info" | "INFO" => Ok(Self::Info),
            "warn" | "Warn" | "WARN" => Ok(Self::Warn),
            "error" | "Error" | "ERROR" => Ok(Self::Error),
            _ => Err(format!("'{s}' is not a valid LogLevel.")),
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        })
    }
}

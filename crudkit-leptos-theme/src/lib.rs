use anyhow::{Context, Result};
use include_dir::{Dir, DirEntry, include_dir};
use indoc::indoc;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

static SCSS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/scss");

const THEMES_FILE: &str = "crudkit-themes.scss";

const THEMES_FILE_CONTENT: &str = indoc!(
    r#"
    @use "./themes/builder";
    @use "./themes/light";
    @use "./themes/dark";
    "#
);

/// Writes CrudKit's SCSS theme into `path`.
///
/// `path` is owned by CrudKit: files in it that are not part of the theme are removed. Files are
/// written in place and only when their content changed, so concurrent generation into the same
/// directory, e.g. by the server and client builds of one application, is safe.
///
/// # Errors
///
/// Will return `Err` if `path` can not be created or if files cannot be written or removed.
pub fn generate(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    let mut expected = HashSet::new();
    write_dir(&SCSS_DIR, path, &mut expected)?;
    let themes_file_path = path.join(THEMES_FILE);
    write_if_changed(&themes_file_path, THEMES_FILE_CONTENT.as_bytes())?;
    expected.insert(themes_file_path);

    remove_unexpected(path, &expected)
}

fn write_dir(dir: &Dir<'_>, root: &Path, expected: &mut HashSet<PathBuf>) -> Result<()> {
    for entry in dir.entries() {
        let target = root.join(entry.path());
        match entry {
            DirEntry::Dir(dir) => {
                std::fs::create_dir_all(&target)
                    .with_context(|| format!("Could not create '{}'", target.display()))?;
                expected.insert(target);
                write_dir(dir, root, expected)?;
            }
            DirEntry::File(file) => {
                write_if_changed(&target, file.contents())?;
                expected.insert(target);
            }
        }
    }
    Ok(())
}

fn write_if_changed(path: &Path, contents: &[u8]) -> Result<()> {
    if std::fs::read(path).is_ok_and(|current| current == contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Could not create '{}'", parent.display()))?;
    }
    std::fs::write(path, contents).with_context(|| format!("Could not write '{}'", path.display()))
}

fn remove_unexpected(dir: &Path, expected: &HashSet<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("Could not read '{}'", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("Could not read an entry of '{}'", dir.display()))?
            .path();
        if path.is_dir() {
            remove_unexpected(&path, expected)?;
            if !expected.contains(&path) {
                remove(std::fs::remove_dir(&path), &path)?;
            }
        } else if !expected.contains(&path) {
            remove(std::fs::remove_file(&path), &path)?;
        }
    }
    Ok(())
}

/// A concurrent generation may already have removed the entry.
fn remove(result: std::io::Result<()>, path: &Path) -> Result<()> {
    match result {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => {
            Err(err).with_context(|| format!("Could not remove '{}'", path.display()))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_replaces_stale_files_and_is_repeatable() {
        let dir = std::env::temp_dir().join(format!("crudkit-theme-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("stale")).expect("create stale dir");
        std::fs::write(dir.join("stale/old.scss"), "old").expect("write stale file");

        generate(&dir).expect("first generation");
        generate(&dir).expect("repeated generation");

        assert!(dir.join(THEMES_FILE).exists());
        assert!(dir.join("themes/builder.scss").exists());
        assert!(!dir.join("stale").exists());
        std::fs::remove_dir_all(&dir).expect("clean up");
    }
}

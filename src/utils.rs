use std::{fs, io::Result, path::Path};

pub fn clear_or_create_dir(path: impl AsRef<Path>) -> Result<()> {
    if !(path.as_ref().exists() && path.as_ref().is_dir()) {
        return fs::create_dir_all(path);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            fs::remove_file(entry_path)?;
        } else if entry_path.is_dir() {
            fs::remove_dir_all(entry_path)?;
        }
    }

    Ok(())
}

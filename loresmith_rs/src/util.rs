use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Write a file atomically by writing a sibling temporary file and renaming it
/// into place. This prevents truncated saves if the process exits mid-write.
///
/// The temporary file is flushed to the disk before the rename so a crash cannot
/// leave a renamed but empty file behind.
pub fn write_atomic(path: impl AsRef<Path>, contents: &[u8]) -> io::Result<()> {
    let path = path.as_ref();
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let temp_name = format!(
        "{}.{}.{}.tmp",
        file_name.to_string_lossy(),
        std::process::id(),
        nonce
    );
    let temp_path = match parent {
        Some(parent) => parent.join(temp_name),
        None => Path::new(&temp_name).to_path_buf(),
    };

    if let Err(err) = write_durably(&temp_path, contents) {
        let _ = fs::remove_file(&temp_path);
        return Err(err);
    }

    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(rename_error) => {
            // Windows does not replace an existing destination with rename.
            if path.exists() && fs::remove_file(path).is_ok() {
                fs::rename(&temp_path, path)
            } else {
                let _ = fs::remove_file(&temp_path);
                Err(rename_error)
            }
        }
    }
}

fn write_durably(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch_dir(tag: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("loresmith_util_{tag}_{unique}"));
        fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    #[test]
    fn writes_new_file_and_overwrites_existing() {
        let dir = scratch_dir("write");
        let path = dir.join("save.json");

        write_atomic(&path, b"{\"turn\":1}").expect("first write");
        assert_eq!(fs::read_to_string(&path).expect("read"), "{\"turn\":1}");

        write_atomic(&path, b"{\"turn\":2}").expect("second write");
        assert_eq!(fs::read_to_string(&path).expect("read"), "{\"turn\":2}");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn leaves_no_temporary_files_behind() {
        let dir = scratch_dir("cleanup");
        let path = dir.join("save.json");

        write_atomic(&path, b"one").expect("first write");
        write_atomic(&path, b"two").expect("second write");

        let leftovers: Vec<String> = fs::read_dir(&dir)
            .expect("list dir")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporary files left: {leftovers:?}");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_paths_without_a_file_name() {
        let err = write_atomic("/", b"nope").expect_err("root has no file name");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn reports_unwritable_destination() {
        let dir = scratch_dir("missing-parent");
        let path = dir.join("no-such-dir").join("save.json");
        assert!(write_atomic(&path, b"x").is_err());
        fs::remove_dir_all(&dir).ok();
    }
}

use std::fs;
use std::io;
use std::path::Path;

/// Write a file atomically by writing a sibling temporary file and renaming it
/// into place. This prevents truncated saves if the process exits mid-write.
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

    fs::write(&temp_path, contents)?;
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

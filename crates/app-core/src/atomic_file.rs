use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) enum AtomicWriteError {
    Write(io::Error),
    Publish(io::Error),
}

pub(crate) fn write(path: &Path, bytes: &[u8]) -> Result<(), AtomicWriteError> {
    write_with_mode(path, bytes, false)
}

pub(crate) fn write_private(path: &Path, bytes: &[u8]) -> Result<(), AtomicWriteError> {
    write_with_mode(path, bytes, true)
}

fn write_with_mode(path: &Path, bytes: &[u8], private: bool) -> Result<(), AtomicWriteError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| {
            AtomicWriteError::Write(io::Error::new(
                io::ErrorKind::InvalidInput,
                "file name is required",
            ))
        })?
        .to_string_lossy();
    let (temporary, mut file) = loop {
        let sequence = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), sequence));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        if private {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temporary) {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(AtomicWriteError::Write(error)),
        }
    };

    let result = file.write_all(bytes).map_err(AtomicWriteError::Write);
    let result = result.and_then(|_| if private { file.sync_all().map_err(AtomicWriteError::Write) } else { Ok(()) });
    drop(file);
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(AtomicWriteError::Publish(error));
    }
    #[cfg(unix)]
    if private {
        fs::File::open(parent).and_then(|directory| directory.sync_all()).map_err(AtomicWriteError::Publish)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_writes_to_one_path_do_not_share_a_temporary_file() {
        let directory = std::env::temp_dir().join(format!(
            "mobile-api-studio-atomic-write-{}-{}",
            std::process::id(),
            NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("mock-rules.json");
        std::thread::scope(|scope| {
            for index in 0..32 {
                let path = &path;
                scope.spawn(move || write(path, format!("rule-{index}").as_bytes()).unwrap());
            }
        });
        assert!(fs::read_to_string(&path).unwrap().starts_with("rule-"));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(all(test, unix))]
mod private_checks {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn recovery_journal_is_private_atomic_and_durable() {
        let dir = std::env::temp_dir().join(format!("mas-private-journal-{}-{}", std::process::id(), NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&dir).unwrap(); let path = dir.join("journal.json");
        write_private(&path, b"first snapshot").unwrap();
        write_private(&path, b"replacement snapshot").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"replacement snapshot");
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
}

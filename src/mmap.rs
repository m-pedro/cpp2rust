use memmap2::{Mmap, MmapOptions};
use std::fs::File;
use std::io;
use std::path::Path;

/// Safe RAII wrapper around a read-only memory-mapped log file.
/// Replaces `MappedFile`, `mmap_open`, and `mmap_close` from `cpp/src/mmap_reader.c`.
pub struct MappedLogFile {
    mmap: Option<Mmap>,
    len: usize,
}

impl MappedLogFile {
    /// Opens `path` and maps its contents into virtual memory for zero-copy reading.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let len = metadata.len() as usize;

        // POSIX mmap returns EINVAL on 0-byte files; handle empty files safely
        if len == 0 {
            return Ok(Self { mmap: None, len: 0 });
        }

        // SAFETY: The underlying file is opened read-only for batch log analysis.
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        Ok(Self {
            mmap: Some(mmap),
            len,
        })
    }

    /// Returns the entire memory-mapped file as a byte slice `&[u8]`.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        match &self.mmap {
            Some(m) => m.as_ref(),
            None => &[],
        }
    }

    /// Returns the size of the mapped file in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the mapped file is 0 bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_mmap_open_and_read_slice() {
        let tmp_path = std::env::temp_dir().join("cpp2rust_day9_test.log");
        {
            let mut f = File::create(&tmp_path).expect("failed to create temp file");
            f.write_all(b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / HTTP/1.1\" 200 64 1.0\n")
                .unwrap();
        }

        let mapped = MappedLogFile::open(&tmp_path).expect("failed to mmap temp file");
        assert!(!mapped.is_empty());
        assert!(mapped.as_slice().starts_with(b"10.0.0.1"));

        let _ = std::fs::remove_file(&tmp_path);
    }
}

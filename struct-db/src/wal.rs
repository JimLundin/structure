use crate::serialization::WalEntry;
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Write-Ahead Log for persistent storage
pub struct Wal {
    path: PathBuf,
    writer: Mutex<BufWriter<File>>,
}

impl Wal {
    /// Open or create a WAL file
    pub fn open<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            path,
            writer: Mutex::new(BufWriter::new(file)),
        })
    }

    /// Append an entry to the WAL
    pub fn append(&self, entry: &WalEntry) -> crate::Result<()> {
        let mut writer = self.writer.lock();
        let serialized = bincode::serialize(entry)?;

        // Write length prefix (u32) followed by data
        let len = serialized.len() as u32;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&serialized)?;
        writer.flush()?;

        Ok(())
    }

    /// Read all entries from the WAL
    pub fn read_all(&self) -> crate::Result<Vec<WalEntry>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop {
            // Read length prefix
            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            let len = u32::from_le_bytes(len_buf) as usize;

            // Read entry data
            let mut entry_buf = vec![0u8; len];
            reader.read_exact(&mut entry_buf)?;

            let entry: WalEntry = bincode::deserialize(&entry_buf)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Compact the WAL by rewriting it with only the given entries
    pub fn compact(&self, entries: &[WalEntry]) -> crate::Result<()> {
        // Write to a temporary file
        let temp_path = self.path.with_extension("tmp");
        let temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)?;

        let mut writer = BufWriter::new(temp_file);

        for entry in entries {
            let serialized = bincode::serialize(entry)?;
            let len = serialized.len() as u32;
            writer.write_all(&len.to_le_bytes())?;
            writer.write_all(&serialized)?;
        }

        writer.flush()?;
        drop(writer);

        // Replace old WAL with new one
        std::fs::rename(&temp_path, &self.path)?;

        // Reopen the file for appending
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        *self.writer.lock() = BufWriter::new(file);

        Ok(())
    }

    /// Compact the WAL atomically using custom temp and target paths
    ///
    /// This operation:
    /// - Writes to temp_path
    /// - Atomically renames to target_path
    /// - Is crash-safe (old WAL preserved until success)
    pub fn compact_atomic(&self, temp_path: &Path, target_path: &Path, entries: &[WalEntry]) -> crate::Result<()> {
        // Write to temporary file
        let temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(temp_path)?;

        let mut writer = BufWriter::new(temp_file);

        for entry in entries {
            let serialized = bincode::serialize(entry)?;
            let len = serialized.len() as u32;
            writer.write_all(&len.to_le_bytes())?;
            writer.write_all(&serialized)?;
        }

        writer.flush()?;
        drop(writer);

        // Atomic rename
        std::fs::rename(temp_path, target_path)?;

        // Reopen the file for appending
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(target_path)?;

        *self.writer.lock() = BufWriter::new(file);

        Ok(())
    }
}

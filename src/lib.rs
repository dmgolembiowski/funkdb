#![allow(unused_imports)]
use anyhow::{Result, Error, bail, anyhow, self as ah};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
#[cfg(any(unix, target_os = "wasi"))]
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_disk_persisted() -> anyhow::Result<()> {
        let path = "tmp.funk";
        let mut db = FunkDb::open(&path).expect("it works in testing");
        db.save()?;
        assert!(fs::remove_file(db.path).is_ok());
        Ok(())
    }

    #[test]
    fn create_db_schema_and_apply_it() {
        let test_schema = r#"
            module default {
                type F____Given {
                    required expires: int32;
                    significance: str,
                }
                type ReasonForLiving {
                    required online: bool;
                    multi f____: F____Given;
                }
            }"#;
        todo!("Create the in-memory object of a schema module. Prepare a transaction. Commit it. Save to disk."); // Try to apply the transaction.");
    }

    #[ignore]
    #[test]
    fn persist_schema_transaction_to_disk() {
        todo!("Commit a database transaction to a disk-persisted db");
    }

    #[ignore]
    #[test]
    fn verify_schema_post_transaction() {
        todo!("Retrieve schema from database file, verify it corresponds to sdl transaction");
    }

    #[ignore]
    #[test]
    fn parse_schema_into_memory_model() {
        todo!("Make the simplest schema in an external file, serialize it, apply it.");
    }

    #[ignore]
    #[test]
    fn insert_query() {
        todo!("Create new insert query and apply it");
    }
}

#[allow(dead_code)]
pub struct FunkDb {
    path: PathBuf,
    stream: Option<UnixStream>,
    file: File,
}

impl FunkDb {
    pub fn new<F: IntoRawFd>(path: PathBuf, fileno: Option<F>, file: File) -> Self {
        let stream = match fileno {
            Some(f) => {
                let fd = f.into_raw_fd();
                Some(unsafe { <UnixStream as FromRawFd>::from_raw_fd(fd) })
            }
            None => { None }
        };
        Self { path, stream, file }
    }
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = PathBuf::from(path.as_ref());
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        Ok(Self::new(path, Option::<UnixStream>::None, file))
    }
    #[allow(dead_code)]
    fn new_server(&mut self, server_path: impl AsRef<Path>, db_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let _sfd = FunkDbServer::bind(server_path, db_path)?;
        todo!("Not implemented");
    }
    pub fn save(&mut self) -> anyhow::Result<()> {
        if self.stream.is_some() { bail!("Not implemented!"); }
        self.file.sync_all()?;
        Ok(())    
    }
}

pub struct FunkDbServer {

}

impl FunkDbServer {
    /// Returns the result of the bind op 
    /// which, assuming the socket path wasn't already taken, should be Ok(i32).
    ///
    /// With the unwrapped return value, the caller can assume 
    /// that there is a unix domain socket at [`path`] which 
    /// is a [`UnixListener`]. 
    /// 
    /// The listener will be used to accept client connections to the database
    /// so that prepared statements can be executed, queries against the 
    /// database can be ran, and transactions to update the schema can be made.
    /// 
    /// Note that [`bind`]'s argument, [`path`], is distinct from the actual 
    /// database file.
    #[allow(dead_code, unused_variables)]
    pub fn bind(server_path: impl AsRef<Path>, db_path: impl AsRef<Path>) -> anyhow::Result<RawFd> {
        let path = server_path.as_ref().to_string_lossy();
        let stream = db_path.as_ref().to_string_lossy();
        let server = UnixListener::bind(db_path)?.set_nonblocking(true);
        
        bail!("This is not yet implemented");
    } 
}
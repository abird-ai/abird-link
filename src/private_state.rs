//! Cross-process serialization of private control-plane state.
//! The stable lock file is independent of the atomically replaced payload.
//! Dropping the file releases the lock on all normal/cancellation paths.

use anyhow::{Context, Result, bail};
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

pub(crate) fn state_dir() -> Result<std::path::PathBuf> {
    Ok(crate::xdg::state_home()?.join("abird/dotlink"))
}

/// Persist correctness state before publishing its dependent external effect.
/// An error after rename means durability is uncertain, not that the new bytes
/// are absent; callers must fail before the dependent effect and reconcile later.
pub(crate) fn write_durable(path: &Path, contents: &[u8]) -> Result<()> {
    crate::setup::atomic_write_private_durable(path, contents, true)
}

/// All owned XDG control-plane directories are initialized through this
/// private-mode boundary before a shell can mount a writable ancestor.
pub(crate) fn prepare_owned_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("cannot create {}", path.display()))?;
    crate::setup::set_private_dir_permissions(path)
}

const MUTATION_ARTIFACT_DIR: &str = "mutation-artifacts-v1";
const MUTATION_ARTIFACT_VERSION: u32 = 1;
const MAX_MUTATION_STATE_BYTES: u64 = 1024 * 1024;
const MUTATION_STATE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationArtifactFile {
    version: u32,
    records: Vec<aw_tools::filesystem::private_artifact::Record>,
}

struct PrivateMutationStore {
    records: PathBuf,
    lock: PathBuf,
}

impl PrivateMutationStore {
    fn for_root(root: &Path) -> Result<Self> {
        Self::for_root_in(&state_dir()?, root)
    }

    fn for_root_in(state_root: &Path, root: &Path) -> Result<Self> {
        let directory = state_root.join(MUTATION_ARTIFACT_DIR);
        prepare_owned_dir(&directory)?;
        let key = mutation_root_key(root);
        Ok(Self {
            records: directory.join(format!("{key}.json")),
            lock: directory.join(format!("{key}.lock")),
        })
    }

    fn error(error: impl std::fmt::Display) -> aw_tools::filesystem::private_artifact::Error {
        aw_tools::filesystem::private_artifact::Error::Store(error.to_string())
    }

    fn read_unlocked(
        &self,
    ) -> std::result::Result<
        Vec<aw_tools::filesystem::private_artifact::Record>,
        aw_tools::filesystem::private_artifact::Error,
    > {
        let metadata = match fs::symlink_metadata(&self.records) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(Self::error(error)),
        };
        if !metadata.file_type().is_file() || metadata.len() > MAX_MUTATION_STATE_BYTES {
            return Err(Self::error("invalid private mutation state file"));
        }
        let bytes = fs::read(&self.records).map_err(Self::error)?;
        let stored: MutationArtifactFile = serde_json::from_slice(&bytes).map_err(Self::error)?;
        if stored.version != MUTATION_ARTIFACT_VERSION
            || stored.records.len() > aw_tools::filesystem::private_artifact::MAX_PRIVATE_ARTIFACTS
        {
            return Err(Self::error(
                "unsupported or oversized private mutation state",
            ));
        }
        Ok(stored.records)
    }

    fn write_unlocked(
        &self,
        records: &[aw_tools::filesystem::private_artifact::Record],
    ) -> std::result::Result<(), aw_tools::filesystem::private_artifact::Error> {
        let bytes = serde_json::to_vec(&MutationArtifactFile {
            version: MUTATION_ARTIFACT_VERSION,
            records: records.to_vec(),
        })
        .map_err(Self::error)?;
        if bytes.len() as u64 > MAX_MUTATION_STATE_BYTES {
            return Err(Self::error("private mutation state exceeds byte ceiling"));
        }
        write_durable(&self.records, &bytes).map_err(Self::error)
    }

    fn with_lock<T>(
        &self,
        f: impl FnOnce(&Self) -> std::result::Result<T, aw_tools::filesystem::private_artifact::Error>,
    ) -> std::result::Result<T, aw_tools::filesystem::private_artifact::Error> {
        let _lock = acquire(&self.lock, MUTATION_STATE_LOCK_TIMEOUT).map_err(Self::error)?;
        f(self)
    }
}

impl aw_tools::filesystem::private_artifact::Store for PrivateMutationStore {
    fn durable(&self) -> bool {
        true
    }

    fn load(
        &self,
    ) -> std::result::Result<
        Vec<aw_tools::filesystem::private_artifact::Record>,
        aw_tools::filesystem::private_artifact::Error,
    > {
        self.with_lock(Self::read_unlocked)
    }

    fn reserve(
        &self,
        record: &aw_tools::filesystem::private_artifact::Record,
    ) -> std::result::Result<(), aw_tools::filesystem::private_artifact::Error> {
        self.with_lock(|store| {
            let mut records = store.read_unlocked()?;
            if records
                .iter()
                .any(|current| current.parent == record.parent && current.name == record.name)
            {
                return Err(aw_tools::filesystem::private_artifact::Error::Conflict);
            }
            if records.len() >= aw_tools::filesystem::private_artifact::MAX_PRIVATE_ARTIFACTS {
                return Err(aw_tools::filesystem::private_artifact::Error::Capacity);
            }
            records.push(record.clone());
            records.sort_by(|a, b| {
                (a.parent.volume, a.parent.object, a.name.as_str()).cmp(&(
                    b.parent.volume,
                    b.parent.object,
                    b.name.as_str(),
                ))
            });
            store.write_unlocked(&records)
        })
    }

    fn finalize(
        &self,
        before: &aw_tools::filesystem::private_artifact::Record,
        after: &aw_tools::filesystem::private_artifact::Record,
    ) -> std::result::Result<(), aw_tools::filesystem::private_artifact::Error> {
        use aw_tools::filesystem::private_artifact::{Error, Kind};
        if before.parent != after.parent
            || before.name != after.name
            || before.parent_path != after.parent_path
            || before.kind != Kind::Stage
            || after.kind != Kind::Stage
            || before.object.is_some()
            || after.object.is_none()
        {
            return Err(Error::Conflict);
        }
        self.with_lock(|store| {
            let mut records = store.read_unlocked()?;
            let Some(index) = records
                .iter()
                .position(|current| current.parent == before.parent && current.name == before.name)
            else {
                return Err(Error::Conflict);
            };
            if &records[index] != before {
                return Err(Error::Conflict);
            }
            records[index] = after.clone();
            records.sort_by(|a, b| {
                (a.parent.volume, a.parent.object, a.name.as_str()).cmp(&(
                    b.parent.volume,
                    b.parent.object,
                    b.name.as_str(),
                ))
            });
            store.write_unlocked(&records)
        })
    }

    fn retire(
        &self,
        record: &aw_tools::filesystem::private_artifact::Record,
    ) -> std::result::Result<(), aw_tools::filesystem::private_artifact::Error> {
        self.with_lock(|store| {
            let mut records = store.read_unlocked()?;
            let Some(index) = records
                .iter()
                .position(|current| current.parent == record.parent && current.name == record.name)
            else {
                return Ok(());
            };
            if &records[index] != record {
                return Err(aw_tools::filesystem::private_artifact::Error::Conflict);
            }
            records.remove(index);
            store.write_unlocked(&records)
        })
    }
}

fn mutation_root_key(root: &Path) -> String {
    use sha2::Digest as _;
    let mut hash = sha2::Sha256::new();
    hash.update(b"dotlink-private-mutation-root-v1\0");
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;
        hash.update(root.as_os_str().as_bytes());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt as _;
        for value in root.as_os_str().encode_wide() {
            hash.update(value.to_le_bytes());
        }
    }
    #[cfg(not(any(unix, windows)))]
    hash.update(root.to_string_lossy().as_bytes());
    hex::encode(hash.finalize())
}

pub(crate) fn mutation_registry(
    root: &Path,
) -> Result<Arc<aw_tools::filesystem::private_artifact::Registry>> {
    let store = Arc::new(PrivateMutationStore::for_root(root)?);
    aw_tools::filesystem::private_artifact::Registry::open(store)
        .map_err(|error| anyhow::anyhow!(error))
}

pub(crate) fn acquire(path: &Path, timeout: Duration) -> Result<File> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("lock path has no parent"))?;
    prepare_owned_dir(parent)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("cannot open state lock {}", path.display()))?;
    crate::setup::set_private_file_permissions(path)?;
    let deadline = Instant::now() + timeout;
    loop {
        match lock.try_lock() {
            Ok(()) => return Ok(lock),
            Err(fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(fs::TryLockError::WouldBlock) => {
                bail!("state is busy: {}; retry", path.display());
            }
            Err(fs::TryLockError::Error(error)) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(fs::TryLockError::Error(error)) => {
                return Err(error).with_context(|| format!("cannot lock {}", path.display()));
            }
        }
    }
}

#[cfg(test)]
mod mutation_store_tests {
    use super::*;
    use aw_tools::filesystem::private_artifact::{Kind, ObjectIdentity, Record, Registry};

    fn record(name: &str, object: u64) -> Record {
        Record {
            kind: Kind::Tombstone,
            parent_path: "dir".into(),
            parent: ObjectIdentity {
                volume: 1,
                object: 2,
            },
            name: name.into(),
            object: Some(ObjectIdentity { volume: 1, object }),
        }
    }

    #[test]
    fn durable_mutation_store_survives_reopen_and_retires_exact_record() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir(&root).unwrap();
        let store = Arc::new(PrivateMutationStore::for_root_in(temp.path(), &root).unwrap());
        let registry = Registry::open(store.clone()).unwrap();
        assert!(registry.is_durable());
        let lease = registry.reserve(record("gone", 3)).unwrap();
        drop(lease);

        let reopened = Registry::open(store.clone()).unwrap();
        assert_eq!(reopened.records().unwrap(), [record("gone", 3)]);
        reopened.retire_record(&record("gone", 3)).unwrap();
        assert!(Registry::open(store).unwrap().records().unwrap().is_empty());
    }

    #[test]
    fn provisional_stage_finalize_is_atomic_and_survives_reopen() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir(&root).unwrap();
        let store = Arc::new(PrivateMutationStore::for_root_in(temp.path(), &root).unwrap());
        let registry = Registry::open(store.clone()).unwrap();
        let parent = ObjectIdentity {
            volume: 7,
            object: 11,
        };
        let mut reservation = registry
            .reserve_name(
                Kind::Stage,
                "dir".into(),
                parent,
                ".aw-stage-random.tmp".into(),
            )
            .unwrap();
        let provisional = reservation.record().clone();
        assert!(provisional.object.is_none());
        assert_eq!(
            Registry::open(store.clone()).unwrap().records().unwrap(),
            [provisional]
        );

        let object = ObjectIdentity {
            volume: 7,
            object: 12,
        };
        let lease = reservation.finalize(object).unwrap();
        let finalized = lease.record().clone();
        assert_eq!(finalized.object, Some(object));
        drop(lease);
        assert_eq!(
            Registry::open(store).unwrap().records().unwrap(),
            [finalized]
        );
    }

    #[test]
    fn mutation_store_conflict_is_cross_registry_and_root_keys_are_distinct() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a");
        let b = temp.path().join("b");
        fs::create_dir(&a).unwrap();
        fs::create_dir(&b).unwrap();
        assert_ne!(mutation_root_key(&a), mutation_root_key(&b));
        let store = Arc::new(PrivateMutationStore::for_root_in(temp.path(), &a).unwrap());
        let first = Registry::open(store.clone()).unwrap();
        let second = Registry::open(store).unwrap();
        drop(first.reserve(record("same", 3)).unwrap());
        assert!(matches!(
            second.reserve(record("same", 4)),
            Err(aw_tools::filesystem::private_artifact::Error::Conflict)
        ));
    }
}

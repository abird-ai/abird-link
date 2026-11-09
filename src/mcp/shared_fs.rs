//! Thin dotlink host binding for the canonical secured filesystem backend.
//! Product authority is already resolved by `AccessPolicy`; this module only
//! prepares pinned shared grants/exclusions and private mutation custody.

use super::AccessPolicy;
use anyhow::{Result, anyhow};
use aw_tools::filesystem::private_artifact::Registry;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

#[cfg(unix)]
use aw_tools_fs_unix::scoped_fs::capability::{
    CapabilitySet, ExclusionSpec, GrantSpec, PreparedGrant,
};
#[cfg(windows)]
use aw_tools_fs_win::scoped_fs::capability::{
    CapabilitySet, ExclusionSpec, GrantSpec, PreparedGrant,
};

#[cfg(unix)]
pub(crate) type SharedBackend = aw_tools_fs_unix::scoped_fs::capability::CapabilitySet;
#[cfg(windows)]
pub(crate) type SharedBackend = aw_tools_fs_win::scoped_fs::capability::CapabilitySet;

#[derive(Clone)]
pub(crate) struct SharedCapabilitySet {
    prepared: Arc<CapabilitySet>,
    #[cfg(windows)]
    unrestricted: Option<Arc<WindowsUnrestricted>>,
}

#[cfg(windows)]
#[derive(Clone)]
struct WindowsUnrestricted {
    access: AccessPolicy,
    protected: Arc<[PathBuf]>,
}

impl std::fmt::Debug for SharedCapabilitySet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SharedCapabilitySet")
            .field("base", &self.prepared.base())
            .finish_non_exhaustive()
    }
}

impl std::ops::Deref for SharedCapabilitySet {
    type Target = CapabilitySet;
    fn deref(&self) -> &Self::Target {
        &self.prepared
    }
}

fn scoped(error: impl std::fmt::Debug) -> anyhow::Error {
    anyhow!("shared filesystem capability: {error:?}")
}

fn exclusions(access: &AccessPolicy, protected: &[PathBuf]) -> Vec<ExclusionSpec> {
    let mut excluded = BTreeMap::<PathBuf, (bool, bool)>::new();
    for path in &access.deny_read_roots {
        excluded.entry(path.clone()).or_default().0 = true;
    }
    for path in &access.deny_write_roots {
        excluded.entry(path.clone()).or_default().1 = true;
    }
    // Dotlink control-plane paths are never model-addressable native paths,
    // including under broad/root grants. Shell has its own explicit policy.
    for path in protected {
        let bits = excluded.entry(path.clone()).or_default();
        bits.0 = true;
        bits.1 = true;
    }
    excluded
        .into_iter()
        .map(|(path, (read, write))| ExclusionSpec { path, read, write })
        .collect()
}

fn prepare_with_registry(
    access: &AccessPolicy,
    protected: &[PathBuf],
    mut registry: impl FnMut(&Path) -> Result<Arc<Registry>>,
) -> Result<SharedCapabilitySet> {
    let mut rights = BTreeMap::<PathBuf, (bool, bool)>::new();
    for root in &access.read_roots {
        rights.entry(root.clone()).or_default().0 = true;
    }
    for root in &access.write_roots {
        rights.entry(root.clone()).or_default().1 = true;
    }

    #[cfg(unix)]
    if access.unrestricted_fs {
        let root = rights.entry(PathBuf::from("/")).or_default();
        root.0 = true;
        root.1 = true;
    }

    // Windows unrestricted execution is prepared per request below. Keep one
    // pinned base anchor for authority/debug state when no finite grant exists;
    // never treat it as the complete unrestricted filesystem authority.
    #[cfg(windows)]
    if access.unrestricted_fs && rights.is_empty() {
        rights.insert(access.base_dir.clone(), (true, true));
    }

    let mut grants = Vec::with_capacity(rights.len());
    for (path, (read, write)) in rights {
        let private = if write {
            registry(&path)?
        } else {
            Registry::memory()
        };
        grants.push(PreparedGrant::new(GrantSpec { path, read, write }, private));
    }

    let exclusions = exclusions(access, protected);

    let set = CapabilitySet::open_prepared_with_exclusions(&access.base_dir, grants, exclusions)
        .map_err(scoped)?;
    set.reconcile_private_artifacts().map_err(scoped)?;
    Ok(SharedCapabilitySet {
        prepared: Arc::new(set),
        #[cfg(windows)]
        unrestricted: None,
    })
}

pub(crate) fn prepare(access: &AccessPolicy, protected: &[PathBuf]) -> Result<SharedCapabilitySet> {
    let set = prepare_with_registry(access, protected, crate::private_state::mutation_registry)?;
    #[cfg(windows)]
    {
        let mut set = set;
        if access.unrestricted_fs {
            set.unrestricted = Some(Arc::new(WindowsUnrestricted {
                access: access.clone(),
                protected: Arc::from(protected.to_vec()),
            }));
        }
        Ok(set)
    }
    #[cfg(not(windows))]
    {
        Ok(set)
    }
}

impl SharedCapabilitySet {
    pub(crate) fn backend_for_request(
        &self,
        _request: &aw_tools::filesystem::request::Request,
    ) -> Result<Arc<CapabilitySet>> {
        #[cfg(windows)]
        if let Some(unrestricted) = &self.unrestricted {
            return unrestricted.backend_for_request(_request);
        }
        Ok(self.prepared.clone())
    }
}

#[cfg(windows)]
impl WindowsUnrestricted {
    fn merge_rights(
        rights: &mut BTreeMap<PathBuf, (bool, bool)>,
        root: PathBuf,
        access: aw_tools::filesystem::request::PathAccess,
    ) {
        let bits = rights.entry(root).or_default();
        match access {
            aw_tools::filesystem::request::PathAccess::Read => bits.0 = true,
            aw_tools::filesystem::request::PathAccess::Write => bits.1 = true,
            aw_tools::filesystem::request::PathAccess::ReadWrite => {
                bits.0 = true;
                bits.1 = true;
            }
        }
    }

    fn model_host_path(&self, model: &str) -> Result<PathBuf> {
        if model == "." || model.is_empty() {
            return Ok(self.access.base_dir.clone());
        }
        if model.starts_with('/') && !aw_tools::filesystem::path::is_unc(model) {
            return Err(anyhow!(
                "POSIX absolute model path is not valid on a Windows filesystem"
            ));
        }
        let host = model.replace('/', "\\");
        let path = PathBuf::from(host);
        if aw_tools::filesystem::path::is_absolute(model) {
            Ok(path)
        } else {
            Ok(self.access.base_dir.join(path))
        }
    }

    fn namespace_root(path: &Path) -> Result<PathBuf> {
        use std::path::Component;
        let mut components = path.components();
        let prefix = match components.next() {
            Some(Component::Prefix(prefix)) => prefix.as_os_str().to_os_string(),
            _ => {
                return Err(anyhow!(
                    "Windows unrestricted path has no absolute namespace prefix"
                ));
            }
        };
        if !matches!(components.next(), Some(Component::RootDir)) {
            return Err(anyhow!("Windows unrestricted path is not absolute"));
        }
        let mut root = PathBuf::from(prefix);
        root.push(Path::new(r"\"));
        root.canonicalize().map_err(Into::into)
    }

    fn observed_final_target_root(path: &Path) -> Result<Option<PathBuf>> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        if !metadata.file_type().is_symlink() {
            return Ok(None);
        }
        let target = path.canonicalize()?;
        Self::namespace_root(&target).map(Some)
    }

    fn backend_for_request(
        &self,
        request: &aw_tools::filesystem::request::Request,
    ) -> Result<Arc<CapabilitySet>> {
        let mut rights = BTreeMap::<PathBuf, (bool, bool)>::new();
        for path in request
            .access_paths()
            .map_err(|error| anyhow!(error.to_string()))?
        {
            let host = self.model_host_path(&path.path)?;
            Self::merge_rights(&mut rights, Self::namespace_root(&host)?, path.access);
            if path.follow_final {
                if let Some(target_root) = Self::observed_final_target_root(&host)? {
                    Self::merge_rights(&mut rights, target_root, path.access);
                }
            }
        }
        if rights.is_empty() {
            return Err(anyhow!(
                "canonical filesystem request has no authority paths"
            ));
        }
        let mut grants = Vec::with_capacity(rights.len());
        for (path, (read, write)) in rights {
            let private = if write {
                crate::private_state::mutation_registry(&path)?
            } else {
                Registry::memory()
            };
            grants.push(PreparedGrant::new(GrantSpec { path, read, write }, private));
        }
        let set = CapabilitySet::open_prepared_with_exclusions(
            &self.access.base_dir,
            grants,
            exclusions(&self.access, &self.protected),
        )
        .map_err(scoped)?;
        set.reconcile_private_artifacts().map_err(scoped)?;
        Ok(Arc::new(set))
    }
}

#[cfg(test)]
pub(crate) fn prepare_root_for_test(root: &Path) -> Result<SharedCapabilitySet> {
    let base = root.canonicalize()?;
    let access = AccessPolicy::from_spec(super::AccessSpec {
        base_dir: base.clone(),
        read_roots: vec![base.clone()],
        write_roots: vec![base],
        deny_read_roots: Vec::new(),
        deny_write_roots: Vec::new(),
        unrestricted_fs: false,
    })?;
    prepare_with_registry(&access, &[], |_| Ok(Registry::memory()))
}

#[cfg(test)]
pub(crate) fn prepare_for_test(
    access: &AccessPolicy,
    protected: &[PathBuf],
) -> Result<SharedCapabilitySet> {
    prepare_with_registry(access, protected, |_| Ok(Registry::memory()))
}

pub(crate) fn authority_value(set: &SharedCapabilitySet) -> Result<serde_json::Value> {
    let identities = set.identities().map_err(scoped)?;
    Ok(serde_json::Value::Array(
        identities
            .into_iter()
            .map(|identity| {
                serde_json::json!({
                    "path": identity.path,
                    "volume": identity.volume,
                    "object": identity.object,
                    "read": identity.read,
                    "write": identity.write,
                })
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::guard_identities;
    use aw_tools::filesystem::{backend::ExplicitPathBackend as _, operations, walk::Backend as _};

    #[test]
    fn prepared_capability_reuses_authorized_roots_and_excludes_policy_holes() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let read = base.join("read");
        let write = base.join("write");
        let secret = read.join("secret");
        std::fs::create_dir_all(&secret).unwrap();
        std::fs::create_dir(&write).unwrap();
        std::fs::write(read.join("visible.txt"), b"visible").unwrap();
        std::fs::write(secret.join("hidden.txt"), b"hidden").unwrap();
        let access = AccessPolicy::from_spec(super::super::AccessSpec {
            base_dir: base.clone(),
            read_roots: vec![read.clone()],
            write_roots: vec![write.clone()],
            deny_read_roots: vec![secret.clone()],
            deny_write_roots: Vec::new(),
            unrestricted_fs: false,
        })
        .unwrap();
        let set = prepare_with_registry(&access, &[], |_| Ok(Registry::memory())).unwrap();
        let listing = set.list("read", 16).unwrap();
        assert!(
            listing
                .entries
                .iter()
                .any(|entry| entry.path == "read/visible.txt")
        );
        assert!(
            !listing
                .entries
                .iter()
                .any(|entry| entry.path.contains("secret"))
        );
        assert!(set.read_bytes("read/secret/hidden.txt", 64).is_err());
        assert_eq!(set.identities().unwrap().len(), 2);
    }

    #[test]
    fn dotlink_policy_preserves_asymmetric_cross_root_lifecycle_authority() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let read = base.join("read");
        let write_a = base.join("write-a");
        let write_b = base.join("write-b");
        for path in [&read, &write_a, &write_b] {
            std::fs::create_dir(path).unwrap();
        }
        std::fs::write(read.join("source.txt"), b"source").unwrap();
        std::fs::write(write_a.join("move.txt"), b"move").unwrap();
        let access = AccessPolicy::from_spec(super::super::AccessSpec {
            base_dir: base.clone(),
            read_roots: vec![read.clone()],
            write_roots: vec![write_a.clone(), write_b.clone()],
            deny_read_roots: Vec::new(),
            deny_write_roots: Vec::new(),
            unrestricted_fs: false,
        })
        .unwrap();
        let set = prepare_with_registry(&access, &[], |_| Ok(Registry::memory())).unwrap();

        assert_eq!(
            operations::copy_path(
                &*set,
                "read/source.txt",
                "write-a/copied.txt",
                false,
                &|| false
            )
            .unwrap(),
            aw_tools::filesystem::status::Primary::Ok
        );
        assert_eq!(
            std::fs::read(write_a.join("copied.txt")).unwrap(),
            b"source"
        );
        assert!(
            operations::copy_path(
                &*set,
                "write-a/copied.txt",
                "read/forbidden.txt",
                false,
                &|| false
            )
            .is_err(),
            "read-only destination must never gain write authority from a readable source"
        );
        assert!(!read.join("forbidden.txt").exists());

        assert_eq!(
            operations::move_path(&*set, "write-a/move.txt", "write-b/moved.txt", &|| false)
                .unwrap(),
            aw_tools::filesystem::status::Primary::Ok
        );
        assert!(!write_a.join("move.txt").exists());
        assert_eq!(std::fs::read(write_b.join("moved.txt")).unwrap(), b"move");
    }

    #[cfg(unix)]
    #[test]
    fn authorized_final_symlink_content_follows_but_lifecycle_copies_link_object() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let links = base.join("links");
        let targets = base.join("targets");
        std::fs::create_dir(&links).unwrap();
        std::fs::create_dir(&targets).unwrap();
        let target = targets.join("target.txt");
        std::fs::write(&target, b"target").unwrap();
        symlink(&target, links.join("alias.txt")).unwrap();
        let access = AccessPolicy::from_spec(super::super::AccessSpec {
            base_dir: base.clone(),
            read_roots: vec![links.clone(), targets.clone()],
            write_roots: vec![links.clone(), targets.clone()],
            deny_read_roots: Vec::new(),
            deny_write_roots: Vec::new(),
            unrestricted_fs: false,
        })
        .unwrap();
        let set = prepare_with_registry(&access, &[], |_| Ok(Registry::memory())).unwrap();

        let resolved = set.resolve_content_path("links/alias.txt", false).unwrap();
        assert_eq!(set.read_bytes(&resolved, 64).unwrap(), b"target");
        operations::copy_path(&*set, "links/alias.txt", "links/copied.txt", false, &|| {
            false
        })
        .unwrap();
        assert!(
            std::fs::symlink_metadata(links.join("copied.txt"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            std::fs::read_link(links.join("copied.txt")).unwrap(),
            std::fs::read_link(links.join("alias.txt")).unwrap(),
            "lifecycle cp must duplicate the final symlink object, not dereference it"
        );
    }

    #[test]
    fn recursive_mutation_rejects_protected_descendant_before_effect() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let tree = base.join("tree");
        let protected = tree.join("protected");
        std::fs::create_dir_all(&protected).unwrap();
        std::fs::write(tree.join("visible.txt"), b"visible").unwrap();
        std::fs::write(protected.join("private.txt"), b"private").unwrap();
        let access = AccessPolicy::from_spec(super::super::AccessSpec {
            base_dir: base.clone(),
            read_roots: vec![base.clone()],
            write_roots: vec![base.clone()],
            deny_read_roots: Vec::new(),
            deny_write_roots: Vec::new(),
            unrestricted_fs: false,
        })
        .unwrap();
        let set = prepare_with_registry(&access, std::slice::from_ref(&protected), |_| {
            Ok(Registry::memory())
        })
        .unwrap();

        assert!(operations::remove_paths(&*set, &["tree".to_owned()], true, &|| false).is_err());
        assert!(tree.exists());
        assert_eq!(std::fs::read(tree.join("visible.txt")).unwrap(), b"visible");
        assert_eq!(
            std::fs::read(protected.join("private.txt")).unwrap(),
            b"private"
        );
    }

    #[cfg(unix)]
    #[test]
    fn final_symlink_cannot_cross_into_resolved_deny_root() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let denied = base.join("denied");
        std::fs::create_dir(&denied).unwrap();
        std::fs::write(denied.join("secret.txt"), b"secret").unwrap();
        symlink(denied.join("secret.txt"), base.join("alias.txt")).unwrap();
        let access = AccessPolicy::from_spec(super::super::AccessSpec {
            base_dir: base.clone(),
            read_roots: vec![base.clone()],
            write_roots: vec![base.clone()],
            deny_read_roots: guard_identities(&base, std::slice::from_ref(&denied)).unwrap(),
            deny_write_roots: guard_identities(&base, std::slice::from_ref(&denied)).unwrap(),
            unrestricted_fs: false,
        })
        .unwrap();
        let set = prepare_with_registry(&access, &[], |_| Ok(Registry::memory())).unwrap();
        assert!(
            set.resolve_content_path("alias.txt", false).is_err(),
            "content final symlink must reauthorize the resolved target against exclusions"
        );
        assert!(set.read_bytes("alias.txt", 64).is_err());
    }

    #[test]
    fn protected_paths_are_read_write_exclusions() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let private = base.join("private");
        std::fs::create_dir(&private).unwrap();
        let access = AccessPolicy::from_spec(super::super::AccessSpec {
            base_dir: base.clone(),
            read_roots: vec![base.clone()],
            write_roots: vec![base.clone()],
            deny_read_roots: Vec::new(),
            deny_write_roots: Vec::new(),
            unrestricted_fs: false,
        })
        .unwrap();
        let set = prepare_with_registry(&access, std::slice::from_ref(&private), |_| {
            Ok(Registry::memory())
        })
        .unwrap();
        assert!(set.list("private", 4).is_err());
    }
}

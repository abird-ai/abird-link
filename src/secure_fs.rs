//! Descriptor-pinned Unix filesystem capabilities used by shell/sandbox setup.
//!
//! Canonical model-facing filesystem semantics live in `aw-tools`. Dotlink keeps
//! only the host-side primitive it still owns: pin already-authorized roots and
//! host mount objects so later sandbox setup cannot be redirected by pathname
//! races.

use std::{
    ffi::CString,
    fs::File,
    io,
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::ffi::OsStrExt as _,
    },
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
#[cfg(any(target_os = "macos", test))]
use std::path::Component;

#[cfg(target_os = "linux")]
const PATH_ONLY: i32 = libc::O_PATH;
#[cfg(target_os = "macos")]
const PATH_ONLY: i32 = libc::O_RDONLY;

#[cfg(target_os = "linux")]
#[repr(C)]
struct OpenHow {
    flags: u64,
    mode: u64,
    resolve: u64,
}

const RESOLVE_NO_XDEV: u64 = 0x01;
const RESOLVE_NO_SYMLINKS: u64 = 0x04;
const RESOLVE_BENEATH: u64 = 0x08;
const SAFE_RESOLVE: u64 = RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV;

#[cfg(target_os = "linux")]
fn openat2(dir: &OwnedFd, path: &Path, flags: i32, mode: u32, resolve: u64) -> io::Result<OwnedFd> {
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in filesystem path"))?;
    let how = OpenHow {
        flags: (flags | libc::O_CLOEXEC) as u64,
        mode: mode as u64,
        resolve,
    };
    // SAFETY: pointers reference live stack/CString memory and the kernel copies
    // the fixed-size open_how before returning a newly owned descriptor.
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            dir.as_raw_fd(),
            path.as_ptr(),
            &how as *const OpenHow,
            std::mem::size_of::<OpenHow>(),
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful openat2 returned a new descriptor owned by this process.
    Ok(unsafe { OwnedFd::from_raw_fd(fd as i32) })
}

/// macOS has no `openat2`; walk from the pinned directory one component at a
/// time. `O_NOFOLLOW` rejects swapped symlinks and device checks preserve the
/// no-cross-mount policy used for prepared roots.
#[cfg(any(target_os = "macos", test))]
fn openat_beneath(
    dir: &OwnedFd,
    path: &Path,
    flags: i32,
    mode: u32,
    resolve: u64,
) -> io::Result<OwnedFd> {
    use std::os::unix::fs::MetadataExt as _;

    fn open_part(dir: &OwnedFd, part: &Path, flags: i32, mode: u32) -> io::Result<OwnedFd> {
        let name = CString::new(part.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in filesystem path"))?;
        // SAFETY: valid descriptor and NUL-terminated path.
        let fd = unsafe {
            libc::openat(
                dir.as_raw_fd(),
                name.as_ptr(),
                flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                mode,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful openat returns a new owned descriptor.
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }

    let mut parts = path.components().peekable();
    let mut cursor = dir.try_clone()?;
    if parts.peek().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty file path",
        ));
    }
    let mut current_device = File::from(cursor.try_clone()?).metadata()?.dev();
    while let Some(part) = parts.next() {
        let name = match part {
            Component::Normal(name) => Path::new(name),
            Component::CurDir => Path::new("."),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "path escapes pinned directory",
                ));
            }
        };
        let final_component = parts.peek().is_none();
        let next = open_part(
            &cursor,
            name,
            if final_component {
                flags
            } else {
                libc::O_RDONLY | libc::O_DIRECTORY
            },
            if final_component { mode } else { 0 },
        )?;
        let metadata = File::from(next.try_clone()?).metadata()?;
        if resolve & RESOLVE_NO_XDEV != 0 && metadata.dev() != current_device {
            return Err(io::Error::from_raw_os_error(libc::EXDEV));
        }
        if !final_component && !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "non-directory path component",
            ));
        }
        current_device = metadata.dev();
        cursor = next;
    }
    Ok(cursor)
}

#[cfg(target_os = "macos")]
fn openat2(dir: &OwnedFd, path: &Path, flags: i32, mode: u32, resolve: u64) -> io::Result<OwnedFd> {
    openat_beneath(dir, path, flags, mode, resolve)
}

#[derive(Debug)]
struct Root {
    path: PathBuf,
    fd: OwnedFd,
    resolve: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FileIdentity {
    pub(crate) dev: u64,
    pub(crate) ino: u64,
    pub(crate) rdev: u64,
}

pub(crate) fn file_identity(file: &File) -> io::Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt as _;
    let metadata = file.metadata()?;
    Ok(FileIdentity {
        dev: metadata.dev(),
        ino: metadata.ino(),
        rdev: metadata.rdev(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RootIdentity {
    pub(crate) path: PathBuf,
    pub(crate) dev: u64,
    pub(crate) ino: u64,
    pub(crate) rdev: u64,
}

/// Immutable root capabilities used by shell/sandbox setup. Filesystem tools use
/// the shared `aw-tools-fs-*` CapabilitySet instead.
#[derive(Debug)]
pub struct SecureFs {
    read: Vec<Root>,
    write: Vec<Root>,
}

impl SecureFs {
    pub fn new(read: &[PathBuf], write: &[PathBuf]) -> Result<Self> {
        fn open_roots(paths: &[PathBuf], already_open: &[Root]) -> Result<Vec<Root>> {
            let slash: OwnedFd = File::open("/")?.into();
            let mut roots = Vec::new();
            for path in paths {
                if let Some(original) = already_open.iter().find(|root| root.path == *path) {
                    roots.push(Root {
                        path: path.clone(),
                        fd: original.fd.try_clone()?,
                        resolve: original.resolve,
                    });
                    continue;
                }
                let relative = path.strip_prefix("/")?;
                let relative = if relative.as_os_str().is_empty() {
                    Path::new(".")
                } else {
                    relative
                };
                let fd = openat2(
                    &slash,
                    relative,
                    PATH_ONLY | libc::O_DIRECTORY,
                    0,
                    RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS,
                )
                .with_context(|| {
                    format!("cannot securely open authorized root {}", path.display())
                })?;
                roots.push(Root {
                    path: path.clone(),
                    fd,
                    resolve: if path == Path::new("/") {
                        RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS
                    } else {
                        SAFE_RESOLVE
                    },
                });
            }
            roots.sort_by_key(|root| std::cmp::Reverse(root.path.components().count()));
            Ok(roots)
        }

        let read = open_roots(read, &[])?;
        let write = open_roots(write, &read)?;
        if let Some(root) = read.first().or_else(|| write.first()) {
            openat2(
                &root.fd,
                Path::new("."),
                PATH_ONLY | libc::O_DIRECTORY,
                0,
                root.resolve,
            )
            .context("secure filesystem access requires descriptor-rooted root opening")?;
        }
        Ok(Self { read, write })
    }

    pub(crate) fn authority_roots(&self) -> Result<(Vec<RootIdentity>, Vec<RootIdentity>)> {
        let identities = |roots: &[Root]| -> Result<Vec<RootIdentity>> {
            let mut values = Vec::with_capacity(roots.len());
            for root in roots {
                let file = File::from(root.fd.try_clone()?);
                let identity = file_identity(&file)?;
                values.push(RootIdentity {
                    path: root.path.clone(),
                    dev: identity.dev,
                    ino: identity.ino,
                    rdev: identity.rdev,
                });
            }
            values.sort_by(|left, right| left.path.cmp(&right.path));
            Ok(values)
        };
        Ok((identities(&self.read)?, identities(&self.write)?))
    }

    fn root<'a, 'b>(&'a self, target: &'b Path, write: bool) -> Result<(&'a Root, &'b Path)> {
        let roots = if write { &self.write } else { &self.read };
        roots
            .iter()
            .find_map(|root| target.strip_prefix(&root.path).ok().map(|rel| (root, rel)))
            .ok_or_else(|| {
                anyhow!(
                    "path has no authorized {} root: {}",
                    if write { "write" } else { "read" },
                    target.display()
                )
            })
    }

    fn open(&self, path: &Path, write: bool, flags: i32, mode: u32) -> Result<File> {
        let (root, relative) = self.root(path, write)?;
        let relative = if relative.as_os_str().is_empty() {
            Path::new(".")
        } else {
            relative
        };
        let fd = openat2(&root.fd, relative, flags, mode, root.resolve)
            .with_context(|| format!("refused unsafe or unavailable path {}", path.display()))?;
        Ok(File::from(fd))
    }

    pub fn open_directory(&self, path: &Path) -> Result<File> {
        self.open(path, false, libc::O_RDONLY | libc::O_DIRECTORY, 0)
    }

    /// Open an opted-in host service/config path by FD, refusing symlink
    /// replacement and verifying the actual pinned object type.
    #[cfg(target_os = "linux")]
    pub fn open_host_mount(
        source: &Path,
        kind: crate::host_services::HostMountKind,
    ) -> Result<(File, PathBuf)> {
        use std::os::unix::fs::FileTypeExt as _;

        let target = std::fs::canonicalize(source)
            .with_context(|| format!("host integration disappeared: {}", source.display()))?;
        let slash: OwnedFd = File::open("/")?.into();
        let relative = target.strip_prefix("/")?;
        let fd = openat2(
            &slash,
            relative,
            PATH_ONLY | libc::O_NOFOLLOW,
            0,
            RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS,
        )
        .with_context(|| format!("unsafe host integration source {}", source.display()))?;
        let file = File::from(fd);
        let metadata = file.metadata()?;
        let valid = match kind {
            crate::host_services::HostMountKind::RegularFile => metadata.is_file(),
            crate::host_services::HostMountKind::Socket => metadata.file_type().is_socket(),
            crate::host_services::HostMountKind::Directory => metadata.is_dir(),
            crate::host_services::HostMountKind::CharacterDevice => {
                metadata.file_type().is_char_device()
            }
        };
        if !valid {
            bail!("host integration has wrong file type: {}", source.display());
        }
        Ok((file, target))
    }

    /// Pin a path just resolved as canonical for a sandbox bind.
    #[cfg(target_os = "linux")]
    pub fn pin_canonical(path: &Path) -> Result<File> {
        let slash: OwnedFd = File::open("/")?.into();
        let relative = path.strip_prefix("/")?;
        let relative = if relative.as_os_str().is_empty() {
            Path::new(".")
        } else {
            relative
        };
        let fd = openat2(
            &slash,
            relative,
            PATH_ONLY | libc::O_NOFOLLOW,
            0,
            RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS,
        )
        .with_context(|| format!("unsafe sandbox mount source {}", path.display()))?;
        Ok(File::from(fd))
    }

    /// Pin an explicitly configured shell-only cache directory.
    #[cfg(target_os = "linux")]
    pub fn open_canonical_directory(path: &Path) -> Result<File> {
        let slash: OwnedFd = File::open("/")?.into();
        let relative = path.strip_prefix("/")?;
        let relative = if relative.as_os_str().is_empty() {
            Path::new(".")
        } else {
            relative
        };
        let fd = openat2(
            &slash,
            relative,
            libc::O_RDONLY | libc::O_DIRECTORY,
            0,
            RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS,
        )
        .with_context(|| format!("unsafe shell cache source {}", path.display()))?;
        Ok(File::from(fd))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved_tempdir() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        (temp, base)
    }

    #[test]
    fn portable_openat_walk_rejects_symlink_and_parent_escape() {
        let (_temp, root) = resolved_tempdir();
        let allowed = root.join("allowed");
        let outside = root.join("outside");
        std::fs::create_dir_all(&allowed).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret"), b"secret").unwrap();
        std::os::unix::fs::symlink(&outside, allowed.join("jump")).unwrap();
        let dir: OwnedFd = File::open(&allowed).unwrap().into();
        for path in [Path::new("jump/secret"), Path::new("../outside/secret")] {
            assert!(
                openat_beneath(&dir, path, libc::O_RDONLY, 0, SAFE_RESOLVE).is_err(),
                "{path:?}"
            );
        }
    }

    #[test]
    fn pinned_root_directory_cannot_open_a_sibling() {
        let (_temp, base) = resolved_tempdir();
        let one = base.join("one");
        let two = base.join("two");
        std::fs::create_dir(&one).unwrap();
        std::fs::create_dir(&two).unwrap();
        let secure = SecureFs::new(std::slice::from_ref(&one), &[]).unwrap();
        assert!(secure.open_directory(&one).is_ok());
        assert!(secure.open_directory(&two).is_err());
        let (read, write) = secure.authority_roots().unwrap();
        assert_eq!(read.len(), 1);
        assert!(write.is_empty());
    }
}

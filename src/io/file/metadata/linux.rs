#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::NativeMetadataValue;
use std::collections::BTreeMap;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

#[repr(C)]
#[derive(Default)]
struct FilesystemAttributes {
    flags: u32,
    extent_size: u32,
    extent_count: u32,
    project_id: u32,
    cow_extent_size: u32,
    padding: [u8; 8],
}

#[expect(
    unsafe_code,
    reason = "Linux ioctl requires FFI with initialized buffers matching the linux/fs.h ABI"
)]
pub(super) fn capture_filesystem_attributes(
    path: &Path,
    metadata: &std::fs::Metadata,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    if !metadata.is_file() && !metadata.is_dir() {
        return Ok(());
    }
    nix::ioctl_read!(get_extended_attributes, b'X', 31, FilesystemAttributes);
    nix::ioctl_read!(get_inode_generation, b'v', 1, libc::c_long);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .with_context(|| {
            format!(
                "Failed to open {} for native metadata capture",
                path.display()
            )
        })?;
    let mut attributes = FilesystemAttributes::default();
    // SAFETY: The live descriptor and initialized repr(C) buffer match FS_IOC_FSGETXATTR.
    match unsafe {
        get_extended_attributes(file.as_raw_fd(), &raw mut attributes)
    } {
        Ok(_) => {
            for (name, value) in [
                ("flags", attributes.flags),
                ("extent_size", attributes.extent_size),
                ("extent_count", attributes.extent_count),
                ("project_id", attributes.project_id),
                ("cow_extent_size", attributes.cow_extent_size),
            ] {
                values.insert(
                    format!("fsxattr.{name}"),
                    NativeMetadataValue::Unsigned(u64::from(value)),
                );
            }
        }
        Err(nix::errno::Errno::ENOTTY | nix::errno::Errno::EOPNOTSUPP) => {}
        Err(error) => {
            return Err(error)
                .context("Failed to capture filesystem extended attributes");
        }
    }
    let mut generation: libc::c_long = 0;
    // SAFETY: The live descriptor and initialized c_long buffer match FS_IOC_GETVERSION.
    match unsafe { get_inode_generation(file.as_raw_fd(), &raw mut generation) }
    {
        Ok(_) => {
            values.insert(
                "inode_generation".to_owned(),
                NativeMetadataValue::Bytes(generation.to_le_bytes().to_vec()),
            );
        }
        Err(nix::errno::Errno::ENOTTY | nix::errno::Errno::EOPNOTSUPP) => {}
        Err(error) => {
            return Err(error).context("Failed to capture inode generation");
        }
    }
    Ok(())
}

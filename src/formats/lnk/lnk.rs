// SPDX-License-Identifier for parts derived from lnk_rs: MIT
// Copyright (c) 2023 Lily Hopkins
// From https://github.com/lilopkins/lnk-rs

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use binrw::{BinReaderExt, BinWrite};
use ctb_utilities::anyhow::anyhow;
use std::io::Cursor;
use std::path::{self, Path, PathBuf};

pub use lnk::encoding;
use lnk::{
    LinkFlags, LinkInfo, LinkTargetIdList, ShellLink, ShellLinkHeader,
    StringData, StringEncoding,
};

#[cfg(test)]
use include_dir::{Dir, include_dir};

#[cfg(test)]
static LNK_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

#[cfg(test)]
pub(crate) fn get_lnk_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&LNK_DATA_DIR, key)
}

fn split_windowsish_path(path: &str) -> (Option<&str>, &str) {
    let sep_index = path.rfind('\\').into_iter().chain(path.rfind('/')).max();

    let Some(sep_index) = sep_index else {
        return (None, path);
    };

    let working_dir = path.get(..sep_index).filter(|d| !d.is_empty());
    // Reason for fallback: if sep_index + 1 exceeds path length, fallback to whole path as file name.
    let file_name = path.get(sep_index.saturating_add(1)..).unwrap_or(path);

    (working_dir, file_name)
}

pub fn create_simple_lnk<P: AsRef<std::path::Path>>(
    target_path: P,
    name: Option<&str>,
) -> Result<Vec<u8>> {
    // NOTE: `ShellLink::new_simple` touches the filesystem (metadata + canonicalize).
    // This helper is meant to be purely in-memory, so we construct a default link and
    // populate the relevant StringData fields ourselves.
    let target = target_path.as_ref().to_string_lossy();
    let (working_dir, file_name) = split_windowsish_path(&target);

    // Reason for fallback: working directory defaults to current directory '.' if no path separator was present in target path.
    let working_dir = working_dir.unwrap_or(".");
    let relative_name = if file_name.is_empty() {
        target.as_ref()
    } else {
        file_name
    };
    let relative_path = format!(".\\{relative_name}");

    let mut lnk = ShellLink::default().with_encoding(&StringEncoding::Unicode);
    if let Some(name) = name {
        lnk.set_name(Some(name.to_owned()));
    } else {
        lnk.set_name(None);
    }
    lnk.set_relative_path(Some(relative_path));
    lnk.set_working_dir(Some(working_dir.to_string()));
    let mut buffer: Vec<u8> = Vec::new();

    let mut w = Cursor::new(&mut buffer);

    debug!("Writing header...");
    // Invoke binwrite
    lnk.header()
        .write_le(&mut w)
        .context("Failed to write LNK header")?;

    let link_flags = *lnk.header().link_flags();
    debug!("Writing StringData...");
    lnk.string_data()
        .write_le_args(&mut w, (link_flags, encoding::UTF_16LE))
        .context("Failed to write LNK string data")?;

    // if link_flags.contains(LinkFlags::HAS_LINK_TARGET_ID_LIST) {
    //     if let None = lnk.linktarget_id_list {
    //         error!("LinkTargetIDList not specified but expected!")
    //     }
    //     debug!("A LinkTargetIDList is marked as present. Writing.");
    //     let mut data: Vec<u8> = lnk.linktarget_id_list.clone().unwrap().into();
    //     w.write_all(&mut data)?;
    // }

    // if link_flags.contains(LinkFlags::HAS_LINK_INFO) {
    //     if let None = lnk.link_info {
    //         error!("LinkInfo not specified but expected!")
    //     }
    //     debug!("LinkInfo is marked as present. Writing.");
    //     let mut data: Vec<u8> = lnk.link_info.clone().unwrap().into();
    //     w.write_all(&mut data)?;
    // }

    // if link_flags.contains(LinkFlags::HAS_NAME) {
    //     if lnk.name_string == None {
    //         error!("Name not specified but expected!")
    //     }
    //     debug!("Name is marked as present. Writing.");
    //     w.write_all(&stringdata::to_data(
    //         lnk.name_string.as_ref().unwrap(),
    //         link_flags,
    //     ))?;
    // }

    // if link_flags.contains(LinkFlags::HAS_RELATIVE_PATH) {
    //     if lnk.relative_path == None {
    //         error!("Relative path not specified but expected!")
    //     }
    //     debug!("Relative path is marked as present. Writing.");
    //     w.write_all(&stringdata::to_data(
    //         lnk.relative_path.as_ref().unwrap(),
    //         link_flags,
    //     ))?;
    // }

    // if link_flags.contains(LinkFlags::HAS_WORKING_DIR) {
    //     if lnk.working_dir == None {
    //         error!("Working Directory not specified but expected!")
    //     }
    //     debug!("Working dir is marked as present. Writing.");
    //     w.write_all(&stringdata::to_data(
    //         lnk.working_dir.as_ref().unwrap(),
    //         link_flags,
    //     ))?;
    // }

    // if link_flags.contains(LinkFlags::HAS_ARGUMENTS) {
    //     if lnk.icon_location == None {
    //         error!("Arguments not specified but expected!")
    //     }
    //     debug!("Arguments are marked as present. Writing.");
    //     w.write_all(&stringdata::to_data(
    //         lnk.command_line_arguments.as_ref().unwrap(),
    //         link_flags,
    //     ))?;
    // }

    // if link_flags.contains(LinkFlags::HAS_ICON_LOCATION) {
    //     if lnk.icon_location == None {
    //         error!("Icon Location not specified but expected!")
    //     }
    //     debug!("Icon Location is marked as present. Writing.");
    //     w.write_all(&stringdata::to_data(
    //         lnk.icon_location.as_ref().unwrap(),
    //         link_flags,
    //     ))?;
    // }

    Ok(buffer)
}

/// Open and parse a shell link
///
/// All string which are stored in the `lnk` file are encoded with either
/// Unicode (UTF-16LE) of any of the Windows code pages. Which of both is
/// being used is specified by the [`LinkFlags::IS_UNICODE`] flag. Microsoft
/// documents this as follows:
///
/// > If this bit is set, the StringData section contains Unicode-encoded
/// > strings; otherwise, it contains strings that are encoded using the
/// > system default code page.
/// >
/// > (<https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/ae350202-3ba9-4790-9e9e-98935f4ee5af>)
///
/// The system default code page is stored in
/// `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Nls\CodePage\ACP`
///
/// Because we do not know what the system default code page was, you must
/// specify this using the `encoding` parameter (see below). If you you do
/// not know the system default code page either, you're lost. There is no
/// way to correctly guess the used code page from the data in the `lnk`
/// file.
///
/// * `path` - path of the `lnk` file to be analyzed
/// * `encoding` - character encoding to be used if the `lnk` file is not
///   Unicode encoded
pub fn read_path_from_lnk<P: AsRef<Path>>(
    lnk_path: P,
    encoding: lnk::Encoding,
) -> Result<path::PathBuf> {
    let bytes = std::fs::read(lnk_path.as_ref()).with_context(|| {
        format!("Failed to read lnk file {}", lnk_path.as_ref().display())
    })?;
    read_path_from_lnk_bytes(&bytes, encoding)
}

pub fn read_path_from_lnk_bytes(
    bytes: &[u8],
    encoding: lnk::Encoding,
) -> Result<PathBuf> {
    let mut reader = Cursor::new(bytes);
    let header: ShellLinkHeader =
        reader.read_le().context("Failed to parse LNK header")?;
    let link_flags = *header.link_flags();

    if link_flags.contains(LinkFlags::HAS_LINK_TARGET_ID_LIST) {
        let _: LinkTargetIdList = reader
            .read_le()
            .context("Failed to parse LinkTargetIdList")?;
    }

    let link_info = if link_flags.contains(LinkFlags::HAS_LINK_INFO) {
        Some(
            reader
                .read_le_args((encoding,))
                .context("Failed to parse LinkInfo")?,
        )
    } else {
        None
    };

    let string_data: StringData = reader
        .read_le_args((link_flags, encoding))
        .context("Failed to parse StringData")?;

    if let Some(info) = link_info.as_ref() {
        return build_path_from_link_info(info);
    }

    build_path_from_string_data(&string_data)
}

fn build_path_from_link_info(info: &LinkInfo) -> Result<PathBuf> {
    let mut base_path = if info
        .link_info_flags()
        .has_common_network_relative_link_and_path_suffix()
    {
        let common = info
            .common_network_relative_link()
            .as_ref()
            .context("Missing CommonNetworkRelativeLink in LinkInfo")?;
        std::panic::catch_unwind(|| common.name()).map_err(|panic_payload| {
            // Reason for fallback: default description used when catch_unwind panic payload is not a str or String.
            let payload_msg = panic_payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "non-string panic payload".to_string());
            anyhow!(
                "Failed to read CommonNetworkRelativeLink name: {payload_msg}"
            )
        })?
    } else {
        let base_path = if let Some(local_base_path_unicode) =
            info.local_base_path_unicode().as_ref()
        {
            local_base_path_unicode.as_str()
        } else if let Some(local_base_path) = info.local_base_path() {
            local_base_path
        } else {
            bail!("Missing local base path in LinkInfo");
        };

        base_path.to_string()
    };

    let common_path = if let Some(common_path_suffix_unicode) =
        info.common_path_suffix_unicode()
    {
        common_path_suffix_unicode.as_str()
    } else {
        info.common_path_suffix()
    };

    if !common_path.is_empty() {
        if !base_path.ends_with('\\') {
            base_path.push('\\');
        }
        base_path.push_str(common_path);
    }

    Ok(PathBuf::from(base_path))
}

fn build_path_from_string_data(string_data: &StringData) -> Result<PathBuf> {
    let relative_path = string_data
        .relative_path()
        .as_ref()
        .context("Missing RelativePath in StringData")?;
    // Reason for fallback: if relative path has no './' or '.\\' prefix, original relative path is retained.
    let relative_path = relative_path
        .strip_prefix(".\\")
        .or(relative_path.strip_prefix("./"))
        .unwrap_or(relative_path);

    if let Some(working_dir) = string_data.working_dir().as_ref() {
        let mut path = PathBuf::from(working_dir);
        path.push(relative_path);
        return Ok(path);
    }

    Ok(PathBuf::from(relative_path))
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use binrw::BinReaderExt;

    static NEXT_TEST_LNK_ID: AtomicU64 = AtomicU64::new(0);

    #[crate::ctb_test]
    fn test_create_lnk_doesnt_explode() {
        create_simple_lnk(
            PathBuf::from(r"C:\Windows\System32\notepad.exe"),
            Some("Notepad"),
        )
        .unwrap();
        create_simple_lnk(
            PathBuf::from(r"C:\Windows\System32\notepad.exe"),
            None,
        )
        .unwrap();
    }

    #[crate::ctb_test]
    fn test_create_lnk_contains_header_and_string_data() {
        let bytes = create_simple_lnk(
            PathBuf::from(r"C:\Windows\System32\notepad.exe"),
            Some("Notepad"),
        )
        .unwrap();

        let mut reader = Cursor::new(&bytes);
        let header: ShellLinkHeader = reader.read_le().unwrap();
        let link_flags = *header.link_flags();
        assert!(link_flags.contains(LinkFlags::HAS_NAME));
        assert!(link_flags.contains(LinkFlags::HAS_RELATIVE_PATH));
        assert!(link_flags.contains(LinkFlags::HAS_WORKING_DIR));

        let string_data: StringData = reader
            .read_le_args((link_flags, encoding::UTF_16LE))
            .unwrap();
        assert_eq!(string_data.name_string().as_deref(), Some("Notepad"));
    }

    #[crate::ctb_test]
    fn test_read_path_from_lnk_bytes_matches_lnk_rs() {
        /* FIXME: Maybe flaky - this test crashed once with this message:

                thread 'tests::test_read_path_from_lnk_matches_lnk_rs' (207123) panicked at src/formats/lnk/lnk.rs:364:14:
        called `Result::unwrap()` on an `Err` value: BinReadError("LinkInfo",
         ╺━━━━━━━━━━━━━━━━━━━━┅ Backtrace ┅━━━━━━━━━━━━━━━━━━━━╸

         0: Error: failed to fill whole buffer
                   While parsing field 'volume_id_size' in VolumeID
             at ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lnk-0.6.4/src/linkinfo.rs:350
         1: While parsing field 'volume_id' in LinkInfo
             at ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lnk-0.6.4/src/linkinfo.rs:205

             */
        let bytes = test_lnk();
        let file_path = test_lnk_path();

        let expected = ShellLink::open(&file_path, encoding::WINDOWS_1252)
            .unwrap()
            .link_target()
            .expect("Expected LinkInfo-based target path");
        let actual =
            read_path_from_lnk_bytes(&bytes, encoding::WINDOWS_1252).unwrap();

        assert_eq!(actual, PathBuf::from(expected));
    }

    #[crate::ctb_test]
    fn test_read_path_from_lnk_matches_lnk_rs() {
        /* FIXME: Maybe flaky - this test crashed once with this message:

        [2026-06-08T05:20:56.742761Z TRACE ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lnk-0.6.4/src/lib.rs:330 log workspace:1156279/workspace] test_read_path_from_lnk_bytes_matches_lnk_rs: Reading file.

        thread 'tests::test_read_path_from_lnk_bytes_matches_lnk_rs' (1156282) panicked at src/formats/lnk/lnk.rs:364:14:
        called `Result::unwrap()` on an `Err` value: BinReadError("ShellLinkHeader",
         ╺━━━━━━━━━━━━━━━━━━━━┅ Backtrace ┅━━━━━━━━━━━━━━━━━━━━╸

         0: Error: failed to fill whole buffer
                   While parsing field 'header_size' in ShellLinkHeader
             at ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lnk-0.6.4/src/header.rs:32


             */
        let lnk_path = test_lnk_path();
        let expected = ShellLink::open(&lnk_path, encoding::WINDOWS_1252)
            .unwrap()
            .link_target()
            .expect("Expected LinkInfo-based target path");

        let actual =
            read_path_from_lnk(&lnk_path, encoding::WINDOWS_1252).unwrap();
        assert_eq!(actual, PathBuf::from(expected));
    }

    fn test_lnk() -> Vec<u8> {
        get_lnk_data("fixtures/test.lnk").expect("Failed to get test lnk data")
    }

    fn test_lnk_path() -> PathBuf {
        let lnk = test_lnk();
        let unique_id = NEXT_TEST_LNK_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ctoolbox-test-{}-{}.lnk",
            std::process::id(),
            unique_id
        ));
        std::fs::write(&path, &lnk).expect("Failed to write test lnk file");
        path
    }
}

/*

// From lnk-rs:

MIT License

Copyright (c) 2023 Lily Hopkins

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

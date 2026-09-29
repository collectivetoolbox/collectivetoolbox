// SPDX-License-Identifier: AGPL-3.0-or-later AND Apache-2.0
// SPDX-License-Identifier for parts derived from szip: Apache-2.0
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// Parts derived from szip:
/*
* Copyright 1997,1998,2021 Michael Schindler michael@compressconsult.com
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*/
// See end of file for full license text for parts derived from szip.

//! Implementation of the `szip` 1.11+ compression format (Michael Schindler, 1997-2000).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;
use anyhow::anyhow;
use std::io::{Read, Write};

pub mod bit_model;
pub mod model;
pub mod qs_model;
pub mod range_coder;
pub mod reorder;
pub mod sort_transform;

use model::SzModel;
use range_coder::{RangeDecoder, RangeEncoder};
use reorder::{forward_delta, inverse_delta, reorder, unreorder};
use sort_transform::{sort_bwt, sort_general, sort_order4, unsort_bwt, unsort_general};

/// Global header magic bytes (`"SZ\n\x04"`).
pub const SZIP_GLOBAL_MAGIC: [u8; 4] = [0x53, 0x5A, 0x0A, 0x04];

/// Block directory header magic bytes (`"BH"`).
pub const SZIP_BLOCK_MAGIC: [u8; 2] = [0x42, 0x48];

/// Default block size in bytes (approx 1.66 MiB matching `-b17`).
pub const DEFAULT_BLOCK_SIZE: usize = 1_703_936;

/// Default context order.
pub const DEFAULT_ORDER: u8 = 6;

/// Default record size in bytes.
pub const DEFAULT_RECORD_SIZE: u8 = 1;

/// Configuration options for compressing szip streams.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SzipOptions {
    /// Maximum uncompressed bytes per block (default: 1,703,936).
    pub block_size: usize,
    /// Context sorting order (0 for unlimited BWT, or 3..255; default: 6).
    pub order: u8,
    /// Elementary record channel size in bytes (1..127; default: 1).
    pub record_size: u8,
    /// Whether to enable differential (delta) coding across samples (default: false).
    pub incremental: bool,
}

impl Default for SzipOptions {
    fn default() -> Self {
        Self {
            block_size: DEFAULT_BLOCK_SIZE,
            order: DEFAULT_ORDER,
            record_size: DEFAULT_RECORD_SIZE,
            incremental: false,
        }
    }
}

fn write_u24<W: Write>(writer: &mut W, val: u32) -> Result<()> {
    let b1 = u8::try_from((val >> 16) & 0xFF)
        .map_err(|e| anyhow!("u24 byte 1 conversion: {e}"))?;
    let b2 = u8::try_from((val >> 8) & 0xFF)
        .map_err(|e| anyhow!("u24 byte 2 conversion: {e}"))?;
    let b3 = u8::try_from(val & 0xFF)
        .map_err(|e| anyhow!("u24 byte 3 conversion: {e}"))?;
    writer.write_all(&[b1, b2, b3])?;
    Ok(())
}

fn read_u24<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 3];
    reader
        .read_exact(&mut buf)
        .context("Failed to read 3-byte integer")?;
    let b1 = u32::from(buf.first().copied().unwrap_or(0));
    let b2 = u32::from(buf.get(1).copied().unwrap_or(0));
    let b3 = u32::from(buf.get(2).copied().unwrap_or(0));
    Ok((b1 << 16) | (b2 << 8) | b3)
}

/// Compresses a stream from `reader` into `writer` using standard szip 1.11+ format.
pub fn compress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    compress_stream_options(reader, writer, &SzipOptions::default())
}

/// Compresses a stream from `reader` into `writer` with explicit options.
pub fn compress_stream_options(
    reader: &mut impl Read,
    writer: &mut impl Write,
    options: &SzipOptions,
) -> Result<u64> {
    if options.order == 1 || options.order == 2 {
        bail!("Szip context order 1 and 2 are invalid; use 0 or >= 3");
    }
    let rec_size = options.record_size & 0x7F;
    if rec_size == 0 {
        bail!("Szip record size must be between 1 and 127");
    }

    // Write global header: "SZ\n\x04" followed by version 1.11 (0x01, 0x0B)
    writer.write_all(&SZIP_GLOBAL_MAGIC)?;
    writer.write_all(&[1, 11])?;

    let mut total_uncompressed = 0u64;
    let mut block_buf = vec![0u8; options.block_size];

    loop {
        let mut bytes_read = 0usize;
        while bytes_read < options.block_size {
            let Some(slice) = block_buf.get_mut(bytes_read..) else {
                break;
            };
            let n = reader.read(slice)?;
            if n == 0 {
                break;
            }
            bytes_read = bytes_read.saturating_add(n);
        }
        if bytes_read == 0 {
            break;
        }

        total_uncompressed = total_uncompressed.saturating_add(
            u64::try_from(bytes_read).map_err(|e| anyhow!("Length conversion: {e}"))?,
        );

        let Some(cur_slice) = block_buf.get_mut(..bytes_read) else {
            continue;
        };
        let buflen_u32 = u32::try_from(bytes_read)
            .map_err(|e| anyhow!("Block length conversion: {e}"))?;

        // Write block directory: "BH", 3-byte buflen, and 0x00 directory terminator
        writer.write_all(&SZIP_BLOCK_MAGIC)?;
        write_u24(writer, buflen_u32)?;
        writer.write_all(&[0x00])?;
        let dir_size = 6u32;

        if bytes_read <= usize::from(options.order) || bytes_read <= 5 {
            // Stored uncompressed block
            writer.write_all(&[0x00])?;
            writer.write_all(cur_slice)?;
            let trailer_len = dir_size.saturating_add(4).saturating_add(buflen_u32);
            write_u24(writer, trailer_len)?;
        } else {
            // Szip compressed block
            writer.write_all(&[0x01])?;

            if rec_size > 1 {
                let mut tmp = vec![0u8; bytes_read];
                reorder(cur_slice, &mut tmp, usize::from(rec_size))?;
                cur_slice.copy_from_slice(&tmp);
            }

            if options.incremental {
                forward_delta(cur_slice);
            }

            let indexlast = if options.order == 4 {
                sort_order4(cur_slice)?
            } else if options.order == 0 {
                sort_bwt(cur_slice)?
            } else {
                sort_general(cur_slice, usize::from(options.order))?
            };

            write_u24(writer, indexlast)?;
            writer.write_all(&[options.order])?;

            let record_byte = (options.record_size & 0x7F)
                | if options.incremental { 0x80 } else { 0 };

            let mut range_enc = RangeEncoder::new(
                &mut *writer,
                record_byte,
                dir_size.saturating_add(5),
            );
            let mut model = SzModel::new(true)?;

            let mut pos = 0usize;
            let mut is_first_run = true;
            while pos < bytes_read {
                let sym = cur_slice.get(pos).copied().unwrap_or(0);
                let mut run_len = 1u32;
                pos = pos.saturating_add(1);
                while pos < bytes_read && cur_slice.get(pos) == Some(&sym) {
                    run_len = run_len.saturating_add(1);
                    pos = pos.saturating_add(1);
                }
                model.encode(&mut range_enc, sym, run_len)?;
                if is_first_run {
                    model.fix_after_first();
                    is_first_run = false;
                }
            }
            range_enc.finish()?;
        }
    }

    Ok(total_uncompressed)
}

fn decompress_stored_block(
    reader: &mut impl Read,
    writer: &mut impl Write,
    buflen: usize,
    dir_size: u32,
    buflen_u32: u32,
) -> Result<u64> {
    let mut buf = vec![0u8; buflen];
    reader
        .read_exact(&mut buf)
        .context("Failed reading stored block payload")?;
    let stored_trailer = read_u24(reader)?;
    let expected_trailer = dir_size.saturating_add(4).saturating_add(buflen_u32);
    if stored_trailer != expected_trailer {
        bail!("Stored block trailer size mismatch");
    }
    writer.write_all(&buf)?;
    u64::try_from(buflen).map_err(|e| anyhow!("Length conversion: {e}"))
}

fn decompress_szip_block(
    reader: &mut impl Read,
    writer: &mut impl Write,
    buflen: usize,
) -> Result<u64> {
    let indexlast = read_u24(reader)?;
    let mut order_buf = [0u8; 1];
    reader
        .read_exact(&mut order_buf)
        .context("Failed reading block order")?;
    let order = order_buf.first().copied().unwrap_or(0);

    let Some((mut range_dec, record_byte)) = RangeDecoder::new(reader.by_ref())? else {
        bail!("Unexpected EOF in range coder stream");
    };

    let incremental = (record_byte & 0x80) != 0;
    let rec_size = usize::from(record_byte & 0x7F);

    let mut model = SzModel::new(false)?;
    let mut decoded_buf = vec![0u8; buflen];

    let mut bytes_left = buflen;
    let mut out_idx = 0usize;
    let mut is_first_run = true;

    while bytes_left > 0 {
        let (sym, run_len_u32) = model.decode(&mut range_dec)?;
        let run_len = usize::try_from(run_len_u32)
            .map_err(|e| anyhow!("Run length conversion: {e}"))?;
        if run_len > bytes_left {
            bail!("Corrupted szip block: run length exceeds block size");
        }
        for _ in 0..run_len {
            if let Some(slot) = decoded_buf.get_mut(out_idx) {
                *slot = sym;
            }
            out_idx = out_idx.saturating_add(1);
        }
        bytes_left = bytes_left.saturating_sub(run_len);
        if is_first_run {
            model.fix_after_first();
            is_first_run = false;
        }
    }

    range_dec.finish()?;

    let mut transformed = vec![0u8; buflen];
    if order == 0 {
        unsort_bwt(&decoded_buf, &mut transformed, indexlast)?;
    } else if order >= 3 {
        unsort_general(&decoded_buf, &mut transformed, indexlast, usize::from(order))?;
    } else {
        bail!("Unsupported context order in block: {order}");
    }

    if incremental {
        inverse_delta(&mut transformed);
    }

    if rec_size > 1 {
        let mut unreordered = vec![0u8; buflen];
        unreorder(&transformed, &mut unreordered, rec_size)?;
        writer.write_all(&unreordered)?;
    } else {
        writer.write_all(&transformed)?;
    }

    u64::try_from(buflen).map_err(|e| anyhow!("Length conversion: {e}"))
}

/// Decompresses an szip 1.11+ stream from `reader` into `writer`.
pub fn decompress_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<u64> {
    let mut total_decompressed = 0u64;

    loop {
        let mut first_byte_buf = [0u8; 1];
        let n = reader.read(&mut first_byte_buf)?;
        if n == 0 {
            break;
        }
        let Some(&first_byte) = first_byte_buf.first() else {
            break;
        };
        let mut ch = first_byte;

        // Check for optional global header: "SZ\n\x04"
        if ch == SZIP_GLOBAL_MAGIC[0] {
            let mut rest_magic = [0u8; 3];
            reader
                .read_exact(&mut rest_magic)
                .context("Failed reading global magic header")?;
            if rest_magic != SZIP_GLOBAL_MAGIC[1..] {
                bail!("Invalid szip global header magic");
            }
            let mut ver = [0u8; 2];
            reader
                .read_exact(&mut ver)
                .context("Failed reading szip version header")?;
            let vmay = ver.first().copied().unwrap_or(0);
            let vmin = ver.get(1).copied().unwrap_or(0);

            if vmay > 1 || (vmay == 1 && vmin > 12) {
                bail!("Unsupported future szip version {vmay}.{vmin}");
            }
            if vmay == 1 && vmin == 10 {
                bail!("szip 1.10 (alpha) format is unsupported due to encoder bugs");
            }

            let mut next_buf = [0u8; 1];
            let n2 = reader.read(&mut next_buf)?;
            if n2 == 0 {
                break;
            }
            ch = next_buf.first().copied().unwrap_or(0);
        }

        // Must be block magic "BH"
        if ch != SZIP_BLOCK_MAGIC[0] {
            bail!("Not an szip block: expected 'B' (0x42), found 0x{ch:02X}");
        }
        let mut h_buf = [0u8; 1];
        reader
            .read_exact(&mut h_buf)
            .context("Failed reading block header 'H'")?;
        if h_buf.first().copied().unwrap_or(0) != SZIP_BLOCK_MAGIC[1] {
            bail!("Invalid block magic: expected 'H' (0x48)");
        }

        let buflen_u32 = read_u24(reader)?;
        let mut term_buf = [0u8; 1];
        reader
            .read_exact(&mut term_buf)
            .context("Failed reading block directory terminator")?;
        if term_buf.first().copied().unwrap_or(0xFF) != 0x00 {
            bail!("Invalid block directory terminator");
        }
        let dir_size = 6u32;
        let buflen = usize::try_from(buflen_u32)
            .map_err(|e| anyhow!("Block length conversion: {e}"))?;

        let mut type_buf = [0u8; 1];
        reader
            .read_exact(&mut type_buf)
            .context("Failed reading block type")?;
        let block_type = type_buf.first().copied().unwrap_or(0xFF);

        if block_type == 0 {
            let n = decompress_stored_block(reader, writer, buflen, dir_size, buflen_u32)?;
            total_decompressed = total_decompressed.saturating_add(n);
        } else if block_type == 1 {
            let n = decompress_szip_block(reader, writer, buflen)?;
            total_decompressed = total_decompressed.saturating_add(n);
        } else {
            bail!("Unknown szip block type: {block_type}");
        }
    }

    Ok(total_decompressed)
}

/// Compresses in-memory byte slice using standard szip 1.11+ format.
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    compress_stream(&mut input, &mut output)?;
    Ok(output)
}

/// Decompresses in-memory byte slice using standard szip 1.11+ format.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut input = data;
    let mut output = Vec::new();
    decompress_stream(&mut input, &mut output)?;
    Ok(output)
}

#[cfg(test)]
#[allow(
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
    use super::*;

    #[crate::ctb_test]
    fn test_szip_roundtrip_empty() {
        let input: &[u8] = b"";
        let compressed = compress(input).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[crate::ctb_test]
    fn test_szip_roundtrip_small() {
        let input = b"Hello, Szip 1.11+ compression!";
        let compressed = compress(input).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[crate::ctb_test]
    fn test_szip_roundtrip_repetitive() {
        let input = vec![b'Z'; 5000];
        let compressed = compress(&input).unwrap();
        assert!(compressed.len() < input.len());
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[crate::ctb_test]
    fn test_szip_roundtrip_options() {
        for order in [0u8, 4, 6] {
            for record_size in [1u8, 3] {
                for incremental in [false, true] {
                    let options = SzipOptions {
                        block_size: 1024,
                        order,
                        record_size,
                        incremental,
                    };
                    let input = b"Test multi-block options roundtrip with varied pattern! 1234567890".repeat(50);
                    let mut compressed = Vec::new();
                    compress_stream_options(&mut input.as_slice(), &mut compressed, &options).unwrap();
                    let decompressed = decompress(&compressed).unwrap();
                    assert_eq!(decompressed, input);
                }
            }
        }
    }
}

/*

From `LICENSE` in szip:

```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

   9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or additional liability.

   END OF TERMS AND CONDITIONS

   APPENDIX: How to apply the Apache License to your work.

      To apply the Apache License to your work, attach the following
      boilerplate notice, with the fields enclosed by brackets "[]"
      replaced with your own identifying information. (Don't include
      the brackets!)  The text should be enclosed in the appropriate
      comment syntax for the file format. We also recommend that a
      file or class name and description of purpose be included on the
      same "printed page" as the copyright notice for easier
      identification within third-party archives.

   Copyright [yyyy] [name of copyright owner]

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```
*/
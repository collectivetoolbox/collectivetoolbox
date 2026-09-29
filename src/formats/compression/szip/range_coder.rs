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

//! Byte-renormalized arithmetic range encoder and decoder (Schindler, 1998).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;
use std::io::{Read, Write};

pub(crate) const TOP_VALUE: u32 = 1 << 31;
pub(crate) const BOTTOM_VALUE: u32 = TOP_VALUE >> 8;
pub(crate) const SHIFT_BITS: u32 = 23;
pub(crate) const EXTRA_BITS: u32 = 7;
pub(crate) const LOW_MASK: u32 = TOP_VALUE.wrapping_sub(1);

/// Schindler 32-bit byte-renormalized arithmetic range encoder.
pub struct RangeEncoder<W: Write> {
    writer: W,
    low: u32,
    range: u32,
    help: u32,
    buffer: u8,
    bytecount: u32,
}

impl<W: Write> RangeEncoder<W> {
    /// Initializes a new range encoder with an initial header byte and start length.
    pub fn new(writer: W, initial_byte: u8, init_length: u32) -> Self {
        Self {
            writer,
            low: 0,
            range: TOP_VALUE,
            buffer: initial_byte,
            help: 0,
            bytecount: init_length,
        }
    }

    /// Normalizes encoder state, writing bytes and carrying out pending carries.
    pub fn normalize(&mut self) -> Result<()> {
        let no_carry_bound = 0xFF_u32 << SHIFT_BITS;
        while self.range <= BOTTOM_VALUE {
            if self.low < no_carry_bound {
                self.writer
                    .write_all(&[self.buffer])
                    .context("Failed to write range coder byte")?;
                while self.help > 0 {
                    self.writer
                        .write_all(&[0xFF])
                        .context("Failed to write carry propagation byte")?;
                    self.help = self.help.saturating_sub(1);
                }
                self.buffer = u8::try_from((self.low >> SHIFT_BITS) & 0xFF)
                    .map_err(|e| anyhow!("Range low shift overflow: {e}"))?;
            } else if (self.low & TOP_VALUE) != 0 {
                let carry_byte = self.buffer.wrapping_add(1);
                self.writer
                    .write_all(&[carry_byte])
                    .context("Failed to write carry byte")?;
                while self.help > 0 {
                    self.writer
                        .write_all(&[0x00])
                        .context("Failed to write zero propagation byte")?;
                    self.help = self.help.saturating_sub(1);
                }
                self.buffer = u8::try_from((self.low >> SHIFT_BITS) & 0xFF)
                    .map_err(|e| anyhow!("Range low shift overflow: {e}"))?;
            } else {
                self.help = self.help.saturating_add(1);
            }
            self.range <<= 8;
            self.low = (self.low << 8) & LOW_MASK;
            self.bytecount = self.bytecount.saturating_add(1);
        }
        Ok(())
    }

    /// Encodes a symbol interval using explicit frequency totals.
    pub fn encode_freq(&mut self, sy_f: u32, lt_f: u32, tot_f: u32) -> Result<()> {
        self.normalize()?;
        if tot_f == 0 {
            bail!("Range encoder total frequency cannot be zero");
        }
        let r = self
            .range
            .checked_div(tot_f)
            .ok_or_else(|| anyhow!("Divide by zero in encode_freq"))?;
        let tmp = r.saturating_mul(lt_f);
        self.low = self.low.wrapping_add(tmp);
        self.range = r.saturating_mul(sy_f);
        Ok(())
    }

    /// Encodes a symbol interval where the total frequency is a power-of-two shift.
    pub fn encode_shift(&mut self, sy_f: u32, lt_f: u32, shift: u32) -> Result<()> {
        self.normalize()?;
        let r = self.range >> shift;
        let tmp = r.saturating_mul(lt_f);
        self.low = self.low.wrapping_add(tmp);
        self.range = r.saturating_mul(sy_f);
        Ok(())
    }

    /// Flushes encoder state, writing final interval bytes and the 3-byte trailer.
    pub fn finish(mut self) -> Result<u32> {
        self.normalize()?;
        self.bytecount = self.bytecount.saturating_add(5);
        let half_bc = self.bytecount >> 1;
        let bottom_mask = BOTTOM_VALUE.saturating_sub(1);
        let tmp = if (self.low & bottom_mask) < half_bc {
            self.low >> SHIFT_BITS
        } else {
            (self.low >> SHIFT_BITS).saturating_add(1)
        };

        if tmp > 0xFF {
            let carry_byte = self.buffer.wrapping_add(1);
            self.writer
                .write_all(&[carry_byte])
                .context("Failed to write final carry byte")?;
            while self.help > 0 {
                self.writer
                    .write_all(&[0x00])
                    .context("Failed to write final zero byte")?;
                self.help = self.help.saturating_sub(1);
            }
        } else {
            self.writer
                .write_all(&[self.buffer])
                .context("Failed to write final buffer byte")?;
            while self.help > 0 {
                self.writer
                    .write_all(&[0xFF])
                    .context("Failed to write final FF byte")?;
                self.help = self.help.saturating_sub(1);
            }
        }

        let tmp_u8 = u8::try_from(tmp & 0xFF)
            .map_err(|e| anyhow!("Final code byte conversion: {e}"))?;
        let bc_high = u8::try_from((self.bytecount >> 16) & 0xFF)
            .map_err(|e| anyhow!("Bytecount high conversion: {e}"))?;
        let bc_mid = u8::try_from((self.bytecount >> 8) & 0xFF)
            .map_err(|e| anyhow!("Bytecount mid conversion: {e}"))?;
        let bc_low = u8::try_from(self.bytecount & 0xFF)
            .map_err(|e| anyhow!("Bytecount low conversion: {e}"))?;

        self.writer
            .write_all(&[tmp_u8, bc_high, bc_mid, bc_low])
            .context("Failed to write range coder block trailer")?;
        self.writer.flush().context("Failed to flush range encoder")?;
        Ok(self.bytecount)
    }
}

/// Schindler 32-bit byte-renormalized arithmetic range decoder.
pub struct RangeDecoder<R: Read> {
    reader: R,
    low: u32,
    range: u32,
    help: u32,
    buffer: u8,
}

impl<R: Read> RangeDecoder<R> {
    fn read_byte_opt(&mut self) -> Result<Option<u8>> {
        let mut buf = [0u8; 1];
        let n = self
            .reader
            .read(&mut buf)
            .context("Failed to read byte from stream")?;
        if n == 0 {
            Ok(None)
        } else {
            match buf.first() {
                Some(&b) => Ok(Some(b)),
                None => Ok(None),
            }
        }
    }

    /// Starts decoding by reading the initial stream byte and priming registers.
    pub fn new(mut reader: R) -> Result<Option<(Self, u8)>> {
        let mut first_buf = [0u8; 1];
        let n1 = reader
            .read(&mut first_buf)
            .context("Failed to read first range coder byte")?;
        if n1 == 0 {
            return Ok(None);
        }
        let Some(&first_byte) = first_buf.first() else {
            return Ok(None);
        };

        let mut second_buf = [0u8; 1];
        let n2 = reader
            .read(&mut second_buf)
            .context("Failed to read second range coder byte")?;
        let [b] = second_buf;
        let buffer_byte = if n2 == 0 { 0 } else { b };

        let low = u32::from(buffer_byte >> 1); // 8 - EXTRA_BITS = 1
        let range = 1_u32 << EXTRA_BITS; // 128

        Ok(Some((
            Self {
                reader,
                low,
                range,
                help: 0,
                buffer: buffer_byte,
            },
            first_byte,
        )))
    }

    /// Renormalizes decoder state, reading incoming bytes into code registers.
    pub fn normalize(&mut self) -> Result<()> {
        while self.range <= BOTTOM_VALUE {
            // Reason for fallback: range decoder shifts in trailing zeroes at EOF to flush pending bits
            let next_byte = self.read_byte_opt()?.unwrap_or(0);
            self.low = ((self.low << 8) | u32::from(self.buffer << EXTRA_BITS))
                | u32::from(next_byte >> 1);
            self.buffer = next_byte;
            self.range <<= 8;
        }
        Ok(())
    }

    /// Computes cumulative frequency corresponding to next symbol.
    pub fn decode_culfreq(&mut self, tot_f: u32) -> Result<u32> {
        self.normalize()?;
        if tot_f == 0 {
            bail!("Divide by zero in decode_culfreq");
        }
        let help = self
            .range
            .checked_div(tot_f)
            .ok_or_else(|| anyhow!("Divide by zero in decode_culfreq"))?;
        let help = if help == 0 { 1 } else { help };
        self.help = help;
        self.low
            .checked_div(help)
            .ok_or_else(|| anyhow!("Division failure in decode_culfreq"))
    }

    /// Computes cumulative frequency corresponding to next symbol with shift.
    pub fn decode_culshift(&mut self, shift: u32) -> Result<u32> {
        self.normalize()?;
        let help = self.range >> shift;
        let help = if help == 0 { 1 } else { help };
        self.help = help;
        self.low
            .checked_div(help)
            .ok_or_else(|| anyhow!("Division failure in decode_culshift"))
    }

    /// Updates internal code registers after symbol frequency lookup.
    pub fn update(&mut self, sy_f: u32, lt_f: u32) {
        let tmp = self.help.saturating_mul(lt_f);
        self.low = self.low.saturating_sub(tmp);
        self.range = self.help.saturating_mul(sy_f);
    }

    /// Completes decoding for the block, running final renormalization.
    pub fn finish(&mut self) -> Result<()> {
        self.normalize()
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
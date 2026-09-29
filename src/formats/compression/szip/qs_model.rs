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

//! Quasistatic adaptive probability model with periodic frequency rescaling.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;

const TBL_SHIFT: u32 = 7;
const TBL_SIZE: usize = 1 << TBL_SHIFT; // 128

/// Quasistatic adaptive probability model.
#[derive(Clone, Debug)]
pub struct QsModel {
    num_symbols: usize,
    left: i32,
    next_left: i32,
    rescale: i32,
    target_rescale: i32,
    incr: i32,
    search_shift: u32,
    cf: Vec<u16>,
    new_f: Vec<u16>,
    search: Option<Vec<u16>>,
}

impl QsModel {
    /// Initializes a quasistatic model.
    pub fn new(
        num_symbols: usize,
        lg_tot_f: u32,
        target_rescale: i32,
        is_compress: bool,
    ) -> Result<Self> {
        let search_shift = if lg_tot_f >= TBL_SHIFT {
            lg_tot_f.saturating_sub(TBL_SHIFT)
        } else {
            0
        };

        let mut cf = vec![0u16; num_symbols.saturating_add(1)];
        let new_f = vec![0u16; num_symbols.saturating_add(1)];

        let total_freq_u32 = 1_u32 << lg_tot_f;
        let total_freq = u16::try_from(total_freq_u32)
            .map_err(|e| anyhow!("Total frequency overflow for u16: {e}"))?;
        if let Some(slot) = cf.get_mut(num_symbols) {
            *slot = total_freq;
        }

        let search = if is_compress {
            None
        } else {
            let mut s = vec![0u16; TBL_SIZE.saturating_add(1)];
            let last_sym = u16::try_from(num_symbols.saturating_sub(1))
                .unwrap_or(0);
            if let Some(slot) = s.get_mut(TBL_SIZE) {
                *slot = last_sym;
            }
            Some(s)
        };

        let mut model = Self {
            num_symbols,
            left: 0,
            next_left: 0,
            rescale: i32::try_from(num_symbols >> 4).unwrap_or(0) | 2,
            target_rescale,
            incr: 0,
            search_shift,
            cf,
            new_f,
            search,
        };

        model.reset_uniform()?;
        Ok(model)
    }

    /// Resets model state with uniform probability distribution.
    fn reset_uniform(&mut self) -> Result<()> {
        self.rescale = i32::try_from(self.num_symbols >> 4).unwrap_or(0) | 2;
        self.next_left = 0;
        let tot = self.cf.get(self.num_symbols).copied().unwrap_or(0);
        let n = u16::try_from(self.num_symbols)
            .map_err(|e| anyhow!("Symbol count conversion: {e}"))?;
        if n == 0 {
            bail!("Symbol count cannot be zero");
        }
        let init_val = tot.checked_div(n).unwrap_or(0);
        let end = usize::from(tot.checked_rem(n).unwrap_or(0));

        for i in 0..end {
            if let Some(slot) = self.new_f.get_mut(i) {
                *slot = init_val.saturating_add(1);
            }
        }
        for i in end..self.num_symbols {
            if let Some(slot) = self.new_f.get_mut(i) {
                *slot = init_val;
            }
        }

        self.do_rescale();
        Ok(())
    }

    /// Rescales collected statistics and recalculates cumulative frequencies.
    pub fn do_rescale(&mut self) {
        if self.next_left > 0 {
            self.incr = self.incr.saturating_add(1);
            self.left = self.next_left;
            self.next_left = 0;
            return;
        }

        if self.rescale < self.target_rescale {
            self.rescale <<= 1;
            if self.rescale > self.target_rescale {
                self.rescale = self.target_rescale;
            }
        }

        let mut cf_val = i32::from(self.cf.get(self.num_symbols).copied().unwrap_or(0));
        let mut missing = cf_val;

        let mut i = self.num_symbols.saturating_sub(1);
        while i > 0 {
            let tmp = i32::from(self.new_f.get(i).copied().unwrap_or(0));
            cf_val = cf_val.saturating_sub(tmp);
            if let Some(slot) = self.cf.get_mut(i) {
                *slot = u16::try_from(cf_val).unwrap_or(0);
            }
            let halved = (tmp >> 1) | 1;
            missing = missing.saturating_sub(halved);
            if let Some(slot) = self.new_f.get_mut(i) {
                *slot = u16::try_from(halved).unwrap_or(1);
            }
            i = i.saturating_sub(1);
        }

        let first_f = i32::from(self.new_f.first().copied().unwrap_or(0));
        let halved_first = (first_f >> 1) | 1;
        missing = missing.saturating_sub(halved_first);
        if let Some(slot) = self.new_f.first_mut() {
            *slot = u16::try_from(halved_first).unwrap_or(1);
        }

        if self.rescale > 0 {
            self.incr = missing.checked_div(self.rescale).unwrap_or(0);
            self.next_left = missing.checked_rem(self.rescale).unwrap_or(0);
            self.left = self.rescale.saturating_sub(self.next_left);
        } else {
            self.incr = 1;
            self.next_left = 0;
            self.left = 1;
        }

        if let Some(search_tbl) = &mut self.search {
            let mut sym_idx = self.num_symbols;
            while sym_idx > 0 {
                let cf_cur = self.cf.get(sym_idx).copied().unwrap_or(0);
                let end = (cf_cur.saturating_sub(1)) >> self.search_shift;
                sym_idx = sym_idx.saturating_sub(1);
                let cf_prev = self.cf.get(sym_idx).copied().unwrap_or(0);
                let mut start = cf_prev >> self.search_shift;
                let sym_u16 = u16::try_from(sym_idx).unwrap_or(0);
                while start <= end {
                    if let Some(slot) = search_tbl.get_mut(usize::from(start)) {
                        *slot = sym_u16;
                    }
                    start = start.saturating_add(1);
                }
            }
        }
    }

    /// Queries the symbol frequency interval: `(symbol_freq, cumulative_freq)`.
    pub fn get_freq(&self, sym: usize) -> (u32, u32) {
        let lt_f = u32::from(self.cf.get(sym).copied().unwrap_or(0));
        let next_cf = u32::from(
            self.cf
                .get(sym.saturating_add(1))
                .copied()
                .unwrap_or(0),
        );
        let sy_f = next_cf.saturating_sub(lt_f);
        (sy_f, lt_f)
    }

    /// Maps cumulative frequency back to symbol index.
    pub fn get_sym(&self, lt_f: u32) -> usize {
        let (mut lo, mut hi) = if let Some(search_tbl) = &self.search {
            let idx = usize::try_from(lt_f >> self.search_shift).unwrap_or(0);
            let s_lo = usize::from(search_tbl.get(idx).copied().unwrap_or(0));
            let s_hi = usize::from(
                search_tbl
                    .get(idx.saturating_add(1))
                    .copied()
                    .unwrap_or(0),
            )
            .saturating_add(1);
            (s_lo, s_hi)
        } else {
            (0, self.num_symbols)
        };

        let lt_f_u16 = u16::try_from(lt_f).unwrap_or(u16::MAX);
        while lo.saturating_add(1) < hi {
            let mid = (lo.saturating_add(hi)) >> 1;
            let cf_mid = self.cf.get(mid).copied().unwrap_or(0);
            if lt_f_u16 < cf_mid {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        lo
    }

    /// Updates probability statistics for symbol `sym`.
    pub fn update(&mut self, sym: usize) {
        if self.left <= 0 {
            self.do_rescale();
        }
        self.left = self.left.saturating_sub(1);
        if let Some(slot) = self.new_f.get_mut(sym) {
            let incr_u16 = u16::try_from(self.incr).unwrap_or(1);
            *slot = slot.saturating_add(incr_u16);
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
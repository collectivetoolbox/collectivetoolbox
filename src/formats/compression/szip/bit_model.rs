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

//! Binary Indexed Tree (Fenwick Tree) probability model with symbol exclusion.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;

const EXCLUSION_MASK: u16 = 0x8000;
const FREQ_MASK: u16 = 0x7FFF;

/// Binary Indexed Tree (Fenwick Tree) frequency model.
#[derive(Clone, Debug)]
pub struct BitModel {
    num_symbols: usize,
    total_freq: i32,
    max_tot_f: i32,
    incr: u16,
    mask: usize,
    f: Vec<u16>,
    cf: Vec<u16>,
}

impl BitModel {
    /// Initializes a new `BitModel` with symbol count and rescaling parameters.
    pub fn new(num_symbols: usize, max_tot_f: i32, rescale: i32) -> Result<Self> {
        let n_i32 = i32::try_from(num_symbols)
            .map_err(|e| anyhow!("Symbol count conversion: {e}"))?;
        let min_max = n_i32.saturating_mul(2);
        let max_tot = if max_tot_f < min_max { min_max } else { max_tot_f };
        ensure!(rescale > 0, "Invalid rescale parameter");
        let half_max = max_tot
            .checked_div(2)
            .ok_or_else(|| anyhow!("Math overflow"))?;
        let mut incr_i32 = half_max
            .checked_div(rescale)
            .ok_or_else(|| anyhow!("Math overflow"))?;
        if incr_i32 < 1 {
            incr_i32 = 1;
        }
        let incr = u16::try_from(incr_i32)
            .map_err(|e| anyhow!("Increment conversion to u16: {e}"))?;

        let mut mask = 1usize;
        let mut temp = num_symbols;
        while temp > 1 {
            temp >>= 1;
            mask <<= 1;
        }

        let f = vec![1u16; num_symbols];
        let cf = vec![0u16; num_symbols.saturating_add(1)];

        let mut model = Self {
            num_symbols,
            total_freq: n_i32,
            max_tot_f: max_tot,
            incr,
            mask,
            f,
            cf,
        };
        model.build_cf();
        Ok(model)
    }

    /// Rebuilds cumulative frequency binary indexed tree.
    fn build_cf(&mut self) {
        self.total_freq = 0;
        let n = self.num_symbols;
        let mut i = 1usize;
        while i <= n {
            let step = i.saturating_mul(2);
            let mut j = i;
            while j <= n {
                let j_minus_1 = j.saturating_sub(1);
                // Reason for fallback: symbol frequency array lookup defaults to 0
                let f_val = self.f.get(j_minus_1).copied().unwrap_or(0);
                let entry_val = if (f_val & EXCLUSION_MASK) != 0 {
                    0u16
                } else {
                    let active = f_val & FREQ_MASK;
                    self.total_freq = self.total_freq.saturating_add(i32::from(active));
                    active
                };
                if let Some(cf_slot) = self.cf.get_mut(j) {
                    *cf_slot = entry_val;
                }
                let mut k = i >> 1;
                while k > 0 {
                    // Reason for fallback: fenwick tree index predecessor lookup defaults to 0
                    let prev_val = j
                        .checked_sub(k)
                        .and_then(|idx| self.cf.get(idx).copied())
                        .unwrap_or(0);
                    if let Some(cf_slot) = self.cf.get_mut(j) {
                        *cf_slot = cf_slot.saturating_add(prev_val);
                    }
                    k >>= 1;
                }
                j = j.saturating_add(step);
            }
            i <<= 1;
        }
    }

    /// Halves cumulative frequencies on total frequency overflow.
    fn scale_freq(&mut self) {
        for slot in &mut self.f {
            let active = *slot & FREQ_MASK;
            let excluded = *slot & EXCLUSION_MASK;
            let scaled = (active.saturating_add(1)) >> 1;
            *slot = scaled | excluded;
        }
        self.build_cf();
    }

    /// Returns current total active frequency.
    #[expect(
        clippy::expect_used,
        reason = "total_freq is range-checked to be strictly positive, and positive i32 fits in u32"
    )]
    pub fn total_freq(&self) -> u32 {
        if self.total_freq <= 0 {
            0
        } else {
            u32::try_from(self.total_freq).expect("Positive i32 fits in u32")
        }
    }

    /// Returns the symbol frequency and strictly less cumulative frequency.
    #[expect(
        clippy::expect_used,
        reason = "lt_f_i32 is range-checked to be non-negative, and non-negative i32 fits in u32"
    )]
    pub fn get_freq(&self, sym: usize) -> (u32, u32) {
        // Reason for fallback: out of bounds symbol frequency lookup defaults to 0
        let sy_f_u16 = self.f.get(sym).copied().unwrap_or(0) & FREQ_MASK;
        let mut idx = sym.saturating_add(1);
        // Reason for fallback: cumulative frequency tree node lookup defaults to 0
        let mut cul = i32::from(self.cf.get(idx).copied().unwrap_or(0));
        idx &= idx.saturating_sub(1);
        while idx > 0 {
            // Reason for fallback: cumulative frequency tree node lookup defaults to 0
            cul = cul.saturating_add(i32::from(self.cf.get(idx).copied().unwrap_or(0)));
            idx &= idx.saturating_sub(1);
        }
        let sy_f = u32::from(sy_f_u16);
        let lt_f_i32 = cul.saturating_sub(i32::from(sy_f_u16));
        let lt_f = if lt_f_i32 <= 0 {
            0
        } else {
            u32::try_from(lt_f_i32).expect("Positive i32 fits in u32")
        };
        (sy_f, lt_f)
    }

    /// Looks up symbol index by cumulative frequency.
    pub fn get_sym(&self, mut lt_f: u32) -> usize {
        let mut sym = 0usize;
        let mut mask = self.mask;
        let n = self.num_symbols;
        while mask > 0 {
            let x = sym | mask;
            // Reason for fallback: cumulative frequency tree node lookup defaults to 0
            let cf_val = u32::from(self.cf.get(x).copied().unwrap_or(0));
            if x <= n && lt_f >= cf_val {
                lt_f = lt_f.saturating_sub(cf_val);
                sym = x;
            }
            mask >>= 1;
        }
        sym
    }

    /// Updates cumulative frequency nodes by delta value.
    fn update_cf(&mut self, sym: usize, delta: i32) {
        self.total_freq = self.total_freq.saturating_add(delta);
        if self.total_freq > self.max_tot_f {
            self.scale_freq();
        } else {
            let mut idx = sym.saturating_add(1);
            let n = self.num_symbols;
            while idx <= n {
                if let Some(cf_slot) = self.cf.get_mut(idx) {
                    let updated = i32::from(*cf_slot).saturating_add(delta);
                    *cf_slot = if updated < 0 {
                        0
                    } else {
                        // Reason for fallback: cumulative frequency clamped to u16::MAX on saturation overflow
                        u16::try_from(updated).unwrap_or(u16::MAX)
                    };
                }
                let next = (idx | idx.saturating_sub(1)).saturating_add(1);
                if next <= idx {
                    break;
                }
                idx = next;
            }
        }
    }

    /// Updates model and deactivates symbol from active model.
    pub fn update_exclude(&mut self, sym: usize) {
        // Reason for fallback: symbol frequency array lookup defaults to 0
        let current_val = self.f.get(sym).copied().unwrap_or(0);
        let active = current_val & FREQ_MASK;
        let delta = 0i32.wrapping_sub(i32::from(active));
        let new_f = (active.saturating_add(self.incr)) | EXCLUSION_MASK;
        if let Some(slot) = self.f.get_mut(sym) {
            *slot = new_f;
        }
        self.update_cf(sym, delta);
    }

    /// Deactivates an active symbol, subtracting its frequency from cumulative tree.
    pub fn deactivate(&mut self, sym: usize) {
        // Reason for fallback: symbol frequency array lookup defaults to 0
        let current_val = self.f.get(sym).copied().unwrap_or(0);
        if (current_val & EXCLUSION_MASK) == 0 {
            let active = current_val & FREQ_MASK;
            let delta = 0i32.wrapping_sub(i32::from(active));
            if let Some(slot) = self.f.get_mut(sym) {
                *slot = active | EXCLUSION_MASK;
            }
            self.update_cf(sym, delta);
        }
    }

    /// Reactivates an excluded symbol, restoring its frequency to cumulative tree.
    pub fn reactivate(&mut self, sym: usize) {
        // Reason for fallback: symbol frequency array lookup defaults to 0
        let current_val = self.f.get(sym).copied().unwrap_or(0);
        if (current_val & EXCLUSION_MASK) != 0 {
            let active = current_val & FREQ_MASK;
            let delta = i32::from(active);
            if let Some(slot) = self.f.get_mut(sym) {
                *slot = active;
            }
            self.update_cf(sym, delta);
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
//! Implementation of Unix `compress` LZW formats:
//! - `CompressLzw` (compress 3.0+ / 4.0 / ncompress, magic `0x1F 0x9D`, block mode)
//! - `CompressLzw2` (compress 2.0, magic `0x1F 0x9D`, non-block mode)
//! - `CompressLzw1` (compress 1.0, headerless stream)
//!
//! Specification reference: `data/docs/compress-ncompress.md`
//! Reference decompressor: `old/unix-tools/gzip-1.14/gzip-1.14/unlzw.c`

// SPDX-License-Identifier for parts derived from from gzip: GPL-3.0-or-later

// Header comment from original unlzh.c:
/* unlzh.c -- decompress files in SCO compress -H (LZH) format.
 * The code in this file is directly derived from the public domain 'ar002'
 * written by Haruhiko Okumura.
 */

// Header comment from original unpack.c:
/* unpack.c -- decompress files in pack format.

   Copyright (C) 1997, 1999, 2006, 2009-2025 Free Software Foundation, Inc.
   Copyright (C) 1992-1993 Jean-loup Gailly

   This program is free software; you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation; either version 3, or (at your option)
   any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

// AUTHORS file for gzip overall:
/* gzip was written by Jean-loup Gailly <jloup@gzip.org>,
and Mark Adler for the decompression code. */

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;
use std::collections::HashMap;
use std::io::{Read, Write};

/// Magic header bytes for `.Z` LZW compress format (`0x1F`, `0x9D`).
pub const LZW_MAGIC: [u8; 2] = [0x1F, 0x9D];

/// Block mode mask bit in header byte 2 (`0x80`).
pub const BLOCK_MODE: u8 = 0x80;

/// Maxbits mask in header byte 2 (`0x1F`).
pub const BIT_MASK: u8 = 0x1F;

/// Reserved bits in header byte 2 (`0x60`).
pub const LZW_RESERVED: u8 = 0x60;

/// Initial bit width for LZW codes (9 bits).
pub const INIT_BITS: u32 = 9;

/// Default maximum bit width for LZW codes (16 bits).
pub const MAX_BITS: u32 = 16;

/// Special CLEAR code emitted in block mode to flush dictionary (256).
pub const CLEAR_CODE: u32 = 256;

/// First free dictionary entry in block mode (257).
pub const FIRST_FREE_BLOCK: u32 = 257;

/// First free dictionary entry in non-block mode (256).
pub const FIRST_FREE_NONBLOCK: u32 = 256;

// -----------------------------------------------------------------------------
// LSB-first Bit Writer matching Spencer Thomas / gzip unlzw.c posbits layout
// -----------------------------------------------------------------------------

struct LzwBitWriter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
    posbits: usize,
    block_posbits: usize,
}

impl<W: Write> LzwBitWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: Vec::with_capacity(4096),
            posbits: 0,
            block_posbits: 0,
        }
    }

    fn write_code(&mut self, code: u32, n_bits: u32) -> Result<()> {
        let byte_pos = self.posbits / 8;
        let shift = self.posbits % 8;
        let needed_len = byte_pos.saturating_add(4);

        if self.buffer.len() < needed_len {
            self.buffer.resize(needed_len, 0);
        }

        let code_u64 = u64::from(code);
        let val = code_u64.checked_shl(u32::try_from(shift)?).unwrap_or(0);

        for i in 0..4 {
            let idx = byte_pos.saturating_add(i);
            let byte_val = u8::try_from((val >> (i.saturating_mul(8))) & 0xFF)?;
            if let Some(b) = self.buffer.get_mut(idx) {
                *b |= byte_val;
            }
        }

        let n_bits_usize = usize::try_from(n_bits)?;
        self.posbits = self.posbits.saturating_add(n_bits_usize);
        self.block_posbits = self.block_posbits.saturating_add(n_bits_usize);
        Ok(())
    }

    fn align_block(&mut self, n_bits: u32) -> Result<()> {
        if self.block_posbits > 0 {
            let n_bits_usize = usize::try_from(n_bits)?;
            let block_bits = n_bits_usize.saturating_mul(8);
            let rem = (self.block_posbits.saturating_sub(1)) % block_bits;
            let pad = block_bits.saturating_sub(rem).saturating_sub(1);
            self.posbits = self.posbits.saturating_add(pad);
            self.block_posbits = 0;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        let final_bytes = (self.posbits.saturating_add(7)) / 8;
        self.buffer.truncate(final_bytes);
        self.writer
            .write_all(&self.buffer)
            .context("Failed to write compress payload")?;
        self.writer.flush().context("Failed to flush inner writer")?;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// LSB-first Bit Reader matching Spencer Thomas / gzip unlzw.c posbits layout
// -----------------------------------------------------------------------------

struct LzwBitReader<'a> {
    data: &'a [u8],
    posbits: usize,
    block_posbits: usize,
}

impl<'a> LzwBitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            posbits: 0,
            block_posbits: 0,
        }
    }

    fn read_code(&mut self, n_bits: u32) -> Result<Option<u32>> {
        let byte_pos = self.posbits / 8;
        if byte_pos >= self.data.len() {
            return Ok(None);
        }

        let b0 = u64::from(*self.data.get(byte_pos).unwrap_or(&0));
        let b1 = u64::from(*self.data.get(byte_pos.saturating_add(1)).unwrap_or(&0));
        let b2 = u64::from(*self.data.get(byte_pos.saturating_add(2)).unwrap_or(&0));
        let b3 = u64::from(*self.data.get(byte_pos.saturating_add(3)).unwrap_or(&0));

        let val = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        let shift = u32::try_from(self.posbits % 8)?;
        let mask = (1u64.checked_shl(n_bits).unwrap_or(0)).saturating_sub(1);

        let code = u32::try_from((val >> shift) & mask)?;
        let n_bits_usize = usize::try_from(n_bits)?;
        self.posbits = self.posbits.saturating_add(n_bits_usize);
        self.block_posbits = self.block_posbits.saturating_add(n_bits_usize);
        Ok(Some(code))
    }

    fn align_block(&mut self, n_bits: u32) -> Result<()> {
        if self.block_posbits > 0 {
            let n_bits_usize = usize::try_from(n_bits)?;
            let block_bits = n_bits_usize.saturating_mul(8);
            let rem = (self.block_posbits.saturating_sub(1)) % block_bits;
            let pad = block_bits.saturating_sub(rem).saturating_sub(1);
            self.posbits = self.posbits.saturating_add(pad);
            self.block_posbits = 0;
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Stream Compression & Decompression Entrypoints
// -----------------------------------------------------------------------------

/// Compresses a stream using the specified LZW format variant (`CompressLzw`, `CompressLzw2`, `CompressLzw1`).
pub fn compress_lzw_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: crate::CompressionFormat,
) -> Result<u64> {
    let maxbits = MAX_BITS;
    let block_mode = match format {
        crate::CompressionFormat::CompressLzw => true,
        crate::CompressionFormat::CompressLzw2 | crate::CompressionFormat::CompressLzw1 => false,
        _ => bail!("Unsupported format for LZW compress: {:?}", format),
    };

    // 1. Header writing
    if format != crate::CompressionFormat::CompressLzw1 {
        let mode_byte = if block_mode {
            BLOCK_MODE | u8::try_from(maxbits)?
        } else {
            u8::try_from(maxbits)?
        };
        writer
            .write_all(&[LZW_MAGIC[0], LZW_MAGIC[1], mode_byte])
            .context("Failed to write compress header")?;
    }

    let mut bit_writer = LzwBitWriter::new(writer);
    let mut dict: HashMap<(u32, u8), u32> = HashMap::with_capacity(4096);

    let mut free_ent = if block_mode {
        FIRST_FREE_BLOCK
    } else {
        FIRST_FREE_NONBLOCK
    };
    let mut n_bits = INIT_BITS;
    let maxmaxcode = 1u32.checked_shl(maxbits).unwrap_or(0);
    let mut maxcode = (1u32.checked_shl(n_bits).unwrap_or(0)).saturating_sub(1);

    let mut input_bytes = Vec::new();
    let total_in = reader
        .read_to_end(&mut input_bytes)
        .context("Failed to read input data")?;

    if input_bytes.is_empty() {
        bit_writer.finish()?;
        return Ok(u64::try_from(total_in)?);
    }

    let first_byte = match input_bytes.first() {
        Some(&b) => b,
        None => bail!("Empty input buffer"),
    };

    let mut ent = u32::from(first_byte);

    for &byte in input_bytes.iter().skip(1) {
        let key = (ent, byte);
        if let Some(&code) = dict.get(&key) {
            ent = code;
        } else {
            bit_writer.write_code(ent, n_bits)?;

            if free_ent < maxmaxcode {
                dict.insert(key, free_ent);
                free_ent = free_ent.saturating_add(1);

                if free_ent > maxcode && n_bits < maxbits {
                    bit_writer.align_block(n_bits)?;
                    n_bits = n_bits.saturating_add(1);
                    maxcode = (1u32.checked_shl(n_bits).unwrap_or(0)).saturating_sub(1);
                }
            } else if block_mode {
                bit_writer.write_code(CLEAR_CODE, n_bits)?;
                bit_writer.align_block(n_bits)?;
                dict.clear();
                free_ent = FIRST_FREE_BLOCK;
                n_bits = INIT_BITS;
                maxcode = (1u32.checked_shl(n_bits).unwrap_or(0)).saturating_sub(1);
            }

            ent = u32::from(byte);
        }
    }

    bit_writer.write_code(ent, n_bits)?;
    bit_writer.finish()?;

    Ok(u64::try_from(total_in)?)
}

/// Decompresses a stream using the specified LZW format variant (`CompressLzw`, `CompressLzw2`, `CompressLzw1`).
pub fn decompress_lzw_stream(
    reader: &mut impl Read,
    writer: &mut impl Write,
    format: crate::CompressionFormat,
) -> Result<u64> {
    let mut input_data = Vec::new();
    reader
        .read_to_end(&mut input_data)
        .context("Failed to read input compress stream")?;

    if input_data.is_empty() {
        return Ok(0);
    }

    let mut data_slice: &[u8] = &input_data;
    let block_mode;
    let maxbits;

    if format != crate::CompressionFormat::CompressLzw1 {
        if data_slice.len() < 3 {
            bail!("Compress stream too short for 3-byte header");
        }
        let h0 = match data_slice.first() {
            Some(&b) => b,
            None => bail!("Missing magic byte 1"),
        };
        let h1 = match data_slice.get(1) {
            Some(&b) => b,
            None => bail!("Missing magic byte 2"),
        };

        if h0 != LZW_MAGIC[0] || h1 != LZW_MAGIC[1] {
            bail!(
                "Invalid compress magic header: expected 0x{:02X}{:02X}, got 0x{:02X}{:02X}",
                LZW_MAGIC[0],
                LZW_MAGIC[1],
                h0,
                h1
            );
        }

        let mode_u8 = match data_slice.get(2) {
            Some(&b) => b,
            None => bail!("Missing mode byte"),
        };

        block_mode = (mode_u8 & BLOCK_MODE) != 0;
        maxbits = u32::from(mode_u8 & BIT_MASK);

        if maxbits < 9 || maxbits > 16 {
            bail!("Invalid compress maxbits param: {maxbits}");
        }

        data_slice = match data_slice.get(3..) {
            Some(sl) => sl,
            None => bail!("Failed to slice compress payload"),
        };
    } else {
        block_mode = false;
        maxbits = MAX_BITS;
    }

    let mut bit_reader = LzwBitReader::new(data_slice);

    let mut prefix = vec![0u32; 65536];
    let mut suffix = vec![0u8; 65536];

    for i in 0..256 {
        if let Some(s) = suffix.get_mut(i) {
            *s = u8::try_from(i)?;
        }
    }

    let mut free_ent = if block_mode {
        FIRST_FREE_BLOCK
    } else {
        FIRST_FREE_NONBLOCK
    };
    let mut n_bits = INIT_BITS;
    let maxmaxcode = 1u32.checked_shl(maxbits).unwrap_or(0);
    let mut maxcode = (1u32.checked_shl(n_bits).unwrap_or(0)).saturating_sub(1);

    let mut oldcode: i32 = -1;
    let mut finchar: u8 = 0;
    let mut stack = vec![0u8; 65536];
    let mut total_written: u64 = 0;

    let total_data_bits = data_slice.len().saturating_mul(8);

    while bit_reader
        .posbits
        .saturating_add(usize::try_from(n_bits)?)
        <= total_data_bits
    {
        if free_ent > maxcode {
            bit_reader.align_block(n_bits)?;
            n_bits = n_bits.saturating_add(1);
            if n_bits == maxbits {
                maxcode = maxmaxcode;
            } else {
                maxcode = (1u32.checked_shl(n_bits).unwrap_or(0)).saturating_sub(1);
            }
            if bit_reader
                .posbits
                .saturating_add(usize::try_from(n_bits)?)
                > total_data_bits
            {
                break;
            }
        }

        let mut code = match bit_reader.read_code(n_bits)? {
            Some(c) => c,
            None => break,
        };

        if oldcode == -1 {
            if code >= 256 {
                bail!("Corrupt compress stream: first code {code} >= 256");
            }
            finchar = u8::try_from(code)?;
            oldcode = i32::try_from(code)?;
            writer
                .write_all(&[finchar])
                .context("Failed to write byte to output stream")?;
            total_written = total_written.saturating_add(1);
            continue;
        }

        if code == CLEAR_CODE && block_mode {
            prefix.fill(0);
            free_ent = FIRST_FREE_NONBLOCK;
            bit_reader.align_block(n_bits)?;
            n_bits = INIT_BITS;
            maxcode = (1u32.checked_shl(n_bits).unwrap_or(0)).saturating_sub(1);
            oldcode = -1;
            continue;
        }

        let incode = code;
        let mut stack_idx: usize = stack.len();

        if code >= free_ent {
            if code > free_ent {
                bail!("Corrupt compress stream: code {code} exceeds free entry {free_ent}");
            }
            stack_idx = stack_idx.saturating_sub(1);
            if let Some(s) = stack.get_mut(stack_idx) {
                *s = finchar;
            }
            code = u32::try_from(oldcode)?;
        }

        while code >= 256 {
            let code_idx = usize::try_from(code)?;
            let suf = match suffix.get(code_idx) {
                Some(&s) => s,
                None => bail!("Corrupt compress stream: invalid suffix index {code}"),
            };
            stack_idx = stack_idx.saturating_sub(1);
            if let Some(s) = stack.get_mut(stack_idx) {
                *s = suf;
            }
            code = match prefix.get(code_idx) {
                Some(&p) => p,
                None => bail!("Corrupt compress stream: invalid prefix index {code}"),
            };
        }

        finchar = u8::try_from(code)?;
        stack_idx = stack_idx.saturating_sub(1);
        if let Some(s) = stack.get_mut(stack_idx) {
            *s = finchar;
        }

        let out_slice = match stack.get(stack_idx..) {
            Some(sl) => sl,
            None => bail!("Stack index out of bounds"),
        };
        writer
            .write_all(out_slice)
            .context("Failed to write decompressed block")?;
        total_written = total_written.saturating_add(u64::try_from(out_slice.len())?);

        if free_ent < maxmaxcode {
            let oldcode_u32 = u32::try_from(oldcode)?;
            let free_idx = usize::try_from(free_ent)?;
            if let Some(p) = prefix.get_mut(free_idx) {
                *p = oldcode_u32;
            }
            if let Some(s) = suffix.get_mut(free_idx) {
                *s = finchar;
            }
            free_ent = free_ent.saturating_add(1);
        }

        oldcode = i32::try_from(incode)?;
    }

    writer.flush().context("Failed to flush output stream")?;
    Ok(total_written)
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
    fn test_compress_lzw_roundtrip() {
        let sample = b"The quick brown fox jumps over the lazy dog! 1234567890 repeating text repeating text repeating text";

        for format in [
            crate::CompressionFormat::CompressLzw,
            crate::CompressionFormat::CompressLzw2,
            crate::CompressionFormat::CompressLzw1,
        ] {
            let mut compressed = Vec::new();
            compress_lzw_stream(&mut &sample[..], &mut compressed, format).unwrap();

            let mut decompressed = Vec::new();
            decompress_lzw_stream(&mut &compressed[..], &mut decompressed, format).unwrap();

            assert_eq!(
                decompressed, sample,
                "Roundtrip failed for LZW format {:?}",
                format
            );
        }
    }
}

/*

License for parts derived from gzip 1.14:

```
                    GNU GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The GNU General Public License is a free, copyleft license for
software and other kinds of works.

  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
the GNU General Public License is intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.  We, the Free Software Foundation, use the
GNU General Public License for most of our software; it applies also to
any other work released this way by its authors.  You can apply it to
your programs, too.

  When we speak of free software, we are referring to freedom, not
price.  Our General Public Licenses are designed to make sure that you
have the freedom to distribute copies of free software (and charge for
them if you wish), that you receive source code or can get it if you
want it, that you can change the software or use pieces of it in new
free programs, and that you know you can do these things.

  To protect your rights, we need to prevent others from denying you
these rights or asking you to surrender the rights.  Therefore, you have
certain responsibilities if you distribute copies of the software, or if
you modify it: responsibilities to respect the freedom of others.

  For example, if you distribute copies of such a program, whether
gratis or for a fee, you must pass on to the recipients the same
freedoms that you received.  You must make sure that they, too, receive
or can get the source code.  And you must show them these terms so they
know their rights.

  Developers that use the GNU GPL protect your rights with two steps:
(1) assert copyright on the software, and (2) offer you this License
giving you legal permission to copy, distribute and/or modify it.

  For the developers' and authors' protection, the GPL clearly explains
that there is no warranty for this free software.  For both users' and
authors' sake, the GPL requires that modified versions be marked as
changed, so that their problems will not be attributed erroneously to
authors of previous versions.

  Some devices are designed to deny users access to install or run
modified versions of the software inside them, although the manufacturer
can do so.  This is fundamentally incompatible with the aim of
protecting users' freedom to change the software.  The systematic
pattern of such abuse occurs in the area of products for individuals to
use, which is precisely where it is most unacceptable.  Therefore, we
have designed this version of the GPL to prohibit the practice for those
products.  If such problems arise substantially in other domains, we
stand ready to extend this provision to those domains in future versions
of the GPL, as needed to protect the freedom of users.

  Finally, every program is threatened constantly by software patents.
States should not allow patents to restrict development and use of
software on general-purpose computers, but in those that do, we wish to
avoid the special danger that patents applied to a free program could
make it effectively proprietary.  To prevent this, the GPL assures that
patents cannot be used to render the program non-free.

  The precise terms and conditions for copying, distribution and
modification follow.

                       TERMS AND CONDITIONS

  0. Definitions.

  "This License" refers to version 3 of the GNU General Public License.

  "Copyright" also means copyright-like laws that apply to other kinds of
works, such as semiconductor masks.

  "The Program" refers to any copyrightable work licensed under this
License.  Each licensee is addressed as "you".  "Licensees" and
"recipients" may be individuals or organizations.

  To "modify" a work means to copy from or adapt all or part of the work
in a fashion requiring copyright permission, other than the making of an
exact copy.  The resulting work is called a "modified version" of the
earlier work or a work "based on" the earlier work.

  A "covered work" means either the unmodified Program or a work based
on the Program.

  To "propagate" a work means to do anything with it that, without
permission, would make you directly or secondarily liable for
infringement under applicable copyright law, except executing it on a
computer or modifying a private copy.  Propagation includes copying,
distribution (with or without modification), making available to the
public, and in some countries other activities as well.

  To "convey" a work means any kind of propagation that enables other
parties to make or receive copies.  Mere interaction with a user through
a computer network, with no transfer of a copy, is not conveying.

  An interactive user interface displays "Appropriate Legal Notices"
to the extent that it includes a convenient and prominently visible
feature that (1) displays an appropriate copyright notice, and (2)
tells the user that there is no warranty for the work (except to the
extent that warranties are provided), that licensees may convey the
work under this License, and how to view a copy of this License.  If
the interface presents a list of user commands or options, such as a
menu, a prominent item in the list meets this criterion.

  1. Source Code.

  The "source code" for a work means the preferred form of the work
for making modifications to it.  "Object code" means any non-source
form of a work.

  A "Standard Interface" means an interface that either is an official
standard defined by a recognized standards body, or, in the case of
interfaces specified for a particular programming language, one that
is widely used among developers working in that language.

  The "System Libraries" of an executable work include anything, other
than the work as a whole, that (a) is included in the normal form of
packaging a Major Component, but which is not part of that Major
Component, and (b) serves only to enable use of the work with that
Major Component, or to implement a Standard Interface for which an
implementation is available to the public in source code form.  A
"Major Component", in this context, means a major essential component
(kernel, window system, and so on) of the specific operating system
(if any) on which the executable work runs, or a compiler used to
produce the work, or an object code interpreter used to run it.

  The "Corresponding Source" for a work in object code form means all
the source code needed to generate, install, and (for an executable
work) run the object code and to modify the work, including scripts to
control those activities.  However, it does not include the work's
System Libraries, or general-purpose tools or generally available free
programs which are used unmodified in performing those activities but
which are not part of the work.  For example, Corresponding Source
includes interface definition files associated with source files for
the work, and the source code for shared libraries and dynamically
linked subprograms that the work is specifically designed to require,
such as by intimate data communication or control flow between those
subprograms and other parts of the work.

  The Corresponding Source need not include anything that users
can regenerate automatically from other parts of the Corresponding
Source.

  The Corresponding Source for a work in source code form is that
same work.

  2. Basic Permissions.

  All rights granted under this License are granted for the term of
copyright on the Program, and are irrevocable provided the stated
conditions are met.  This License explicitly affirms your unlimited
permission to run the unmodified Program.  The output from running a
covered work is covered by this License only if the output, given its
content, constitutes a covered work.  This License acknowledges your
rights of fair use or other equivalent, as provided by copyright law.

  You may make, run and propagate covered works that you do not
convey, without conditions so long as your license otherwise remains
in force.  You may convey covered works to others for the sole purpose
of having them make modifications exclusively for you, or provide you
with facilities for running those works, provided that you comply with
the terms of this License in conveying all material for which you do
not control copyright.  Those thus making or running the covered works
for you must do so exclusively on your behalf, under your direction
and control, on terms that prohibit them from making any copies of
your copyrighted material outside their relationship with you.

  Conveying under any other circumstances is permitted solely under
the conditions stated below.  Sublicensing is not allowed; section 10
makes it unnecessary.

  3. Protecting Users' Legal Rights From Anti-Circumvention Law.

  No covered work shall be deemed part of an effective technological
measure under any applicable law fulfilling obligations under article
11 of the WIPO copyright treaty adopted on 20 December 1996, or
similar laws prohibiting or restricting circumvention of such
measures.

  When you convey a covered work, you waive any legal power to forbid
circumvention of technological measures to the extent such circumvention
is effected by exercising rights under this License with respect to
the covered work, and you disclaim any intention to limit operation or
modification of the work as a means of enforcing, against the work's
users, your or third parties' legal rights to forbid circumvention of
technological measures.

  4. Conveying Verbatim Copies.

  You may convey verbatim copies of the Program's source code as you
receive it, in any medium, provided that you conspicuously and
appropriately publish on each copy an appropriate copyright notice;
keep intact all notices stating that this License and any
non-permissive terms added in accord with section 7 apply to the code;
keep intact all notices of the absence of any warranty; and give all
recipients a copy of this License along with the Program.

  You may charge any price or no price for each copy that you convey,
and you may offer support or warranty protection for a fee.

  5. Conveying Modified Source Versions.

  You may convey a work based on the Program, or the modifications to
produce it from the Program, in the form of source code under the
terms of section 4, provided that you also meet all of these conditions:

    a) The work must carry prominent notices stating that you modified
    it, and giving a relevant date.

    b) The work must carry prominent notices stating that it is
    released under this License and any conditions added under section
    7.  This requirement modifies the requirement in section 4 to
    "keep intact all notices".

    c) You must license the entire work, as a whole, under this
    License to anyone who comes into possession of a copy.  This
    License will therefore apply, along with any applicable section 7
    additional terms, to the whole of the work, and all its parts,
    regardless of how they are packaged.  This License gives no
    permission to license the work in any other way, but it does not
    invalidate such permission if you have separately received it.

    d) If the work has interactive user interfaces, each must display
    Appropriate Legal Notices; however, if the Program has interactive
    interfaces that do not display Appropriate Legal Notices, your
    work need not make them do so.

  A compilation of a covered work with other separate and independent
works, which are not by their nature extensions of the covered work,
and which are not combined with it such as to form a larger program,
in or on a volume of a storage or distribution medium, is called an
"aggregate" if the compilation and its resulting copyright are not
used to limit the access or legal rights of the compilation's users
beyond what the individual works permit.  Inclusion of a covered work
in an aggregate does not cause this License to apply to the other
parts of the aggregate.

  6. Conveying Non-Source Forms.

  You may convey a covered work in object code form under the terms
of sections 4 and 5, provided that you also convey the
machine-readable Corresponding Source under the terms of this License,
in one of these ways:

    a) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by the
    Corresponding Source fixed on a durable physical medium
    customarily used for software interchange.

    b) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by a
    written offer, valid for at least three years and valid for as
    long as you offer spare parts or customer support for that product
    model, to give anyone who possesses the object code either (1) a
    copy of the Corresponding Source for all the software in the
    product that is covered by this License, on a durable physical
    medium customarily used for software interchange, for a price no
    more than your reasonable cost of physically performing this
    conveying of source, or (2) access to copy the
    Corresponding Source from a network server at no charge.

    c) Convey individual copies of the object code with a copy of the
    written offer to provide the Corresponding Source.  This
    alternative is allowed only occasionally and noncommercially, and
    only if you received the object code with such an offer, in accord
    with subsection 6b.

    d) Convey the object code by offering access from a designated
    place (gratis or for a charge), and offer equivalent access to the
    Corresponding Source in the same way through the same place at no
    further charge.  You need not require recipients to copy the
    Corresponding Source along with the object code.  If the place to
    copy the object code is a network server, the Corresponding Source
    may be on a different server (operated by you or a third party)
    that supports equivalent copying facilities, provided you maintain
    clear directions next to the object code saying where to find the
    Corresponding Source.  Regardless of what server hosts the
    Corresponding Source, you remain obligated to ensure that it is
    available for as long as needed to satisfy these requirements.

    e) Convey the object code using peer-to-peer transmission, provided
    you inform other peers where the object code and Corresponding
    Source of the work are being offered to the general public at no
    charge under subsection 6d.

  A separable portion of the object code, whose source code is excluded
from the Corresponding Source as a System Library, need not be
included in conveying the object code work.

  A "User Product" is either (1) a "consumer product", which means any
tangible personal property which is normally used for personal, family,
or household purposes, or (2) anything designed or sold for incorporation
into a dwelling.  In determining whether a product is a consumer product,
doubtful cases shall be resolved in favor of coverage.  For a particular
product received by a particular user, "normally used" refers to a
typical or common use of that class of product, regardless of the status
of the particular user or of the way in which the particular user
actually uses, or expects or is expected to use, the product.  A product
is a consumer product regardless of whether the product has substantial
commercial, industrial or non-consumer uses, unless such uses represent
the only significant mode of use of the product.

  "Installation Information" for a User Product means any methods,
procedures, authorization keys, or other information required to install
and execute modified versions of a covered work in that User Product from
a modified version of its Corresponding Source.  The information must
suffice to ensure that the continued functioning of the modified object
code is in no case prevented or interfered with solely because
modification has been made.

  If you convey an object code work under this section in, or with, or
specifically for use in, a User Product, and the conveying occurs as
part of a transaction in which the right of possession and use of the
User Product is transferred to the recipient in perpetuity or for a
fixed term (regardless of how the transaction is characterized), the
Corresponding Source conveyed under this section must be accompanied
by the Installation Information.  But this requirement does not apply
if neither you nor any third party retains the ability to install
modified object code on the User Product (for example, the work has
been installed in ROM).

  The requirement to provide Installation Information does not include a
requirement to continue to provide support service, warranty, or updates
for a work that has been modified or installed by the recipient, or for
the User Product in which it has been modified or installed.  Access to a
network may be denied when the modification itself materially and
adversely affects the operation of the network or violates the rules and
protocols for communication across the network.

  Corresponding Source conveyed, and Installation Information provided,
in accord with this section must be in a format that is publicly
documented (and with an implementation available to the public in
source code form), and must require no special password or key for
unpacking, reading or copying.

  7. Additional Terms.

  "Additional permissions" are terms that supplement the terms of this
License by making exceptions from one or more of its conditions.
Additional permissions that are applicable to the entire Program shall
be treated as though they were included in this License, to the extent
that they are valid under applicable law.  If additional permissions
apply only to part of the Program, that part may be used separately
under those permissions, but the entire Program remains governed by
this License without regard to the additional permissions.

  When you convey a copy of a covered work, you may at your option
remove any additional permissions from that copy, or from any part of
it.  (Additional permissions may be written to require their own
removal in certain cases when you modify the work.)  You may place
additional permissions on material, added by you to a covered work,
for which you have or can give appropriate copyright permission.

  Notwithstanding any other provision of this License, for material you
add to a covered work, you may (if authorized by the copyright holders of
that material) supplement the terms of this License with terms:

    a) Disclaiming warranty or limiting liability differently from the
    terms of sections 15 and 16 of this License; or

    b) Requiring preservation of specified reasonable legal notices or
    author attributions in that material or in the Appropriate Legal
    Notices displayed by works containing it; or

    c) Prohibiting misrepresentation of the origin of that material, or
    requiring that modified versions of such material be marked in
    reasonable ways as different from the original version; or

    d) Limiting the use for publicity purposes of names of licensors or
    authors of the material; or

    e) Declining to grant rights under trademark law for use of some
    trade names, trademarks, or service marks; or

    f) Requiring indemnification of licensors and authors of that
    material by anyone who conveys the material (or modified versions of
    it) with contractual assumptions of liability to the recipient, for
    any liability that these contractual assumptions directly impose on
    those licensors and authors.

  All other non-permissive additional terms are considered "further
restrictions" within the meaning of section 10.  If the Program as you
received it, or any part of it, contains a notice stating that it is
governed by this License along with a term that is a further
restriction, you may remove that term.  If a license document contains
a further restriction but permits relicensing or conveying under this
License, you may add to a covered work material governed by the terms
of that license document, provided that the further restriction does
not survive such relicensing or conveying.

  If you add terms to a covered work in accord with this section, you
must place, in the relevant source files, a statement of the
additional terms that apply to those files, or a notice indicating
where to find the applicable terms.

  Additional terms, permissive or non-permissive, may be stated in the
form of a separately written license, or stated as exceptions;
the above requirements apply either way.

  8. Termination.

  You may not propagate or modify a covered work except as expressly
provided under this License.  Any attempt otherwise to propagate or
modify it is void, and will automatically terminate your rights under
this License (including any patent licenses granted under the third
paragraph of section 11).

  However, if you cease all violation of this License, then your
license from a particular copyright holder is reinstated (a)
provisionally, unless and until the copyright holder explicitly and
finally terminates your license, and (b) permanently, if the copyright
holder fails to notify you of the violation by some reasonable means
prior to 60 days after the cessation.

  Moreover, your license from a particular copyright holder is
reinstated permanently if the copyright holder notifies you of the
violation by some reasonable means, this is the first time you have
received notice of violation of this License (for any work) from that
copyright holder, and you cure the violation prior to 30 days after
your receipt of the notice.

  Termination of your rights under this section does not terminate the
licenses of parties who have received copies or rights from you under
this License.  If your rights have been terminated and not permanently
reinstated, you do not qualify to receive new licenses for the same
material under section 10.

  9. Acceptance Not Required for Having Copies.

  You are not required to accept this License in order to receive or
run a copy of the Program.  Ancillary propagation of a covered work
occurring solely as a consequence of using peer-to-peer transmission
to receive a copy likewise does not require acceptance.  However,
nothing other than this License grants you permission to propagate or
modify any covered work.  These actions infringe copyright if you do
not accept this License.  Therefore, by modifying or propagating a
covered work, you indicate your acceptance of this License to do so.

  10. Automatic Licensing of Downstream Recipients.

  Each time you convey a covered work, the recipient automatically
receives a license from the original licensors, to run, modify and
propagate that work, subject to this License.  You are not responsible
for enforcing compliance by third parties with this License.

  An "entity transaction" is a transaction transferring control of an
organization, or substantially all assets of one, or subdividing an
organization, or merging organizations.  If propagation of a covered
work results from an entity transaction, each party to that
transaction who receives a copy of the work also receives whatever
licenses to the work the party's predecessor in interest had or could
give under the previous paragraph, plus a right to possession of the
Corresponding Source of the work from the predecessor in interest, if
the predecessor has it or can get it with reasonable efforts.

  You may not impose any further restrictions on the exercise of the
rights granted or affirmed under this License.  For example, you may
not impose a license fee, royalty, or other charge for exercise of
rights granted under this License, and you may not initiate litigation
(including a cross-claim or counterclaim in a lawsuit) alleging that
any patent claim is infringed by making, using, selling, offering for
sale, or importing the Program or any portion of it.

  11. Patents.

  A "contributor" is a copyright holder who authorizes use under this
License of the Program or a work on which the Program is based.  The
work thus licensed is called the contributor's "contributor version".

  A contributor's "essential patent claims" are all patent claims
owned or controlled by the contributor, whether already acquired or
hereafter acquired, that would be infringed by some manner, permitted
by this License, of making, using, or selling its contributor version,
but do not include claims that would be infringed only as a
consequence of further modification of the contributor version.  For
purposes of this definition, "control" includes the right to grant
patent sublicenses in a manner consistent with the requirements of
this License.

  Each contributor grants you a non-exclusive, worldwide, royalty-free
patent license under the contributor's essential patent claims, to
make, use, sell, offer for sale, import and otherwise run, modify and
propagate the contents of its contributor version.

  In the following three paragraphs, a "patent license" is any express
agreement or commitment, however denominated, not to enforce a patent
(such as an express permission to practice a patent or covenant not to
sue for patent infringement).  To "grant" such a patent license to a
party means to make such an agreement or commitment not to enforce a
patent against the party.

  If you convey a covered work, knowingly relying on a patent license,
and the Corresponding Source of the work is not available for anyone
to copy, free of charge and under the terms of this License, through a
publicly available network server or other readily accessible means,
then you must either (1) cause the Corresponding Source to be so
available, or (2) arrange to deprive yourself of the benefit of the
patent license for this particular work, or (3) arrange, in a manner
consistent with the requirements of this License, to extend the patent
license to downstream recipients.  "Knowingly relying" means you have
actual knowledge that, but for the patent license, your conveying the
covered work in a country, or your recipient's use of the covered work
in a country, would infringe one or more identifiable patents in that
country that you have reason to believe are valid.

  If, pursuant to or in connection with a single transaction or
arrangement, you convey, or propagate by procuring conveyance of, a
covered work, and grant a patent license to some of the parties
receiving the covered work authorizing them to use, propagate, modify
or convey a specific copy of the covered work, then the patent license
you grant is automatically extended to all recipients of the covered
work and works based on it.

  A patent license is "discriminatory" if it does not include within
the scope of its coverage, prohibits the exercise of, or is
conditioned on the non-exercise of one or more of the rights that are
specifically granted under this License.  You may not convey a covered
work if you are a party to an arrangement with a third party that is
in the business of distributing software, under which you make payment
to the third party based on the extent of your activity of conveying
the work, and under which the third party grants, to any of the
parties who would receive the covered work from you, a discriminatory
patent license (a) in connection with copies of the covered work
conveyed by you (or copies made from those copies), or (b) primarily
for and in connection with specific products or compilations that
contain the covered work, unless you entered into that arrangement,
or that patent license was granted, prior to 28 March 2007.

  Nothing in this License shall be construed as excluding or limiting
any implied license or other defenses to infringement that may
otherwise be available to you under applicable patent law.

  12. No Surrender of Others' Freedom.

  If conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License.  If you cannot convey a
covered work so as to satisfy simultaneously your obligations under this
License and any other pertinent obligations, then as a consequence you may
not convey it at all.  For example, if you agree to terms that obligate you
to collect a royalty for further conveying from those to whom you convey
the Program, the only way you could satisfy both those terms and this
License would be to refrain entirely from conveying the Program.

  13. Use with the GNU Affero General Public License.

  Notwithstanding any other provision of this License, you have
permission to link or combine any covered work with a work licensed
under version 3 of the GNU Affero General Public License into a single
combined work, and to convey the resulting work.  The terms of this
License will continue to apply to the part which is the covered work,
but the special requirements of the GNU Affero General Public License,
section 13, concerning interaction through a network will apply to the
combination as such.

  14. Revised Versions of this License.

  The Free Software Foundation may publish revised and/or new versions of
the GNU General Public License from time to time.  Such new versions will
be similar in spirit to the present version, but may differ in detail to
address new problems or concerns.

  Each version is given a distinguishing version number.  If the
Program specifies that a certain numbered version of the GNU General
Public License "or any later version" applies to it, you have the
option of following the terms and conditions either of that numbered
version or of any later version published by the Free Software
Foundation.  If the Program does not specify a version number of the
GNU General Public License, you may choose any version ever published
by the Free Software Foundation.

  If the Program specifies that a proxy can decide which future
versions of the GNU General Public License can be used, that proxy's
public statement of acceptance of a version permanently authorizes you
to choose that version for the Program.

  Later license versions may give you additional or different
permissions.  However, no additional obligations are imposed on any
author or copyright holder as a result of your choosing to follow a
later version.

  15. Disclaimer of Warranty.

  THERE IS NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY
APPLICABLE LAW.  EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT
HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM "AS IS" WITHOUT WARRANTY
OF ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO,
THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE.  THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE PROGRAM
IS WITH YOU.  SHOULD THE PROGRAM PROVE DEFECTIVE, YOU ASSUME THE COST OF
ALL NECESSARY SERVICING, REPAIR OR CORRECTION.

  16. Limitation of Liability.

  IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN WRITING
WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MODIFIES AND/OR CONVEYS
THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES, INCLUDING ANY
GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES ARISING OUT OF THE
USE OR INABILITY TO USE THE PROGRAM (INCLUDING BUT NOT LIMITED TO LOSS OF
DATA OR DATA BEING RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD
PARTIES OR A FAILURE OF THE PROGRAM TO OPERATE WITH ANY OTHER PROGRAMS),
EVEN IF SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF
SUCH DAMAGES.

  17. Interpretation of Sections 15 and 16.

  If the disclaimer of warranty and limitation of liability provided
above cannot be given local legal effect according to their terms,
reviewing courts shall apply local law that most closely approximates
an absolute waiver of all civil liability in connection with the
Program, unless a warranty or assumption of liability accompanies a
copy of the Program in return for a fee.

                     END OF TERMS AND CONDITIONS

            How to Apply These Terms to Your New Programs

  If you develop a new program, and you want it to be of the greatest
possible use to the public, the best way to achieve this is to make it
free software which everyone can redistribute and change under these terms.

  To do so, attach the following notices to the program.  It is safest
to attach them to the start of each source file to most effectively
state the exclusion of warranty; and each file should have at least
the "copyright" line and a pointer to where the full notice is found.

    <one line to give the program's name and a brief idea of what it does.>
    Copyright (C) <year>  <name of author>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

Also add information on how to contact you by electronic and paper mail.

  If the program does terminal interaction, make it output a short
notice like this when it starts in an interactive mode:

    <program>  Copyright (C) <year>  <name of author>
    This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
    This is free software, and you are welcome to redistribute it
    under certain conditions; type `show c' for details.

The hypothetical commands `show w' and `show c' should show the appropriate
parts of the General Public License.  Of course, your program's commands
might be different; for a GUI interface, you would use an "about box".

  You should also get your employer (if you work as a programmer) or school,
if any, to sign a "copyright disclaimer" for the program, if necessary.
For more information on this, and how to apply and follow the GNU GPL, see
<https://www.gnu.org/licenses/>.

  The GNU General Public License does not permit incorporating your program
into proprietary programs.  If your program is a subroutine library, you
may consider it more useful to permit linking proprietary applications with
the library.  If this is what you want to do, use the GNU Lesser General
Public License instead of this License.  But first, please read
<https://www.gnu.org/licenses/why-not-lgpl.html>.
```
*/
//! X86 and X64 instruction set analyzer.

// Approach to this is inspired by https://github.com/IrreducibleOSS/instruction-set-analyzer though this is an independent reimplementation (and doesn't support iterating directories).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use iced_x86::{Code, Decoder, DecoderOptions};
use object::read::archive::ArchiveFile;
use object::{Architecture, File, FileKind, Object, ObjectSection, SectionKind};
use std::collections::BTreeSet;

/// Extracts a sorted, deduplicated list of CPU instruction set names (e.g.
/// `"SSE2"`, `"AVX"`, `"AVX2"`, `"AVX512F"`) found within the given object file
/// or archive file data.
///
/// # Errors
/// Returns an error if the input data is neither a valid object file nor a
/// valid archive file.
pub fn extract_instruction_sets(data: &[u8]) -> Result<Vec<String>> {
    let mut features = BTreeSet::new();
    process_data(data, &mut features)?;
    Ok(features.into_iter().collect())
}

fn process_data(data: &[u8], features: &mut BTreeSet<String>) -> Result<()> {
    let kind = FileKind::parse(data).context("Failed to parse file format")?;

    match kind {
        FileKind::Archive => {
            let archive =
                ArchiveFile::parse(data).context("Failed to parse archive header")?;
            for member_result in archive.members() {
                if let Ok(member) = member_result {
                    if let Ok(member_data) = member.data(data) {
                        let _ = process_data(member_data, features);
                    }
                }
            }
        }
        _ => {
            let file = File::parse(data).context("Failed to parse object file")?;
            let bitness = match file.architecture() {
                Architecture::X86_64 | Architecture::X86_64_X32 => 64_u32,
                Architecture::I386 => 32_u32,
                _ => return Ok(()),
            };

            for section in file.sections() {
                if section.kind() == SectionKind::Text {
                    if let Ok(section_data) = section.uncompressed_data() {
                        let decoder =
                            Decoder::new(bitness, &section_data, DecoderOptions::NONE);
                        for instr in decoder {
                            if instr.code() != Code::INVALID {
                                for cpuid in instr.cpuid_features() {
                                    features.insert(format!("{cpuid:?}"));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
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
    use super::*;

    fn create_sample_elf() -> Vec<u8> {
        let mut elf = Vec::new();
        // --- ELF64 Header (64 bytes) ---
        // e_ident
        elf.extend_from_slice(&[
            0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        // e_type (1 = ET_REL)
        elf.extend_from_slice(&[1, 0]);
        // e_machine (62 = EM_X86_64)
        elf.extend_from_slice(&[62, 0]);
        // e_version (1)
        elf.extend_from_slice(&[1, 0, 0, 0]);
        // e_entry (0)
        elf.extend_from_slice(&[0; 8]);
        // e_phoff (0)
        elf.extend_from_slice(&[0; 8]);
        // e_shoff (97)
        elf.extend_from_slice(&[97, 0, 0, 0, 0, 0, 0, 0]);
        // e_flags (0)
        elf.extend_from_slice(&[0, 0, 0, 0]);
        // e_ehsize (64)
        elf.extend_from_slice(&[64, 0]);
        // e_phentsize (0)
        elf.extend_from_slice(&[0, 0]);
        // e_phnum (0)
        elf.extend_from_slice(&[0, 0]);
        // e_shentsize (64)
        elf.extend_from_slice(&[64, 0]);
        // e_shnum (3)
        elf.extend_from_slice(&[3, 0]);
        // e_shstrndx (2)
        elf.extend_from_slice(&[2, 0]);

        // --- Offset 64..80: .text section contents (16 bytes) ---
        // addpd xmm0, xmm1 (SSE2)
        elf.extend_from_slice(&[0x66, 0x0f, 0x58, 0xc1]);
        // vaddpd xmm0, xmm1, xmm1 (AVX)
        elf.extend_from_slice(&[0xc5, 0xf9, 0x58, 0xc1]);
        // vpaddd ymm0, ymm1, ymm2 (AVX2)
        elf.extend_from_slice(&[0xc5, 0xdd, 0xfe, 0xc2]);
        // nop padding
        elf.extend_from_slice(&[0x90, 0x90, 0x90, 0x90]);

        // --- Offset 80..97: .shstrtab section contents (17 bytes) ---
        elf.extend_from_slice(b"\0.text\0.shstrtab\0");

        // --- Offset 97..289: Section Headers (3 * 64 bytes = 192 bytes) ---
        // Section Header 0: NULL
        elf.extend_from_slice(&[0; 64]);

        // Section Header 1: .text
        elf.extend_from_slice(&[1, 0, 0, 0]); // sh_name (offset 1 in shstrtab)
        elf.extend_from_slice(&[1, 0, 0, 0]); // sh_type (SHT_PROGBITS = 1)
        elf.extend_from_slice(&[6, 0, 0, 0, 0, 0, 0, 0]); // sh_flags (ALLOC|EXEC = 6)
        elf.extend_from_slice(&[0; 8]); // sh_addr
        elf.extend_from_slice(&[64, 0, 0, 0, 0, 0, 0, 0]); // sh_offset (64)
        elf.extend_from_slice(&[16, 0, 0, 0, 0, 0, 0, 0]); // sh_size (16)
        elf.extend_from_slice(&[0; 4]); // sh_link
        elf.extend_from_slice(&[0; 4]); // sh_info
        elf.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]); // sh_addralign
        elf.extend_from_slice(&[0; 8]); // sh_entsize

        // Section Header 2: .shstrtab
        elf.extend_from_slice(&[7, 0, 0, 0]); // sh_name (offset 7 in shstrtab)
        elf.extend_from_slice(&[3, 0, 0, 0]); // sh_type (SHT_STRTAB = 3)
        elf.extend_from_slice(&[0; 8]); // sh_flags
        elf.extend_from_slice(&[0; 8]); // sh_addr
        elf.extend_from_slice(&[80, 0, 0, 0, 0, 0, 0, 0]); // sh_offset (80)
        elf.extend_from_slice(&[17, 0, 0, 0, 0, 0, 0, 0]); // sh_size (17)
        elf.extend_from_slice(&[0; 4]); // sh_link
        elf.extend_from_slice(&[0; 4]); // sh_info
        elf.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]); // sh_addralign
        elf.extend_from_slice(&[0; 8]); // sh_entsize

        elf
    }

    #[crate::ctb_test]
    fn test_invalid_file_data() {
        let invalid_data = b"not a valid binary or archive";
        assert!(extract_instruction_sets(invalid_data).is_err());
    }

    #[crate::ctb_test]
    fn test_minimal_elf_x86_64() {
        let elf_bytes = create_sample_elf();
        let result = extract_instruction_sets(&elf_bytes).unwrap();
        assert!(result.contains(&"SSE2".to_string()));
        assert!(result.contains(&"AVX".to_string()));
        assert!(result.contains(&"AVX2".to_string()));
    }

    #[crate::ctb_test]
    fn test_archive_containing_elf() {
        let elf_bytes = create_sample_elf();
        let mut ar = Vec::new();
        // Archive magic
        ar.extend_from_slice(b"!<arch>\n");
        // Member header (60 bytes)
        ar.extend_from_slice(b"sample.o/       "); // name (16)
        ar.extend_from_slice(b"0           "); // mtime (12)
        ar.extend_from_slice(b"0     "); // uid (6)
        ar.extend_from_slice(b"0     "); // gid (6)
        ar.extend_from_slice(b"644     "); // mode (8)
        let size_str = format!("{:<10}", elf_bytes.len());
        ar.extend_from_slice(size_str.as_bytes()); // size (10)
        ar.extend_from_slice(b"`\n"); // magic (2)
        // Member contents
        ar.extend_from_slice(&elf_bytes);

        let result = extract_instruction_sets(&ar).unwrap();
        assert!(result.contains(&"SSE2".to_string()));
        assert!(result.contains(&"AVX".to_string()));
        assert!(result.contains(&"AVX2".to_string()));
    }
}


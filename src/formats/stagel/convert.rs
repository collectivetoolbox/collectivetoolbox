#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub fn run_stagel_bootstrap_convert(
    debug_build: bool,
    typecheck_build: bool,
    cache_dir: Option<&std::path::Path>,
    input_file: &std::path::Path,
    target_lang: &str,
) -> Result<Vec<u8>> {
    let input_bytes = std::fs::read(input_file).with_context(|| {
        format!("Failed to read input file: {input_file:?}")
    })?;

    // Reason for fallback: default module stem name "input" used if file_stem is invalid UTF-8 or absent.
    let filename = input_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("input");

    let do_convert = || -> Result<Vec<u8>> {
        let parsed = crate::parse::parse(&input_bytes, filename)?;
        let codegen_out = crate::codegen::codegen(
            &parsed,
            target_lang,
            debug_build,
            typecheck_build,
        )?;
        Ok(codegen_out)
    };

    if let Some(cache_dir) = cache_dir {
        use sha2::{Digest, Sha512};
        let sha512_hex = |data: &[u8]| -> String {
            let mut hasher = Sha512::new();
            hasher.update(data);
            bin2hex(hasher.finalize())
        };

        let exe_path = std::env::current_exe()?;
        let exe_bytes = std::fs::read(&exe_path)?;
        let exe_hash = sha512_hex(&exe_bytes);

        let input_hash = sha512_hex(&input_bytes);

        let debug_flag = if debug_build { "--debug" } else { "--no-debug" };
        let typecheck_flag = if typecheck_build {
            "--runtime-type-checks"
        } else {
            "--no-runtime-type-checks"
        };
        let config_str =
            format!("{debug_flag}:{typecheck_flag}:{target_lang}\n");
        let config_hash = sha512_hex(config_str.as_bytes());

        let hash_a = input_hash.get(0..1).context("Invalid hash")?;
        let hash_b = input_hash.get(1..2).context("Invalid hash")?;
        let hash_c = input_hash.get(2..3).context("Invalid hash")?;

        let cache_subdir = cache_dir.join(format!(
            ".stagel-cache/1/{exe_hash}/{exe_hash}/{config_hash}/{hash_a}/{hash_b}/{hash_c}"
        ));

        std::fs::create_dir_all(&cache_subdir)?;
        let cache_file = cache_subdir.join(&input_hash);

        if cache_file.exists() {
            let cached_bytes = std::fs::read(&cache_file)?;
            return Ok(cached_bytes);
        }

        let output_bytes = do_convert()?;
        let tmp_file = cache_subdir.join(format!("{input_hash}.tmp"));
        std::fs::write(&tmp_file, &output_bytes)?;
        std::fs::rename(&tmp_file, &cache_file)?;

        Ok(output_bytes)
    } else {
        do_convert()
    }
}

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;


    #[crate::ctb_test("tokio")]
    async fn test_x86_instruction_sets_command() {
        assert!(crate::routing::is_lightweight_command(
            "x86-instruction-sets"
        ));
        let temp_dir = tempfile::tempdir().unwrap();
        let sample_file = temp_dir.path().join("dummy.bin");
        std::fs::write(&sample_file, b"not a binary").unwrap();

        let cmd = Command::X86InstructionSets { path: sample_file };
        assert!(run_lightweight_command(&cmd).await.is_err());
    }


}
#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;


    #[crate::ctb_test("tokio")]
    async fn test_wfparser_wfscan_commands() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let temp_file_path = temp_dir.path().join("wf_test.pan");
        std::fs::write(&temp_file_path, b"(Hello <tag> World)")
            .expect("Write temp file");

        let parser_cmd = Command::Wfparser {
            file: temp_file_path.clone(),
        };
        let parser_result = run_lightweight_command(&parser_cmd)
            .await
            .expect("Run parser command");
        match parser_result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(
                    String::from_utf8_lossy(&stdout),
                    "(Hello   World)\n"
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let scan_cmd = Command::Wfscan {
            file: temp_file_path.clone(),
        };
        let scan_result = run_lightweight_command(&scan_cmd)
            .await
            .expect("Run scan command");
        match scan_result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), " hello world \n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

}
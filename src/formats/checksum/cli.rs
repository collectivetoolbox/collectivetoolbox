#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

            let data = read_file_or_stdin(file.as_path())?;
            let hash_algo =
                ctb_formats_checksum::HashAlgorithm::try_from(algo.as_str())?;
            let output = format!(
                "{}\n",
                ctb_formats_checksum::hash_hex(&data, hash_algo, *prefix_0x)
            );
            Ok(ToolResult::immediate_ok(output.into_bytes()))

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;


    #[crate::ctb_test("tokio")]
    async fn test_csum_command() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let temp_file_path = temp_dir.path().join("csum_test_temp.txt");
        std::fs::write(&temp_file_path, b"hello world")
            .expect("Write temp file");

        let cmd = Command::Csum {
            algo: "xxhash32".to_string(),
            file: temp_file_path.clone(),
            prefix_0x: false,
        };
        let result = run_lightweight_command(&cmd)
            .await
            .expect("Run lightweight command");
        match result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), "cebb6622\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let cmd_0x = Command::Csum {
            algo: "xxhash32".to_string(),
            file: temp_file_path.clone(),
            prefix_0x: true,
        };
        let result_0x = run_lightweight_command(&cmd_0x)
            .await
            .expect("Run lightweight command");
        match result_0x {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), "0xcebb6622\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

}
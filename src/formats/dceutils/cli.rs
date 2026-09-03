#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test("tokio")]
    async fn test_dceutils_php_to_csv_command() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let random_num = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let php_filename = format!("test_cmd_{random_num}.php");
        let php_path = temp_dir.path().join(&php_filename);

        let php_content = r"<?php
$my_test_array = array('a' => '1', 'b' => '2');
?>";
        std::fs::write(&php_path, php_content).expect("Write temp PHP file");

        let expected_csv_name = format!("{php_filename}-my_test_array.csv");
        let expected_csv_path = std::path::Path::new(&expected_csv_name);

        if expected_csv_path.exists() {
            let _ = std::fs::remove_file(expected_csv_path);
        }

        let cmd = Command::DceutilsPhpToCsv {
            php_file: php_path.clone(),
        };
        let result = run_lightweight_command(&cmd)
            .await
            .expect("Run lightweight command");
        match result {
            ToolResult::Immediate { .. } => {
                assert!(expected_csv_path.exists());
                let csv_content = std::fs::read_to_string(expected_csv_path)
                    .expect("Read CSV content");
                assert_eq!(csv_content, "a,1\nb,2\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let _ = std::fs::remove_file(expected_csv_path);
    }


}
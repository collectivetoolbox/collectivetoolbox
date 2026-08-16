use ctb_utilities::string::to_char;

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

/// Implements the CLI base2base helper
pub fn base2base {
    let (input, from_base, to_base) = if args.len() >= 3
                && let (Ok(from), Ok(to)) = (
                    args.first()
                        .ok_or_else(|| anyhow!("Missing from_base"))?
                        .parse::<u8>(),
                    args.get(1)
                        .ok_or_else(|| anyhow!("Missing to_base"))?
                        .parse::<u8>(),
                )
                && from >= 2
                && from <= 36
                && to >= 2
                && to <= 36
            {
                let input =
                    args.get(2..).map(|s| s.join(" ")).unwrap_or_default();
                (input, Some(from), Some(to))
            } else if args.is_empty() {
                bail!("Invalid arguments! Usage: base2base [FROM_BASE TO_BASE INPUT] or [INPUT]");
            } else {
                let input = args.join(" ");
                (input, None, None)
            };

            run_base_convert(
                &from_base,
                &to_base,
                &StringInput {
                    input: input.clone(),
                },
                base_args,
            )
}

/// Implements the CLI hex2bin helper
pub fn hex2bin {
                let input_str = if let Some(val) = value {
                val.clone()
            } else {
                use std::io::Read;
                let mut buf = Vec::new();
                std::io::stdin().read_to_end(&mut buf)?;
                String::from_utf8(buf).context("Input is not valid UTF-8")?
            };
            let decoded = ctb_formats_hexdump::hex2bin(&input_str)?;
            Ok(ToolResult::immediate_ok(decoded))
}

/// Implements the CLI bin2hex helper
pub fn bin2hex {
    let input_bytes = if let Some(file_path) = file {
                read_file_or_stdin(file_path.as_path())?
            } else if let Some(val) = value {
                val.as_bytes().to_vec()
            } else {
                read_file_or_stdin(std::path::Path::new("-"))?
            };
            let encoded = if *hd {
                ctb_formats_hexdump::to_hex_dump(&input_bytes)
            } else if *hf {
                ctb_formats_hexdump::to_fancy_hex_dump(&input_bytes)
            } else {
                ctb_formats_hexdump::bin2hex(&input_bytes)
            };
            if let Some(out_path) = output {
                if out_path.as_path() == std::path::Path::new("-") {
                    Ok(ToolResult::immediate_ok(encoded.into_bytes()))
                } else {
                    std::fs::write(out_path, encoded.as_bytes()).with_context(
                        || {
                            format!(
                                "Failed to write output file: {path_display}",
                                path_display = out_path.display()
                            )
                        },
                    )?;
                    Ok(ToolResult::immediate_ok(Vec::new()))
                }
            } else {
                Ok(ToolResult::immediate_ok(encoded.into_bytes()))
            }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;


    #[crate::ctb_test("tokio")]
    async fn test_hex2bin_and_bin2hex_commands() {
        let cmd = Command::Hex2Bin {
            value: Some("48656c6c6f".to_string()),
        };
        let result = run_lightweight_command(&cmd).await.expect("Run hex2bin");
        match result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(stdout, b"Hello");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let cmd2 = Command::Bin2Hex {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            hd: false,
            hf: false,
        };
        let result2 =
            run_lightweight_command(&cmd2).await.expect("Run bin2hex");
        match result2 {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(stdout, b"48656c6c6f");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_bin2hex_options() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let in_path = temp_dir.path().join("test_in.bin");
        let out_path = temp_dir.path().join("test_out.hex");

        let data = b"Hello, World!\x00\x01\xff";
        std::fs::write(&in_path, data).expect("Write temp in file");

        // Test -f and -o with default format
        let cmd_file = Command::Bin2Hex {
            value: None,
            file: Some(in_path.clone()),
            output: Some(out_path.clone()),
            hd: false,
            hf: false,
        };
        let res_file = run_lightweight_command(&cmd_file)
            .await
            .expect("Run bin2hex with -f and -o");
        match res_file {
            ToolResult::Immediate { stdout, .. } => {
                assert!(stdout.is_empty());
                let written =
                    std::fs::read_to_string(&out_path).expect("Read output file");
                assert_eq!(written, ctb_formats_hexdump::bin2hex(data));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Test --hd (classic hex dump format)
        let cmd_hd = Command::Bin2Hex {
            value: None,
            file: Some(in_path.clone()),
            output: None,
            hd: true,
            hf: false,
        };
        let res_hd = run_lightweight_command(&cmd_hd)
            .await
            .expect("Run bin2hex with --hd");
        match res_hd {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(
                    String::from_utf8(stdout).expect("UTF-8 stdout"),
                    ctb_formats_hexdump::to_hex_dump(data)
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Test --hf (fancy hex dump format)
        let cmd_hf = Command::Bin2Hex {
            value: None,
            file: Some(in_path.clone()),
            output: None,
            hd: false,
            hf: true,
        };
        let res_hf = run_lightweight_command(&cmd_hf)
            .await
            .expect("Run bin2hex with --hf");
        match res_hf {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(
                    String::from_utf8(stdout).expect("UTF-8 stdout"),
                    ctb_formats_hexdump::to_fancy_hex_dump(data)
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Test CLI parsing
        let parsed = parse_invocation(Some(vec![
            "ctoolbox".to_string(),
            "bin2hex".to_string(),
            "-f".to_string(),
            in_path.to_str().expect("Valid path").to_string(),
            "-o".to_string(),
            out_path.to_str().expect("Valid path").to_string(),
            "--hd".to_string(),
        ]))
        .expect("Parse CLI invocation");

        if let Invocation::User(cli) = parsed {
            if let Some(Command::Bin2Hex {
                file,
                output,
                hd,
                hf,
                ..
            }) = cli.command
            {
                assert_eq!(file, Some(in_path));
                assert_eq!(output, Some(out_path));
                assert!(hd);
                assert!(!hf);
            } else {
                panic!("Expected Command::Bin2Hex");
            }
        } else {
            panic!("Expected Invocation::User");
        }
    }
}
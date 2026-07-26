#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use anyhow::{Result, anyhow};
use clap::Subcommand;
use ctb_formats_stagel::convert::run_stagel_bootstrap_convert;
use ctb_utilities::json::maybe_value::MaybeOption;
use std::path::PathBuf;

use crate::base_conversion::{BaseArgs, run_base_convert};
use crate::utilities::{
    fork, get_this_executable, upgrade_in_place,
    wait_for_ctoolbox_exit_and_clean_up,
};
use crate::{StringInput, ToolResult, generate_help_bytes};

/// Return true if this is a command that can be run without booting.
pub fn is_lightweight_command(command: &str) -> bool {
    matches!(
        command,
        "csum"
            | "compress"
            | "decompress"
            | "base2base"
            | "hex2dec"
            | "dec2hex"
            | "hexfmt"
            | "hex2bin"
            | "bin2hex"
            | "ctb-asset-bundle-extract"
            | "js-lint"
            | "js-test"
            | "gdb_instructions_generate"
            | "ia"
            | "pan2csv"
            | "stagel-bootstrap-parse"
            | "stagel-bootstrap-convert"
            | "pan2parsejson"
            | "pdf2txt"
            | "pdf2json"
            | "pdf2md"
            | "warcat"
            | "help"
            | "ctb-dev-sign"
            | "ctb-dev-gz-sha256"
            | "ctb-dev-key-create"
            | "ctb-dev-release-check"
            | "ctb-dev-release-expire"
            | "install"
            | "make-offline-installer"
            | "ts-check"
            | "update"
            | "uninstall"
            | "ctb-upgrade-canary"
            | "jq"
            | "json-escape"
            | "dceutils_php_to_csv"
    )
}

#[derive(Subcommand, Debug)]
pub enum IACommand {
    /// Verify a local Internet Archive item directory against its files XML.
    Verify {
        /// Path to the item directory, or omit to use the current directory.
        item_path: Option<PathBuf>,
        /// Override the identifier if it cannot be inferred from the path.
        #[arg(long)]
        identifier: Option<String>,
        /// Fetch the current files XML from archive.org instead of using a
        /// local copy.
        #[arg(long)]
        check_live: bool,
    },
    /// Print the expected sha1 for a file in an Internet Archive item.
    Sha1 {
        /// A local file path, item/file path, or archive.org download URL.
        target: String,
        /// Override the identifier if it cannot be inferred from the path.
        #[arg(long)]
        identifier: Option<String>,
        /// Fetch live metadata instead of reading a local files XML.
        #[arg(long)]
        check_live: bool,
    },
    /// Print the expected md5 for a file in an Internet Archive item.
    Md5 {
        /// A local file path, item/file path, or archive.org download URL.
        target: String,
        /// Override the identifier if it cannot be inferred from the path.
        #[arg(long)]
        identifier: Option<String>,
        /// Fetch live metadata instead of reading a local files XML.
        #[arg(long)]
        check_live: bool,
    },
    /// Check whether an item contains a particular file.
    #[command(name = "contains")]
    Contains {
        /// An item identifier, item/file path, or archive.org URL.
        target: String,
        /// File path inside the item to check for.
        desired_file: String,
    },
    /// List item files one per line.
    #[command(name = "listplain")]
    ListPlain {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Fetch live item metadata as pretty JSON.
    Metadata {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Fetch the live `_files.xml` document.
    #[command(name = "filesxml")]
    FilesXml {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Fetch the live `_meta.xml` document.
    #[command(name = "metaxml")]
    MetaXml {
        /// An item identifier or archive.org URL.
        target: String,
    },
    /// Download an item or file from archive.org.
    Download {
        /// An item identifier, item/file path, or archive.org download URL.
        target: String,
        /// Destination directory. Defaults to the current directory.
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
    /// Download a single file and write it to stdout.
    #[command(name = "downloadAsStream")]
    DownloadAsStream {
        /// An item/file path or archive.org download URL.
        target: String,
    },
    /// Download a single file into the current directory.
    #[command(name = "downloadHere")]
    DownloadHere {
        /// An item/file path or archive.org download URL.
        target: String,
        /// Destination directory. Defaults to the current directory.
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
    /// Download an item or file, then verify the downloaded content.
    Checkeddl {
        /// An item identifier, item/file path, or archive.org download URL.
        target: String,
        /// Destination directory. Defaults to the current directory.
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Wait for the provided PID to shutdown, then clean up
    #[command(name = "waitshutdown")]
    WaitShutdown {
        /// Process ID to wait for
        #[arg(long)]
        pid: u32,
    },
    /// Wait for the provided PID to shutdown, then start ctoolbox with the
    /// given port
    #[command(name = "waitrestart")]
    WaitRestart {
        /// Process ID to wait for
        #[arg(long)]
        pid: u32,
        /// Port to pass to the new ctoolbox instance
        #[arg(long)]
        port: u16,
    },
    /// Wait for the provided PID to shutdown, then upgrade the ctoolbox
    /// instance in place, then restart it. (Will need to copy the executable
    /// before upgrading it probably because Windows locks running executables.)
    #[command(name = "waitupgrade")]
    WaitUpgrade {
        /// Process ID to wait for
        #[arg(long)]
        pid: u32,
        /// Path to the temporary file holding the new ctoolbox executable
        #[arg(long)]
        temp_path: PathBuf,
        /// Path to the installed ctoolbox executable
        #[arg(long)]
        target_path: PathBuf,
        /// Port to pass to the new ctoolbox instance
        #[arg(long)]
        port: u16,
    },
    /// Convert from one base to another (for base <= 36)
    #[command(name = "base2base")]
    Base2Base {
        /// All positional arguments for custom parsing
        #[arg(required = true)]
        args: Vec<String>,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Convert from hexadecimal to decimal
    #[command(name = "hex2dec")]
    Hex2Dec {
        #[command(flatten)]
        string_input: StringInput,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Convert from decimal to hexadecimal
    #[command(name = "dec2hex")]
    Dec2Hex {
        #[command(flatten)]
        string_input: StringInput,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Reformat hexdumps
    #[command(name = "hexfmt")]
    Hexfmt {
        #[command(flatten)]
        string_input: StringInput,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Convert a hexadecimal string to binary data
    #[command(name = "hex2bin")]
    Hex2Bin {
        /// Hexadecimal string. If not provided, reads from stdin.
        value: Option<String>,
    },
    /// Convert binary data to a hexadecimal string
    #[command(name = "bin2hex")]
    Bin2Hex {
        /// Data to convert. If not provided, reads from stdin.
        value: Option<String>,
    },
    /// Generate GDB instructions from symbols
    #[command(name = "gdb_instructions_generate")]
    GdbInstructionsGenerate {},
    /// Internet Archive utilities.
    #[command(name = "ia")]
    IA {
        #[command(subcommand)]
        command: IACommand,
    },
    /// Convert a .pan file to CSV output
    #[command(name = "pan2csv")]
    Pan2Csv {
        /// Input PAN file path
        pan_file: PathBuf,
    },
    /// Parse a StageL file to token output
    #[command(name = "stagel-bootstrap-parse")]
    StagelBootstrapParse {
        /// Input StageL file path
        input_file: PathBuf,
    },
    /// Translate a StageL file using the bootstrap compiler
    #[command(name = "stagel-bootstrap-convert")]
    StagelBootstrapConvert {
        /// Optional cache directory (if specified, enables caching)
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Disable debug build
        #[arg(long)]
        no_debug: bool,
        /// Disable runtime type checks
        #[arg(long)]
        no_runtime_type_checks: bool,
        /// Input StageL file path
        input_file: PathBuf,
        /// Target language (js or bash)
        target_lang: String,
    },
    /// Convert a .pan file to JSON of parse
    #[command(name = "pan2parsejson")]
    Pan2ParseJson {
        /// Input PAN file path
        pan_file: PathBuf,
    },
    /// Convert a PDF file to text output
    #[command(name = "pdf2txt")]
    Pdf2Txt {
        /// Input PDF file path (or - for stdin)
        pdf_file: PathBuf,
    },
    /// Convert a PDF file to JSON output
    #[command(name = "pdf2json")]
    Pdf2Json {
        /// Input PDF file path (or - for stdin)
        pdf_file: PathBuf,
    },
    /// Convert a PDF file to Markdown output
    #[command(name = "pdf2md")]
    Pdf2Md {
        /// Input PDF file path (or - for stdin)
        pdf_file: PathBuf,
    },
    /// WARC archiving tool
    #[command(
        name = "warcat",
        trailing_var_arg = true,
        allow_hyphen_values = true,
        disable_help_flag = true
    )]
    Warcat {
        /// Arguments passed to warcat
        args: Vec<String>,
    },
    /// Extract a ctoolbox asset bundle to a directory tree.
    #[command(name = "ctb-asset-bundle-extract")]
    CtbAssetBundleExtract {
        /// Path to the `.rsrc` asset bundle.
        bundle_path: PathBuf,
        /// Output directory for the extracted assets.
        output_dir: Option<PathBuf>,
    },
    /// Lint JavaScript and TypeScript sources.
    #[command(name = "js-lint")]
    JsLint {
        #[command(flatten)]
        cli: ctb_formats_javascript::js_lint::JsLintCli,
    },
    /// Type-check TypeScript code.
    #[command(name = "ts-check")]
    TsCheck {
        #[command(flatten)]
        args: ctb_formats_javascript::ts_check::TsCheckArgs,
    },
    /// Run JavaScript tests.
    #[command(name = "js-test")]
    JsTest {
        #[command(flatten)]
        args: ctb_formats_javascript::js_test::JsTestArgs,
    },
    /// Calculate checksum for a file or stdin
    #[command(name = "csum")]
    Csum {
        /// Hash algorithm type (`xxhash32`, `xxhash64`, `xxhash3_64`, `xxhash3_128`)
        algo: String,
        /// Input file path (or - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
        /// Prefix the output hex string with 0x
        #[arg(long = "prefix-0x")]
        prefix_0x: bool,
    },
    /// Compress a file or stdin using single-stream compression format
    #[command(name = "compress")]
    Compress {
        /// Compression format (`brotli` / `.br`, `gzip` / `.gz`, `deflate` / `.deflate`, `zlib` / `.zz` / `.zl`)
        format: String,
        /// Input file path (or - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
        /// Output file path or - for stdout
        #[arg(short = 'o', long = "output", alias = "file", value_name = "OUTPUT")]
        output: Option<PathBuf>,
        /// Force overwrite without prompting
        #[arg(long = "force")]
        force: bool,
    },
    /// Decompress a compressed file or stdin
    #[command(name = "decompress")]
    Decompress {
        /// Optional compression format (`brotli`, `gzip`, `deflate`, `zlib`). If omitted, detected from file extension or magic bytes.
        format: Option<String>,
        /// Input file path (or - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
        /// Output file path or - for stdout
        #[arg(short = 'o', long = "output", alias = "file", value_name = "OUTPUT")]
        output: Option<PathBuf>,
        /// Force overwrite without prompting
        #[arg(long = "force")]
        force: bool,
    },
    /// Process a file using wfparser logic
    #[command(name = "wfparser")]
    Wfparser {
        /// Input file path (or - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
    },
    /// Process a file using wfscan logic
    #[command(name = "wfscan")]
    Wfscan {
        /// Input file path (or - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
    },
    /// Convert PHP data file arrays to CSV files
    #[command(name = "dceutils_php_to_csv")]
    DceutilsPhpToCsv {
        /// Path to the PHP data file
        php_file: PathBuf,
    },
    /// Show help
    Help,
    /// Unimplemented, just an example
    ShowNode {
        /// Example parameter
        #[arg(short, long)]
        id: i128,
    },
    /// Sign release artifacts for distribution (developer command)
    #[command(name = "ctb-dev-sign")]
    DevSign {
        /// Directory containing release artifacts to sign.
        /// Defaults to ~/ctb_release/input if not specified.
        #[allow(clippy::doc_markdown, reason = "doc comment references local path conventions")]
        #[arg(long)]
        input_dir: Option<PathBuf>,
        /// Directory to write signed chunks and manifest.
        /// Defaults to ~/ctb_release/releases if not specified.
        #[allow(clippy::doc_markdown, reason = "doc comment references local path conventions")]
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Target for this release (e.g. linux-x64, linux-x86, windows-x64,
        /// mac-x64, mac-arm64).
        /// Defaults to current platform if not specified.
        #[arg(long)]
        platform: Option<String>,
    },
    /// Compress a tar file with the server's gzip settings and print its SHA256 hex (developer command)
    #[command(name = "ctb-dev-gz-sha256")]
    DevGzSha256 {
        /// Path to the tar file
        path: PathBuf,
    },

    /// Generate a signing keypair for `ctb-dev-sign`.
    ///
    /// If `--write` is provided, writes the generated keys into local
    /// `pc_settings.json` as `dev_signing_private_key` and
    /// `dev_signing_public_key`.
    #[command(name = "ctb-dev-key-create")]
    DevKeyCreate {
        /// Write keys into local `pc_settings.json`.
        #[arg(long)]
        write: bool,
    },
    /// Verify an uploaded release: check signature and chunk hashes
    #[command(name = "ctb-dev-release-check")]
    DevReleaseCheck {
        /// Path to the manifest file to verify.
        /// Defaults to `{storage_dir}/releases/ctb-{platform}-latest.json` if
        /// not specified.
        #[arg(long)]
        manifest: Option<PathBuf>,
        /// Path to the chunks directory.
        /// Defaults to `{storage_dir}/releases/bh/` if not specified.
        #[arg(long)]
        chunks_dir: Option<PathBuf>,

        /// Target to verify (e.g. linux-x64, linux-x86).
        /// Defaults to the current platform.
        #[arg(long)]
        platform: Option<String>,
    },
    /// Delete old release chunks and manifests to reclaim disk space.
    ///
    /// Scans the releases directory for manifest files, builds a set of all
    /// chunk hashes referenced by manifests newer than the threshold, then
    /// deletes any chunks not in the keep set along with old manifest files.
    #[command(name = "ctb-dev-release-expire")]
    DevReleaseExpire {
        /// Only expire chunks from releases older than this many days.
        /// Defaults to 30 days if not specified.
        #[arg(long, default_value = "30")]
        older_than: u32,
        /// Path to the releases directory.
        /// Defaults to `{storage_dir}/releases/` if not specified.
        #[arg(long)]
        releases_dir: Option<PathBuf>,
    },
    /// Launch the installer GUI (or TUI if --no-gui).
    ///
    /// Installs from files in the current directory if present, otherwise
    /// downloads from the update server.
    Install {
        /// Use text-mode installer instead of GUI.
        #[arg(long)]
        no_gui: bool,
        /// Run in unattended mode with default options (implies --no-gui).
        #[arg(long)]
        unattended: bool,
    },
    /// Download and package the latest offline installer bundle.
    #[command(name = "make-offline-installer")]
    MakeOfflineInstaller {
        /// Output path for the generated bundle.
        output: PathBuf,
        /// Target platform to fetch. Defaults to the current platform.
        #[arg(long)]
        platform: Option<String>,
        /// Release version to fetch. Defaults to latest.
        #[arg(long)]
        version: Option<String>,
        /// URL of the update server.
        /// Defaults to the configured server URL.
        #[arg(long)]
        server_url: Option<String>,
    },
    /// Check for updates and optionally apply them.
    ///
    /// By default, prompts the user if an update is available. Use
    /// --unattended to automatically apply updates without prompting.
    Update {
        /// Automatically apply updates without prompting.
        #[arg(long)]
        unattended: bool,
        /// URL of the update server.
        /// Defaults to the configured server URL.
        #[arg(long)]
        server_url: Option<String>,
    },
    /// Uninstall ctoolbox from this system.
    ///
    /// Reads installation.json to determine installed files, removes them,
    /// removes the desktop entry, and removes PATH modifications.
    Uninstall {
        /// Use text-mode uninstaller instead of GUI.
        #[arg(long)]
        no_gui: bool,
        /// Run without prompting for confirmation.
        #[arg(long)]
        unattended: bool,
    },
    /// Internal: Post-upgrade canary validation.
    ///
    /// This command is run by the new binary after an upgrade. If the process
    /// survives 30 seconds, the backup is deleted and the process exits. If
    /// it crashes, the caller's watchdog logic restores the backup.
    #[command(name = "ctb-upgrade-canary")]
    UpgradeCanary {
        /// Path to the backup copy of the previous binary.
        #[arg(long)]
        backup_path: PathBuf,
        /// Path to the installed binary location.
        #[arg(long)]
        target_path: PathBuf,
        /// Optional port to restart ctoolbox with after successful validation.
        #[arg(long)]
        port: Option<u16>,
    },
    /// Lightweight JQ implementation
    #[command(name = "jq")]
    Jq {
        /// Use raw output (no quotes around strings)
        #[arg(short = 'r', long = "raw-output")]
        raw_output: bool,
        /// The JQ query string
        query: String,
        /// Optional path to the JSON file, or - for stdin
        file: Option<PathBuf>,
    },
    /// Escape a string to be a valid JSON string value (enclosed in double
    /// quotes).
    #[command(name = "json-escape")]
    JsonEscape {
        /// The string to escape. If not provided, reads from stdin.
        value: Option<String>,
    },
}

fn read_file_or_stdin(path: &std::path::Path) -> Result<Vec<u8>> {
    use std::io::Read;
    if path.to_str() == Some("-") {
        let mut buf = Vec::new();
        std::io::stdin().lock().read_to_end(&mut buf)
            .context("Failed to read from stdin")?;
        Ok(buf)
    } else {
        std::fs::read(path).with_context(|| {
            format!(
                "Could not read file: {path_display}",
                path_display = path.display()
            )
        })
    }
}

fn check_overwrite_prompt(path: &std::path::Path, force: bool) -> Result<bool> {
    use std::io::IsTerminal;
    if force || path == std::path::Path::new("-") || !path.exists() {
        return Ok(true);
    }
    if std::io::stdin().is_terminal() {
        eprint!(
            "Output file '{path_display}' already exists. Overwrite? [y/N] ",
            path_display = path.display()
        );
        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        let trimmed = response.trim().to_ascii_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            return Ok(false);
        }
    }
    Ok(true)
}

// ---------------------------
// Command Execution
// ---------------------------

#[allow(clippy::too_many_lines, reason = "uniform tool command router is naturally long")]
pub async fn run_lightweight_command(cmd: &Command) -> Result<ToolResult> {
    match cmd {
        Command::WaitShutdown { pid } => {
            wait_for_ctoolbox_exit_and_clean_up(*pid);
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::WaitRestart { pid, port } => {
            wait_for_ctoolbox_exit_and_clean_up(*pid);
            fork(
                &get_this_executable(),
                vec!["--ctoolbox-ipc-port", &port.to_string().as_str()],
            );
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::WaitUpgrade {
            pid,
            temp_path,
            target_path,
            port,
        } => {
            wait_for_ctoolbox_exit_and_clean_up(*pid);
            upgrade_in_place(temp_path, target_path)?;
            fork(
                target_path,
                vec!["--ctoolbox-ipc-port", &port.to_string().as_str()],
            );
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::Base2Base { args, base_args } => {
            let (input, from_base, to_base) = match args.len() {
                1 => (
                    args.first().ok_or_else(|| anyhow!("Empty arguments"))?,
                    None,
                    None,
                ),
                3 => {
                    let from_base = Some(
                        args.first()
                            .ok_or_else(|| anyhow!("Missing from_base"))?
                            .parse::<u8>()?,
                    );
                    let to_base = Some(
                        args.get(1)
                            .ok_or_else(|| anyhow!("Missing to_base"))?
                            .parse::<u8>()?,
                    );
                    let input = args.get(2).ok_or_else(|| anyhow!("Missing input"))?;
                    (input, from_base, to_base)
                }
                _ => {
                    eprintln!(
                        "Invalid arguments! Usage: base2base [FROM_BASE TO_BASE INPUT] or [INPUT]"
                    );
                    std::process::exit(1);
                }
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
        Command::Hex2Dec {
            string_input,
            base_args,
        } => run_base_convert(&Some(16), &Some(10), string_input, base_args),
        Command::Dec2Hex {
            string_input,
            base_args,
        } => run_base_convert(&Some(10), &Some(16), string_input, base_args),
        Command::Hexfmt {
            string_input,
            base_args,
        } => run_base_convert(&Some(16), &Some(16), string_input, base_args),
        Command::Hex2Bin { value } => {
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
        Command::Bin2Hex { value } => {
            let input_bytes = if let Some(val) = value {
                val.as_bytes().to_vec()
            } else {
                use std::io::Read;
                let mut buf = Vec::new();
                std::io::stdin().read_to_end(&mut buf)?;
                buf
            };
            let encoded = ctb_formats_hexdump::bin2hex(&input_bytes);
            Ok(ToolResult::immediate_ok(encoded.into_bytes()))
        }
        Command::GdbInstructionsGenerate {} => {
            // FIXME: Use a streaming ToolResult here
            crate::utilities::debug_tools::generate_gdb_instructions_streaming(
            )?;
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::IA { command } => match command {
            IACommand::Verify {
                item_path,
                identifier,
                check_live,
            } => {
                let item_path = if let Some(item_path) = item_path {
                    item_path.clone()
                } else {
                    std::env::current_dir()
                        .context("Failed to get current directory")?
                };
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::verify(
                        item_path.as_path(),
                        identifier.as_deref(),
                        *check_live,
                    )?,
                ))
            }
            IACommand::Sha1 {
                target,
                identifier,
                check_live,
            } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::iasha1(
                    target,
                    identifier.as_deref(),
                    *check_live,
                )?,
            )),
            IACommand::Md5 {
                target,
                identifier,
                check_live,
            } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::iamd5(
                    target,
                    identifier.as_deref(),
                    *check_live,
                )?,
            )),
            IACommand::Contains {
                target,
                desired_file,
            } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::contains(target, desired_file)?,
            )),
            IACommand::ListPlain { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::listplain(target)?,
            )),
            IACommand::Metadata { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::metadata(target)?,
            )),
            IACommand::FilesXml { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::filesxml(target)?,
            )),
            IACommand::MetaXml { target } => Ok(ToolResult::immediate_ok(
                ctb_formats_internetarchive::metaxml(target)?,
            )),
            IACommand::Download { target, output_dir } => {
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::download(
                        target,
                        output_dir.as_deref(),
                    )?,
                ))
            }
            IACommand::DownloadAsStream { target } => {
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::download_as_stream(target)?,
                ))
            }
            IACommand::DownloadHere { target, output_dir } => {
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::download_here(
                        target,
                        output_dir.as_deref(),
                    )?,
                ))
            }
            IACommand::Checkeddl { target, output_dir } => {
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::checkeddl(
                        target,
                        output_dir.as_deref(),
                    )?,
                ))
            }
        },
        Command::Pan2Csv { pan_file } => {
            let data = read_file_or_stdin(pan_file.as_path())?;
            let output = ctb_formats_pan::output::pan_to_csv(&data, false)?.into_bytes();
            Ok(ToolResult::immediate_ok(output))
        }
        Command::StagelBootstrapParse {
            input_file,
        } => {
            let data = std::fs::read(&input_file)
                .with_context(|| format!("Failed to read input file: {:?}", input_file))?;
            let filename = input_file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let output = ctb_formats_stagel::parse::parse(&data, &filename)?;
            Ok(ToolResult::immediate_ok(output))
        }
        Command::StagelBootstrapConvert {
            cache_dir,
            no_debug,
            no_runtime_type_checks,
            input_file,
            target_lang,
        } => {
            let output = run_stagel_bootstrap_convert(
                !no_debug,
                !no_runtime_type_checks,
                cache_dir.as_deref(),
                &input_file,
                &target_lang,
            )?;
            Ok(ToolResult::immediate_ok(output))
        }
        Command::Pan2ParseJson { pan_file } => {
            let data = read_file_or_stdin(pan_file.as_path())?;
            let output = ctb_formats_pan::output::pan_to_parse_json(&data)?.into_bytes();
            Ok(ToolResult::immediate_ok(output))
        }
        Command::Pdf2Txt { pdf_file } => {
            let data = read_file_or_stdin(pdf_file.as_path())?;
            let output = ctb_formats_pdf::pdf2txt(&data)?;
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
        Command::Pdf2Json { pdf_file } => {
            let data = read_file_or_stdin(pdf_file.as_path())?;
            let output = ctb_formats_pdf::pdf2json(&data)?;
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
        Command::Pdf2Md { pdf_file } => {
            let data = read_file_or_stdin(pdf_file.as_path())?;
            let output = ctb_formats_pdf::pdf2md(&data)?;
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
        Command::Warcat { args } => {
            let mut warcat_args = vec!["warcat".to_string()];
            warcat_args.extend(args.clone());
            let exit_code = ctb_formats_warc::run_warcat(warcat_args);
            Ok(ToolResult::Immediate {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code,
            })
        }
        Command::CtbAssetBundleExtract {
            bundle_path,
            output_dir,
        } => {
            let cwd = std::env::current_dir()
                .expect("Failed to get current directory");
            let cwd = cwd.as_path();
            let output_dir =
                ctb_formats_ctb_asset_bundle::extract_asset_bundle(
                    bundle_path.as_path(),
                    output_dir.as_deref().unwrap_or(cwd),
                )?;
            Ok(ToolResult::immediate_ok(
                format!("Extracted asset bundle to {}\n", output_dir.display())
                    .into_bytes(),
            ))
        }
        Command::JsLint { cli } => Ok(ToolResult::Immediate {
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: ctb_formats_javascript::js_lint::run_cli(cli)?,
        }),
        Command::TsCheck { args } => Ok(ToolResult::Immediate {
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: ctb_formats_javascript::ts_check::run_typecheck_args(
                args,
            )?,
        }),
        Command::JsTest { args } => Ok(ToolResult::Immediate {
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: ctb_formats_javascript::js_test::run_test_args(
                args,
            )?,
        }),
        Command::Csum { algo, file, prefix_0x } => {
            let data = read_file_or_stdin(file.as_path())?;
            let hash_algo = ctb_formats_checksum::HashAlgorithm::from(algo.clone());
            let output = format!(
                "{}\n",
                ctb_formats_checksum::hash_hex(&data, hash_algo, *prefix_0x)
            );
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
        Command::Compress {
            format,
            file,
            output,
            force,
        } => {
            let compression_format =
                ctb_formats_compression::CompressionFormat::try_from(format.as_str())?;
            let data = read_file_or_stdin(file.as_path())?;
            let compressed =
                ctb_formats_compression::compress(&data, compression_format)?;

            let target_path = match output {
                Some(out_path) => out_path.clone(),
                None => {
                    if file.as_path() == std::path::Path::new("-") {
                        PathBuf::from("-")
                    } else {
                        PathBuf::from(format!(
                            "{}.{}",
                            file.display(),
                            compression_format.extension()
                        ))
                    }
                }
            };

            if target_path.as_path() == std::path::Path::new("-") {
                Ok(ToolResult::immediate_ok(compressed))
            } else {
                if !check_overwrite_prompt(&target_path, *force)? {
                    return Ok(ToolResult::immediate_err(
                        "Operation cancelled.\n".as_bytes().to_vec(),
                        1,
                    ));
                }
                std::fs::write(&target_path, &compressed)
                    .with_context(|| format!("Failed to write to {}", target_path.display()))?;
                Ok(ToolResult::immediate_ok(Vec::new()))
            }
        }
        Command::Decompress {
            format,
            file,
            output,
            force,
        } => {
            let (input_path, raw_format) = match format {
                Some(fmt_str) => {
                    if let Ok(parsed_fmt) =
                        ctb_formats_compression::CompressionFormat::try_from(fmt_str.as_str())
                    {
                        (file.clone(), Some(parsed_fmt))
                    } else {
                        let in_path = PathBuf::from(fmt_str);
                        let inferred = in_path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .and_then(ctb_formats_compression::CompressionFormat::from_extension);
                        (in_path, inferred)
                    }
                }
                None => {
                    let inferred = file
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .and_then(ctb_formats_compression::CompressionFormat::from_extension);
                    (file.clone(), inferred)
                }
            };

            let data = read_file_or_stdin(input_path.as_path())?;

            let compression_format = match raw_format {
                Some(fmt) => fmt,
                None => ctb_formats_compression::CompressionFormat::from_magic_bytes(&data)
                    .ok_or_else(|| {
                        anyhow!(
                            "Could not determine compression format for '{}'",
                            input_path.display()
                        )
                    })?,
            };

            let decompressed =
                ctb_formats_compression::decompress(&data, compression_format)?;

            let target_path = match output {
                Some(out_path) => out_path.clone(),
                None => {
                    if input_path.as_path() == std::path::Path::new("-") {
                        PathBuf::from("-")
                    } else {
                        let filename_str = input_path.to_string_lossy();
                        let known_exts = [".br", ".gz", ".deflate", ".zz", ".zl"];
                        let mut stripped = None;
                        for ext in known_exts {
                            if filename_str.to_ascii_lowercase().ends_with(ext) {
                                let cut_len = filename_str.len().saturating_sub(ext.len());
                                if let Some(prefix) = filename_str.get(..cut_len) {
                                    stripped = Some(PathBuf::from(prefix));
                                }
                                break;
                            }
                        }
                        match stripped {
                            Some(s_path) => s_path,
                            None => PathBuf::from(format!("{}.decompressed", input_path.display())),
                        }
                    }
                }
            };

            if target_path.as_path() == std::path::Path::new("-") {
                Ok(ToolResult::immediate_ok(decompressed))
            } else {
                if !check_overwrite_prompt(&target_path, *force)? {
                    return Ok(ToolResult::immediate_err(
                        "Operation cancelled.\n".as_bytes().to_vec(),
                        1,
                    ));
                }
                std::fs::write(&target_path, &decompressed)
                    .with_context(|| format!("Failed to write to {}", target_path.display()))?;
                Ok(ToolResult::immediate_ok(Vec::new()))
            }
        }
        Command::Wfparser { file } => {
            let data = read_file_or_stdin(file.as_path())?;
            let output = ctb_formats_wfscan::wfparse(&data)?;
            Ok(ToolResult::immediate_ok(output))
        }
        Command::Wfscan { file } => {
            let data = read_file_or_stdin(file.as_path())?;
            let output = ctb_formats_wfscan::wfscan(&data)?;
            Ok(ToolResult::immediate_ok(output))
        }
        Command::DceutilsPhpToCsv { php_file } => {
            ctb_formats_dceutils::to_csv::php_file_to_csv_files(php_file)?;
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::Help => Ok(ToolResult::immediate_ok(generate_help_bytes())),
        Command::ShowNode { .. } => Err(anyhow!(
            "ShowNode should not be run as lightweight command; it needs the full environment"
        )),
        Command::DevSign {
            input_dir,
            output_dir,
            platform,
        } => {
            let summary = ctb_installer::dev_sign::run_dev_sign(
                input_dir.as_deref(),
                output_dir.as_deref(),
                platform.as_deref(),
            )?;
            println!("{summary}");
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::DevGzSha256 { path } => {
            let sha = ctb_installer::dev_sign::compress_gz_sha256(path)?;
            println!("{sha}");
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::DevKeyCreate { write } => {
            use crate::utilities::pc_settings::PcSettings;
            use ctb_installer::signing::{
                KeyId, generate_keypair, private_key_to_base64,
                public_key_to_base64,
            };

            let (private_key, public_key) = generate_keypair();
            let private_b64 = private_key_to_base64(&private_key);
            let public_b64 = public_key_to_base64(&public_key);
            let key_id = KeyId::from_public_key(&public_key);

            if *write {
                PcSettings::apply_patch(PcSettings {
                    dev_signing_private_key: MaybeOption::Value(
                        private_b64.clone(),
                    ),
                    dev_signing_public_key: MaybeOption::Value(
                        public_b64.clone(),
                    ),
                    ..Default::default()
                })?;
            }

            println!("Signing keypair generated");
            println!("Key ID: {key_id}");
            println!();
            println!("dev_signing_private_key (base64): {private_b64}");
            println!("dev_signing_public_key (base64):  {public_b64}");
            println!();
            println!(
                "Server `release_public_key` should be set to the public key above."
            );

            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::DevReleaseCheck {
            manifest,
            chunks_dir,
            platform,
        } => {
            let summary = ctb_installer::release_check::run_dev_release_check(
                manifest.as_deref(),
                chunks_dir.as_deref(),
                platform.as_deref(),
            )?;
            println!("{summary}");
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::DevReleaseExpire {
            older_than,
            releases_dir,
        } => {
            let summary = ctb_installer::release_expire::run_release_expire(
                *older_than,
                releases_dir.as_deref(),
            )?;
            println!("{summary}");
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::Install { no_gui, unattended } => {
            // Unattended implies no_gui
            let use_tui = *no_gui || *unattended;

            if use_tui {
                ctb_installer::tui::run_installer(*unattended)?;
            } else {
                ctb_installer::gui::run_installer()?;
            }
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::MakeOfflineInstaller {
            output,
            platform,
            version,
            server_url,
        } => {
            ctb_installer::tarball::download_offline_bundle_to_path(
                output,
                server_url.as_deref(),
                platform.as_deref(),
                version.as_deref(),
            )?;
            println!(
                "Offline installer bundle written to {}",
                output.display()
            );
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::Update {
            unattended,
            server_url,
        } => {
            ctb_installer::install::run_update_check(
                server_url.as_deref(),
                *unattended,
            )?;
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::Uninstall { no_gui, unattended } => {
            let use_tui = *no_gui || *unattended;

            if *unattended {
                // Run unattended uninstall (no prompts)
                ctb_installer::install::run_uninstall_unattended()?;
            } else if use_tui {
                ctb_installer::tui::run_uninstall()?;
            } else {
                ctb_installer::gui::run_uninstall()?;
            }
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::UpgradeCanary {
            backup_path,
            target_path,
            port,
        } => {
            ctb_installer::upgrade::run_canary_validation(
                backup_path,
                target_path,
                *port,
            )?;
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::Jq {
            raw_output,
            query,
            file,
        } => {
            use std::io::Read;

            let input_bytes = if let Some(path) = file {
                read_file_or_stdin(path)?
            } else {
                let mut buf = Vec::new();
                std::io::stdin().read_to_end(&mut buf)?;
                buf
            };
            let input_str = String::from_utf8(input_bytes)
                .context("Input is not valid UTF-8")?;

            let cli = ctb_utilities::json::Cli {
                raw_output: *raw_output,
                compact_output: true,
                ..Default::default()
            };
            let output = ctb_utilities::json::jq_implementation(query, &input_str, cli)?;
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
        Command::JsonEscape { value } => {
            let input_str = if let Some(val) = value {
                val.clone()
            } else {
                use std::io::Read;
                let mut buf = Vec::new();
                std::io::stdin().read_to_end(&mut buf)?;
                String::from_utf8(buf).context("Input is not valid UTF-8")?
            };
            let escaped = ctb_utilities::json::json_escape(&input_str)?;
            Ok(ToolResult::immediate_ok(escaped.into_bytes()))
        }
    }
}

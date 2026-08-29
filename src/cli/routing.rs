// SPDX-License-Identifier: AGPL-3.0-or-later
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

//! Subcommand definitions and argument routing for the CLI.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

// Any nontrivial logic should go in the associated crate, rather than this file.

use anyhow::{Result, anyhow};
use clap::Subcommand;
use ctb_formats_stagel::convert::run_stagel_bootstrap_convert;
use ctb_utilities::json::maybe_value::MaybeOption;
use std::path::PathBuf;

use crate::base_conversion::{BaseArgs, run_base2base, run_base_convert};
use crate::utilities::{
    fork, get_this_executable, upgrade_in_place,
    wait_for_ctoolbox_exit_and_clean_up,
};
use crate::{StringInput, ToolResult, generate_help_bytes};

/// Return true if this is a command that can be run without booting.
pub fn is_lightweight_command(command: &str) -> bool {
    matches!(
        command,
        "adduser"
            | "csum"
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
            | "pan2macro"
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
            | "x86-instruction-sets"
            | "range_gen"
            | "range-gen"
            | "character_description"
            | "character-description"
            | "chardesc"
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
        /// Only verify files with source="original".
        #[arg(long)]
        original: bool,
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
        /// Only download files with source="original".
        #[arg(long)]
        original: bool,
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
        /// Only download and verify files with source="original".
        #[arg(long)]
        original: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Register a new non-administrator local user
    #[command(name = "adduser")]
    AddUser {
        /// Username for the new non-administrator account
        username: String,
        /// Read password from stdin even if running interactively
        #[arg(long = "password-stdin")]
        password_stdin: bool,
    },
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
    #[command(
        name = "base2base",
        after_help = "Examples:\n  $ ctoolbox base2base 10 16 \"255 16 10\"\n  ff 10 a\n\n  $ ctoolbox base2base 2 10 \"1101 1010\"\n  13 10\n\n  $ ctoolbox base2base 16 2 --prefix \"0b\" 1f 2a\n  0b11111 0b101010\n\n  $ ctoolbox base2base 10 16 --bytes 255 128\n  ff 80"
    )]
    Base2Base {
        /// All positional arguments for custom parsing
        #[arg(required = true)]
        args: Vec<String>,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Convert from hexadecimal to decimal
    #[command(
        name = "hex2dec",
        after_help = "Examples:\n  $ ctoolbox hex2dec \"1A 2B 3C\"\n  26 43 60\n\n  $ ctoolbox hex2dec \"0x1A 0x2B\"\n  26 43\n\n  $ ctoolbox hex2dec -s \", \" \"FF 80 00\"\n  255, 128, 0"
    )]
    Hex2Dec {
        #[command(flatten)]
        string_input: StringInput,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Convert from decimal to hexadecimal
    #[command(
        name = "dec2hex",
        after_help = "Examples:\n  $ ctoolbox dec2hex \"255 128 64\"\n  ff 80 40\n\n  $ ctoolbox dec2hex --prefix \"0x\" \"10 20 30\"\n  0xa 0x14 0x1e\n\n  $ ctoolbox dec2hex --bytes \"255 16\"\n  ff 10"
    )]
    Dec2Hex {
        #[command(flatten)]
        string_input: StringInput,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Reformat hexdumps
    #[command(
        name = "hexfmt",
        after_help = "Examples:\n  $ ctoolbox hexfmt \"1a2b3c4d\"\n  1a2b3c4d\n\n  $ ctoolbox hexfmt -s \" \" \"1a 2b 3c 4d\"\n  1a 2b 3c 4d\n\n  $ ctoolbox hexfmt --prefix \"0x\" \"de ad be ef\"\n  0xde 0xad 0xbe 0xef"
    )]
    Hexfmt {
        #[command(flatten)]
        string_input: StringInput,
        #[command(flatten)]
        base_args: BaseArgs,
    },
    /// Convert a hexadecimal string to binary data
    #[command(
        name = "hex2bin",
        after_help = "Examples:\n  $ ctoolbox hex2bin \"48656c6c6f\"\n  Hello\n\n  $ echo \"48 65 6c 6c 6f\" | ctoolbox hex2bin\n  Hello\n\n  $ ctoolbox hex2bin -f file.hex -o file.bin\n  $ ctoolbox hex2bin \"48656c6c6f\" > output.bin"
    )]
    Hex2Bin(ctb_formats_hexdump::cli::Hex2BinArgs),
    /// Convert binary data to a hexadecimal string or hex dump
    #[command(
        name = "bin2hex",
        after_help = "Examples:\n  $ ctoolbox bin2hex \"Hello\"\n  48656c6c6f\n\n  $ echo -n \"Hello\" | ctoolbox bin2hex\n  48656c6c6f\n\n  $ cat file.exe | ctoolbox bin2hex\n  4d5a...\n\n  $ ctoolbox bin2hex -f file.bin -o file.hex\n  $ ctoolbox bin2hex --hd -f file.bin\n  $ ctoolbox bin2hex --hf \"Hello\""
    )]
    Bin2Hex(ctb_formats_hexdump::cli::Bin2HexArgs),
    /// Generate a range of numbers in various bases
    #[command(
        name = "range_gen",
        alias = "range-gen",
        after_help = "Examples:\n  $ ctoolbox range_gen 1 10\n  1\n  2\n  3\n  4\n  5\n  6\n  7\n  8\n  9\n  10\n\n  $ ctoolbox range_gen -s 2 1 10\n  1\n  3\n  5\n  7\n  9\n\n  $ ctoolbox range_gen -b 16 -t -S, 18D0C 18D12\n  18D0C,18D0D,18D0E,18D0F,18D10,18D11,18D12,\n\n  $ ctoolbox range_gen -b hex 0x00 0x10\n  00\n  01\n  02\n  03\n  04\n  05\n  06\n  07\n  08\n  09\n  0A\n  0B\n  0C\n  0D\n  0E\n  0F\n  10"
    )]
    RangeGen(ctb_formats_math::range_generator::RangeGenArgs),
    /// Describe Unicode characters with annotations, aliases, and meanings
    #[command(
        name = "character_description",
        alias = "character-description",
        alias = "chardesc",
        after_help = "Examples:\n  $ ctoolbox character_description \"Hello\"\n  $ ctoolbox character_description -f input.txt -o output.txt\n  $ ctoolbox character_description --codepoint U+1F602\n  $ ctoolbox character_description --wuc-compat \"Hello\""
    )]
    CharacterDescription(
        ctb_formats_unicode::cli::CharacterDescriptionArgs,
    ),
    /// Generate GDB instructions from symbols
    #[command(name = "gdb_instructions_generate")]
    GdbInstructionsGenerate {},
    /// Analyze an x86/x64 object or archive file and list CPU instruction set features
    #[command(name = "x86-instruction-sets")]
    X86InstructionSets {
        /// Path to the object or archive file
        path: PathBuf,
    },
    /// Internet Archive utilities.
    #[command(name = "ia")]
    IA {
        #[command(subcommand)]
        command: IACommand,
    },
    /// Convert a .pan file to CSV output
    #[command(name = "pan2csv")]
    Pan2Csv {
        /// Format data using Panorama output patterns (also strips subsequent lines of text fields by default)
        #[arg(long, short = 'p')]
        patterns: bool,
        /// When formatting with patterns, keep full multiline text strings instead of truncating at first newline
        #[arg(long = "keep-multiline")]
        keep_multiline: bool,
        /// Include CSV header row with field names (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        header: bool,
        /// Do not include CSV header row (alias for --header=false)
        #[arg(long = "no-header")]
        no_header: bool,
        /// Export delimiter / format: commas (default), tabs, tabs-no-quotes, wordperfect
        #[arg(long, short = 'd', default_value = "commas")]
        delimiter: String,
        /// Output character encoding (utf8, mac, windows)
        #[arg(long, default_value = "utf8")]
        encoding: String,
        /// Line terminator: crlf (\r\n) or lf (\n)
        #[arg(long)]
        crlf: bool,
        /// Replicate export inconsistencies and legacy double-encoding behavior
        #[arg(long = "replicate-double-encoding")]
        replicate_double_encoding: bool,
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
    /// Extract a macro/procedure from a .pan file
    #[command(name = "pan2macro")]
    Pan2Macro {
        /// Output character encoding (mac, windows, utf8)
        #[arg(long, default_value = "windows")]
        encoding: String,
        /// Input PAN file path
        pan_file: PathBuf,
        /// Name of the macro/procedure to extract
        macro_name: String,
    },
    /// Parse a macro/procedure into AST JSON from a .pan file
    #[command(name = "pan2ast")]
    Pan2Ast {
        /// Output character encoding (mac, windows, utf8)
        #[arg(long, default_value = "windows")]
        encoding: String,
        /// Input PAN file path
        pan_file: PathBuf,
        /// Name of the macro/procedure to parse
        macro_name: String,
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
        /// Output directory for the extracted assets. Defaults to the current directory. The bundle name will be appended as a subdirectory, such as extracting to a folder `test` will place the extracted files within `test/bundle_v3-extracted/`.
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
    #[command(name = "compress", after_help = ctb_formats_compression::COMPRESSION_AFTER_HELP.as_str())]
    Compress(ctb_formats_compression::cli::CliCompressArgs),
    /// Decompress a compressed file or stdin
    #[command(name = "decompress", after_help = ctb_formats_compression::COMPRESSION_AFTER_HELP.as_str())]
    Decompress {
        /// Optional compression format (e.g. `br`, `gz`, `deflate`, `zlib`). If omitted, detected from file extension or magic bytes.
        format: Option<String>,
        /// Input file path (or - for stdin)
        #[arg(default_value = "-")]
        file: PathBuf,
        /// Output file path or - for stdout. Defaults to stdout when input is stdin, or strips the compression extension (or appends `.decompressed` if none recognized) when decompressing a file.
        #[arg(
            short = 'o',
            long = "output",
            alias = "file",
            value_name = "OUTPUT"
        )]
        output: Option<PathBuf>,
        /// Force overwrite of existing destination file without confirmation
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
        #[expect(
            clippy::doc_markdown,
            reason = "doc comment references local path conventions"
        )]
        #[arg(long)]
        input_dir: Option<PathBuf>,
        /// Directory to write signed chunks and manifest.
        /// Defaults to ~/ctb_release/releases if not specified.
        #[expect(
            clippy::doc_markdown,
            reason = "doc comment references local path conventions"
        )]
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
        std::io::stdin()
            .lock()
            .read_to_end(&mut buf)
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

#[expect(
    clippy::too_many_lines,
    reason = "uniform tool command router is naturally long"
)]
pub async fn run_lightweight_command(cmd: &Command) -> Result<ToolResult> {
    match cmd {
        Command::AddUser {
            username,
            password_stdin,
        } => {
            use std::io::IsTerminal;
            use std::io::Write;

            let password_str =
                if !password_stdin && std::io::stdin().is_terminal() {
                    print!("Enter password for '{username}': ");
                    std::io::stdout().flush()?;
                    let mut p1 = String::new();
                    std::io::stdin().read_line(&mut p1)?;
                    let p1 = p1.trim_end_matches(['\r', '\n']).to_string();

                    print!("Confirm password: ");
                    std::io::stdout().flush()?;
                    let mut p2 = String::new();
                    std::io::stdin().read_line(&mut p2)?;
                    let p2 = p2.trim_end_matches(['\r', '\n']).to_string();

                    if p1 != p2 {
                        bail!("Passwords do not match");
                    }
                    p1
                } else {
                    let mut p = String::new();
                    std::io::stdin().read_line(&mut p)?;
                    p.trim_end_matches(['\r', '\n']).to_string()
                };

            if password_str.is_empty() {
                bail!("Password cannot be empty");
            }

            let password =
                ctb_utilities::password::Password::from_string(&password_str);
            let user =
                ctb_storage::user::add_non_admin_user(username, &password)?;

            println!(
                "User '{name}' registered successfully with ID {id}.",
                name = user.name(),
                id = user.local_id()
            );
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::WaitShutdown { pid } => {
            wait_for_ctoolbox_exit_and_clean_up(*pid);
            Ok(ToolResult::immediate_ok(Vec::new()))
        }
        Command::WaitRestart { pid, port } => {
            wait_for_ctoolbox_exit_and_clean_up(*pid);
            fork(
                &get_this_executable()?,
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
            run_base2base(args, base_args)
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
        Command::Hex2Bin(args) => {
            let out = ctb_formats_hexdump::cli::execute_cli_hex2bin(
                args.clone(),
                read_file_or_stdin,
            )?;
            let bytes = match out {
                Some(b) => b,
                None => Vec::new(),
            };
            Ok(ToolResult::immediate_ok(bytes))
        }
        Command::Bin2Hex(args) => {
            let out = ctb_formats_hexdump::cli::execute_cli_bin2hex(
                args.clone(),
                read_file_or_stdin,
            )?;
            let bytes = match out {
                Some(b) => b,
                None => Vec::new(),
            };
            Ok(ToolResult::immediate_ok(bytes))
        }
        Command::RangeGen(args) => {
            let output = ctb_formats_math::range_generator::range_cli_handler(args)?;
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
        Command::CharacterDescription(args) => {
            let out = ctb_formats_unicode::cli::execute_cli_character_description(
                args.clone(),
                read_file_or_stdin,
            )?;
            let bytes = match out {
                Some(b) => b,
                None => Vec::new(),
            };
            Ok(ToolResult::immediate_ok(bytes))
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
                original,
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
                        *original,
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
            IACommand::Download {
                target,
                output_dir,
                original,
            } => {
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::download(
                        target,
                        output_dir.as_deref(),
                        *original,
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
            IACommand::Checkeddl {
                target,
                output_dir,
                original,
            } => {
                Ok(ToolResult::immediate_ok(
                    ctb_formats_internetarchive::checkeddl(
                        target,
                        output_dir.as_deref(),
                        *original,
                    )?,
                ))
            }
        },
        Command::Pan2Csv {
            patterns,
            keep_multiline,
            header,
            no_header,
            delimiter,
            encoding,
            crlf,
            replicate_double_encoding,
            pan_file,
        } => {
            let data = read_file_or_stdin(pan_file.as_path())?;
            let include_header = if *no_header { false } else { *header };
            let enc = match encoding.to_ascii_lowercase().as_str() {
                "mac" | "macroman" | "mac-roman" | "macintosh" => {
                    ctb_formats_pan::output::PanCsvEncoding::MacRoman
                }
                "win" | "windows" | "win1252" | "windows-1252" | "panwindows" => {
                    ctb_formats_pan::output::PanCsvEncoding::Windows
                }
                "utf8-windows" | "windows-utf8" | "utf8-win" | "win-utf8" => {
                    ctb_formats_pan::output::PanCsvEncoding::Utf8Windows
                }
                _ => ctb_formats_pan::output::PanCsvEncoding::Utf8,
            };
            let delim = match delimiter.to_ascii_lowercase().replace('_', "-").as_str() {
                "tab" | "tabs" | "tsv" => {
                    ctb_formats_pan::output::PanExportDelimiter::Tabs
                }
                "tab-no-quotes"
                | "tabs-no-quotes"
                | "tab-without-quotes"
                | "tabs-without-quotes"
                | "tabs-w/o-quotes"
                | "tsv-no-quotes" => {
                    ctb_formats_pan::output::PanExportDelimiter::TabsWithoutQuotes
                }
                "wordperfect" | "wp" => {
                    ctb_formats_pan::output::PanExportDelimiter::WordPerfect
                }
                _ => ctb_formats_pan::output::PanExportDelimiter::Commas,
            };
            let opts = ctb_formats_pan::output::PanCsvOptions {
                output_patterns: *patterns,
                truncate_multiline: !*keep_multiline,
                include_header,
                encoding: enc,
                delimiter: delim,
                crlf: *crlf
                    || (delim != ctb_formats_pan::output::PanExportDelimiter::WordPerfect
                        && enc == ctb_formats_pan::output::PanCsvEncoding::Windows),
                replicate_double_encoding: *replicate_double_encoding,
            };
            let output =
                ctb_formats_pan::output::pan_to_csv_with_options(&data, &opts)?;
            Ok(ToolResult::immediate_ok(output))
        }
        Command::StagelBootstrapParse { input_file } => {
            let data = std::fs::read(input_file).with_context(|| {
                format!("Failed to read input file: {input_file:?}")
            })?;
            let filename = input_file
                .file_stem()
                .ok_or_else(|| anyhow::anyhow!("Input file path {:?} has no valid file stem", input_file))?
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
                input_file,
                target_lang,
            )?;
            Ok(ToolResult::immediate_ok(output))
        }
        Command::Pan2ParseJson { pan_file } => {
            let data = read_file_or_stdin(pan_file.as_path())?;
            let output =
                ctb_formats_pan::output::pan_to_parse_json(&data)?.into_bytes();
            Ok(ToolResult::immediate_ok(output))
        }
        Command::Pan2Macro {
            encoding,
            pan_file,
            macro_name,
        } => {
            let data = read_file_or_stdin(pan_file.as_path())?;
            let enc = match encoding.to_ascii_lowercase().as_str() {
                "mac" | "macroman" | "mac-roman" | "macintosh" => {
                    ctb_formats_pan::output::PanCsvEncoding::MacRoman
                }
                "win" | "windows" | "win1252" | "windows-1252" | "panwindows" => {
                    ctb_formats_pan::output::PanCsvEncoding::Windows
                }
                "utf8-windows" | "windows-utf8" | "utf8-win" | "win-utf8" => {
                    ctb_formats_pan::output::PanCsvEncoding::Utf8Windows
                }
                _ => ctb_formats_pan::output::PanCsvEncoding::Windows,
            };
            let output = ctb_formats_pan::output::pan_to_macro_with_encoding(
                &data,
                &macro_name,
                enc,
            )?
            .into_bytes();
            Ok(ToolResult::immediate_ok(output))
        }
        Command::Pan2Ast {
            encoding,
            pan_file,
            macro_name,
        } => {
            let data = read_file_or_stdin(pan_file.as_path())?;
            let enc = match encoding.to_ascii_lowercase().as_str() {
                "mac" | "macroman" | "mac-roman" | "macintosh" => {
                    ctb_formats_pan::output::PanCsvEncoding::MacRoman
                }
                "win" | "windows" | "win1252" | "windows-1252" | "panwindows" => {
                    ctb_formats_pan::output::PanCsvEncoding::Windows
                }
                "utf8-windows" | "windows-utf8" | "utf8-win" | "win-utf8" => {
                    ctb_formats_pan::output::PanCsvEncoding::Utf8Windows
                }
                _ => ctb_formats_pan::output::PanCsvEncoding::Windows,
            };
            let output = ctb_formats_pan::output::pan_to_ast_with_encoding(
                &data,
                &macro_name,
                enc,
            )?
            .into_bytes();
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
                .context("Failed to get current directory")?;
            let cwd = cwd.as_path();
            // Reason for fallback: when optional output_dir CLI parameter is omitted, extraction defaults to current working directory (cwd).
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
            exit_code: ctb_formats_javascript::js_test::run_test_args(args)?,
        }),
        Command::Csum {
            algo,
            file,
            prefix_0x,
        } => {
            let data = read_file_or_stdin(file.as_path())?;
            let hash_algo =
                ctb_formats_checksum::HashAlgorithm::try_from(algo.as_str())?;
            let output = format!(
                "{}\n",
                ctb_formats_checksum::hash_hex(&data, hash_algo, *prefix_0x)
            );
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
        Command::Compress(args) => {
            let cli_output =
                ctb_formats_compression::cli::execute_cli_compress(
                    args.clone(),
                    read_file_or_stdin,
                    check_overwrite_prompt,
                )?;
            match cli_output {
                ctb_formats_compression::cli::CliCompressionOutput::Stdout(bytes) => {
                    Ok(ToolResult::immediate_ok(bytes))
                }
                ctb_formats_compression::cli::CliCompressionOutput::FileWritten(_) => {
                    Ok(ToolResult::immediate_ok(Vec::new()))
                }
                ctb_formats_compression::cli::CliCompressionOutput::Cancelled => {
                    Ok(ToolResult::immediate_err(
                        "Operation cancelled.\n".as_bytes().to_vec(),
                        1,
                    ))
                }
            }
        }
        Command::Decompress {
            format,
            file,
            output,
            force,
        } => {
            let cli_output =
                ctb_formats_compression::cli::execute_cli_decompress(
                    ctb_formats_compression::cli::CliDecompressArgs {
                        format: format.clone(),
                        input_path: file.clone(),
                        output_path: output.clone(),
                        force: *force,
                    },
                    read_file_or_stdin,
                    check_overwrite_prompt,
                )?;
            match cli_output {
                ctb_formats_compression::cli::CliCompressionOutput::Stdout(bytes) => {
                    Ok(ToolResult::immediate_ok(bytes))
                }
                ctb_formats_compression::cli::CliCompressionOutput::FileWritten(_) => {
                    Ok(ToolResult::immediate_ok(Vec::new()))
                }
                ctb_formats_compression::cli::CliCompressionOutput::Cancelled => {
                    Ok(ToolResult::immediate_err(
                        "Operation cancelled.\n".as_bytes().to_vec(),
                        1,
                    ))
                }
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
            let output =
                ctb_utilities::json::jq_implementation(query, &input_str, cli)?;
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
        Command::X86InstructionSets { path } => {
            let data = std::fs::read(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            let sets = ctb_formats_x86::extract_instruction_sets(&data)?;
            let mut output = sets.join("\n");
            if !output.is_empty() {
                output.push('\n');
            }
            Ok(ToolResult::immediate_ok(output.into_bytes()))
        }
    }
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
    use clap::CommandFactory;

    #[crate::ctb_test]
    fn test_compress_command_help() {
        let mut cmd = crate::Cli::command();
        let compress_sub = cmd
            .find_subcommand_mut("compress")
            .expect("compress subcommand should exist");
        let mut help_buf = Vec::new();
        compress_sub
            .write_help(&mut help_buf)
            .expect("Should format help");
        let help_str = String::from_utf8(help_buf).expect("UTF-8 help");

        assert!(help_str.contains("Supported compression formats:"));
        assert!(help_str.contains("br, brotli: Brotli compressed stream"));
        assert!(help_str.contains("gz, gzip: GNU gzip format"));
        assert!(help_str.contains("sco, compress-h, compress-sco, sco-compress: `compress`: SCO `compress -H` format"));
    }

    #[crate::ctb_test("tokio")]
    async fn test_x86_instruction_sets_command() {
        assert!(is_lightweight_command("x86-instruction-sets"));
        let temp_dir = tempfile::tempdir().unwrap();
        let sample_file = temp_dir.path().join("dummy.bin");
        std::fs::write(&sample_file, b"not a binary").unwrap();

        let cmd = Command::X86InstructionSets {
            path: sample_file,
        };
        assert!(run_lightweight_command(&cmd).await.is_err());
    }

    #[crate::ctb_test("tokio")]
    async fn test_adduser_command() -> Result<()> {
        assert!(is_lightweight_command("adduser"));

        let username = format!("cli_user_{}", function_name!());
        ctb_storage::user::User::delete_by_name(&username).ok();

        // Ensure allow_local_account_creation is true for this test
        let mut settings =
            ctb_utilities::pc_settings::PcSettings::load().unwrap_or_default();
        settings.allow_local_account_creation =
            ctb_utilities::json::maybe_value::MaybeValue::Value(true);
        settings.save()?;

        let _cmd = Command::AddUser {
            username: username.clone(),
            password_stdin: true,
        };

        // Note: Password comes from stdin or add_non_admin_user directly.
        // Let's test calling add_non_admin_user
        let password =
            ctb_utilities::password::Password::from_string(
                ctb_utilities::password::TEST_USER_PASS,
            );
        let user = ctb_storage::user::add_non_admin_user(&username, &password)?;
        assert_eq!(user.name(), username);
        assert!(!user.is_admin());

        user.delete()?;
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn test_base2base_positional_args() -> Result<()> {
        let cmd = Command::Base2Base {
            args: vec![
                "16".to_string(),
                "2".to_string(),
                "1f".to_string(),
                "2a".to_string(),
            ],
            base_args: BaseArgs {
                bytes: false,
                no_pad: false,
                prefix: "0b".to_string(),
                separator: " ".to_string(),
                lowercase: true,
                filter_chars: true,
                collapse_filtered: false,
                collapse_only: Vec::new(),
                parse_prefixes: true,
                limit: 0,
                pad: false,
                pad_l: 1,
                quiet: false,
            },
        };
        let res = run_lightweight_command(&cmd).await?;
        match res {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                let output = String::from_utf8(stdout)?;
                assert_eq!(output.trim(), "0b11111 0b101010");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
        Ok(())
    }
}

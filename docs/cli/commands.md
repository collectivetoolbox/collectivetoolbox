# Collective Toolbox CLI Reference

This document is automatically generated from the `ctoolbox` CLI command definitions.

## Overview

```text
Collective Toolbox

Usage: ctoolbox [OPTIONS] [COMMAND]

Commands:
  adduser                    Register a new non-administrator local user
  waitshutdown               Wait for the provided PID to shutdown, then clean up
  waitrestart                Wait for the provided PID to shutdown, then start ctoolbox with the given port
  waitupgrade                Wait for the provided PID to shutdown, then upgrade the ctoolbox instance in place, then restart it. (Will need to copy the executable before upgrading it probably because Windows locks running executables.)
  base2base                  Convert from one base to another (for base <= 36)
  hex2dec                    Convert from hexadecimal to decimal
  dec2hex                    Convert from decimal to hexadecimal
  hexfmt                     Reformat hexdumps
  hex2bin                    Convert a hexadecimal string to binary data
  bin2hex                    Convert binary data to a hexadecimal string or hex dump
  range_gen                  Generate a range of numbers in various bases
  gdb_instructions_generate  Generate GDB instructions from symbols
  x86-instruction-sets       Analyze an x86/x64 object or archive file and list CPU instruction set features
  ia                         Internet Archive utilities
  pan2csv                    Convert a .pan file to CSV output
  stagel-bootstrap-parse     Parse a StageL file to token output
  stagel-bootstrap-convert   Translate a StageL file using the bootstrap compiler
  pan2parsejson              Convert a .pan file to JSON of parse
  pdf2txt                    Convert a PDF file to text output
  pdf2json                   Convert a PDF file to JSON output
  pdf2md                     Convert a PDF file to Markdown output
  warcat                     WARC archiving tool
  ctb-asset-bundle-extract   Extract a ctoolbox asset bundle to a directory tree
  js-lint                    Lint JavaScript and TypeScript sources
  ts-check                   Type-check TypeScript code
  js-test                    Run JavaScript tests
  csum                       Calculate checksum for a file or stdin
  compress                   Compress a file or stdin using single-stream compression format
  decompress                 Decompress a compressed file or stdin
  wfparser                   Process a file using wfparser logic
  wfscan                     Process a file using wfscan logic
  dceutils_php_to_csv        Convert PHP data file arrays to CSV files
  help                       Show help
  show-node                  Unimplemented, just an example
  ctb-dev-sign               Sign release artifacts for distribution (developer command)
  ctb-dev-gz-sha256          Compress a tar file with the server's gzip settings and print its SHA256 hex (developer command)
  ctb-dev-key-create         Generate a signing keypair for `ctb-dev-sign`
  ctb-dev-release-check      Verify an uploaded release: check signature and chunk hashes
  ctb-dev-release-expire     Delete old release chunks and manifests to reclaim disk space
  install                    Launch the installer GUI (or TUI if --no-gui)
  make-offline-installer     Download and package the latest offline installer bundle
  update                     Check for updates and optionally apply them
  uninstall                  Uninstall ctoolbox from this system
  ctb-upgrade-canary         Internal: Post-upgrade canary validation
  jq                         Lightweight JQ implementation
  json-escape                Escape a string to be a valid JSON string value (enclosed in double quotes)

Options:
      --ctoolbox-ipc-port <CTOOLBOX_IPC_PORT>  
      --no-update                              Skip automatic update checks on startup
      --use-bundled-tls-validator              Use bundled certificate roots for this run only
      --use-system-tls-validator               Use the system certificate store for this run only
  -h, --help                                   Print help
  -V, --version                                Print version
```

## Subcommands

### `ctoolbox adduser`

```text
Register a new non-administrator local user

Usage: ctoolbox adduser [OPTIONS] <USERNAME>

Arguments:
  <USERNAME>  Username for the new non-administrator account

Options:
      --password-stdin  Read password from stdin even if running interactively
  -h, --help            Print help
```

### `ctoolbox base2base`

```text
Convert from one base to another (for base <= 36)

Usage: ctoolbox base2base [OPTIONS] <ARGS>...

Arguments:
  <ARGS>...  All positional arguments for custom parsing

Options:
  -b, --bytes                          Shortcut for -n -q --limit 255 --pad
      --no-pad                         Invalid unless using --bytes option. Turns off padding
      --prefix <PREFIX>                Add prefix to each output number (e.g. 0x) [default: ""]
  -s, --separator <SEPARATOR>          Separator inserted after numeric output values (except the last one) [default: " "]
      --lowercase                      Output numbers in base 11+ using lowercase letters, rather than the default of uppercase. Does not change the case of input characters that are not parts of numbers
  -f, --filter-chars                   Whether to filter out bytes that aren't digits in the input base
  -c, --collapse-filtered              Should filtered characters be totally ignored for parsing numbers? E.g. `10_000` would get the _ filtered out and be treated as 10000
      --collapse-only <COLLAPSE_ONLY>  A list of filtered characters to collapse, leaving others as spaces [default: []]
      --parse-prefixes                 Whether to interpret existing prefixes (e.g. 0x) in the input. If set to false, it may produce silly results in some cases, like when converting hex with 0x prefixes to another base. If you also ask it to add prefixes, you'll get three prefixes for each number! (Because it will take 0 as a number, then pass through x, then take the actual number.)
  -l, --limit <LIMIT>                  Limit width for each number. Input numbers will be split up if longer than this value (0x0404 would be read as 0x04 04). The value of this argument should be the maximum value that you need to represent, and the width in bytes will be derived from that dependent on the base. Set to 0 to disable limiting [default: 0]
  -p, --pad                            Zero-pad the left of each number to the number of digits determined by the limit argument. Requires a limit to be set
  -P, --pad-l <PAD_L>                  Zero-pad the left of each number to at least this many digits. Set to 0 or 1 to turn off [default: 1]
  -q, --quiet                          Suppress warning messages
  -h, --help                           Print help (see more with '--help')

Examples:
  $ ctoolbox base2base 10 16 "255 16 10"
  ff 10 a

  $ ctoolbox base2base 2 10 "1101 1010"
  13 10

  $ ctoolbox base2base 16 2 --prefix "0b" 1f 2a
  0b11111 0b101010

  $ ctoolbox base2base 10 16 --bytes 255 128
  ff 80
```

### `ctoolbox bin2hex`

```text
Convert binary data to a hexadecimal string or hex dump

Usage: ctoolbox bin2hex [OPTIONS] [VALUE]

Arguments:
  [VALUE]  Data to convert. If not provided, reads from stdin or file

Options:
  -f, --file <FILE>      Input file path (or - for stdin)
  -o, --output <OUTPUT>  Output file path (or - for stdout)
      --hd               Output in classic hex dump format
      --hf               Output in fancy hex dump format
  -h, --help             Print help

Examples:
  $ ctoolbox bin2hex "Hello"
  48656c6c6f

  $ echo -n "Hello" | ctoolbox bin2hex
  48656c6c6f

  $ cat file.exe | ctoolbox bin2hex
  4d5a...

  $ ctoolbox bin2hex -f file.bin -o file.hex
  $ ctoolbox bin2hex --hd -f file.bin
  $ ctoolbox bin2hex --hf "Hello"
```

### `ctoolbox compress`

```text
Compress a file or stdin using single-stream compression format

Usage: ctoolbox compress [OPTIONS] <FORMAT> [FILE]

Arguments:
  <FORMAT>  Compression format (e.g. `br`, `gz`, `deflate`, `zlib`). See table below for allowed values
  [FILE]    Input file path (or - for stdin) [default: -]

Options:
  -o, --output <OUTPUT>  Output file path or - for stdout
      --force            Force overwrite without prompting
  -h, --help             Print help

Supported compression formats:
  br, brotli: Brotli compressed stream (RFC 9841)
  gz, gzip: GNU gzip format (RFC 1952)
  deflate, raw-deflate: Raw DEFLATE compressed stream (RFC 1951)
  zl, zz, zlib, zlib-deflate: Zlib-wrapped DEFLATE stream (RFC 1950)
  bz, bz2, bzip2: Bzip2 compressed stream
  compress, compress3, compress4, compress-3.0, compress-4.0: `compress` format, modern LZW block format
  sco, compress-h, compress-sco, sco-compress: `compress`: SCO `compress -H` format
  compress2, compress-2.0: `compress` 2.0 (LZW non-block format)
  compress16, compress1.6, compress-1.6, lzw-sorted-chain: `compress` 1.6 (LZW sorted chain format)
  compress1, compress-1.0: `compress` 1.0 (LZW headerless format)
  pack: `pack` format, common version (Huffman)
  opack, oldpack, old-pack, pts-opack, early-pack: `pack` format, early PDP-11 Unix binary tree
  compact, uncompact: `compact` (McMaster Adaptive Huffman)
  lz4: LZ4 compression
  lzma: LZMA compression
  lzma2: LZMA2 compression
  lz, lzip: Lzip compression
  xz, xzip: XZ compression
  zst, zstd: Zstandard compression
  lzo: LZO compression
```

### `ctoolbox csum`

```text
Calculate checksum for a file or stdin

Usage: ctoolbox csum [OPTIONS] <ALGO> [FILE]

Arguments:
  <ALGO>  Hash algorithm type (`xxhash32`, `xxhash64`, `xxhash3_64`, `xxhash3_128`)
  [FILE]  Input file path (or - for stdin) [default: -]

Options:
      --prefix-0x  Prefix the output hex string with 0x
  -h, --help       Print help
```

### `ctoolbox ctb-asset-bundle-extract`

```text
Extract a ctoolbox asset bundle to a directory tree

Usage: ctoolbox ctb-asset-bundle-extract <BUNDLE_PATH> [OUTPUT_DIR]

Arguments:
  <BUNDLE_PATH>  Path to the `.rsrc` asset bundle
  [OUTPUT_DIR]   Output directory for the extracted assets. Defaults to the current directory. The bundle name will be appended as a subdirectory, such as extracting to a folder `test` will place the extracted files within `test/bundle_v3-extracted/`

Options:
  -h, --help  Print help
```

### `ctoolbox ctb-dev-gz-sha256`

```text
Compress a tar file with the server's gzip settings and print its SHA256 hex (developer command)

Usage: ctoolbox ctb-dev-gz-sha256 <PATH>

Arguments:
  <PATH>  Path to the tar file

Options:
  -h, --help  Print help
```

### `ctoolbox ctb-dev-key-create`

```text
Generate a signing keypair for `ctb-dev-sign`

Usage: ctoolbox ctb-dev-key-create [OPTIONS]

Options:
      --write  Write keys into local `pc_settings.json`
  -h, --help   Print help (see more with '--help')
```

### `ctoolbox ctb-dev-release-check`

```text
Verify an uploaded release: check signature and chunk hashes

Usage: ctoolbox ctb-dev-release-check [OPTIONS]

Options:
      --manifest <MANIFEST>      Path to the manifest file to verify. Defaults to `{storage_dir}/releases/ctb-{platform}-latest.json` if not specified
      --chunks-dir <CHUNKS_DIR>  Path to the chunks directory. Defaults to `{storage_dir}/releases/bh/` if not specified
      --platform <PLATFORM>      Target to verify (e.g. linux-x64, linux-x86). Defaults to the current platform
  -h, --help                     Print help
```

### `ctoolbox ctb-dev-release-expire`

```text
Delete old release chunks and manifests to reclaim disk space

Usage: ctoolbox ctb-dev-release-expire [OPTIONS]

Options:
      --older-than <OLDER_THAN>      Only expire chunks from releases older than this many days. Defaults to 30 days if not specified [default: 30]
      --releases-dir <RELEASES_DIR>  Path to the releases directory. Defaults to `{storage_dir}/releases/` if not specified
  -h, --help                         Print help (see more with '--help')
```

### `ctoolbox ctb-dev-sign`

```text
Sign release artifacts for distribution (developer command)

Usage: ctoolbox ctb-dev-sign [OPTIONS]

Options:
      --input-dir <INPUT_DIR>    Directory containing release artifacts to sign. Defaults to ~/ctb_release/input if not specified
      --output-dir <OUTPUT_DIR>  Directory to write signed chunks and manifest. Defaults to ~/ctb_release/releases if not specified
      --platform <PLATFORM>      Target for this release (e.g. linux-x64, linux-x86, windows-x64, mac-x64, mac-arm64). Defaults to current platform if not specified
  -h, --help                     Print help
```

### `ctoolbox ctb-upgrade-canary`

```text
Internal: Post-upgrade canary validation

Usage: ctoolbox ctb-upgrade-canary [OPTIONS] --backup-path <BACKUP_PATH> --target-path <TARGET_PATH>

Options:
      --backup-path <BACKUP_PATH>  Path to the backup copy of the previous binary
      --target-path <TARGET_PATH>  Path to the installed binary location
      --port <PORT>                Optional port to restart ctoolbox with after successful validation
  -h, --help                       Print help (see more with '--help')
```

### `ctoolbox dceutils_php_to_csv`

```text
Convert PHP data file arrays to CSV files

Usage: ctoolbox dceutils_php_to_csv <PHP_FILE>

Arguments:
  <PHP_FILE>  Path to the PHP data file

Options:
  -h, --help  Print help
```

### `ctoolbox dec2hex`

```text
Convert from decimal to hexadecimal

Usage: ctoolbox dec2hex [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input number or string

Options:
  -b, --bytes                          Shortcut for -n -q --limit 255 --pad
      --no-pad                         Invalid unless using --bytes option. Turns off padding
      --prefix <PREFIX>                Add prefix to each output number (e.g. 0x) [default: ""]
  -s, --separator <SEPARATOR>          Separator inserted after numeric output values (except the last one) [default: " "]
      --lowercase                      Output numbers in base 11+ using lowercase letters, rather than the default of uppercase. Does not change the case of input characters that are not parts of numbers
  -f, --filter-chars                   Whether to filter out bytes that aren't digits in the input base
  -c, --collapse-filtered              Should filtered characters be totally ignored for parsing numbers? E.g. `10_000` would get the _ filtered out and be treated as 10000
      --collapse-only <COLLAPSE_ONLY>  A list of filtered characters to collapse, leaving others as spaces [default: []]
      --parse-prefixes                 Whether to interpret existing prefixes (e.g. 0x) in the input. If set to false, it may produce silly results in some cases, like when converting hex with 0x prefixes to another base. If you also ask it to add prefixes, you'll get three prefixes for each number! (Because it will take 0 as a number, then pass through x, then take the actual number.)
  -l, --limit <LIMIT>                  Limit width for each number. Input numbers will be split up if longer than this value (0x0404 would be read as 0x04 04). The value of this argument should be the maximum value that you need to represent, and the width in bytes will be derived from that dependent on the base. Set to 0 to disable limiting [default: 0]
  -p, --pad                            Zero-pad the left of each number to the number of digits determined by the limit argument. Requires a limit to be set
  -P, --pad-l <PAD_L>                  Zero-pad the left of each number to at least this many digits. Set to 0 or 1 to turn off [default: 1]
  -q, --quiet                          Suppress warning messages
  -h, --help                           Print help (see more with '--help')

Examples:
  $ ctoolbox dec2hex "255 128 64"
  ff 80 40

  $ ctoolbox dec2hex --prefix "0x" "10 20 30"
  0xa 0x14 0x1e

  $ ctoolbox dec2hex --bytes "255 16"
  ff 10
```

### `ctoolbox decompress`

```text
Decompress a compressed file or stdin

Usage: ctoolbox decompress [OPTIONS] [FORMAT] [FILE]

Arguments:
  [FORMAT]  Optional compression format (e.g. `br`, `gz`, `deflate`, `zlib`). If omitted, detected from file extension or magic bytes
  [FILE]    Input file path (or - for stdin) [default: -]

Options:
  -o, --output <OUTPUT>  Output file path or - for stdout
      --force            Force overwrite without prompting
  -h, --help             Print help

Supported compression formats:
  br, brotli: Brotli compressed stream (RFC 9841)
  gz, gzip: GNU gzip format (RFC 1952)
  deflate, raw-deflate: Raw DEFLATE compressed stream (RFC 1951)
  zl, zz, zlib, zlib-deflate: Zlib-wrapped DEFLATE stream (RFC 1950)
  bz, bz2, bzip2: Bzip2 compressed stream
  compress, compress3, compress4, compress-3.0, compress-4.0: `compress` format, modern LZW block format
  sco, compress-h, compress-sco, sco-compress: `compress`: SCO `compress -H` format
  compress2, compress-2.0: `compress` 2.0 (LZW non-block format)
  compress16, compress1.6, compress-1.6, lzw-sorted-chain: `compress` 1.6 (LZW sorted chain format)
  compress1, compress-1.0: `compress` 1.0 (LZW headerless format)
  pack: `pack` format, common version (Huffman)
  opack, oldpack, old-pack, pts-opack, early-pack: `pack` format, early PDP-11 Unix binary tree
  compact, uncompact: `compact` (McMaster Adaptive Huffman)
  lz4: LZ4 compression
  lzma: LZMA compression
  lzma2: LZMA2 compression
  lz, lzip: Lzip compression
  xz, xzip: XZ compression
  zst, zstd: Zstandard compression
  lzo: LZO compression
```

### `ctoolbox gdb_instructions_generate`

```text
Generate GDB instructions from symbols

Usage: ctoolbox gdb_instructions_generate

Options:
  -h, --help  Print help
```

### `ctoolbox hex2bin`

```text
Convert a hexadecimal string to binary data

Usage: ctoolbox hex2bin [OPTIONS] [VALUE]

Arguments:
  [VALUE]  Hexadecimal string. If not provided, reads from stdin or file

Options:
  -f, --file <FILE>      Input file path (or - for stdin)
  -o, --output <OUTPUT>  Output file path (or - for stdout)
  -h, --help             Print help

Examples:
  $ ctoolbox hex2bin "48656c6c6f"
  Hello

  $ echo "48 65 6c 6c 6f" | ctoolbox hex2bin
  Hello

  $ ctoolbox hex2bin -f file.hex -o file.bin
  $ ctoolbox hex2bin "48656c6c6f" > output.bin
```

### `ctoolbox hex2dec`

```text
Convert from hexadecimal to decimal

Usage: ctoolbox hex2dec [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input number or string

Options:
  -b, --bytes                          Shortcut for -n -q --limit 255 --pad
      --no-pad                         Invalid unless using --bytes option. Turns off padding
      --prefix <PREFIX>                Add prefix to each output number (e.g. 0x) [default: ""]
  -s, --separator <SEPARATOR>          Separator inserted after numeric output values (except the last one) [default: " "]
      --lowercase                      Output numbers in base 11+ using lowercase letters, rather than the default of uppercase. Does not change the case of input characters that are not parts of numbers
  -f, --filter-chars                   Whether to filter out bytes that aren't digits in the input base
  -c, --collapse-filtered              Should filtered characters be totally ignored for parsing numbers? E.g. `10_000` would get the _ filtered out and be treated as 10000
      --collapse-only <COLLAPSE_ONLY>  A list of filtered characters to collapse, leaving others as spaces [default: []]
      --parse-prefixes                 Whether to interpret existing prefixes (e.g. 0x) in the input. If set to false, it may produce silly results in some cases, like when converting hex with 0x prefixes to another base. If you also ask it to add prefixes, you'll get three prefixes for each number! (Because it will take 0 as a number, then pass through x, then take the actual number.)
  -l, --limit <LIMIT>                  Limit width for each number. Input numbers will be split up if longer than this value (0x0404 would be read as 0x04 04). The value of this argument should be the maximum value that you need to represent, and the width in bytes will be derived from that dependent on the base. Set to 0 to disable limiting [default: 0]
  -p, --pad                            Zero-pad the left of each number to the number of digits determined by the limit argument. Requires a limit to be set
  -P, --pad-l <PAD_L>                  Zero-pad the left of each number to at least this many digits. Set to 0 or 1 to turn off [default: 1]
  -q, --quiet                          Suppress warning messages
  -h, --help                           Print help (see more with '--help')

Examples:
  $ ctoolbox hex2dec "1A 2B 3C"
  26 43 60

  $ ctoolbox hex2dec "0x1A 0x2B"
  26 43

  $ ctoolbox hex2dec -s ", " "FF 80 00"
  255, 128, 0
```

### `ctoolbox hexfmt`

```text
Reformat hexdumps

Usage: ctoolbox hexfmt [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input number or string

Options:
  -b, --bytes                          Shortcut for -n -q --limit 255 --pad
      --no-pad                         Invalid unless using --bytes option. Turns off padding
      --prefix <PREFIX>                Add prefix to each output number (e.g. 0x) [default: ""]
  -s, --separator <SEPARATOR>          Separator inserted after numeric output values (except the last one) [default: " "]
      --lowercase                      Output numbers in base 11+ using lowercase letters, rather than the default of uppercase. Does not change the case of input characters that are not parts of numbers
  -f, --filter-chars                   Whether to filter out bytes that aren't digits in the input base
  -c, --collapse-filtered              Should filtered characters be totally ignored for parsing numbers? E.g. `10_000` would get the _ filtered out and be treated as 10000
      --collapse-only <COLLAPSE_ONLY>  A list of filtered characters to collapse, leaving others as spaces [default: []]
      --parse-prefixes                 Whether to interpret existing prefixes (e.g. 0x) in the input. If set to false, it may produce silly results in some cases, like when converting hex with 0x prefixes to another base. If you also ask it to add prefixes, you'll get three prefixes for each number! (Because it will take 0 as a number, then pass through x, then take the actual number.)
  -l, --limit <LIMIT>                  Limit width for each number. Input numbers will be split up if longer than this value (0x0404 would be read as 0x04 04). The value of this argument should be the maximum value that you need to represent, and the width in bytes will be derived from that dependent on the base. Set to 0 to disable limiting [default: 0]
  -p, --pad                            Zero-pad the left of each number to the number of digits determined by the limit argument. Requires a limit to be set
  -P, --pad-l <PAD_L>                  Zero-pad the left of each number to at least this many digits. Set to 0 or 1 to turn off [default: 1]
  -q, --quiet                          Suppress warning messages
  -h, --help                           Print help (see more with '--help')

Examples:
  $ ctoolbox hexfmt "1a2b3c4d"
  1a2b3c4d

  $ ctoolbox hexfmt -s " " "1a 2b 3c 4d"
  1a 2b 3c 4d

  $ ctoolbox hexfmt --prefix "0x" "de ad be ef"
  0xde 0xad 0xbe 0xef
```

### `ctoolbox ia`

```text
Internet Archive utilities

Usage: ctoolbox ia <COMMAND>

Commands:
  verify            Verify a local Internet Archive item directory against its files XML
  sha1              Print the expected sha1 for a file in an Internet Archive item
  md5               Print the expected md5 for a file in an Internet Archive item
  contains          Check whether an item contains a particular file
  listplain         List item files one per line
  metadata          Fetch live item metadata as pretty JSON
  filesxml          Fetch the live `_files.xml` document
  metaxml           Fetch the live `_meta.xml` document
  download          Download an item or file from archive.org
  downloadAsStream  Download a single file and write it to stdout
  downloadHere      Download a single file into the current directory
  checkeddl         Download an item or file, then verify the downloaded content

Options:
  -h, --help  Print help
```

### `ctoolbox ia checkeddl`

```text
Download an item or file, then verify the downloaded content

Usage: ctoolbox ia checkeddl [OPTIONS] <TARGET>

Arguments:
  <TARGET>  An item identifier, item/file path, or archive.org download URL

Options:
      --output-dir <OUTPUT_DIR>  Destination directory. Defaults to the current directory
      --original                 Only download and verify files with source="original"
  -h, --help                     Print help
```

### `ctoolbox ia contains`

```text
Check whether an item contains a particular file

Usage: ctoolbox ia contains <TARGET> <DESIRED_FILE>

Arguments:
  <TARGET>        An item identifier, item/file path, or archive.org URL
  <DESIRED_FILE>  File path inside the item to check for

Options:
  -h, --help  Print help
```

### `ctoolbox ia download`

```text
Download an item or file from archive.org

Usage: ctoolbox ia download [OPTIONS] <TARGET>

Arguments:
  <TARGET>  An item identifier, item/file path, or archive.org download URL

Options:
      --output-dir <OUTPUT_DIR>  Destination directory. Defaults to the current directory
      --original                 Only download files with source="original"
  -h, --help                     Print help
```

### `ctoolbox ia downloadAsStream`

```text
Download a single file and write it to stdout

Usage: ctoolbox ia downloadAsStream <TARGET>

Arguments:
  <TARGET>  An item/file path or archive.org download URL

Options:
  -h, --help  Print help
```

### `ctoolbox ia downloadHere`

```text
Download a single file into the current directory

Usage: ctoolbox ia downloadHere [OPTIONS] <TARGET>

Arguments:
  <TARGET>  An item/file path or archive.org download URL

Options:
      --output-dir <OUTPUT_DIR>  Destination directory. Defaults to the current directory
  -h, --help                     Print help
```

### `ctoolbox ia filesxml`

```text
Fetch the live `_files.xml` document

Usage: ctoolbox ia filesxml <TARGET>

Arguments:
  <TARGET>  An item identifier or archive.org URL

Options:
  -h, --help  Print help
```

### `ctoolbox ia listplain`

```text
List item files one per line

Usage: ctoolbox ia listplain <TARGET>

Arguments:
  <TARGET>  An item identifier or archive.org URL

Options:
  -h, --help  Print help
```

### `ctoolbox ia md5`

```text
Print the expected md5 for a file in an Internet Archive item

Usage: ctoolbox ia md5 [OPTIONS] <TARGET>

Arguments:
  <TARGET>  A local file path, item/file path, or archive.org download URL

Options:
      --identifier <IDENTIFIER>  Override the identifier if it cannot be inferred from the path
      --check-live               Fetch live metadata instead of reading a local files XML
  -h, --help                     Print help
```

### `ctoolbox ia metadata`

```text
Fetch live item metadata as pretty JSON

Usage: ctoolbox ia metadata <TARGET>

Arguments:
  <TARGET>  An item identifier or archive.org URL

Options:
  -h, --help  Print help
```

### `ctoolbox ia metaxml`

```text
Fetch the live `_meta.xml` document

Usage: ctoolbox ia metaxml <TARGET>

Arguments:
  <TARGET>  An item identifier or archive.org URL

Options:
  -h, --help  Print help
```

### `ctoolbox ia sha1`

```text
Print the expected sha1 for a file in an Internet Archive item

Usage: ctoolbox ia sha1 [OPTIONS] <TARGET>

Arguments:
  <TARGET>  A local file path, item/file path, or archive.org download URL

Options:
      --identifier <IDENTIFIER>  Override the identifier if it cannot be inferred from the path
      --check-live               Fetch live metadata instead of reading a local files XML
  -h, --help                     Print help
```

### `ctoolbox ia verify`

```text
Verify a local Internet Archive item directory against its files XML

Usage: ctoolbox ia verify [OPTIONS] [ITEM_PATH]

Arguments:
  [ITEM_PATH]  Path to the item directory, or omit to use the current directory

Options:
      --identifier <IDENTIFIER>  Override the identifier if it cannot be inferred from the path
      --check-live               Fetch the current files XML from archive.org instead of using a local copy
      --original                 Only verify files with source="original"
  -h, --help                     Print help
```

### `ctoolbox install`

```text
Launch the installer GUI (or TUI if --no-gui)

Usage: ctoolbox install [OPTIONS]

Options:
      --no-gui      Use text-mode installer instead of GUI
      --unattended  Run in unattended mode with default options (implies --no-gui)
  -h, --help        Print help (see more with '--help')
```

### `ctoolbox jq`

```text
Lightweight JQ implementation

Usage: ctoolbox jq [OPTIONS] <QUERY> [FILE]

Arguments:
  <QUERY>  The JQ query string
  [FILE]   Optional path to the JSON file, or - for stdin

Options:
  -r, --raw-output  Use raw output (no quotes around strings)
  -h, --help        Print help
```

### `ctoolbox js-lint`

```text
Lint JavaScript and TypeScript sources

Usage: ctoolbox js-lint [OPTIONS] [FILES]...
       ctoolbox js-lint <COMMAND>

Commands:
  rules  
  run    

Arguments:
  [FILES]...  Set the input file to use

Options:
      --rule <RULE_CODE>  Run a certain rule
      --config <CONFIG>   Load config from file
      --format <FORMAT>   Configure output format [default: pretty] [possible values: compact, pretty]
  -h, --help              Print help
```

### `ctoolbox js-lint rules`

```text
Usage: ctoolbox js-lint rules [OPTIONS] [RULE_NAME]

Arguments:
  [RULE_NAME]  Show detailed information about rule. If omitted, show the list of all rules

Options:
      --json  
  -h, --help  Print help
```

### `ctoolbox js-lint run`

```text
Usage: ctoolbox js-lint run [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Set the input file to use

Options:
      --rule <RULE_CODE>  Run a certain rule
      --config <CONFIG>   Load config from file
      --format <FORMAT>   Configure output format [default: pretty] [possible values: compact, pretty]
  -h, --help              Print help
```

### `ctoolbox js-test`

```text
Run JavaScript tests

Usage: ctoolbox js-test <FOLDER>

Arguments:
  <FOLDER>  Folder containing JavaScript tests

Options:
  -h, --help  Print help
```

### `ctoolbox json-escape`

```text
Escape a string to be a valid JSON string value (enclosed in double quotes)

Usage: ctoolbox json-escape [VALUE]

Arguments:
  [VALUE]  The string to escape. If not provided, reads from stdin

Options:
  -h, --help  Print help
```

### `ctoolbox make-offline-installer`

```text
Download and package the latest offline installer bundle

Usage: ctoolbox make-offline-installer [OPTIONS] <OUTPUT>

Arguments:
  <OUTPUT>  Output path for the generated bundle

Options:
      --platform <PLATFORM>      Target platform to fetch. Defaults to the current platform
      --version <VERSION>        Release version to fetch. Defaults to latest
      --server-url <SERVER_URL>  URL of the update server. Defaults to the configured server URL
  -h, --help                     Print help
```

### `ctoolbox pan2csv`

```text
Convert a .pan file to CSV output

Usage: ctoolbox pan2csv <PAN_FILE>

Arguments:
  <PAN_FILE>  Input PAN file path

Options:
  -h, --help  Print help
```

### `ctoolbox pan2parsejson`

```text
Convert a .pan file to JSON of parse

Usage: ctoolbox pan2parsejson <PAN_FILE>

Arguments:
  <PAN_FILE>  Input PAN file path

Options:
  -h, --help  Print help
```

### `ctoolbox pdf2json`

```text
Convert a PDF file to JSON output

Usage: ctoolbox pdf2json <PDF_FILE>

Arguments:
  <PDF_FILE>  Input PDF file path (or - for stdin)

Options:
  -h, --help  Print help
```

### `ctoolbox pdf2md`

```text
Convert a PDF file to Markdown output

Usage: ctoolbox pdf2md <PDF_FILE>

Arguments:
  <PDF_FILE>  Input PDF file path (or - for stdin)

Options:
  -h, --help  Print help
```

### `ctoolbox pdf2txt`

```text
Convert a PDF file to text output

Usage: ctoolbox pdf2txt <PDF_FILE>

Arguments:
  <PDF_FILE>  Input PDF file path (or - for stdin)

Options:
  -h, --help  Print help
```

### `ctoolbox range_gen`

```text
Generate a range of numbers in various bases

Usage: ctoolbox range_gen [OPTIONS] <START> <END>

Arguments:
  <START>  Starting value of the range
  <END>    Ending value of the range

Options:
  -s, --step <STEP>            Step size (defaults to "1") [default: 1]
  -b, --base <BASE>            Number base (e.g. "10", "16", "2", "64", "hex", "bin", "oct") [default: 10]
  -S, --separator <SEPARATOR>  Separator between output items (defaults to newline) [default: "\n"]
  -t, --trailing               Append a trailing separator to the output
  -h, --help                   Print help

Examples:
  $ ctoolbox range_gen 1 10
  1
  2
  3
  4
  5
  6
  7
  8
  9
  10

  $ ctoolbox range_gen -s 2 1 10
  1
  3
  5
  7
  9

  $ ctoolbox range_gen -b 16 -t -S, 18D0C 18D12
  18D0C,18D0D,18D0E,18D0F,18D10,18D11,18D12,

  $ ctoolbox range_gen -b hex 0x00 0x10
  00
  01
  02
  03
  04
  05
  06
  07
  08
  09
  0A
  0B
  0C
  0D
  0E
  0F
  10
```

### `ctoolbox show-node`

```text
Unimplemented, just an example

Usage: ctoolbox show-node --id <ID>

Options:
  -i, --id <ID>  Example parameter
  -h, --help     Print help
```

### `ctoolbox stagel-bootstrap-convert`

```text
Translate a StageL file using the bootstrap compiler

Usage: ctoolbox stagel-bootstrap-convert [OPTIONS] <INPUT_FILE> <TARGET_LANG>

Arguments:
  <INPUT_FILE>   Input StageL file path
  <TARGET_LANG>  Target language (js or bash)

Options:
      --cache-dir <CACHE_DIR>   Optional cache directory (if specified, enables caching)
      --no-debug                Disable debug build
      --no-runtime-type-checks  Disable runtime type checks
  -h, --help                    Print help
```

### `ctoolbox stagel-bootstrap-parse`

```text
Parse a StageL file to token output

Usage: ctoolbox stagel-bootstrap-parse <INPUT_FILE>

Arguments:
  <INPUT_FILE>  Input StageL file path

Options:
  -h, --help  Print help
```

### `ctoolbox ts-check`

```text
Type-check TypeScript code

Usage: ctoolbox ts-check [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Set the input file(s) or directories to use

Options:
      --config <CONFIG>           Load config from file
      --format <FORMAT>           Configure output format [default: pretty] [possible values: compact, pretty]
      --add-types <ADD_TYPES>...  Dynamically patch tsconfig to add paths mapping for these types from the compiler's types folder
  -h, --help                      Print help
```

### `ctoolbox uninstall`

```text
Uninstall ctoolbox from this system

Usage: ctoolbox uninstall [OPTIONS]

Options:
      --no-gui      Use text-mode uninstaller instead of GUI
      --unattended  Run without prompting for confirmation
  -h, --help        Print help (see more with '--help')
```

### `ctoolbox update`

```text
Check for updates and optionally apply them

Usage: ctoolbox update [OPTIONS]

Options:
      --unattended               Automatically apply updates without prompting
      --server-url <SERVER_URL>  URL of the update server. Defaults to the configured server URL
  -h, --help                     Print help (see more with '--help')
```

### `ctoolbox waitrestart`

```text
Wait for the provided PID to shutdown, then start ctoolbox with the given port

Usage: ctoolbox waitrestart --pid <PID> --port <PORT>

Options:
      --pid <PID>    Process ID to wait for
      --port <PORT>  Port to pass to the new ctoolbox instance
  -h, --help         Print help
```

### `ctoolbox waitshutdown`

```text
Wait for the provided PID to shutdown, then clean up

Usage: ctoolbox waitshutdown --pid <PID>

Options:
      --pid <PID>  Process ID to wait for
  -h, --help       Print help
```

### `ctoolbox waitupgrade`

```text
Wait for the provided PID to shutdown, then upgrade the ctoolbox instance in place, then restart it. (Will need to copy the executable before upgrading it probably because Windows locks running executables.)

Usage: ctoolbox waitupgrade --pid <PID> --temp-path <TEMP_PATH> --target-path <TARGET_PATH> --port <PORT>

Options:
      --pid <PID>                  Process ID to wait for
      --temp-path <TEMP_PATH>      Path to the temporary file holding the new ctoolbox executable
      --target-path <TARGET_PATH>  Path to the installed ctoolbox executable
      --port <PORT>                Port to pass to the new ctoolbox instance
  -h, --help                       Print help
```

### `ctoolbox warcat`

```text
WARC archive tool

Usage: warcat [OPTIONS] <COMMAND>

Commands:
  export   Decodes a WARC file to messages in a easier-to-process format such as JSON
  import   Encodes a WARC file from messages in a format of the `export` subcommand
  list     Provides a listing of the WARC records
  get      Returns a single WARC record
  extract  Extracts resources for casual viewing of the WARC contents
  verify   Perform specification and integrity checks on WARC files
  self     Self-installer and uninstaller
  help     Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet                  Disable any progress messages
      --log-level <LOG_LEVEL>  Filter log messages by level [default: off] [possible values: trace, debug, info, warn, error, off]
      --log-file <LOG_FILE>    Write log messages to the given file instead of standard error
      --log-json               Write log messages as JSON sequences instead of a console logging format
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
```

### `ctoolbox warcat export`

```text
Decodes a WARC file to messages in a easier-to-process format such as JSON

Usage: warcat export [OPTIONS]

Options:
      --input <INPUT>              Path to a WARC file [default: -]
      --compression <COMPRESSION>  Specify the compression format of the input WARC file [default: auto] [possible values: auto, none, gzip, zstandard]
      --output <OUTPUT>            Path for the output messages [default: -]
      --format <FORMAT>            Format for the output messages [default: json-seq] [possible values: json-seq, jsonl, cbor-seq]
      --no-block                   Do not output block messages
      --extract                    Output extract messages
  -h, --help                       Print help (see more with '--help')
```

### `ctoolbox warcat extract`

```text
Extracts resources for casual viewing of the WARC contents

Usage: warcat extract [OPTIONS]

Options:
      --input <INPUT>
          Path to the WARC file [default: -]
      --compression <COMPRESSION>
          Compression format of the input WARC file [default: auto] [possible values: auto, none, gzip, zstandard]
      --output <OUTPUT>
          Path to the output directory [default: ./]
      --continue-on-error
          Whether to ignore errors
      --include <INCLUDE>
          Select only records with a field
      --include-pattern <INCLUDE_PATTERN>
          Select only records matching a regular expression
      --exclude <EXCLUDE>
          Do not select records with a field
      --exclude-pattern <EXCLUDE_PATTERN>
          Do not select records matching a regular expression
  -h, --help
          Print help (see more with '--help')
```

### `ctoolbox warcat get`

```text
Returns a single WARC record

Usage: warcat get <COMMAND>

Commands:
  export   Output export messages
  extract  Extract a resource
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `ctoolbox warcat get export`

```text
Output export messages

Usage: warcat get export [OPTIONS] --position <POSITION>

Options:
      --input <INPUT>              Path of the WARC file [default: -]
      --compression <COMPRESSION>  Compression format of the input WARC file [default: auto] [possible values: auto, none, gzip, zstandard]
      --position <POSITION>        Position where the record is located in the input WARC file
      --id <ID>                    The ID of the record to extract
      --output <OUTPUT>            Path for the output messages [default: -]
      --format <FORMAT>            Format for the output messages [default: json-seq] [possible values: json-seq, jsonl, cbor-seq]
      --no-block                   Do not output block messages
      --extract                    Output extract messages
  -h, --help                       Print help (see more with '--help')
```

### `ctoolbox warcat get extract`

```text
Extract a resource

Usage: warcat get extract [OPTIONS] --position <POSITION>

Options:
      --input <INPUT>              [default: -]
      --compression <COMPRESSION>  Compression format of the input WARC file [default: auto] [possible values: auto, none, gzip, zstandard]
      --position <POSITION>        Position where the record is located in the input WARC file
      --id <ID>                    The ID of the record to extract
      --output <OUTPUT>            Path for the output file [default: -]
  -h, --help                       Print help (see more with '--help')
```

### `ctoolbox warcat import`

```text
Encodes a WARC file from messages in a format of the `export` subcommand

Usage: warcat import [OPTIONS]

Options:
      --input <INPUT>
          Path to the input messages [default: -]
      --format <FORMAT>
          Format for the input messages [default: json-seq] [possible values: json-seq, jsonl, cbor-seq]
      --output <OUTPUT>
          Path of the output WARC file [default: -]
      --compression <COMPRESSION>
          Compression format of the output WARC file [default: auto] [possible values: auto, none, gzip, zstandard]
      --compression-level <COMPRESSION_LEVEL>
          Level of compression for the output [default: high] [possible values: balanced, high, low]
  -h, --help
          Print help (see more with '--help')
```

### `ctoolbox warcat list`

```text
Provides a listing of the WARC records

Usage: warcat list [OPTIONS]

Options:
      --input <INPUT>              Path of the WARC file [default: -]
      --compression <COMPRESSION>  Compression format of the input WARC file [default: auto] [possible values: auto, none, gzip, zstandard]
      --output <OUTPUT>            Path to output listings [default: -]
      --format <FORMAT>            Format of the output [default: json-seq] [possible values: json-seq, jsonl, cbor-seq, csv]
      --field <FIELD>              Fields to include in the listing [default: :position,WARC-Record-ID,WARC-Type,Content-Type,WARC-Target-URI]
  -h, --help                       Print help (see more with '--help')
```

### `ctoolbox warcat self`

```text
Self-installer and uninstaller

Usage: warcat self <COMMAND>

Commands:
  install    Launch the interactive self-installer
  uninstall  Launch the interactive uninstaller
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `ctoolbox warcat self install`

```text
Launch the interactive self-installer

Usage: warcat self install [OPTIONS]

Options:
      --quiet  Install automatically without user interaction
  -h, --help   Print help
```

### `ctoolbox warcat self uninstall`

```text
Launch the interactive uninstaller

Usage: warcat self uninstall [OPTIONS]

Options:
      --quiet  Uninstall automatically without user interaction
  -h, --help   Print help
```

### `ctoolbox warcat verify`

```text
Perform specification and integrity checks on WARC files

Usage: warcat verify [OPTIONS]

Options:
      --input <INPUT>                  Path to the WARC file [default: -]
      --compression <COMPRESSION>      Compression format of the input WARC file [default: auto] [possible values: auto, none, gzip, zstandard]
      --output <OUTPUT>                Path to output problems [default: -]
      --format <FORMAT>                Format of the output [default: json-seq] [possible values: json-seq, jsonl, cbor-seq, csv]
      --exclude-check <EXCLUDE_CHECK>  Do not perform check [possible values: mandatory-fields, known-record-type, content-type, concurrent-to, block-digest, payload-digest, ip-address, refers-to, refers-to-target-uri, refers-to-date, target-uri, truncated, warcinfo-id, filename, profile, segment, record-at-time-compression]
      --database <DATABASE>            Database filename for storing temporary intermediate data
  -h, --help                           Print help (see more with '--help')
```

### `ctoolbox wfparser`

```text
Process a file using wfparser logic

Usage: ctoolbox wfparser [FILE]

Arguments:
  [FILE]  Input file path (or - for stdin) [default: -]

Options:
  -h, --help  Print help
```

### `ctoolbox wfscan`

```text
Process a file using wfscan logic

Usage: ctoolbox wfscan [FILE]

Arguments:
  [FILE]  Input file path (or - for stdin) [default: -]

Options:
  -h, --help  Print help
```

### `ctoolbox x86-instruction-sets`

```text
Analyze an x86/x64 object or archive file and list CPU instruction set features

Usage: ctoolbox x86-instruction-sets <PATH>

Arguments:
  <PATH>  Path to the object or archive file

Options:
  -h, --help  Print help
```


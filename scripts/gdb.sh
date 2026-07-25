#!/usr/bin/env bash
set -euxo pipefail

BINARY=~/ctoolbox/ctoolbox/target/debug/ctoolbox
OUT=trace.gdb
LOGFILE=gdb_log.txt
TMPSYMS=$(mktemp)

# Resolve path
BINARY_PATH=$(realpath "$BINARY")

echo "Binary: $BINARY_PATH"
echo "Generating symbol list..."

# List defined text symbols, demangle, and filter Rust-like symbols.
# Use nm -C (demangle) or nm + c++filt if needed.
# -D may be required on some systems; try plain nm first.
if nm --version >/dev/null 2>&1; then
  nm -n --defined-only "$BINARY_PATH" 2>/dev/null |\
    awk '$2 ~ /^[Tt]$/ { print $3 }' > "$TMPSYMS"
else
  echo "nm not found" >&2
  exit 1
fi

# Demangle (c++filt works for Rust symbols too) and filter
if command -v c++filt >/dev/null 2>&1; then
  echo "Got"
  cat "$TMPSYMS" | head -n 5 || true
  echo "Demangling and filtering symbols..."
  #c++filt < "$TMPSYMS"
  c++filt < "$TMPSYMS" | \
    # Keep symbols that look like Rust paths (contain ::) and skip closures and local symbols that are noisy.
    grep '::' | \
    # Exclude common noisy patterns (closures, __rust_maybe_uninit, vtable, etc.)
    grep -Ev '::\{\{.*\}\}|__rust|_ZN|vtable|__cxx|llvm' \
    > "${TMPSYMS}.filtered" || true
    cat "$TMPSYMS.filtered" | head -n 5 || true
else
  cp "$TMPSYMS" "${TMPSYMS}.filtered"
fi

SYMLIST="${TMPSYMS}.filtered"
COUNT=$(wc -l < "$SYMLIST" || echo 0)
echo "Found $COUNT candidate symbols (post-filter)."

if [ "$COUNT" -eq 0 ]; then
  echo "No symbols found after filtering. Writing an empty trace.gdb with instructions."
  cat > "$OUT" <<EOF
set pagination off
set logging on $LOGFILE
run
EOF
  exit 0
fi

echo "Writing $OUT..."

cat > "$OUT" <<EOF
set pagination off
set height 0
set logging on $LOGFILE
# Automatically continue after printing a one-frame backtrace
EOF

if [[ -n "$(which ctoolbox)" ]]; then
    cat "$SYMLIST" | ctoolbox gdb_instructions_generate >> "$OUT"
else
    CHUNK=1000
    i=0
    while IFS= read -r sym; do
        # Escape quotes/backslashes in symbol for gdb
        esc=$(printf '%s' "$sym" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
        echo "break $esc" >> "$OUT"
        cat >> "$OUT" <<'GDBCMD'
commands
    silent
    bt 1
    continue
end
GDBCMD
        i=$((i+1))
    done < "$SYMLIST"
fi

# Run at startup
cat >> "$OUT" <<EOF

run
EOF

echo "Done. Generated $OUT with $i breakpoints. Run it with:"
echo "  rust-gdb -x $OUT --args $BINARY_PATH [program-args...]"
echo "GDB will log to $LOGFILE in the current directory."

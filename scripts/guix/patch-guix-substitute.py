#!/usr/bin/env python3
"""Patch Guix substitute.scm to log detailed debugging and error tracebacks."""

import glob
import os
import re

def patch_substitute_file(path: str):
    print(f"Checking {path}...")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    # Add debug logging and exception catching in download-nar
    old_restore = """      ;; Unpack the Nar at INPUT into DESTINATION.
      (define cpu-usage
        (with-cpu-usage-monitoring
         (restore-file hashed destination
                       #:dump-file (if (and destination-in-store?
                                            deduplicate?)
                                       dump-file/deduplicate*
                                       dump-file))))"""

    new_restore = """      ;; Unpack the Nar at INPUT into DESTINATION with detailed error logging.
      (format (current-error-port) "[substitute-debug] Unpacking ~a (comp: ~a, dl-size: ~a)...\\n" destination compression dl-size)
      (define cpu-usage
        (catch #t
          (lambda ()
            (with-cpu-usage-monitoring
             (restore-file hashed destination
                           #:dump-file (if (and destination-in-store?
                                                deduplicate?)
                                           dump-file/deduplicate*
                                           dump-file))))
          (lambda (key . args)
            (format (current-error-port) "[substitute-error] restore-file failed for ~a: key=~a args=~s\\n" destination key args)
            (display-backtrace (make-stack #t) (current-error-port))
            (apply throw key args))))
      (format (current-error-port) "[substitute-debug] Unpack complete for ~a.\\n" destination)"""

    if old_restore in content:
        content = content.replace(old_restore, new_restore)
        # Also patch exception handling in download-nar around get-hash
        old_hash = """      ;; Check whether we got the data announced in NARINFO.
      (let ((actual (get-hash)))
        (if (bytevector=? actual expected)"""

        new_hash = """      ;; Check whether we got the data announced in NARINFO.
      (let ((actual (get-hash)))
        (format (current-error-port) "[substitute-debug] Hash check for ~a: match=~a\\n" destination (bytevector=? actual expected))
        (if (bytevector=? actual expected)"""

        content = content.replace(old_hash, new_hash)

        os.chmod(path, 0o644)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Successfully patched {path}")

        # Delete corresponding .go file so Guile recompiles from .scm
        go_path = path.replace(".scm", ".go")
        if os.path.exists(go_path):
            os.remove(go_path)
            print(f"Removed {go_path}")

def main():
    paths = glob.glob("/gnu/store/*-guix-*/share/guile/site/3.0/guix/scripts/substitute.scm")
    for p in paths:
        patch_substitute_file(p)

if __name__ == "__main__":
    main()

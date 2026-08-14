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
      (format (current-error-port) "[substitute-debug] Unpacking ~a (comp: ~a, dl-size: ~a)...\\n" destination compression download-size)
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

    # Also fix any previously patched dl-size
    content = content.replace("dl-size: ~a)...\\n\" destination compression dl-size)", "dl-size: ~a)...\\n\" destination compression download-size)")

    # Add hash check debug log
    old_hash = """      ;; Check whether we got the data announced in NARINFO.
      (let ((actual (get-hash)))
        (if (bytevector=? actual expected)"""

    new_hash = """      ;; Check whether we got the data announced in NARINFO.
      (let ((actual (get-hash)))
        (format (current-error-port) "[substitute-debug] Hash check for ~a: match=~a\\n" destination (bytevector=? actual expected))
        (if (bytevector=? actual expected)"""

    if old_hash in content:
        content = content.replace(old_hash, new_hash)

    # Wrap process-substitution in catch #t to catch any unhandled exceptions
    old_proc_sub = """  (guard (c ((network-error? c)
             (when (http-get-error? c)
               (warning (G_ "download from '~a' failed: ~a, ~s~%")
                        (uri->string (http-get-error-uri c))
                        (http-get-error-code c)
                        (http-get-error-reason c)))
             (format (current-error-port)
                     (G_ "retrying download of '~a' with other substitute URLs...~%")
                     store-item)
             (process-substitution/fallback port narinfo destination
                                            #:cache-urls cache-urls
                                            #:acl acl
                                            #:deduplicate? deduplicate?
                                            #:print-build-trace?
                                            print-build-trace?)))
    (download-nar narinfo destination
                  #:status-port port
                  #:deduplicate? deduplicate?
                  #:print-build-trace? print-build-trace?)))"""

    new_proc_sub = """  (catch #t
    (lambda ()
      (guard (c ((network-error? c)
                 (when (http-get-error? c)
                   (warning (G_ "download from '~a' failed: ~a, ~s~%")
                            (uri->string (http-get-error-uri c))
                            (http-get-error-code c)
                            (http-get-error-reason c)))
                 (format (current-error-port)
                         (G_ "retrying download of '~a' with other substitute URLs...~%")
                         store-item)
                 (process-substitution/fallback port narinfo destination
                                                #:cache-urls cache-urls
                                                #:acl acl
                                                #:deduplicate? deduplicate?
                                                #:print-build-trace?
                                                print-build-trace?)))
        (download-nar narinfo destination
                      #:status-port port
                      #:deduplicate? deduplicate?
                      #:print-build-trace? print-build-trace?)))
    (lambda (key . args)
      (format (current-error-port) "[substitute-fatal] Unhandled error during substitution of ~a: key=~a args=~s\\n" store-item key args)
      (display-backtrace (make-stack #t) (current-error-port))
      (apply throw key args))))"""

    if old_proc_sub in content:
        content = content.replace(old_proc_sub, new_proc_sub)

    os.chmod(path, 0o644)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Successfully patched {path}")

    # Compile with guild if available, or update .go timestamp
    go_path = path.replace("/share/guile/site/3.0/", "/lib/guile/3.0/site-ccache/").replace(".scm", ".go")
    if not os.path.exists(go_path):
        go_path = path.replace(".scm", ".go")

    guild_paths = glob.glob("/gnu/store/*-guile-*/bin/guild")
    compiled = False
    if guild_paths and os.path.exists(guild_paths[0]):
        guild_bin = guild_paths[0]
        pkg_share = re.sub(r"/guix/scripts/.*", "", path)
        cmd = f"{guild_bin} compile -L {pkg_share} -o {go_path} {path}"
        ret = os.system(cmd)
        if ret == 0:
            compiled = True
            print(f"Compiled {go_path}")

    if not compiled and os.path.exists(go_path):
        # Ensure .go timestamp is strictly newer than .scm
        mtime = os.path.getmtime(path) + 10
        os.utime(go_path, (mtime, mtime))

def main():
    paths = glob.glob("/gnu/store/*-guix-*/share/guile/site/3.0/guix/scripts/substitute.scm")
    for p in paths:
        patch_substitute_file(p)

if __name__ == "__main__":
    main()

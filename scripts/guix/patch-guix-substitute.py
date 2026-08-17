#!/usr/bin/env python3
"""Patch Guix substitute.scm, ui.scm, serialization.scm, and utils.scm to log detailed debugging."""

import glob
import os
import re

def compile_or_touch_scm(path: str):
    go_path = path.replace("/share/guile/site/3.0/", "/lib/guile/3.0/site-ccache/").replace(".scm", ".go")
    if not os.path.exists(go_path):
        go_path = path.replace(".scm", ".go")

    guild_paths = glob.glob("/gnu/store/*-guile-*/bin/guild")
    compiled = False
    if guild_paths and os.path.exists(guild_paths[0]):
        guild_bin = guild_paths[0]
        pkg_share = re.sub(r"/guix/scripts/.*|/guix/.*", "", path)
        cmd = f"{guild_bin} compile -L {pkg_share} -o {go_path} {path}"
        ret = os.system(cmd)
        if ret == 0:
            compiled = True
            print(f"Compiled {go_path}")

    if not compiled and os.path.exists(go_path):
        mtime = os.path.getmtime(path) + 10
        os.utime(go_path, (mtime, mtime))

def patch_substitute_file(path: str):
    print(f"Checking substitute {path}...")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    # Clear LD_PRELOAD and log args at top of guix-substitute
    old_cmd = """(define-command (guix-substitute . args)
  (category internal)
  (synopsis "implement the build daemon's substituter protocol")"""

    new_cmd = """(define-command (guix-substitute . args)
  (category internal)
  (synopsis "implement the build daemon's substituter protocol")
  (false-if-exception (unsetenv "LD_PRELOAD"))
  (format (current-error-port) "[substitute-debug] guix-substitute invoked with args: ~s\\n" args)"""

    if old_cmd in content and new_cmd not in content:
        content = content.replace(old_cmd, new_cmd)

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
            (display-backtrace (make-stack #t) (current-error-port) 0 30)
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
      (display-backtrace (make-stack #t) (current-error-port) 0 30)
      (apply throw key args))))"""

    if old_proc_sub in content:
        content = content.replace(old_proc_sub, new_proc_sub)

    os.chmod(path, 0o644)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Successfully patched {path}")
    compile_or_touch_scm(path)

def patch_utils_file(path: str):
    print(f"Checking utils {path}...")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    old_filtered = """(define (filtered-port command input)
  "Return an input port where data drained from INPUT is filtered through
COMMAND (a list).  In addition, return a list of PIDs that the caller must
wait.  When INPUT is a file port, it must be unbuffered; otherwise, any
buffered data is lost."
  (let loop ((input input)"""

    new_filtered = """(define (filtered-port command input)
  "Return an input port where data drained from INPUT is filtered through
COMMAND (a list).  In addition, return a list of PIDs that the caller must
wait.  When INPUT is a file port, it must be unbuffered; otherwise, any
buffered data is lost."
  (false-if-exception (unsetenv "LD_PRELOAD"))
  (let loop ((input input)"""

    if old_filtered in content and new_filtered not in content:
        content = content.replace(old_filtered, new_filtered)
        os.chmod(path, 0o644)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Successfully patched {path}")
        compile_or_touch_scm(path)

def patch_ui_file(path: str):
    print(f"Checking ui {path}...")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    old_nar_error = """             ((nar-error? c)
              (let ((file (nar-error-file c))
                    (port (nar-error-port c)))"""

    new_nar_error = """             ((nar-error? c)
              (let ((file (nar-error-file c))
                    (port (nar-error-port c)))
                (format (current-error-port) "[ui-debug] &nar-error intercepted: file=~s port=~s (closed: ~s)\\n" file port (and (port? port) (port-closed? port)))
                (display-backtrace (make-stack #t) (current-error-port) 0 30)"""

    if old_nar_error in content and new_nar_error not in content:
        content = content.replace(old_nar_error, new_nar_error)
        os.chmod(path, 0o644)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Successfully patched {path}")
        compile_or_touch_scm(path)

def patch_serialization_file(path: str):
    print(f"Checking serialization {path}...")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    old_sig = """    (let ((signature (read-string port)))
      (unless (equal? signature %archive-version-1)"""

    new_sig = """    (let ((signature (read-string port)))
      (unless (equal? signature %archive-version-1)
        (format (current-error-port) "[serialization-debug] Nar signature mismatch on port ~s: got ~s, expected ~s\\n" port signature %archive-version-1)
        (display-backtrace (make-stack #t) (current-error-port) 0 30)"""

    if old_sig in content and new_sig not in content:
        content = content.replace(old_sig, new_sig)

    old_short = """(define (get-bytevector-n* port count)
  (let ((bv (get-bytevector-n port count)))
    (when (or (eof-object? bv)
              (< (bytevector-length bv) count))
      (raise (condition (&nar-error"""

    new_short = """(define (get-bytevector-n* port count)
  (let ((bv (get-bytevector-n port count)))
    (when (or (eof-object? bv)
              (< (bytevector-length bv) count))
      (format (current-error-port) "[serialization-debug] short read on port ~s: expected ~a bytes, got ~s\\n" port count bv)
      (display-backtrace (make-stack #t) (current-error-port) 0 30)
      (raise (condition (&nar-error"""

    if old_short in content and new_short not in content:
        content = content.replace(old_short, new_short)

    os.chmod(path, 0o644)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Successfully patched {path}")
    compile_or_touch_scm(path)

def patch_gnu_build_system_file(path: str):
    print(f"Checking gnu-build-system {path}...")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()

    old_unpack = """        ;; Attempt to change into child directory.
        (and=> (first-subdirectory ".") chdir))))"""

    new_unpack = """        ;; Attempt to change into child directory.
        (and=> (first-subdirectory ".") chdir)
        (for-each (lambda (f)
                    (false-if-exception (make-file-writable f)))
                  (find-files "." #:directories? #t)))))"""

    if old_unpack in content and new_unpack not in content:
        content = content.replace(old_unpack, new_unpack)
        os.chmod(path, 0o644)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Successfully patched {path}")
        compile_or_touch_scm(path)

def main():
    for p in glob.glob("/gnu/store/*-guix-*/share/guile/site/3.0/guix/scripts/substitute.scm"):
        patch_substitute_file(p)

    for p in glob.glob("/gnu/store/*-guix-*/share/guile/site/3.0/guix/utils.scm"):
        patch_utils_file(p)

    for p in glob.glob("/gnu/store/*-guix-*/share/guile/site/3.0/guix/ui.scm"):
        patch_ui_file(p)

    for p in glob.glob("/gnu/store/*-guix-*/share/guile/site/3.0/guix/serialization.scm"):
        patch_serialization_file(p)

    for p in glob.glob("/gnu/store/*-guix-*/share/guile/site/3.0/guix/build/gnu-build-system.scm"):
        patch_gnu_build_system_file(p)

if __name__ == "__main__":
    main()

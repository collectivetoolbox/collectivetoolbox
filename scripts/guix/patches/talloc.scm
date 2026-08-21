;;; This file is part of Collective Toolbox, a database and document workspace and utilities.
;;; Copyright (C) 2026 Collective Toolbox Developers
;;; Contact: info@collectivetoolbox.com
;;;
;;; This Scheme program is free software; you can redistribute it and/or modify
;;; it under the terms of the GNU General Public License as published by the
;;; Free Software Foundation; either version 3 of the License, or (at your
;;; option) any later version.
;;;
;;; This Scheme program is distributed in the hope that it will be useful, but
;;; WITHOUT ANY WARRANTY; without even the implied warranty of
;;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;;; GNU General Public License for more details.
;;;
;;; You should have received a copy of the GNU General Public License
;;; along with this Scheme program.  If not, see <http://www.gnu.org/licenses/>.

;;; Patch for talloc, tdb, tevent, and ldb cross-compilation with python disabled.

(define-module (patches talloc)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (gnu packages base)
  #:use-module (gnu packages check)
  #:use-module (gnu packages databases)
  #:use-module (gnu packages pkg-config)
  #:use-module (gnu packages python)
  #:use-module (gnu packages samba)
  #:export (talloc-fixed-proc
            talloc-fixed
            tdb-fixed-proc
            tdb-fixed
            tevent-fixed-proc
            tevent-fixed
            ldb-fixed-proc
            ldb-fixed))

(define %samba-cross-answers
  (string-append
   "Checking uname sysname type: \"Linux\"\n"
   "Checking uname machine type: \"i686\"\n"
   "Checking uname release type: \"6.12.0\"\n"
   "Checking uname version type: \"#1\"\n"
   "Checking getconf LFS_CFLAGS: NO\n"
   "Checking for large file support without additional flags: NO\n"
   "Checking for -D_FILE_OFFSET_BITS=64: OK\n"
   "Checking for -D_LARGE_FILES: NO\n"
   "Checking correct behavior of strtoll: OK\n"
   "Checking for working strptime: OK\n"
   "Checking for HAVE_IFACE_GETIFADDRS: OK\n"
   "Checking for HAVE_IFACE_IFCONF: OK\n"
   "Checking for HAVE_IFACE_IFREQ: OK\n"
   "Checking for HAVE_IFACE_AIX: NO\n"
   "Checking for HAVE_SECURE_MKSTEMP: OK\n"
   "Checking for library constructor support: OK\n"
   "Checking for library destructor support: OK\n"
   "Checking for -Wl,--version-script support: OK\n"
   "Checking for rpath library support: OK\n"
   "Checking for HAVE_VISIBILITY_ATTR: OK\n"
   "Checking for simple C program: OK\n"
   "Checking compiler accepts ['-Werror']: OK\n"
   "Checking linker accepts ['-Wl,-rpath,.']: OK\n"))

(define (talloc-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (append python-wrapper which pkg-config)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases)
        `(modify-phases ,phases
           (add-before 'configure 'patch-samba-cross
             (lambda _
               (when (file-exists? "buildtools/wafsamba/samba_cross.py")
                 (substitute* "buildtools/wafsamba/samba_cross.py"
                   (("cross_answers_incomplete = True")
                    "cross_answers_incomplete = False")
                   (("return ANSWER_UNKNOWN")
                    "return ANSWER_OK")))))
           (replace 'configure
             (lambda* (#:key outputs (target #f) #:allow-other-keys)
               (let* ((out (assoc-ref outputs "out"))
                      (py (or (which "python3") (which "python")))
                      (target-pkg-cfg (and target (which (string-append target "-pkg-config"))))
                      (host-pkg-cfg (which "pkg-config"))
                      (pkg-cfg (or target-pkg-cfg host-pkg-cfg))
                      (bin-dir (string-append (getcwd) "/build-bin"))
                      (cross-pkg-path (or (getenv "CROSS_PKG_CONFIG_PATH")
                                          (getenv "PKG_CONFIG_PATH") "")))
                 (mkdir-p bin-dir)
                 (when pkg-cfg
                   (symlink pkg-cfg (string-append bin-dir "/pkg-config"))
                   (when target
                     (symlink pkg-cfg (string-append bin-dir "/" target "-pkg-config"))))
                 (setenv "PATH" (string-append bin-dir ":" (getenv "PATH")))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (when pkg-cfg
                   (setenv "PKGCONFIG" (string-append bin-dir "/pkg-config"))
                   (setenv "PKG_CONFIG" (string-append bin-dir "/pkg-config")))
                 (when (and cross-pkg-path (not (string-null? cross-pkg-path)))
                   (setenv "PKG_CONFIG_PATH" cross-pkg-path))
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display ,%samba-cross-answers)))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
                       (setenv "CFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (setenv "CPPFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (invoke "sh" "./configure"
                               (string-append "--prefix=" out)
                               "--cross-compile"
                               "--cross-answers=cross-answers.txt"
                               "--disable-python"))
                     (invoke "sh" "./configure"
                             (string-append "--prefix=" out))))))))))))

(define (tdb-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (append python-wrapper which pkg-config)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases)
        `(modify-phases ,phases
           (add-before 'configure 'patch-samba-cross
             (lambda _
               (when (file-exists? "buildtools/wafsamba/samba_cross.py")
                 (substitute* "buildtools/wafsamba/samba_cross.py"
                   (("cross_answers_incomplete = True")
                    "cross_answers_incomplete = False")
                   (("return ANSWER_UNKNOWN")
                    "return ANSWER_OK")))))
           (replace 'configure
             (lambda* (#:key outputs (target #f) #:allow-other-keys)
               (let* ((out (assoc-ref outputs "out"))
                      (py (or (which "python3") (which "python")))
                      (target-pkg-cfg (and target (which (string-append target "-pkg-config"))))
                      (host-pkg-cfg (which "pkg-config"))
                      (pkg-cfg (or target-pkg-cfg host-pkg-cfg))
                      (bin-dir (string-append (getcwd) "/build-bin"))
                      (cross-pkg-path (or (getenv "CROSS_PKG_CONFIG_PATH")
                                          (getenv "PKG_CONFIG_PATH") "")))
                 (mkdir-p bin-dir)
                 (when pkg-cfg
                   (symlink pkg-cfg (string-append bin-dir "/pkg-config"))
                   (when target
                     (symlink pkg-cfg (string-append bin-dir "/" target "-pkg-config"))))
                 (setenv "PATH" (string-append bin-dir ":" (getenv "PATH")))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (when pkg-cfg
                   (setenv "PKGCONFIG" (string-append bin-dir "/pkg-config"))
                   (setenv "PKG_CONFIG" (string-append bin-dir "/pkg-config")))
                 (when (and cross-pkg-path (not (string-null? cross-pkg-path)))
                   (setenv "PKG_CONFIG_PATH" cross-pkg-path))
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display ,%samba-cross-answers)))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
                       (setenv "CFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (setenv "CPPFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (invoke "sh" "./configure"
                               (string-append "--prefix=" out)
                               "--cross-compile"
                               "--cross-answers=cross-answers.txt"
                               "--bundled-libraries=NONE"
                               "--disable-python"))
                     (invoke "sh" "./configure"
                             (string-append "--prefix=" out)
                             "--bundled-libraries=NONE")))))))))))

(define (tevent-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (append python-wrapper which pkg-config)))
    (inputs
     (modify-inputs (package-inputs pkg)
       (append cmocka)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases)
        `(modify-phases ,phases
           (add-before 'configure 'patch-samba-cross
             (lambda _
               (when (file-exists? "buildtools/wafsamba/samba_cross.py")
                 (substitute* "buildtools/wafsamba/samba_cross.py"
                   (("cross_answers_incomplete = True")
                    "cross_answers_incomplete = False")
                   (("return ANSWER_UNKNOWN")
                    "return ANSWER_OK")))))
           (replace 'configure
             (lambda* (#:key outputs (target #f) #:allow-other-keys)
               (let* ((out (assoc-ref outputs "out"))
                      (py (or (which "python3") (which "python")))
                      (target-pkg-cfg (and target (which (string-append target "-pkg-config"))))
                      (host-pkg-cfg (which "pkg-config"))
                      (pkg-cfg (or target-pkg-cfg host-pkg-cfg))
                      (bin-dir (string-append (getcwd) "/build-bin"))
                      (cross-pkg-path (or (getenv "CROSS_PKG_CONFIG_PATH")
                                          (getenv "PKG_CONFIG_PATH") "")))
                 (mkdir-p bin-dir)
                 (when pkg-cfg
                   (symlink pkg-cfg (string-append bin-dir "/pkg-config"))
                   (when target
                     (symlink pkg-cfg (string-append bin-dir "/" target "-pkg-config"))))
                 (setenv "PATH" (string-append bin-dir ":" (getenv "PATH")))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (when pkg-cfg
                   (setenv "PKGCONFIG" (string-append bin-dir "/pkg-config"))
                   (setenv "PKG_CONFIG" (string-append bin-dir "/pkg-config")))
                 (when (and cross-pkg-path (not (string-null? cross-pkg-path)))
                   (setenv "PKG_CONFIG_PATH" cross-pkg-path))
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display ,%samba-cross-answers)))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
                       (setenv "CFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (setenv "CPPFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (invoke "sh" "./configure"
                               (string-append "--prefix=" out)
                               "--cross-compile"
                               "--cross-answers=cross-answers.txt"
                               "--bundled-libraries=NONE"
                               "--disable-python"))
                     (invoke "sh" "./configure"
                             (string-append "--prefix=" out)
                             "--bundled-libraries=NONE")))))))))))

(define (ldb-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (append python-wrapper which pkg-config)))
    (inputs
     (modify-inputs (package-inputs pkg)
       (append cmocka)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:phases phases)
        `(modify-phases ,phases
           (add-before 'configure 'patch-samba-cross
             (lambda _
               (when (file-exists? "buildtools/wafsamba/samba_cross.py")
                 (substitute* "buildtools/wafsamba/samba_cross.py"
                   (("cross_answers_incomplete = True")
                    "cross_answers_incomplete = False")
                   (("return ANSWER_UNKNOWN")
                    "return ANSWER_OK")))
               (when (file-exists? "wscript")
                 (substitute* "wscript"
                   (("deps='cmocka ldb")
                    "deps='cmocka ldb replace ")
                   (("deps=\"cmocka ldb")
                    "deps=\"cmocka ldb replace ")))))
           (replace 'configure
             (lambda* (#:key outputs (target #f) #:allow-other-keys)
               (let* ((out (assoc-ref outputs "out"))
                      (py (or (which "python3") (which "python")))
                      (target-pkg-cfg (and target (which (string-append target "-pkg-config"))))
                      (host-pkg-cfg (which "pkg-config"))
                      (pkg-cfg (or target-pkg-cfg host-pkg-cfg))
                      (bin-dir (string-append (getcwd) "/build-bin"))
                      (cross-pkg-path (or (getenv "CROSS_PKG_CONFIG_PATH")
                                          (getenv "PKG_CONFIG_PATH") "")))
                 (mkdir-p bin-dir)
                 (when pkg-cfg
                   (symlink pkg-cfg (string-append bin-dir "/pkg-config"))
                   (when target
                     (symlink pkg-cfg (string-append bin-dir "/" target "-pkg-config"))))
                 (setenv "PATH" (string-append bin-dir ":" (getenv "PATH")))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (when pkg-cfg
                   (setenv "PKGCONFIG" (string-append bin-dir "/pkg-config"))
                   (setenv "PKG_CONFIG" (string-append bin-dir "/pkg-config")))
                 (when (and cross-pkg-path (not (string-null? cross-pkg-path)))
                   (setenv "PKG_CONFIG_PATH" cross-pkg-path))
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display ,%samba-cross-answers)))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
                       (setenv "CFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (setenv "CPPFLAGS" "-D_FILE_OFFSET_BITS=64")
                       (invoke "sh" "./configure"
                               (string-append "--prefix=" out)
                               (string-append "--with-modulesdir=" out "/lib/ldb/modules")
                               "--cross-compile"
                               "--cross-answers=cross-answers.txt"
                               "--bundled-libraries=NONE"
                               "--without-ldb-lmdb"
                               "--disable-python"))
                     (invoke "sh" "./configure"
                             (string-append "--prefix=" out)
                             (string-append "--with-modulesdir=" out "/lib/ldb/modules")
                             "--bundled-libraries=NONE")))))))))))

(define talloc-fixed
  (talloc-fixed-proc talloc))

(define tdb-fixed
  (tdb-fixed-proc tdb))

(define tevent-fixed
  (tevent-fixed-proc tevent))

(define ldb-fixed
  (ldb-fixed-proc ldb))

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
  #:use-module (gnu packages databases)
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

(define (talloc-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (append python-wrapper which)))
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
               (let ((out (assoc-ref outputs "out"))
                     (py (or (which "python3") (which "python"))))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display "")))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
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
       (append python-wrapper which)))
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
               (let ((out (assoc-ref outputs "out"))
                     (py (or (which "python3") (which "python"))))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display "")))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
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
       (append python-wrapper which)))
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
               (let ((out (assoc-ref outputs "out"))
                     (py (or (which "python3") (which "python"))))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display "")))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
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
       (append python-wrapper which)))
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
               (let ((out (assoc-ref outputs "out"))
                     (py (or (which "python3") (which "python"))))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (if target
                     (begin
                       (with-output-to-file "cross-answers.txt"
                         (lambda () (display "")))
                       (setenv "CC" (string-append target "-gcc"))
                       (setenv "AR" (string-append target "-ar"))
                       (setenv "RANLIB" (string-append target "-ranlib"))
                       (setenv "HOSTCC" "gcc")
                       (invoke "sh" "./configure"
                               (string-append "--prefix=" out)
                               "--cross-compile"
                               "--cross-answers=cross-answers.txt"
                               "--bundled-libraries=NONE"
                               "--disable-python"))
                     (invoke "sh" "./configure"
                             (string-append "--prefix=" out)
                             "--bundled-libraries=NONE")))))))))))

(define talloc-fixed
  (talloc-fixed-proc talloc))

(define tdb-fixed
  (tdb-fixed-proc tdb))

(define tevent-fixed
  (tevent-fixed-proc tevent))

(define ldb-fixed
  (ldb-fixed-proc ldb))

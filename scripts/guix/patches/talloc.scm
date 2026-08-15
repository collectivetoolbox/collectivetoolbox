;;; Patch for talloc, tevent, ldb cross-compilation with native python.
;;; Copyright 2026 Collective Toolbox contributors
;;; This Scheme program is free software; you can redistribute it and/or modify it
;;; under the terms of the GNU General Public License as published by
;;; the Free Software Foundation; either version 3 of the License, or (at
;;; your option) any later version.
;;;
;;; This Scheme program is distributed in the hope that it will be useful, but
;;; WITHOUT ANY WARRANTY; without even the implied warranty of
;;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;;; GNU General Public License for more details.
;;;
;;; You should have received a copy of the GNU General Public License
;;; along with this Scheme program.  If not, see <http://www.gnu.org/licenses/>.

(define-module (patches talloc)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (gnu packages base)
  #:use-module (gnu packages python)
  #:use-module (gnu packages samba)
  #:export (talloc-fixed-proc
            talloc-fixed
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
           (replace 'configure
             (lambda* (#:key outputs #:allow-other-keys)
               (let ((out (assoc-ref outputs "out"))
                     (py (or (which "python3") (which "python"))))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (invoke "sh" "./configure"
                         (string-append "--prefix=" out)))))))))))

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
           (replace 'configure
             (lambda* (#:key outputs #:allow-other-keys)
               (let ((out (assoc-ref outputs "out"))
                     (py (or (which "python3") (which "python"))))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (invoke "sh" "./configure"
                         (string-append "--prefix=" out)
                         "--bundled-libraries=NONE"))))))))))

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
           (replace 'configure
             (lambda* (#:key outputs #:allow-other-keys)
               (let ((out (assoc-ref outputs "out"))
                     (py (or (which "python3") (which "python"))))
                 (setenv "CONFIG_SHELL" (which "sh"))
                 (setenv "PYTHON" py)
                 (invoke "sh" "./configure"
                         (string-append "--prefix=" out)
                         "--bundled-libraries=NONE"))))))))))

(define talloc-fixed
  (talloc-fixed-proc talloc))

(define tevent-fixed
  (tevent-fixed-proc tevent))

(define ldb-fixed
  (ldb-fixed-proc ldb))

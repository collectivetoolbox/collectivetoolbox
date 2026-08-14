;;; Patch for fltk to fix %output unbound variable error in cross-compilation.
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

(define-module (patches fltk)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (ice-9 match)
  #:use-module (gnu packages fltk)
  #:export (fltk-fixed-proc fltk-1.3-fixed fltk-fixed))

(define (fltk-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (map (match-lambda
            ((name (? package? p))
             (list name ((@ (patches) apply-patches) p)))
            ((name (? package? p) output)
             (list name ((@ (patches) apply-patches) p) output))
            (other other))
          (package-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(list "--enable-shared"
                (string-append "DSOFLAGS=-Wl,-rpath=" #$output "/lib")
                (string-append "PKGCONFIG="
                               (or (which "i686-linux-gnu-pkg-config")
                                   (which "pkg-config")
                                   "pkg-config"))))
       ((#:phases phases)
        #~(modify-phases #$phases
            (add-before 'configure 'symlink-pkg-config
              (lambda _
                (let ((pkg-config (or (which "i686-linux-gnu-pkg-config")
                                      (which "pkg-config"))))
                  (when pkg-config
                    (let ((bin-dir (string-append (getcwd) "/ctb-bin")))
                      (mkdir-p bin-dir)
                      (symlink pkg-config (string-append bin-dir "/pkg-config"))
                      (setenv "PATH" (string-append bin-dir ":" (getenv "PATH"))))))))))))))

(define fltk-1.3-fixed
  (fltk-fixed-proc fltk-1.3))

(define fltk-fixed
  (fltk-fixed-proc fltk))



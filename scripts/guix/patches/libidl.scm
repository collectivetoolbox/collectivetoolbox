;;; Patch for libidl to bypass cross-compilation AC_RUN_IFELSE checks.
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

(define-module (patches libidl)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnome)
  #:export (libidl-fixed-proc libidl-fixed))

(define (libidl-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'fix-cross-compilation
              (lambda _
                (substitute* "configure"
                  (("if test \"\\$cross_compiling\" = yes; then" all)
                   "if false; then"))
                (setenv "libIDL_cv_long_long_format" "ll")))))))))

(define libidl-fixed
  (libidl-fixed-proc libidl))

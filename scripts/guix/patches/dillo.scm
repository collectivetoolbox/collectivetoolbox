;;; Patch for Dillo web browser cross-compilation.
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

(define-module (patches dillo)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (ice-9 match)
  #:use-module (gnu packages web-browsers)
  #:export (dillo-fixed-proc dillo-fixed))

(define (dillo-fixed-proc pkg)
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
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'set-fltk-config
              (lambda* (#:key inputs #:allow-other-keys)
                (let ((fltk (assoc-ref inputs "fltk")))
                  (when fltk
                    (let ((fltk-config (string-append fltk "/bin/fltk-config")))
                      (when (file-exists? fltk-config)
                        (setenv "FLTK_CONFIG" fltk-config)
                        (setenv "PATH" (string-append fltk "/bin:" (getenv "PATH")))))))))))))))

(define dillo-fixed
  (dillo-fixed-proc dillo))

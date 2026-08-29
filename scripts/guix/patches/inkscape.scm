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

;;; Patch for inkscape to fix wrap-program when GI_TYPELIB_PATH is unset and disable tests.

(define-module (patches inkscape)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages inkscape)
  #:export (inkscape-fixed-proc inkscape-fixed))

(define (inkscape-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (delete 'check)
            (replace 'wrap-program
              (lambda* (#:key inputs outputs #:allow-other-keys)
                (let ((out (assoc-ref outputs "out"))
                      (python (false-if-exception (search-input-file inputs "bin/python")))
                      (gi-path (getenv "GI_TYPELIB_PATH"))
                      (pythonpath (getenv "GUIX_PYTHONPATH"))
                      (pixbuf-file (getenv "GDK_PIXBUF_MODULE_FILE")))
                  (apply wrap-program (string-append (or out #$output) "/bin/inkscape")
                         `(,@(if python `(("PATH" prefix (,(dirname python)))) '())
                           ,@(if pythonpath `(("GUIX_PYTHONPATH" prefix (,pythonpath))) '())
                           ,@(if pixbuf-file `(("GDK_PIXBUF_MODULE_FILE" = (,pixbuf-file))) '())
                           ,@(if gi-path `(("GI_TYPELIB_PATH" ":" prefix (,gi-path))) '()))))))))))))

(define inkscape-fixed
  (inkscape-fixed-proc inkscape))

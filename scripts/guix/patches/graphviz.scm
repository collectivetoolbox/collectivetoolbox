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

;;; Patch for graphviz to safely handle optional guile bindings phase.

(define-module (patches graphviz)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages graphviz)
  #:export (graphviz-fixed-proc
            graphviz-fixed))

(define (graphviz-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:phases phases)
        #~(modify-phases #$phases
            (replace 'move-guile-bindings
              (lambda* (#:key outputs #:allow-other-keys)
                (let* ((out (assoc-ref outputs "out"))
                       (lib (string-append out "/lib"))
                       (src (string-append lib "/graphviz/guile/libgv_guile.so"))
                       (extdir (string-append lib "/guile/3.0/extensions")))
                  (when (file-exists? src)
                    (mkdir-p extdir)
                    (rename-file src (string-append extdir "/libgv_guile.so"))))))))))))

(define graphviz-fixed
  (graphviz-fixed-proc graphviz))

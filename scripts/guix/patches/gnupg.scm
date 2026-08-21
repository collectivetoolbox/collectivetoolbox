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

;;; Patch for gnupg cross-compilation npth detection.

(define-module (patches gnupg)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnupg)
  #:export (gnupg-fixed-proc gnupg-fixed))

(define (gnupg-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'fix-npth-check
              (lambda* (#:key inputs #:allow-other-keys)
                (let ((npth (assoc-ref inputs "npth")))
                  (substitute* "configure"
                    (("have_npth=no")
                     "have_npth=yes")
                    (("NPTH_CFLAGS=\"\"")
                     (format #f "NPTH_CFLAGS=\"-I~a/include\"" npth))
                    (("NPTH_LIBS=\"\"")
                     (format #f "NPTH_LIBS=\"-L~a/lib -lnpth\"" npth))))))))))))

(define gnupg-fixed #f)

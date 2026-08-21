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

;;; Patch for cairo to ensure libdir is lib.

(define-module (patches cairo)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gtk)
  #:export (cairo-fixed-proc cairo-fixed))

(define (cairo-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "poppler" "libspectre" "ghostscript")))
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(cons* "--libdir=lib"
                 "-Dspectre=disabled"
                 "-Dtests=disabled"
                 "-Dgtk_doc=false"
                 #$flags))))))

(define cairo-fixed
  (cairo-fixed-proc cairo))


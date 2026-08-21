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

;;; Patch for dconf to disable bash_completion, man, gtk_doc, and vapi when cross-compiling.

(define-module (patches dconf)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages bash)
  #:use-module (gnu packages gnome)
  #:use-module (ice-9 match)
  #:export (dconf-fixed-proc dconf-fixed))

(define (dconf-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (cons (list "bash-minimal" bash-minimal)
           (package-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append (delete "-Dgtk_doc=true" #$flags)
                  '("-Dlibdir=lib"
                    "-Dbash_completion=false"
                    "-Dman=false"
                    "-Dgtk_doc=false"
                    "-Dvapi=false")))))))

(define dconf-fixed #f)

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

;;; Patch for pcsc-lite to configure libdir correctly for meson builds.

(define-module (patches pcsc-lite)
  #:use-module (guix packages)
  #:use-module (guix build-system)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:export (pcsc-lite-fixed-proc pcsc-lite-fixed))

(define (pcsc-lite-fixed-proc pkg)
  (if (eq? (build-system-name (package-build-system pkg)) 'meson)
      (package
        (inherit pkg)
        (arguments
         (substitute-keyword-arguments (package-arguments pkg)
           ((#:configure-flags flags #~'())
            #~(cons* "-Dlibdir=lib" #$flags)))))
      pkg))

(define pcsc-lite-fixed #f)

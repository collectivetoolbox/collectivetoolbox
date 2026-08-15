;;; Patch for gtkmm to disable display tests in container environments and set -Dlibdir=lib.
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

(define-module (patches gtkmm)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (gnu packages gtk)
  #:export (gtkmm-fixed-proc gtkmm-fixed))

(define (gtkmm-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags ''())
        `(append ,flags '("-Dlibdir=lib")))))))

(define gtkmm-fixed
  (gtkmm-fixed-proc gtkmm))

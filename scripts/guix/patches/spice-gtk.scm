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

;;; Patch for spice-gtk on i686-linux to ensure libraries are installed into lib/ instead of lib64/ and disable tests.

(define-module (patches spice-gtk)
  #:use-module (guix packages)
  #:use-module (guix gexp)
  #:use-module (gnu packages spice)
  #:export (spice-gtk-fixed))

(define spice-gtk-fixed
  (package
    (inherit spice-gtk)
    (arguments
      (cons* #:tests? #f
             #:configure-flags #~'("-Dlibdir=lib")
             (package-arguments spice-gtk)))))

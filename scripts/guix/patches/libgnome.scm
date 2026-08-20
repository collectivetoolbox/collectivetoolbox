;;; Patch for libgnome to set GCONFTOOL and disable schema installation when cross-compiling.
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

(define-module (patches libgnome)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (gnu packages gnome)
  #:use-module (ice-9 match)
  #:export (libgnome-fixed-proc libgnome-fixed))

(define (libgnome-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (append gconf)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags ''())
        `(append ,flags
                 '("ac_cv_path_GCONFTOOL=true"
                   "--disable-schemas-install"
                   "--with-gconf-source=xml::/etc/gconf/gconf.xml.defaults")))))))

(define libgnome-fixed #f)

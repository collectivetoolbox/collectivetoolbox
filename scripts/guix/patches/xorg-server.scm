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

;;; Patch for xorg-server cross-compilation and non-chroot builds.

(define-module (patches xorg-server)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages xorg)
  #:export (xorg-server-fixed-proc xorg-server-fixed))

(define (xorg-server-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-after 'unpack 'fix-xkb-install
              (lambda _
                (substitute* "xkb/Makefile.in"
                  (("^install-dist_xkbcompiledDATA:.*$")
                   "install-dist_xkbcompiledDATA:\n\t@true\n"))))))))))

(define xorg-server-fixed
  (xorg-server-fixed-proc xorg-server))

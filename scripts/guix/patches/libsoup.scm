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

;;; Patch for libsoup and libsoup-minimal cross-compilation with -Dlibdir=lib, disabled introspection/tls_check, and no samba dependency.

(define-module (patches libsoup)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnome)
  #:export (libsoup-fixed-proc
            libsoup-fixed
            libsoup-minimal-fixed-proc
            libsoup-minimal-fixed
            libsoup-minimal-2-fixed-proc
            libsoup-minimal-2-fixed))

(define (libsoup-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "samba")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  '("-Dlibdir=lib"
                    "-Dtls_check=false"
                    "-Dtests=false"
                    "-Dntlm=disabled"
                    "-Dintrospection=disabled"
                    "-Dvapi=disabled"
                    "-Dsysprof=disabled")))))))

(define libsoup-minimal-fixed-proc libsoup-fixed-proc)
(define libsoup-minimal-2-fixed-proc libsoup-fixed-proc)

(define libsoup-fixed (libsoup-fixed-proc libsoup))
(define libsoup-minimal-fixed (libsoup-fixed-proc libsoup-minimal))
(define libsoup-minimal-2-fixed (libsoup-fixed-proc libsoup-minimal-2))

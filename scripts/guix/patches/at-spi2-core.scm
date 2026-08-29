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

;;; Patch for at-spi2-core to disable introspection and docs when cross-compiling.

(define-module (patches at-spi2-core)
  #:use-module (guix packages)
  #:use-module (guix gexp)
  #:use-module (guix utils)
  #:use-module (gnu packages gtk)
  #:export (at-spi2-core-fixed-proc at-spi2-core-fixed))

(define (at-spi2-core-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags ''())
        #~(list "-Dintrospection=disabled"
                "-Ddocs=false"
                "--libdir=lib"
                (string-append "-Dc_link_args=-Wl,-rpath=" #$output "/lib")
                (string-append "-Dcpp_link_args=-Wl,-rpath=" #$output "/lib")))
       ((#:phases _ #~%standard-phases) #~%standard-phases)
       ((#:tests? _ #f) #f)))
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection"
               "python-dbusmock-for-tests"
               "python-pytest")))))

(define at-spi2-core-fixed
  (at-spi2-core-fixed-proc at-spi2-core))

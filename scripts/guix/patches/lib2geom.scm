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

;;; Patch for lib2geom to disable Boost Python and Cython bindings and tests.

(define-module (patches lib2geom)
  #:use-module (guix packages)
  #:use-module (guix gexp)
  #:use-module (guix utils)
  #:use-module (gnu packages graphics)
  #:export (lib2geom-fixed-proc lib2geom-fixed))

(define (lib2geom-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "python-pycairo")))
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "python-wrapper")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (delete 'patch-python-lib-install-path)))
       ((#:configure-flags flags #~'())
        #~(list "-D2GEOM_BUILD_SHARED=ON"
                "-D2GEOM_BOOST_PYTHON=OFF"
                "-D2GEOM_CYTHON_BINDINGS=OFF"
                "-D2GEOM_TESTING=OFF"
                "-DCMAKE_INSTALL_LIBDIR=lib"))
       ((#:tests? _ #f) #f)))))

(define lib2geom-fixed
  (lib2geom-fixed-proc lib2geom))

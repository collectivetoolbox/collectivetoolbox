;;; Patch for abseil-cpp to fix cross-compilation CMake flags.
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

(define-module (patches abseil-cpp)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages cpp)
  #:export (abseil-cpp-fixed-proc abseil-cpp-fixed))

(define (abseil-cpp-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(list "-DBUILD_SHARED_LIBS=ON"
                "-DABSL_BUILD_TESTING=OFF"
                "-DABSL_USE_EXTERNAL_GOOGLETEST=OFF"))))))

(define abseil-cpp-fixed
  (abseil-cpp-fixed-proc abseil-cpp))

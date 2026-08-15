;;; Patch for x265 cross-compilation.
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

(define-module (patches x265)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages video)
  #:export (x265-fixed-proc x265-fixed))

(define (x265-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(list "-DENABLE_PIC=TRUE"
                "-DENABLE_ASSEMBLY=OFF"
                "-DLINKED_10BIT=OFF"
                "-DLINKED_12BIT=OFF"
                "-DENABLE_CLI=OFF"
                "-DENABLE_SHARED=ON"
                (string-append "-DCMAKE_INSTALL_PREFIX=" #$output)))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (delete 'build-10-bit)
            (delete 'build-12-bit)))))))

(define x265-fixed
  (x265-fixed-proc x265))

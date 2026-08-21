;;; Patch for srt cross-compilation with unit tests disabled.
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

(define-module (patches srt)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages networking)
  #:export (srt-fixed-proc srt-fixed))

(define (srt-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags ''())
        #~(list (string-append "-DCMAKE_INSTALL_BINDIR=" #$output "/bin")
                "-DCMAKE_INSTALL_INCLUDEDIR=include"
                "-DENABLE_STATIC=OFF"
                "-DENABLE_UNITTESTS=OFF"))))))

(define srt-fixed
  (srt-fixed-proc srt))

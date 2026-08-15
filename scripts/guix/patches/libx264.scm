;;; Patch for libx264 to supply config in native-inputs and --cross-prefix.
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

(define-module (patches libx264)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages autotools)
  #:use-module (gnu packages video)
  #:export (libx264-fixed-proc libx264-fixed))

(define (libx264-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (cons `("config" ,config)
           (package-native-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(append #$flags '("--cross-prefix=i686-linux-gnu-")))))))

(define libx264-fixed
  (libx264-fixed-proc libx264))

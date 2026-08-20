;;; Patch for pulseaudio to disable tests, doxygen, and man when cross-compiling, and install into lib/.
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

(define-module (patches pulseaudio)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages pulseaudio)
  #:use-module (ice-9 match)
  #:export (pulseaudio-fixed-proc pulseaudio-fixed))

(define (pulseaudio-fixed-proc pkg)
  (package
    (inherit pkg)

    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  '("-Dlibdir=lib"
                    "-Dtests=false"
                    "-Ddoxygen=false"
                    "-Dman=false")))))))

(define pulseaudio-fixed #f)

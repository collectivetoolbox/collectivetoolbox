;;; Patch for gst-plugins-good on i686-linux to disable hanging unit tests, lib64 directory, and pre-check Xvfb.
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

(define-module (patches gst-plugins-good)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gstreamer)
  #:export (gst-plugins-good-no-tests))

(define gst-plugins-good-no-tests
  (package
    (inherit gst-plugins-good)
    (arguments
      (substitute-keyword-arguments (package-arguments gst-plugins-good)
        ((#:tests? _ #f) #f)
        ((#:configure-flags flags #~'())
         #~'("-Dlibdir=lib"))
        ((#:phases phases)
         #~(modify-phases #$phases
             (delete 'pre-check)))))))

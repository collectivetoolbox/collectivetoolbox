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

;;; Patch for gstreamer to handle optional doc cache generator and disable tests during cross-compilation.

(define-module (patches gstreamer)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gstreamer)
  #:use-module (ice-9 match)
  #:export (gstreamer-fixed-proc gstreamer-fixed))

(define (gstreamer-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(cons* "--libdir=lib"
                 "-Dintrospection=disabled"
                 "-Ddoc=disabled"
                 "-Dexamples=disabled"
                 "-Dtests=disabled"
                 #$flags))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases %standard-phases
            (add-after 'patch-shebangs 'do-not-capture-python
              (lambda _
                (let ((script (string-append
                               #$output "/libexec/gstreamer-1.0/"
                               "gst-plugins-doc-cache-generator")))
                  (when (file-exists? script)
                    (substitute* script
                      (((which "python3"))
                       "/usr/bin/env python3"))))))))))))

(define gstreamer-fixed #f)

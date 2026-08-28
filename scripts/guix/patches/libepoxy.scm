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

;;; Patch for libepoxy to ensure resilient Mesa library resolution and flags.

(define-module (patches libepoxy)
  #:use-module (gnu packages gl)
  #:use-module (guix gexp)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:export (libepoxy-fixed-proc libepoxy-fixed))

(define-public (libepoxy-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:validate-runpath? _ #t) #f)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(cons*
           "--libdir=lib"
           "-Ddocs=false"
           "-Dtests=false"
           #$flags))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (replace 'patch-paths
              (lambda* (#:key inputs #:allow-other-keys)
                (define (find-mesa-lib file)
                  (or (false-if-exception (search-input-file inputs (string-append "lib/" file)))
                      (false-if-exception (search-input-file inputs (string-append "lib64/" file)))
                      file))
                (substitute* (find-files "." "\\.[ch]$")
                  (("libGL\\.so\\.1") (find-mesa-lib "libGL.so.1"))
                  (("libEGL\\.so\\.1") (find-mesa-lib "libEGL.so.1"))
                  (("libGLESv1_CM\\.so\\.1") (find-mesa-lib "libGLESv1_CM.so.1"))
                  (("libGLESv2\\.so\\.2") (find-mesa-lib "libGLESv2.so.2")))))))))))

(define-public libepoxy-fixed #f)

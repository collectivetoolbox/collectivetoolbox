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

;;; Patch for json-glib cross-compilation with fixed libdir and disabled introspection.

(define-module (patches json-glib)
  #:use-module (guix packages)
  #:use-module (guix gexp)
  #:use-module (guix utils)
  #:use-module (gnu packages gnome)
  #:export (json-glib-fixed-proc json-glib-fixed))

(define (get-keyword kw args default)
  (let loop ((rest args))
    (cond ((null? rest) default)
          ((null? (cdr rest)) default)
          ((eq? (car rest) kw) (cadr rest))
          (else (loop (cddr rest))))))

(define (json-glib-fixed-proc pkg)
  (let ((has-custom-phases? (get-keyword #:phases (package-arguments pkg) #f)))
    (package
      (inherit pkg)
      (arguments
       (if has-custom-phases?
           (substitute-keyword-arguments (package-arguments pkg)
             ((#:configure-flags flags ''())
              #~(list "-Dintrospection=disabled"
                      "-Dman=false"
                      "-Dgtk_doc=disabled"
                      "-Dtests=false"
                      "--libdir=lib"
                      (string-append "-Dc_link_args=-Wl,-rpath=" #$output "/lib")))
             ((#:phases phases)
              #~(modify-phases #$phases
                  (replace 'move-docs
                    (lambda* (#:key outputs #:allow-other-keys)
                      (let ((out (assoc-ref outputs "out"))
                            (doc (assoc-ref outputs "doc")))
                        (when (and doc (file-exists? (string-append out "/share/doc")))
                          (copy-recursively (string-append out "/share/doc")
                                            (string-append doc "/share/doc"))
                          (delete-file-recursively (string-append out "/share/doc"))))))))
             ((#:tests? _ #f) #f))
           (substitute-keyword-arguments (package-arguments pkg)
             ((#:configure-flags flags ''())
              #~(list "-Dintrospection=disabled"
                      "-Dman=false"
                      "-Dgtk_doc=disabled"
                      "-Dtests=false"
                      "--libdir=lib"
                      (string-append "-Dc_link_args=-Wl,-rpath=" #$output "/lib")))
             ((#:tests? _ #f) #f))))
      (native-inputs
       (modify-inputs (package-native-inputs pkg)
         (delete "gobject-introspection"))))))

(define json-glib-fixed
  (json-glib-fixed-proc json-glib-minimal))

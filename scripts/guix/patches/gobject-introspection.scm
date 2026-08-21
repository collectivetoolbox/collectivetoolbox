;;; Stub for gobject-introspection to prevent target cross-compilation of Python/pyproject tools.
;;; Copyright 2026 Collective Toolbox contributors
;;; This Scheme program is free software; you can redistribute it and/or modify it
;;; under the terms of the GNU General Public License as published by
;;; the Free Software Foundation; either version 3 of the License, or (at
;;; your option) any later version.

(define-module (patches gobject-introspection)
  #:use-module (guix packages)
  #:use-module (guix build-system trivial)
  #:use-module (gnu packages glib)
  #:export (gobject-introspection-fixed-proc
            gobject-introspection-fixed))

(define (gobject-introspection-fixed-proc pkg)
  (package
    (inherit pkg)
    (build-system trivial-build-system)
    (inputs '())
    (native-inputs '())
    (propagated-inputs '())
    (arguments
     `(#:modules ((guix build utils))
       #:builder
       (begin
         (use-modules (guix build utils))
         (let ((out (assoc-ref %outputs "out")))
           (mkdir-p (string-append out "/bin"))
           (mkdir-p (string-append out "/lib"))
           (mkdir-p (string-append out "/share"))
           #t))))))

(define gobject-introspection-fixed
  (gobject-introspection-fixed-proc gobject-introspection))

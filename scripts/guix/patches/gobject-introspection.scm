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
         (let* ((out (assoc-ref %outputs "out"))
                (bin (string-append out "/bin"))
                (lib (string-append out "/lib"))
                (pkgconfig (string-append lib "/pkgconfig"))
                (share (string-append out "/share")))
           (mkdir-p bin)
           (mkdir-p pkgconfig)
           (mkdir-p (string-append share "/gir-1.0"))
           (mkdir-p (string-append share "/gobject-introspection-1.0"))

           ;; Create dummy g-ir binaries
           (for-each
             (lambda (name)
               (let ((file (string-append bin "/" name)))
                 (call-with-output-file file
                   (lambda (port)
                     (display "#!/bin/sh\nexit 0\n" port)))
                 (chmod file #o755)))
             '("g-ir-scanner" "g-ir-compiler" "g-ir-generate" "g-ir-inspect"))

           ;; Create pkg-config files
           (let ((pc-content
                   (string-append
                     "prefix=" out "\n"
                     "libdir=" lib "\n"
                     "includedir=" out "/include\n"
                     "bindir=" bin "\n"
                     "g_ir_scanner=" bin "/g-ir-scanner\n"
                     "g_ir_compiler=" bin "/g-ir-compiler\n"
                     "g_ir_generate=" bin "/g-ir-generate\n"
                     "gir_dir=" share "/gir-1.0\n"
                     "\n"
                     "Name: gobject-introspection\n"
                     "Description: GObject Introspection\n"
                     "Version: 1.86.0\n"
                     "Requires: glib-2.0, gobject-2.0\n"
                     "Cflags: -I${includedir}\n"
                     "Libs: -L${libdir}\n")))
             (call-with-output-file (string-append pkgconfig "/gobject-introspection-1.0.pc")
               (lambda (port) (display pc-content port)))
             (call-with-output-file (string-append pkgconfig "/gobject-introspection-no-export-1.0.pc")
               (lambda (port) (display pc-content port))))
           #t))))))

(define gobject-introspection-fixed
  (gobject-introspection-fixed-proc gobject-introspection))

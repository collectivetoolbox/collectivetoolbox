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

;;; Stub for gobject-introspection to prevent target cross-compilation of Python/pyproject tools.

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
    (inputs
     (list (@ (gnu packages bash) bash-minimal)))
    (native-inputs '())
    (propagated-inputs '())
    (arguments
     `(#:modules ((guix build utils))
       #:builder
       (begin
         (use-modules (guix build utils))
         (let* ((out (assoc-ref %outputs "out"))
                (bash (assoc-ref %build-inputs "bash-minimal"))
                (sh (string-append bash "/bin/sh"))
                (bin (string-append out "/bin"))
                (lib (string-append out "/lib"))
                (pkgconfig (string-append lib "/pkgconfig"))
                (share (string-append out "/share")))
           (mkdir-p bin)
           (mkdir-p pkgconfig)
           (mkdir-p (string-append share "/gir-1.0"))
           (mkdir-p (string-append share "/gobject-introspection-1.0"))

           ;; Create dummy g-ir binaries with proper interpreter and version response
           (for-each
             (lambda (name)
               (let ((file (string-append bin "/" name)))
                 (call-with-output-file file
                   (lambda (port)
                     (format port "#!~a\nif [ \"$1\" = \"--version\" ] || [ \"$1\" = \"-v\" ]; then\n  echo \"g-ir-scanner 1.86.0\"\nfi\nexit 0\n" sh)))
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

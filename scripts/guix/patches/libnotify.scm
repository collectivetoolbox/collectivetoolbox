;;; Patch for libnotify to disable introspection and doc generation when cross-compiling, and install into lib/.
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

(define-module (patches libnotify)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnome)
  #:use-module (ice-9 match)
  #:export (libnotify-fixed-proc libnotify-fixed))

(define (libnotify-fixed-proc pkg)
  (package
    (inherit pkg)

    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  '("-Dlibdir=lib"
                    "-Dintrospection=disabled"
                    "-Dgtk_doc=false"
                    "-Dtests=false"
                    "-Dman=false"
                    "-Ddocbook_docs=disabled")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (replace 'move-doc
              (lambda* (#:key outputs #:allow-other-keys)
                (let* ((out (assoc-ref outputs "out"))
                       (doc (assoc-ref outputs "doc"))
                       (old (string-append out "/share/doc"))
                       (new (and doc (string-append doc "/share/doc"))))
                  (when (and doc (file-exists? old))
                    (mkdir-p (dirname new))
                    (rename-file old new)))))))))))

(define libnotify-fixed #f)

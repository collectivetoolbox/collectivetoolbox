;;; Patch for libxkbcommon cross-compilation with -Dlibdir=lib, disabled docs, and robust symlink-pc.
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

(define-module (patches libxkbcommon)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages xdisorg)
  #:export (libxkbcommon-fixed-proc libxkbcommon-fixed))

(define (libxkbcommon-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  '("-Dlibdir=lib"
                    "-Denable-docs=false")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (delete 'move-doc)
            (replace 'symlink-pc
              (lambda* (#:key inputs outputs #:allow-other-keys)
                (let* ((out (assoc-ref outputs "out"))
                       (libxml2 (assoc-ref inputs "libxml2"))
                       (stem "/lib/pkgconfig/libxml-2.0.pc"))
                  (mkdir-p (string-append out "/lib/pkgconfig"))
                  (when (and libxml2 (file-exists? (string-append libxml2 stem)))
                    (symlink (string-append libxml2 stem)
                             (string-append out stem))))))))))))

(define libxkbcommon-fixed
  (libxkbcommon-fixed-proc libxkbcommon))

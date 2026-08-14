;;; Patch for freeglut to configure OpenGL legacy lookup in CMake.
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

(define-module (patches freeglut)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (ice-9 match)
  #:use-module (gnu packages gl)
  #:export (freeglut-fixed-proc freeglut-fixed))

(define (freeglut-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (map (match-lambda
            ((name (? package? p))
             (list name ((@ (patches) apply-patches) p)))
            ((name (? package? p) output)
             (list name ((@ (patches) apply-patches) p) output))
            (other other))
          (package-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-after 'unpack 'patch-opengl-lookup
              (lambda* (#:key inputs #:allow-other-keys)
                (let* ((mesa (assoc-ref inputs "mesa"))
                       (mesa-lib (and mesa
                                      (or (and (file-exists? (string-append mesa "/lib/libGL.so"))
                                               (string-append mesa "/lib/libGL.so"))
                                          (and (file-exists? (string-append mesa "/lib64/libGL.so"))
                                               (string-append mesa "/lib64/libGL.so"))
                                          (string-append mesa "/lib/libGL.so")))))
                  (substitute* "CMakeLists.txt"
                    (("FIND_PACKAGE\\(OpenGL REQUIRED\\)")
                     (if mesa-lib
                         (string-append "set(OPENGL_gl_LIBRARY \"" mesa-lib "\")\n"
                                        "set(OPENGL_INCLUDE_DIR \"" mesa "/include\")\n"
                                        "set(OPENGL_FOUND TRUE)")
                         "find_library(OPENGL_gl_LIBRARY GL)\nfind_path(OPENGL_INCLUDE_DIR GL/gl.h)\nset(OPENGL_FOUND TRUE)"))))))))))))

(define freeglut-fixed
  (freeglut-fixed-proc freeglut))

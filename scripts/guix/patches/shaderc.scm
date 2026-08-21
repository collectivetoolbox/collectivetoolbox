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

;;; Patch for shaderc cross-compilation avoiding (which "spirv-dis") failure.

(define-module (patches shaderc)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages vulkan)
  #:export (shaderc-fixed-proc shaderc-fixed))

(define (shaderc-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (replace 'do-not-look-for-bundled-sources
              (lambda _
                (substitute* "CMakeLists.txt"
                  (("add_subdirectory\\(third_party\\)")
                   ""))
                (substitute* "glslc/test/CMakeLists.txt"
                  (("\\$<TARGET_FILE:spirv-dis>")
                   (or (which "spirv-dis") "spirv-dis")))
                ;; Do not attempt to use git to encode version information.
                (substitute* "glslc/CMakeLists.txt"
                  (("add_dependencies\\(glslc_exe build-version\\)")
                   ""))
                (call-with-output-file "glslc/src/build-version.inc"
                  (lambda (port)
                    (format port "\"~a\"\n\"~a\"\n\"~a\"~%"
                            #$(package-version shaderc)
                            #$(package-version (@ (gnu packages vulkan) spirv-tools))
                            #$(package-version (@ (gnu packages vulkan) glslang)))))))))))))

(define shaderc-fixed
  (shaderc-fixed-proc shaderc))

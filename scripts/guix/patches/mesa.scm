;;; Mesa cross-build fixes for the local Guix overlay.
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

(define-module (patches mesa)
  #:use-module (gnu packages gl)
  #:use-module (guix gexp)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:export (mesa-libclc-pkg-config-fixed))

(define-public mesa-libclc-pkg-config-fixed
  (package
    (inherit mesa)
    (arguments
     (substitute-keyword-arguments (package-arguments mesa)
       ((#:phases phases)
        #~(modify-phases #$phases
            (add-before 'configure 'expose-cross-discovery-metadata
              (lambda* (#:key inputs native-inputs #:allow-other-keys)
                (define (prepend-env-path variable entry)
                  (when (and entry (file-exists? entry))
                    (let ((current (or (getenv variable) "")))
                      (setenv variable
                              (if (string=? current "")
                                  entry
                                  (string-append entry ":" current))))))

                (define (input-ref name)
                  (or (assoc-ref inputs name)
                      (assoc-ref native-inputs name)))

                (define (find-input-by-prefix prefix)
                  (let ((match (lambda (lst)
                                 (find (lambda (pair)
                                         (string-prefix? prefix (car pair)))
                                       lst))))
                    (or (and=> (match inputs) cdr)
                        (and=> (match native-inputs) cdr))))

                (let ((libclc (input-ref "libclc"))
                      (llvm (input-ref "llvm-for-mesa"))
                      (clang (or (input-ref "clang")
                                 (input-ref "clang-18")))
                      (spirv-tools (input-ref "spirv-tools"))
                      (llvm-spirv (input-ref "spirv-llvm-translator"))
                      (cross-gcc (or (input-ref "cross-gcc")
                                     (input-ref "gcc-cross")
                                     (find-input-by-prefix "cross-gcc")
                                     (find-input-by-prefix "gcc-cross")))
                      (cross-binutils (or (input-ref "cross-binutils")
                                          (input-ref "binutils-cross")
                                          (find-input-by-prefix "binutils-cross")
                                          (find-input-by-prefix "cross-binutils"))))
                  (when libclc
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append libclc "/share/pkgconfig")))
                  (when llvm
                    (let* ((llvm-overlay (string-append (getcwd) "/ctb-llvm-overlay"))
                        (llvm-bin (string-append llvm "/bin"))
                           (llvm-lib (string-append llvm "/lib"))
                        (overlay-bin (string-append llvm-overlay "/bin"))
                           (overlay-lib (string-append llvm-overlay "/lib"))
                           (overlay-include (string-append llvm-overlay "/include"))
                           (overlay-cmake (string-append overlay-lib "/cmake"))
                           (overlay-llvm-cmake (string-append overlay-cmake "/llvm")))
                      (mkdir-p overlay-bin)
                      (mkdir-p overlay-lib)
                      (mkdir-p overlay-include)
                      (invoke
                       "sh"
                       "-c"
                       (string-append
                        "for entry in " llvm-bin "/*; do "
                        "name=${entry##*/}; "
                        "ln -s \"$entry\" \"" overlay-bin "/$name\"; "
                        "done"))
                      (when cross-gcc
                        (let ((gcc-bin (string-append cross-gcc "/bin")))
                          (invoke
                           "sh"
                           "-c"
                           (string-append
                            "for entry in " gcc-bin "/*; do "
                            "name=${entry##*/}; "
                            "if [ ! -e \"" overlay-bin "/$name\" ]; then "
                            "ln -s \"$entry\" \"" overlay-bin "/$name\"; "
                            "fi; "
                            "done"))))
                      (when cross-binutils
                        (let ((cross-bin (string-append cross-binutils "/bin")))
                          (invoke
                           "sh"
                           "-c"
                           (string-append
                            "for entry in " cross-bin "/*; do "
                            "name=${entry##*/}; "
                            "if [ ! -e \"" overlay-bin "/$name\" ]; then "
                            "ln -s \"$entry\" \"" overlay-bin "/$name\"; "
                            "fi; "
                            "done"))))
                      (invoke
                       "sh"
                       "-c"
                       (string-append
                        "for entry in " llvm-lib "/*; do "
                        "name=${entry##*/}; "
                        "if [ \"$name\" != cmake ]; then "
                        "ln -s \"$entry\" \"" overlay-lib "/$name\"; "
                        "fi; "
                        "done"))
                      (invoke
                       "sh"
                       "-c"
                       (string-append
                        "for entry in " llvm "/include/*; do "
                        "name=${entry##*/}; "
                        "ln -s \"$entry\" \"" overlay-include "/$name\"; "
                        "done"))
                      (when clang
                        (let ((clang-lib (string-append clang "/lib"))
                              (clang-inc (string-append clang "/include")))
                          (invoke
                           "sh"
                           "-c"
                           (string-append
                            "for entry in " clang-lib "/*; do "
                            "name=${entry##*/}; "
                            "if [ \"$name\" != cmake ] && [ ! -e \"" overlay-lib "/$name\" ]; then "
                            "ln -s \"$entry\" \"" overlay-lib "/$name\"; "
                            "fi; "
                            "done"))
                          (invoke
                           "sh"
                           "-c"
                           (string-append
                            "for entry in " clang-inc "/*; do "
                            "name=${entry##*/}; "
                            "if [ ! -e \"" overlay-include "/$name\" ]; then "
                            "ln -s \"$entry\" \"" overlay-include "/$name\"; "
                            "fi; "
                            "done"))))
                      (copy-recursively (string-append llvm-lib "/cmake") overlay-cmake)
                      (chmod (string-append overlay-llvm-cmake "/LLVMConfig.cmake") #o644)
                      (chmod (string-append overlay-llvm-cmake "/LLVMExports-release.cmake") #o644)
                      (invoke
                       "sed"
                       "-E"
                       "-i"
                       "s@IMPORTED_LOCATION_RELEASE \"([^\"]+)\"@IMPORTED_LOCATION_RELEASE \"\\1\"\\n  IMPORTED_LOCATION \"\\1\"@g"
                       (string-append overlay-llvm-cmake "/LLVMExports-release.cmake"))
                      (let ((port (open-file
                                   (string-append overlay-llvm-cmake "/LLVMConfig.cmake")
                                   "a")))
                        (display
                         (string-append
                          "\nset(LLVM_TARGETS_TO_BUILD \"${LLVM_ALL_TARGETS}\")\n"
                          "set(LLVM_LIBRARY_DIR \"" overlay-lib "\")\n"
                          "set(LLVM_LIBRARY_DIRS \"" overlay-lib "\")\n"
                          "set(LLVM_IMPORTED_LOCATION_CTB \""
                          overlay-lib "/libLLVM.so.18.1\")\n"
                          "if(TARGET LLVM)\n"
                          "  set_target_properties(LLVM PROPERTIES IMPORTED_LOCATION \"${LLVM_IMPORTED_LOCATION_CTB}\")\n"
                          "endif()\n"
                          "if(TARGET llvm-tblgen)\n"
                          "  set_target_properties(llvm-tblgen PROPERTIES IMPORTED_LOCATION \""
                          overlay-bin "/llvm-tblgen\")\n"
                          "endif()\n")
                         port)
                        (close-port port))
                      (setenv "LLVM_DIR" overlay-llvm-cmake)
                      (prepend-env-path "CMAKE_PREFIX_PATH" llvm-overlay)
                      (prepend-env-path "PATH" overlay-bin)))
                  (when spirv-tools
                    (prepend-env-path "CMAKE_PREFIX_PATH" spirv-tools)
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append spirv-tools "/lib/pkgconfig"))
                    (prepend-env-path "C_INCLUDE_PATH" (string-append spirv-tools "/include"))
                    (prepend-env-path "CPLUS_INCLUDE_PATH" (string-append spirv-tools "/include"))
                    (prepend-env-path "CROSS_C_INCLUDE_PATH" (string-append spirv-tools "/include"))
                    (prepend-env-path "CROSS_CPLUS_INCLUDE_PATH" (string-append spirv-tools "/include"))
                    (prepend-env-path "CPATH" (string-append spirv-tools "/include")))
                  (when llvm-spirv
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append llvm-spirv "/lib/pkgconfig"))
                    (prepend-env-path "C_INCLUDE_PATH" (string-append llvm-spirv "/include"))
                    (prepend-env-path "CPLUS_INCLUDE_PATH" (string-append llvm-spirv "/include"))
                    (prepend-env-path "CROSS_C_INCLUDE_PATH" (string-append llvm-spirv "/include"))
                    (prepend-env-path "CROSS_CPLUS_INCLUDE_PATH" (string-append llvm-spirv "/include"))
                    (prepend-env-path "CPATH" (string-append llvm-spirv "/include"))))))
            (add-after 'expose-cross-discovery-metadata 'force-cmake-llvm-discovery
              (lambda _
                (substitute* "meson.build"
                  (("method : host_machine\\.system\\(\\) == 'windows' \\? 'auto' : 'config-tool',")
                   "method : 'cmake',")
                  (("dep_spirv_tools = dependency\\(")
                   "_unused_spirv_tools = dependency(")
                  (("required : with_clc,")
                   "required : false,")
                  (("if dep_spirv_tools\\.found\\(\\)")
                   "dep_spirv_tools = null_dep\nif false")
                  (("dep_llvmspirvlib = dependency\\('LLVMSPIRVLib', required : true")
                   "dep_llvmspirvlib = null_dep # dependency('LLVMSPIRVLib', required : false"))
                (substitute* "src/compiler/clc/meson.build"
                  (("and not has_spirv_link_workaround") "and false")
                  (("if has_spirv_link_workaround") "if true"))))
            (add-before 'build 'preserve-cross-path
              (lambda _
                (let ((overlay-bin (string-append (getcwd) "/ctb-llvm-overlay/bin")))
                  (when (file-exists? overlay-bin)
                    (let ((current (or (getenv "PATH") "")))
                      (setenv "PATH" (string-append overlay-bin ":" current)))))))))))))
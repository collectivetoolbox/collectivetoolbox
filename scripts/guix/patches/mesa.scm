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

                (let ((libclc (input-ref "libclc"))
                      (llvm (input-ref "llvm-for-mesa"))
                      (spirv-tools (input-ref "spirv-tools"))
                      (llvm-spirv (input-ref "spirv-llvm-translator")))
                  ;; Cross dependency discovery for Mesa currently misses some
                  ;; Guix-provided metadata unless we expose the relevant build
                  ;; prefixes explicitly.
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
                           (overlay-cmake (string-append overlay-lib "/cmake"))
                           (overlay-llvm-cmake (string-append overlay-cmake "/llvm")))
                      (mkdir-p overlay-bin)
                      (mkdir-p overlay-lib)
                      (invoke
                       "sh"
                       "-c"
                       (string-append
                        "for entry in " llvm-bin "/*; do "
                        "name=${entry##*/}; "
                        "ln -s \"$entry\" \"" overlay-bin "/$name\"; "
                        "done"))
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
                          "\nset(LLVM_IMPORTED_LOCATION_CTB \""
                          overlay-lib "/libLLVM.so.18.1\")\n"
                          "if(TARGET LLVM)\n"
                          "  set_target_properties(LLVM PROPERTIES IMPORTED_LOCATION \"LLVM_IMPORTED_LOCATION_CTB\")\n"
                          "endif()\n"
                          "if(TARGET llvm-tblgen)\n"
                          "  set_target_properties(llvm-tblgen PROPERTIES IMPORTED_LOCATION \""
                          overlay-bin "/llvm-tblgen\")\n"
                          "endif()\n")
                         port)
                        (close-port port))
                      (setenv "LLVM_DIR" overlay-llvm-cmake)
                      (prepend-env-path "CMAKE_PREFIX_PATH" llvm-overlay)
                      (prepend-env-path "PATH" (string-append llvm "/bin"))))
                  (when spirv-tools
                    (prepend-env-path "CMAKE_PREFIX_PATH" spirv-tools)
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append spirv-tools "/lib/pkgconfig")))
                  (when llvm-spirv
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append llvm-spirv "/lib/pkgconfig"))))))
            (add-after 'expose-cross-discovery-metadata 'force-cmake-llvm-discovery
              (lambda _
                (substitute* "meson.build"
                  (("method : host_machine\\.system\\(\\) == 'windows' \\? 'auto' : 'config-tool',")
                   "method : 'cmake',"))))))))))
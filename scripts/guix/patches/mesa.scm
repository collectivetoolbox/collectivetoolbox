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
  #:use-module (ice-9 ftw)
  #:export (mesa-libclc-pkg-config-fixed-proc
            mesa-libclc-pkg-config-fixed))

(define-public (mesa-libclc-pkg-config-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:validate-runpath? _ #t) #f)
       ((#:configure-flags flags #~'())
        #~(cons*
           "-Damd-use-llvm=false"
           "-Dllvm=disabled"
           (map (lambda (flag)
                  (cond
                   ((string-prefix? "-Dgallium-drivers=" flag)
                    "-Dgallium-drivers=crocus,i915,r300,nouveau,virgl,svga,softpipe,zink")
                   ((string-prefix? "-Dvulkan-drivers=" flag)
                    "-Dvulkan-drivers=intel_hasvk,virtio")
                   ((string-prefix? "-Dllvm=" flag)
                    "-Dllvm=disabled")
                   (else flag)))
                #$flags)))
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

                (define (symlink-dir-contents src dest)
                  (when (and src (file-exists? src))
                    (for-each
                     (lambda (entry)
                       (unless (member entry '("." ".."))
                         (let ((src-path (string-append src "/" entry))
                               (dest-path (string-append dest "/" entry)))
                           (unless (file-exists? dest-path)
                             (symlink src-path dest-path)))))
                     (or ((@ (ice-9 ftw) scandir) src) '()))))

                (let ((libclc (or (input-ref "libclc")
                                  (find-input-by-prefix "libclc")))
                      (llvm (or (input-ref "llvm-for-mesa")
                                (input-ref "llvm")
                                (find-input-by-prefix "llvm")))
                      (spirv-tools (or (input-ref "spirv-tools")
                                       (find-input-by-prefix "spirv-tools")))
                      (llvm-spirv (or (input-ref "spirv-llvm-translator")
                                      (input-ref "llvm-spirv")
                                      (find-input-by-prefix "spirv-llvm-translator")
                                      (find-input-by-prefix "llvm-spirv")))
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
                     (string-append libclc "/share/pkgconfig"))
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append libclc "/lib/pkgconfig")))
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
                      (symlink-dir-contents llvm-bin overlay-bin)
                      (when cross-gcc
                        (symlink-dir-contents (string-append cross-gcc "/bin") overlay-bin))
                      (when cross-binutils
                        (symlink-dir-contents (string-append cross-binutils "/bin") overlay-bin))
                      (when (file-exists? llvm-lib)
                        (for-each
                         (lambda (entry)
                           (unless (or (member entry '("." "..")) (string=? entry "cmake"))
                             (let ((src-path (string-append llvm-lib "/" entry))
                                   (dest-path (string-append overlay-lib "/" entry)))
                               (unless (file-exists? dest-path)
                                 (symlink src-path dest-path)))))
                         (or ((@ (ice-9 ftw) scandir) llvm-lib) '())))
                      (symlink-dir-contents (string-append llvm "/include") overlay-include)
                      (copy-recursively (string-append llvm-lib "/cmake") overlay-cmake)
                      (chmod (string-append overlay-llvm-cmake "/LLVMConfig.cmake") #o644)
                      (chmod (string-append overlay-llvm-cmake "/LLVMExports-release.cmake") #o644)
                      (substitute* (string-append overlay-llvm-cmake "/LLVMExports-release.cmake")
                        (("IMPORTED_LOCATION_RELEASE \"([^\"]+)\"" _ path)
                         (string-append "IMPORTED_LOCATION_RELEASE \"" path "\"\n  IMPORTED_LOCATION \"" path "\"")))
                      (let ((port (open-file
                                   (string-append overlay-llvm-cmake "/LLVMConfig.cmake")
                                   "a")))
                        (display
                          (string-append
                           "\nset(LLVM_TARGETS_TO_BUILD \"${LLVM_ALL_TARGETS}\")\n"
                           "set(LLVM_LIBRARY_DIR \"" llvm-lib "\")\n"
                           "set(LLVM_LIBRARY_DIRS \"" llvm-lib "\")\n"
                           "set(LLVM_IMPORTED_LOCATION_CTB \""
                           llvm-lib "/libLLVM.so.18.1\")\n"
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
                  (("with_driver_using_cl = \\[")
                   "with_driver_using_cl = false\n_unused_driver_cl = [")
                  (("dep_clc = dependency\\('libclc'\\)")
                   "dep_clc = dependency('libclc', native : true, required : false)")
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

(define-public mesa-libclc-pkg-config-fixed
  (mesa-libclc-pkg-config-fixed-proc mesa))
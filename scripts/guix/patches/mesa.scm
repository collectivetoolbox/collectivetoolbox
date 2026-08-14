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
           "-Dllvm=enabled"
           (map (lambda (flag)
                  (cond
                   ((string-prefix? "-Dgallium-drivers=" flag)
                    "-Dgallium-drivers=crocus,i915,r300,nouveau,virgl,svga,llvmpipe,softpipe,zink")
                   ((string-prefix? "-Dvulkan-drivers=" flag)
                    "-Dvulkan-drivers=swrast,intel_hasvk,virtio")
                   ((string-prefix? "-Dllvm=" flag)
                    "-Dllvm=enabled")
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

                (define (entry->path entry)
                  (cond
                   ((not (pair? entry)) #f)
                   ((string? (cdr entry)) (cdr entry))
                   ((and (pair? (cdr entry)) (string? (cadr entry))) (cadr entry))
                   (else #f)))

                (define (input-ref name)
                  (let ((extract (lambda (val)
                                   (cond
                                    ((string? val) val)
                                    ((and (pair? val) (string? (car val))) (car val))
                                    ((and (pair? val) (string? (cdr val))) (cdr val))
                                    ((and (pair? val) (pair? (cdr val)) (string? (cadr val))) (cadr val))
                                    (else #f)))))
                    (or (and (list? inputs) (and=> (assoc-ref inputs name) extract))
                        (and (list? native-inputs) (and=> (assoc-ref native-inputs name) extract)))))

                (define (find-input-by-prefix prefix)
                  (let ((match (lambda (lst)
                                 (and (list? lst)
                                      (find (lambda (entry)
                                              (and (pair? entry)
                                                   (string? (car entry))
                                                   (string-prefix? prefix (car entry))))
                                            lst)))))
                    (or (and=> (match inputs) entry->path)
                        (and=> (match native-inputs) entry->path))))

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
                           (wrapper-script (string-append overlay-bin "/llvm-config")))
                      (mkdir-p overlay-bin)
                      (mkdir-p overlay-lib)
                      (mkdir-p overlay-include)
                      (symlink-dir-contents llvm-bin overlay-bin)
                      (when cross-gcc
                        (symlink-dir-contents (string-append cross-gcc "/bin") overlay-bin))
                      (when cross-binutils
                        (symlink-dir-contents (string-append cross-binutils "/bin") overlay-bin))
                      (symlink-dir-contents llvm-lib overlay-lib)
                      (symlink-dir-contents (string-append llvm "/include") overlay-include)
                      ;; Create shell-based llvm-config wrapper for cross host
                      (call-with-output-file wrapper-script
                        (lambda (p)
                          (format p "#!/bin/sh~%")
                          (format p "llvm_dir=~s~%" llvm)
                          (format p "res=\"\"~%")
                          (format p "for arg in \"$@\"; do~%")
                          (format p "  case \"$arg\" in~%")
                          (format p "    --version) res=\"$res 18.1.8\" ;;~%")
                          (format p "    --prefix) res=\"$res $llvm_dir\" ;;~%")
                          (format p "    --bindir) res=\"$res $llvm_dir/bin\" ;;~%")
                          (format p "    --includedir) res=\"$res $llvm_dir/include\" ;;~%")
                          (format p "    --libdir) res=\"$res $llvm_dir/lib\" ;;~%")
                          (format p "    --cppflags|--cflags|--cxxflags) res=\"$res -I$llvm_dir/include -D_GNU_SOURCE -D__STDC_CONSTANT_MACROS -D__STDC_FORMAT_MACROS -D__STDC_LIMIT_MACROS\" ;;~%")
                          (format p "    --ldflags) res=\"$res -L$llvm_dir/lib -Wl,-rpath,$llvm_dir/lib\" ;;~%")
                          (format p "    --libs|--libfiles|--libnames) res=\"$res -L$llvm_dir/lib -lLLVM-18\" ;;~%")
                          (format p "    --system-libs) res=\"$res -lz -lzstd -lm\" ;;~%")
                          (format p "    --shared-mode) res=\"$res shared\" ;;~%")
                          (format p "    --has-rtti) res=\"$res YES\" ;;~%")
                          (format p "    --targets-built) res=\"$res X86\" ;;~%")
                          (format p "    --host-target) res=\"$res i686-unknown-linux-gnu\" ;;~%")
                          (format p "    --build-mode) res=\"$res Release\" ;;~%")
                          (format p "    --assertion-mode) res=\"$res OFF\" ;;~%")
                          (format p "    --components) res=\"$res all all-targets engine executionengine mc mcjit native orcjit target x86 x86asmparser x86codegen x86desc x86disassembler x86info\" ;;~%")
                          (format p "  esac~%")
                          (format p "done~%")
                          (format p "if [ -n \"$res\" ]; then echo \"$res\" | sed 's/^ //'; fi~%")))
                      (chmod wrapper-script #o755)
                      (symlink wrapper-script (string-append overlay-bin "/i686-linux-gnu-llvm-config"))
                      (symlink wrapper-script (string-append overlay-bin "/llvm-config-18"))
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
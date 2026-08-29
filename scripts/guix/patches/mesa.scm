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

;;; Mesa cross-build fixes for the local Guix overlay.

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
       ((#:tests? _ #f) #f)
       ((#:validate-runpath? _ #t) #f)
       ((#:configure-flags flags #~'())
        #~(cons*
           "--libdir=lib"
           #$(if (%current-target-system)
                 #~(cons*
                    "--cross-file=../mesa-26.0.2/mesa-llvm-cross.ini"
                    "-Damd-use-llvm=false"
                    "-Dllvm=enabled"
                    (map (lambda (flag)
                           (cond
                            ((string-prefix? "-Dgallium-drivers=" flag)
                             "-Dgallium-drivers=r300,nouveau,virgl,svga,llvmpipe,softpipe,zink")
                            ((string-prefix? "-Dvulkan-drivers=" flag)
                             "-Dvulkan-drivers=swrast,virtio")
                            ((string-prefix? "-Dllvm=" flag)
                             "-Dllvm=enabled")
                            (else flag)))
                         #$flags))
                 flags)))
       ((#:phases phases)
        (if (%current-target-system)
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

                    (define (find-store-input name)
                      (let* ((all-env (string-append (or (getenv "LIBRARY_PATH") "") ":"
                                                     (or (getenv "PATH") "") ":"
                                                     (or (getenv "CPATH") "") ":"
                                                     (or (getenv "PKG_CONFIG_PATH_FOR_BUILD") "") ":"
                                                     (or (getenv "PKG_CONFIG_PATH") "")))
                             (dirs (string-split all-env #\:))
                             (match (find (lambda (dir)
                                            (and (not (string-null? dir))
                                                 (string-contains dir name)
                                                 (file-exists? dir)))
                                          dirs)))
                        (and match
                             (let ((parent (string-append match "/..")))
                               (if (file-exists? parent)
                                   (canonicalize-path parent)
                                   match)))))

                    (let ((libclc (or (input-ref "libclc")
                                      (find-input-by-prefix "libclc")
                                      (find-store-input "libclc")))
                          (llvm (or (input-ref "llvm-for-mesa")
                                    (input-ref "llvm")
                                    (find-input-by-prefix "llvm-for-mesa")
                                    (find-store-input "llvm-for-mesa")
                                    (find-input-by-prefix "llvm")
                                    (find-store-input "llvm-18")
                                    (find-store-input "llvm")))
                          (spirv-tools (or (input-ref "spirv-tools")
                                           (find-input-by-prefix "spirv-tools")
                                           (find-store-input "spirv-tools")))
                          (llvm-spirv (or (input-ref "spirv-llvm-translator")
                                          (input-ref "llvm-spirv")
                                          (find-input-by-prefix "spirv-llvm-translator")
                                          (find-input-by-prefix "llvm-spirv")
                                          (find-store-input "spirv-llvm-translator")))
                          (cross-gcc (or (input-ref "cross-gcc")
                                         (input-ref "gcc-cross")
                                         (find-input-by-prefix "cross-gcc")
                                         (find-input-by-prefix "gcc-cross")
                                         (find-store-input "gcc-cross")))
                          (cross-binutils (or (input-ref "cross-binutils")
                                              (input-ref "binutils-cross")
                                              (find-input-by-prefix "binutils-cross")
                                              (find-input-by-prefix "cross-binutils")
                                              (find-store-input "binutils-cross"))))
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
                          (when (file-exists? wrapper-script)
                            (delete-file wrapper-script))
                          (call-with-output-file wrapper-script
                            (lambda (p)
                              (format p "#!/bin/sh\n")
                              (format p "llvm_dir=~s\n" llvm)
                              (format p "res=\"\"\n")
                              (format p "for arg in \"$@\"; do\n")
                              (format p "  case \"$arg\" in\n")
                              (format p "    --version) res=\"$res 18.1.8\" ;;\n")
                              (format p "    --prefix) res=\"$res $llvm_dir\" ;;\n")
                              (format p "    --bindir) res=\"$res $llvm_dir/bin\" ;;\n")
                              (format p "    --includedir) res=\"$res $llvm_dir/include\" ;;\n")
                              (format p "    --libdir) res=\"$res $llvm_dir/lib\" ;;\n")
                              (format p "    --cppflags|--cflags|--cxxflags) res=\"$res -I$llvm_dir/include -D_GNU_SOURCE -D__STDC_CONSTANT_MACROS -D__STDC_FORMAT_MACROS -D__STDC_LIMIT_MACROS\" ;;\n")
                              (format p "    --ldflags) res=\"$res -L$llvm_dir/lib -Wl,-rpath,$llvm_dir/lib\" ;;\n")
                              (format p "    --libs|--libfiles|--libnames) res=\"$res -L$llvm_dir/lib -lLLVM-18\" ;;\n")
                              (format p "    --system-libs) res=\"$res -lz -lzstd -lm\" ;;\n")
                              (format p "    --shared-mode) res=\"$res shared\" ;;\n")
                              (format p "    --has-rtti) res=\"$res YES\" ;;\n")
                              (format p "    --targets-built) res=\"$res X86\" ;;\n")
                              (format p "    --host-target) res=\"$res i686-unknown-linux-gnu\" ;;\n")
                              (format p "    --build-mode) res=\"$res Release\" ;;\n")
                              (format p "    --assertion-mode) res=\"$res OFF\" ;;\n")
                              (format p "    --components) res=\"$res aarch64 aarch64asmparser aarch64codegen aarch64desc aarch64disassembler aarch64info aarch64utils aggressiveinstcombine all all-targets amdgpu amdgpuasmparser amdgpucodegen amdgpudesc amdgpudisassembler amdgpuinfo amdgputargetmca amdgpuutils analysis arm armasmparser armcodegen armdesc armdisassembler arminfo armutils asmparser asmprinter avr avrasmparser avrcodegen avrdesc avrdisassembler avrinfo binaryformat bitreader bitstreamreader bitwriter bpf bpfasmparser bpfcodegen bpfdesc bpfdisassembler bpfinfo cfguard codegen codegentypes core coroutines coverage debuginfobtf debuginfocodeview debuginfodwarf debuginfogsym debuginfologicalview debuginfomsf debuginfopdb demangle dlltooldriver dwarflinker dwarflinkerclassic dwarflinkerparallel dwp engine executionengine extensions filecheck frontenddriver frontendhlsl frontendoffloading frontendopenacc frontendopenmp fuzzercli fuzzmutate globalisel hexagon hexagonasmparser hexagoncodegen hexagondesc hexagondisassembler hexagoninfo hipstdpar instcombine instrumentation interfacestub interpreter ipo irprinter irreader jitlink lanai lanaiasmparser lanaicodegen lanaidesc lanaidisassembler lanaiinfo libdriver lineeditor linker loongarch loongarchasmparser loongarchcodegen loongarchdesc loongarchdisassembler loongarchinfo lto mc mca mcdisassembler mcjit mcparser mips mipsasmparser mipscodegen mipsdesc mipsdisassembler mipsinfo mirparser msp430 msp430asmparser msp430codegen msp430desc msp430disassembler msp430info native nativecodegen nvptx nvptxcodegen nvptxdesc nvptxinfo objcarcopts objcopy object objectyaml option orcdebugging orcjit orcshared orctargetprocess passes powerpc powerpcasmparser powerpccodegen powerpcdesc powerpcdisassembler powerpcinfo profiledata remarks riscv riscvasmparser riscvcodegen riscvdesc riscvdisassembler riscvinfo riscvtargetmca runtimedyld scalaropts selectiondag sparc sparcasmparser sparccodegen sparcdesc sparcdisassembler sparcinfo support symbolize systemz systemzasmparser systemzcodegen systemzdesc systemzdisassembler systemzinfo tablegen target targetparser textapi textapibinaryreader transformutils ve veasmparser vecodegen vectorize vedesc vedisassembler veinfo webassembly webassemblyasmparser webassemblycodegen webassemblydesc webassemblydisassembler webassemblyinfo webassemblyutils windowsdriver windowsmanifest x86 x86asmparser x86codegen x86desc x86disassembler x86info x86targetmca xcore xcorecodegen xcoredesc xcoredisassembler xcoreinfo xray\" ;;\n")
                              (format p "  esac\n")
                              (format p "done\n")
                              (format p "if [ -n \"$res\" ]; then echo \"$res\" | sed 's/^ //'; fi\n")))
                          (chmod wrapper-script #o755)
                          (setenv "LLVM_CONFIG" wrapper-script)
                          (setenv "LLVM_CONFIG_PATH" wrapper-script)
                          (for-each
                           (lambda (ver)
                             (let ((name1 (string-append overlay-bin "/llvm-config" ver))
                                   (name2 (string-append overlay-bin "/i686-linux-gnu-llvm-config" ver)))
                               (unless (file-exists? name1) (symlink wrapper-script name1))
                               (unless (file-exists? name2) (symlink wrapper-script name2))))
                           '("" "-14" "-15" "-16" "-17" "-18" "-19" "-20" "-21" "-22" "-23" "-24" "-25"))
                           (let* ((orig-cmake (or (which "cmake")
                                                  "/gnu/store/slczra1cc6dfjd3pvzmbpkfwrrps7f28-cmake-minimal-cross-3.31.10/bin/cmake"))
                                  (cmake-wrapper (string-append overlay-bin "/cmake-cross-wrapper"))
                                  (cross-override (string-append (getcwd) "/mesa-llvm-cross.ini")))
                             (when (file-exists? cmake-wrapper) (delete-file cmake-wrapper))
                              (call-with-output-file cmake-wrapper
                                (lambda (p)
                                  (format p "#!~a\n" (or (which "bash") "/bin/bash"))
                                  (format p "args=()\n")
                                  (format p "for arg in \"$@\"; do\n")
                                  (format p "  case \"$arg\" in\n")
                                  (format p "    -DLLVM_MESON_PACKAGE_NAMES=*)\n")
                                  (format p "      args+=(\"-DLLVM_MESON_PACKAGE_NAMES=LLVM;LLVM-18;LLVM18;LLVM-18.1;LLVM18.1\") ;;\n")
                                  (format p "    -DLLVM_MESON_VERSIONS=*)\n")
                                  (format p "      args+=(\"-DLLVM_MESON_VERSIONS=18.1.8;18.1.0;18.0;18.0.0;18\") ;;\n")
                                  (format p "    *)\n")
                                  (format p "      args+=(\"$arg\") ;;\n")
                                  (format p "  esac\n")
                                  (format p "done\n")
                                  (format p "exec ~s -DLLVM_DIR=~s/lib/cmake/llvm -DLLVM_ROOT=~s -DCMAKE_PREFIX_PATH=~s -DCMAKE_FIND_ROOT_PATH=~s \"${args[@]}\"\n"
                                          orig-cmake llvm llvm llvm llvm)))
                             (chmod cmake-wrapper #o755)
                             (when (file-exists? cross-override) (delete-file cross-override))
                             (call-with-output-file cross-override
                               (lambda (p)
                                 (format p "[binaries]\n")
                                 (format p "cmake = '~a'\n" cmake-wrapper)
                                 (format p "llvm-config = '~a'\n" wrapper-script)
                                 (for-each
                                  (lambda (v)
                                    (format p "llvm-config~a = '~a'\n" v wrapper-script)
                                    (format p "i686-linux-gnu-llvm-config~a = '~a'\n" v wrapper-script))
                                  '("-14" "-15" "-16" "-17" "-18" "-19" "-20" "-21" "-22" "-23" "-24" "-25")))))
                          (prepend-env-path "PATH" overlay-bin)))
                      (let ((cross-override (string-append (getcwd) "/mesa-llvm-cross.ini")))
                        (unless (file-exists? cross-override)
                          (call-with-output-file cross-override
                            (lambda (p)
                              (format p "[binaries]\n")))))
                      (when llvm
                        (setenv "LLVM_DIR" (string-append llvm "/lib/cmake/llvm"))
                        (setenv "LLVM_ROOT" llvm)
                        (setenv "CMAKE_IGNORE_PATH" "/usr;/usr/lib/llvm-20;/usr/include/llvm-20")
                        (prepend-env-path "CMAKE_PREFIX_PATH" llvm))
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
                      (("method : host_machine\\.system\\(\\) == 'windows' \\? 'auto' : 'config-tool'")
                       "method : 'config-tool'")
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
                     (let* ((build-dir (getcwd))
                            (src-dir (string-append build-dir "/../mesa-26.0.2"))
                            (overlay-bin (or (and (file-exists? (string-append src-dir "/ctb-llvm-overlay/bin"))
                                                  (string-append src-dir "/ctb-llvm-overlay/bin"))
                                             (and (file-exists? (string-append build-dir "/ctb-llvm-overlay/bin"))
                                                  (string-append build-dir "/ctb-llvm-overlay/bin")))))
                       (when overlay-bin
                         (let ((wrapper (string-append overlay-bin "/llvm-config")))
                           (when (file-exists? wrapper)
                             (setenv "LLVM_CONFIG" wrapper)
                             (setenv "LLVM_CONFIG_PATH" wrapper)))
                         (setenv "CMAKE_IGNORE_PATH" "/usr;/usr/lib/llvm-20;/usr/include/llvm-20")
                         (let ((current (or (getenv "PATH") "")))
                           (setenv "PATH" (string-append overlay-bin ":" current)))))))
                 (add-after 'install 'symlink-lib-and-lib64
                   (lambda* (#:key outputs #:allow-other-keys)
                     (let ((out (assoc-ref outputs "out")))
                       (when out
                         (let ((lib (string-append out "/lib"))
                               (lib64 (string-append out "/lib64")))
                           (when (and (file-exists? lib64) (not (file-exists? lib)))
                             (symlink "lib64" lib))
                           (when (and (file-exists? lib) (not (file-exists? lib64)))
                             (symlink "lib" lib64))))))))
            #~(modify-phases #$phases
                (add-after 'install 'symlink-lib-and-lib64
                  (lambda* (#:key outputs #:allow-other-keys)
                    (let ((out (assoc-ref outputs "out")))
                      (when out
                        (let ((lib (string-append out "/lib"))
                              (lib64 (string-append out "/lib64")))
                          (when (and (file-exists? lib64) (not (file-exists? lib)))
                            (symlink "lib64" lib))
                          (when (and (file-exists? lib) (not (file-exists? lib64)))
                            (symlink "lib" lib64))))))))))))))

(define-public mesa-libclc-pkg-config-fixed #f)

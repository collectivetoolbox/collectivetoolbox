;;; Patch for icecat to include Bugzilla 1360870 patches and cross-compilation fixes.
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

(define-module (patches icecat)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (guix build-system trivial)
  #:use-module (gnu packages)
  #:use-module (gnu packages compression)
  #:use-module (gnu packages cross-base)
  #:use-module (gnu packages gnuzilla)
  #:use-module (gnu packages rust)
  #:use-module (ice-9 match)
  #:use-module ((srfi srfi-1) #:hide (zip))
  #:export (icecat-minimal-fixed-proc
            icecat-fixed-proc))

(define patch-dir
  (string-append (or (and=> (current-filename) dirname)
                     (string-append (getcwd) "/scripts/guix/patches"))
                 "/icecat/bugzilla1360870"))

(define bugzilla1360870-patches
  (list (local-file (string-append patch-dir "/6695a4a7a649") "icecat-bug1360870-1.patch")
        (local-file (string-append patch-dir "/5e854a4d5fcc") "icecat-bug1360870-2.patch")
        (local-file (string-append patch-dir "/0dc4ed5913a7") "icecat-bug1360870-3.patch")
        (local-file (string-append patch-dir "/61e88493ad15") "icecat-bug1360870-4.patch")
        (local-file (string-append patch-dir "/be5b6a995776") "icecat-bug1360870-5.patch")))

(define (icecat-minimal-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "libgnome")))
    (native-inputs
     (cons* (list "rust-sysroot-for-i686-linux-gnu" (make-rust-sysroot "i686-linux-gnu"))
            (list "gcc-cross-lib" (cross-gcc "i686-linux-gnu" #:libc (cross-libc "i686-linux-gnu")) "lib")
            (list "unzip" (@ (gnu packages compression) unzip))
            (list "zip" (@ (gnu packages compression) zip))
            (package-native-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases _)
        #~(modify-phases %standard-phases
            (add-after 'unpack 'apply-guix-specific-patches
              (lambda _
                (for-each
                 (lambda (file) (invoke "patch" "--force" "-p1" "-i" file))
                 '(#$(local-file (search-patch "icecat-compare-paths.patch"))
                   #$(local-file (search-patch "icecat-use-system-wide-dir.patch"))
                   #$(local-file (search-patch "icecat-fhs-configure-option.patch"))
                   #$(local-file (search-patch "icecat-adjust-mozilla-desktop.patch"))))))
            (add-after 'apply-guix-specific-patches 'remove-bundled-libraries
              (lambda _
                (for-each (lambda (file)
                            (format #t "deleting '~a'...~%" file)
                            (delete-file-recursively file))
                          '("ipc/chromium/src/third_party/libevent"
                            "js/src/ctypes/libffi"
                            "media/libjpeg"
                            "media/libvpx"
                            "modules/freetype2"))))
            (add-after 'remove-bundled-libraries 'fix-ffmpeg-runtime-linker
              (lambda* (#:key inputs #:allow-other-keys)
                (substitute* "dom/media/platforms/ffmpeg/FFmpegRuntimeLinker.cpp"
                  (("libavcodec\\.so")
                   (search-input-file inputs "lib/libavcodec.so")))))
            (add-after 'fix-ffmpeg-runtime-linker 'build-sandbox-whitelist
              (lambda* (#:key inputs #:allow-other-keys)
                (define (runpath-of lib)
                  (call-with-input-file lib
                    (compose elf-dynamic-info-runpath
                             elf-dynamic-info
                             parse-elf
                             get-bytevector-all)))
                (define (runpaths-of-input label)
                  (let* ((dir (string-append (assoc-ref inputs label) "/lib"))
                         (libs (find-files dir "\\.so$")))
                    (append-map runpath-of libs)))
                (let* ((whitelist
                        (map (cut string-append <> "/")
                             (delete-duplicates
                               `(,(string-append (assoc-ref inputs "shared-mime-info")
                                                 "/share/mime")
                                 ,(string-append (assoc-ref inputs "font-dejavu")
                                                 "/share/fonts")
                                 "/run/current-system/profile/share/fonts"
                                 ,@(append-map runpaths-of-input
                                               '("mesa" "ffmpeg"))))))
                       (whitelist-string (string-join whitelist ","))
                       (port (open-file "browser/app/profile/icecat.js" "a")))
                  (format #t "setting 'security.sandbox.content.read_path_whitelist' to '~a'~%"
                          whitelist-string)
                  (format port "~%pref(\"security.sandbox.content.read_path_whitelist\", ~S);~%"
                          whitelist-string)
                  (close-output-port port))))
            (add-after 'patch-source-shebangs 'patch-cargo-checksums
              (lambda _
                (use-modules (guix build cargo-utils))
                (let ((null-hash "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"))
                  (for-each (lambda (file)
                              (format #t "patching checksums in ~a~%" file)
                              (substitute* file
                                (("^checksum = \".*\"")
                                 (string-append "checksum = \"" null-hash "\""))))
                            (find-files "." "Cargo.lock$"))
                  (for-each generate-all-checksums
                            '("services"
                              "js"
                              "third_party/rust"
                              "dom/media"
                              "dom/webauthn"
                              "toolkit"
                              "gfx"
                              "storage"
                              "modules"
                              "xpcom/rust"
                              "media"
                              "mozglue/static/rust"
                              "netwerk"
                              "remote"
                              "intl"
                              "servo"
                              "security/manager/ssl"
                              "build")))))
            (add-after 'patch-cargo-checksums 'remove-cargo-frozen-flag
              (lambda _
                (substitute* "build/RunCbindgen.py"
                  (("args.append\\(\"--frozen\"\\)") "pass"))
                (substitute* "config/makefiles/rust.mk"
                  (("cargo_build_flags \\+= --frozen") ""))))
            (add-after 'remove-cargo-frozen-flag 'wrap-rustc
              (lambda* (#:key inputs native-inputs #:allow-other-keys)
                (let* ((build-inputs (or native-inputs inputs))
                       (rust-target-sysroot (assoc-ref build-inputs "rust-sysroot-for-i686-linux-gnu"))
                       (rust-host (assoc-ref build-inputs "rust"))
                       (real-rustc (search-input-file build-inputs "bin/rustc"))
                       (combined-sysroot (string-append (getcwd) "/combined-rust-sysroot"))
                       (bin-dir (string-append (getcwd) "/rustc-wrapper/bin")))
                  (when rust-target-sysroot
                    (mkdir-p (string-append combined-sysroot "/lib/rustlib"))
                    (when (file-exists? (string-append rust-host "/lib/rustlib/x86_64-unknown-linux-gnu"))
                      (symlink (string-append rust-host "/lib/rustlib/x86_64-unknown-linux-gnu")
                               (string-append combined-sysroot "/lib/rustlib/x86_64-unknown-linux-gnu")))
                    (when (file-exists? (string-append rust-target-sysroot "/lib/rustlib/i686-unknown-linux-gnu"))
                      (symlink (string-append rust-target-sysroot "/lib/rustlib/i686-unknown-linux-gnu")
                               (string-append combined-sysroot "/lib/rustlib/i686-unknown-linux-gnu")))
                    (mkdir-p bin-dir)
                    (call-with-output-file (string-append bin-dir "/rustc")
                      (lambda (port)
                        (format port "#!~a
has_sysroot=0
for arg in \"$@\"; do
  case \"$arg\" in
    --sysroot|--sysroot=*) has_sysroot=1 ;;
  esac
done
if [ $has_sysroot -eq 1 ]; then
  exec ~a \"$@\"
else
  exec ~a --sysroot ~a \"$@\"
fi
"
                                (which "sh") real-rustc real-rustc combined-sysroot)))
                    (chmod (string-append bin-dir "/rustc") #o755)
                    (setenv "PATH" (string-append bin-dir ":" (getenv "PATH")))))))
            (delete 'bootstrap)
            (replace 'configure
              (lambda* (#:key outputs configure-flags inputs native-inputs target #:allow-other-keys)
                (let* ((bash (which "bash"))
                       (abs-srcdir (getcwd))
                       (flags `(,(string-append "--prefix=" #$output)
                                ,(string-append "--with-l10n-base="
                                                abs-srcdir "/l10n")
                                ,@configure-flags))
                       (build-inputs (or native-inputs inputs))
                       (rust-target-sysroot (assoc-ref build-inputs "rust-sysroot-for-i686-linux-gnu"))
                       (combined-sysroot (string-append abs-srcdir "/combined-rust-sysroot"))
                       (gcc-lib (assoc-ref build-inputs "gcc-cross-lib"))
                       (cxx-inc (false-if-exception (search-input-directory build-inputs "include/c++")))
                       (target-cxx-inc (and cxx-inc (string-append cxx-inc "/" (or target "i686-linux-gnu"))))
                       (cxx-backward (and cxx-inc (string-append cxx-inc "/backward")))
                       (stubs-file (false-if-exception (search-input-file inputs "include/gnu/stubs-32.h")))
                       (libc-inc (and stubs-file (dirname (dirname stubs-file))))
                       (linux-ver-file (false-if-exception (search-input-file inputs "include/linux/version.h")))
                       (kernel-inc (and linux-ver-file (dirname (dirname linux-ver-file))))
                       (libc-so-file (false-if-exception (search-input-file inputs "lib/libc.so")))
                       (libc-lib (and libc-so-file (dirname libc-so-file)))
                       (crtbegin-dir (and gcc-lib
                                          (let ((files (find-files gcc-lib "^crtbeginS\\.o$")))
                                            (and (not (null? files)) (dirname (car files))))))
                       (libgcc-s-dir (and gcc-lib
                                          (let ((files (find-files gcc-lib "^libgcc_s\\.so$")))
                                            (and (not (null? files)) (dirname (car files))))))
                       (input-lib-dirs (filter file-exists?
                                              (map (match-lambda
                                                     ((_ . dir) (string-append dir "/lib")))
                                                   inputs)))
                       (input-inc-dirs (filter file-exists?
                                              (map (match-lambda
                                                     ((_ . dir) (string-append dir "/include")))
                                                   inputs)))
                       (input-L-flags (string-join (map (lambda (d) (string-append "-L" d)) input-lib-dirs) " "))
                       (input-I-flags (string-join (map (lambda (d) (string-append "-isystem " d)) input-inc-dirs) " "))
                       (extra-link-flags (string-append
                                          (if (and crtbegin-dir (file-exists? crtbegin-dir))
                                              (string-append "-B" crtbegin-dir " -L" crtbegin-dir " ")
                                              "")
                                          (if (and libc-lib (file-exists? libc-lib))
                                              (string-append "-B" libc-lib " -L" libc-lib " ")
                                              "")
                                          (if (and libgcc-s-dir (file-exists? libgcc-s-dir))
                                              (string-append "-L" libgcc-s-dir " ")
                                              "")
                                          input-L-flags " "))
                       (extra-cxx-flags (string-append
                                         (if (and cxx-inc (file-exists? cxx-inc))
                                             (string-append "-isystem " cxx-inc " ")
                                             "")
                                         (if (and target-cxx-inc (file-exists? target-cxx-inc))
                                             (string-append "-isystem " target-cxx-inc " ")
                                             "")
                                         (if (and cxx-backward (file-exists? cxx-backward))
                                             (string-append "-isystem " cxx-backward " ")
                                             "")
                                         (if (and libc-inc (file-exists? libc-inc))
                                             (string-append "-isystem " libc-inc " ")
                                             "")
                                         (if (and kernel-inc (file-exists? kernel-inc))
                                             (string-append "-isystem " kernel-inc " ")
                                             "")
                                         input-I-flags " "
                                         extra-link-flags))
                       (extra-c-flags (string-append
                                       (if (and libc-inc (file-exists? libc-inc))
                                           (string-append "-isystem " libc-inc " ")
                                           "")
                                       (if (and kernel-inc (file-exists? kernel-inc))
                                           (string-append "-isystem " kernel-inc " ")
                                           "")
                                       input-I-flags " "
                                       extra-link-flags)))
                  (setenv "SHELL" bash)
                  (setenv "CONFIG_SHELL" bash)

                  (setenv "AR" "llvm-ar")
                  (setenv "NM" "llvm-nm")
                  (setenv "CC" (string-append "clang " extra-c-flags))
                  (setenv "CXX" (string-append "clang++ " extra-cxx-flags))
                  (setenv "LDFLAGS" (string-append "-Wl,-rpath="
                                                   #$output "/lib/icecat "
                                                   extra-link-flags))
                  (setenv "LIBRARY_PATH" (string-join input-lib-dirs ":"))

                  (setenv "MACH_BUILD_PYTHON_NATIVE_PACKAGE_SOURCE" "system")
                  (setenv "MOZ_BUILD_DATE" "20260101000000")
                  (setenv "MOZ_APP_REMOTINGNAME" "Icecat")

                  (when (and target-cxx-inc (file-exists? target-cxx-inc))
                    (setenv "BINDGEN_CFLAGS" (string-append "--target=" (or target "i686-linux-gnu") " " extra-cxx-flags)))
                  (when rust-target-sysroot
                    (setenv "RUSTFLAGS" (string-append (or (getenv "RUSTFLAGS") "") " --sysroot " combined-sysroot)))

                  (let ((obj-dir (or (false-if-exception
                                      (first (scandir "." (cut string-prefix? "obj-" <>))))
                                     "obj-i686-pc-linux-gnu")))
                    (setenv "GUIX_PYTHONPATH"
                            (string-append (getcwd) "/" obj-dir "/_virtualenvs/build")))

                  (mkdir ".mozbuild")
                  (setenv "MOZBUILD_STATE_PATH"
                          (string-append (getcwd) "/.mozbuild"))

                  (format #t "build directory: ~s~%" (getcwd))
                  (format #t "configure flags: ~s~%" flags)

                  (call-with-output-file "mozconfig"
                    (lambda (port)
                      (format port "export CC=\"clang ~a\"\n" extra-c-flags)
                      (format port "export CXX=\"clang++ ~a\"\n" extra-cxx-flags)
                      (format port "export AR=\"llvm-ar\"\n")
                      (format port "export NM=\"llvm-nm\"\n")
                      (format port "export LDFLAGS=\"-Wl,-rpath=~a/lib/icecat ~a\"\n"
                              #$output extra-link-flags)
                      (when rust-target-sysroot
                        (format port "export RUSTC=\"~a/rustc-wrapper/bin/rustc\"\n" abs-srcdir)
                        (format port "export RUSTFLAGS=\"--sysroot ~a\"\n" combined-sysroot))
                      (when (and target-cxx-inc (file-exists? target-cxx-inc))
                        (format port "export BINDGEN_CFLAGS=\"--target=~a ~a\"\n"
                                (or target "i686-linux-gnu") extra-cxx-flags))
                      (for-each (lambda (flag)
                                  (format port "ac_add_options ~a\n" flag))
                                flags)))

                  (invoke "./mach" "configure"))))
            (replace 'build
              (lambda* (#:key (make-flags '()) (parallel-build? #t)
                        #:allow-other-keys)
                (apply invoke "./mach" "build"
                       `(,(string-append
                           "-j" (number->string (if parallel-build?
                                                    (parallel-job-count)
                                                    1)))
                         ,@make-flags))))
            (add-after 'build 'neutralise-store-references
              (lambda _
                (let* ((obj-dir (match (scandir "." (cut string-prefix? "obj-" <>))
                                  ((dir) dir)))
                       (file (string-append
                              obj-dir
                              "/dist/bin/chrome/toolkit/content/global/buildconfig.html")))
                  (substitute* file
                    (("[0-9a-df-np-sv-z]{32}" hash)
                     (string-append (string-take hash 8)
                                    "<!-- Guix: not a runtime dependency -->"
                                    (string-drop hash 8)))))))
            (replace 'install
              (lambda* (#:key outputs #:allow-other-keys)
                (invoke "./mach" "install")
                (install-file (first (find-files "." "geckodriver"))
                              (string-append #$output "/bin"))
                (let ((policies.json (string-append
                                      #$output
                                      "/lib/icecat/distribution/policies.json")))
                  (mkdir-p (dirname policies.json))
                  (call-with-output-file policies.json
                    (lambda (p)
                      (format p "\
{
  \"policies\": {
    \"DisableFirefoxAccounts\": true,
    \"DisableTelemetry\": true,
    \"DisablePocket\": true
  }
}~%"))))))
            (add-after 'install 'wrap-program
              (lambda* (#:key inputs #:allow-other-keys)
                (let* ((lib (string-append #$output "/lib"))
                       (gtk (assoc-ref inputs "gtk+"))
                       (gtk-share (string-append gtk "/share"))
                       (ld-libs (cons
                                 (string-append (assoc-ref inputs "libcanberra") "/lib/gtk-3.0/modules")
                                 (map (lambda (label)
                                        (string-append (assoc-ref inputs label) "/lib"))
                                      '("libpng-apng"
                                        "libxscrnsaver"
                                        "mesa"
                                        "pciutils"
                                        "mit-krb5"
                                        "eudev"
                                        "pulseaudio"
                                        "libnotify")))))
                   (wrap-program (car (find-files lib "^icecat$"))
                     `("XDG_DATA_DIRS" prefix (,gtk-share))
                     `("LD_LIBRARY_PATH" prefix ,ld-libs)))))
            (add-after 'wrap-program 'install-desktop-entry
              (lambda _
                (let* ((desktop-file (string-append "toolkit/mozapps/installer"
                                                    "/linux/rpm/mozilla.desktop"))
                       (applications (string-append #$output "/share/applications")))
                  (substitute* desktop-file
                    (("@MOZ_APP_NAME@") "icecat")
                    (("^Exec=icecat") (string-append "Exec=" #$output "/bin/icecat"))
                    (("^Icon=.*") "Icon=icecat\n"))
                  (install-file desktop-file applications))))))))))

(define (icecat-fixed-proc pkg)
  (package
    (inherit pkg)
    (build-system trivial-build-system)
    (native-inputs
     (list (list "icecat-l10n" icecat-l10n)))
    (inputs
     (list (list "icecat-minimal" (icecat-minimal-fixed-proc icecat-minimal))))
    (arguments
     `(#:modules ((guix build union)
                  (guix build utils))
       #:builder
       (begin
          (use-modules (guix build union)
                       (guix build utils))
          (let* ((base (assoc-ref %build-inputs "icecat-minimal"))
                 (l10n (assoc-ref %build-inputs "icecat-l10n"))
                 (out (assoc-ref %outputs "out"))
                 (name "icecat")
                 (wrapper (string-append "lib/" name "/" name))
                 (real-binary (string-append "lib/" name "/." name "-real"))
                 (desktop-file (or (and (file-exists? (string-append base "/share/applications/" name ".desktop"))
                                        (string-append "share/applications/" name ".desktop"))
                                   (and (file-exists? (string-append base "/share/applications/mozilla.desktop"))
                                        "share/applications/mozilla.desktop"))))
            (union-build out (list base l10n)
                         #:create-all-directories? #t)
            (when (file-exists? (string-append out "/" wrapper))
              (delete-file (string-append out "/" wrapper))
              (copy-file (string-append base "/" wrapper) (string-append out "/" wrapper))
              (substitute* (string-append out "/" wrapper)
                ((base) out)))
            (when (file-exists? (string-append out "/bin/" name))
              (delete-file (string-append out "/bin/" name))
              (symlink (string-append out "/" wrapper)
                       (string-append out "/bin/" name)))
            (when (file-exists? (string-append out "/" real-binary))
              (delete-file (string-append out "/" real-binary))
              (copy-file (string-append base "/" real-binary) (string-append out "/" real-binary)))
            (when (and desktop-file (file-exists? (string-append out "/" desktop-file)))
              (delete-file (string-append out "/" desktop-file))
              (copy-file (string-append base "/" desktop-file) (string-append out "/" desktop-file))
              (substitute* (string-append out "/" desktop-file)
                ((base) out)))
            #t))))))

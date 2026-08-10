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
                    (prepend-env-path "CMAKE_PREFIX_PATH" llvm)
                    (prepend-env-path "PATH" (string-append llvm "/bin")))
                  (when spirv-tools
                    (prepend-env-path "CMAKE_PREFIX_PATH" spirv-tools)
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append spirv-tools "/lib/pkgconfig")))
                  (when llvm-spirv
                    (prepend-env-path
                     "PKG_CONFIG_PATH"
                     (string-append llvm-spirv "/lib/pkgconfig"))))))))))))
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
            (add-before 'configure 'expose-libclc-pkg-config
              (lambda* (#:key native-inputs #:allow-other-keys)
                (let ((libclc (assoc-ref native-inputs "libclc")))
                  (when libclc
                    (let ((pkgconfig-dir
                           (string-append libclc "/share/pkgconfig"))
                          (current-path
                           (or (getenv "PKG_CONFIG_PATH") "")))
                      ;; Cross pkg-config only sees target inputs by default,
                      ;; but Mesa resolves libclc as a dependency during the
                      ;; configure phase.
                      (setenv "PKG_CONFIG_PATH"
                              (if (string-null? current-path)
                                  pkgconfig-dir
                                  (string-append
                                   pkgconfig-dir
                                   ":"
                                   current-path))))))))))))))
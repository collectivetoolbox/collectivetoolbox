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

;;; Patch for gpgme cross-compilation with target gpg-error-config and libassuan-config.

(define-module (patches gpgme)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnupg)
  #:use-module (gnu packages pkg-config)
  #:export (gpgme-fixed-proc gpgme-fixed))

(define (gpgme-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (cons `("pkg-config" ,pkg-config)
           (package-native-inputs pkg)))

    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  '("--disable-gpg-test"
                    "--disable-gpgsm-test"
                    "--disable-gpgconf-test"
                    "--disable-g13-test")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'set-config-scripts
              (lambda* (#:key inputs #:allow-other-keys)
                (let ((gpg-error (assoc-ref inputs "libgpg-error"))
                      (libassuan (assoc-ref inputs "libassuan")))
                  (when gpg-error
                    (setenv "GPG_ERROR_CONFIG" (string-append gpg-error "/bin/gpg-error-config")))
                  (when libassuan
                    (setenv "LIBASSUAN_CONFIG" (string-append libassuan "/bin/libassuan-config")))
                  (setenv "GPGRT_CONFIG" "no"))))))))))

(define gpgme-fixed #f)

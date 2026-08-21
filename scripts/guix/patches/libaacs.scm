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

;;; Patch for libaacs and libbdplus to find libgcrypt and libgpg-error configs when cross-compiling.

(define-module (patches libaacs)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnupg)
  #:use-module (gnu packages video)
  #:export (libaacs-fixed-proc
            libaacs-fixed
            libbdplus-fixed))

(define (libaacs-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (append libgpg-error)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'set-libgcrypt-config
              (lambda* (#:key inputs #:allow-other-keys)
                (let ((gcrypt (assoc-ref inputs "libgcrypt"))
                      (gpg-error (assoc-ref inputs "libgpg-error")))
                  (when gcrypt
                    (let ((gcrypt-config (string-append gcrypt "/bin/libgcrypt-config")))
                      (when (file-exists? gcrypt-config)
                        (setenv "LIBGCRYPT_CONFIG" gcrypt-config)
                        (setenv "PATH" (string-append gcrypt "/bin:" (getenv "PATH"))))))
                  (when gpg-error
                    (let ((gpg-config (string-append gpg-error "/bin/gpg-error-config")))
                      (when (file-exists? gpg-config)
                        (setenv "GPG_ERROR_CONFIG" gpg-config)
                        (setenv "PATH" (string-append gpg-error "/bin:" (getenv "PATH")))))))))))))))

(define libaacs-fixed
  (libaacs-fixed-proc libaacs))

(define libbdplus-fixed
  (libaacs-fixed-proc libbdplus))

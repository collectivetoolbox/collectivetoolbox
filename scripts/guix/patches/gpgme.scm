;;; Patch for gpgme cross-compilation with pkg-config and libgpg-error in native-inputs.
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
     (cons* `("pkg-config" ,pkg-config)
            `("libgpg-error" ,libgpg-error)
            (package-native-inputs pkg)))
    (inputs
     (map (lambda (input)
            (if (pair? input)
                (list (car input) ((@ (patches) apply-patches) (cadr input)))
                ((@ (patches) apply-patches) input)))
          (package-inputs pkg)))
    (propagated-inputs
     (map (lambda (input)
            (if (pair? input)
                (list (car input) ((@ (patches) apply-patches) (cadr input)))
                ((@ (patches) apply-patches) input)))
          (package-propagated-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  (list (string-append "--with-libgpg-error-prefix="
                                       #$(this-package-input "libgpg-error"))
                        (string-append "--with-libassuan-prefix="
                                       #$(this-package-input "libassuan")))))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'set-gpgrt-config
              (lambda _
                (setenv "GPGRT_CONFIG" "gpgrt-config")))))))))

(define gpgme-fixed
  (gpgme-fixed-proc gpgme))

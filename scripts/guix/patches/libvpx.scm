;;; Patch for libvpx to support cross-compilation.
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

(define-module (patches libvpx)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (ice-9 match)
  #:use-module (gnu packages video)
  #:export (libvpx-fixed-proc libvpx-fixed))

(define (map-input-list inputs)
  (map (match-lambda
         (((? string? name) (? package? p))
          (list name ((@ (patches) apply-patches) p)))
         (((? string? name) (? package? p) (? string? output))
          (list name ((@ (patches) apply-patches) p) output))
         (((? package? p) (? string? output))
          (list ((@ (patches) apply-patches) p) output))
         ((? package? p)
          ((@ (patches) apply-patches) p))
         (other other))
       inputs))

(define (libvpx-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (map-input-list (package-inputs pkg)))
    (native-inputs
     (map-input-list (package-native-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (replace 'configure
              (lambda* (#:key outputs configure-flags target #:allow-other-keys)
                (let ((out (assoc-ref outputs "out")))
                  (when target
                    (setenv "CROSS" (string-append target "-"))
                    (setenv "CC" (string-append target "-gcc"))
                    (setenv "CXX" (string-append target "-g++"))
                    (setenv "AR" (string-append target "-ar"))
                    (setenv "NM" (string-append target "-nm"))
                    (setenv "RANLIB" (string-append target "-ranlib"))
                    (setenv "AS" "yasm"))
                  (apply invoke "./configure"
                         (append
                          (if target
                              (list (string-append "--target="
                                                   (if (string-prefix? "i686" target)
                                                       "x86-linux-gcc"
                                                       "generic-gnu")))
                              '())
                          configure-flags)))))))))))

(define libvpx-fixed #f)

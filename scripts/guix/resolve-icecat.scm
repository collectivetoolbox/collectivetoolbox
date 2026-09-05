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

;;; Script to deterministically query the prebuilt GNU Icecat binary store path
;;; and full transitive source closure using native Guix Scheme procedures.

(use-modules (guix store)
             (guix packages)
             (guix derivations)
             (guix monads)
             (guix gexp)
             (gnu packages gnuzilla)
             (ice-9 match)
             (patches))

(define (resolve-icecat-binary)
  (with-store store
    (let* ((pkg ((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat)))
           (drv (package-cross-derivation store pkg "i686-linux-gnu" "x86_64-linux"))
           (out (derivation->output-path drv)))
      (if (and (valid-path? store out) (file-exists? out))
          (begin
            (display out)
            (newline)
            #t)
          (begin
            (format (current-error-port)
                    "Error: Icecat binary output path ~a is not valid or missing from /gnu/store.~%"
                    out)
            #f)))))

(define (resolve-icecat-sources)
  (with-store store
    (let* ((pkgs (list ((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat-minimal))
                       ((@ (patches) apply-patches) (@ (gnu packages gnuzilla) icecat))))
           (sources (all-transitive-sources pkgs))
           (missing '()))
      (for-each
       (lambda (src)
         (let* ((lowered (run-with-store store (lower-object src)))
                (out (if (derivation? lowered)
                         (derivation->output-path lowered)
                         lowered)))
           (if (and (valid-path? store out) (file-exists? out))
               (begin
                 (display out)
                 (newline))
               (set! missing (cons (list (or (origin-actual-file-name src)
                                             (origin-file-name src)
                                             "unnamed-origin")
                                         out)
                                   missing)))))
       sources)
      (if (null? missing)
          #t
          (begin
            (format (current-error-port)
                    "Error: The following ~a source file(s) in Icecat closure are missing from /gnu/store:~%"
                    (length missing))
            (for-each (lambda (m)
                        (format (current-error-port) "  - ~a: ~a~%" (car m) (cadr m)))
                      missing)
            #f)))))

(define (resolve-system-sources)
  (with-store store
    (let* ((os (load (string-append (dirname (current-filename)) "/v86-os.scm")))
           (pkgs (operating-system-packages os))
           (sources (all-transitive-sources pkgs)))
      (for-each
       (lambda (src)
         (let* ((lowered (run-with-store store (lower-object src)))
                (out (if (derivation? lowered)
                         (derivation->output-path lowered)
                         lowered)))
           (if (and (valid-path? store out) (file-exists? out))
               (begin
                 (display out)
                 (newline))
               #f)))
       sources)
      #t)))

(match (command-line)
  ((_ "binary")
   (if (resolve-icecat-binary)
       (exit 0)
       (exit 1)))
  ((_ "sources")
   (if (resolve-icecat-sources)
       (exit 0)
       (exit 1)))
  ((_ "system-sources")
   (if (resolve-system-sources)
       (exit 0)
       (exit 1)))
  ((_ "both")
   (if (and (resolve-icecat-binary)
            (resolve-icecat-sources))
       (exit 0)
       (exit 1)))
  (_
   (format (current-error-port) "Usage: resolve-icecat.scm [binary|sources|system-sources|both]~%")
   (exit 2)))

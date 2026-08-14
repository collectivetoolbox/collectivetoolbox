;;; Patch for icecat to include Bugzilla 1360870 patches for module service workers.
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
  #:use-module (gnu packages gnuzilla)
  #:export (icecat-patched))

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

(define icecat-patched
  (package
    (inherit icecat)
    (source
      (origin
        (inherit (package-source icecat))
        (patches (append bugzilla1360870-patches
                         (origin-patches (package-source icecat))))))))

;;; Patch for cups, cups-minimal, and cups-filters to disable tests and propagate patches.
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

(define-module (patches cups)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages cups)
  #:export (cups-minimal-fixed-proc
            cups-minimal-fixed
            cups-fixed-proc
            cups-fixed
            cups-filters-fixed-proc
            cups-filters-fixed))

(define (cups-minimal-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)))))

(define cups-minimal-fixed
  (cups-minimal-fixed-proc cups-minimal))

(define (cups-filters-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)))))

(define cups-filters-fixed
  (cups-filters-fixed-proc cups-filters))

(define (cups-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "cups-filters")))
    (arguments
     (substitute-keyword-arguments (package-arguments cups-minimal)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  (list "--with-languages=all")))))))

(define cups-fixed
  (cups-fixed-proc cups))

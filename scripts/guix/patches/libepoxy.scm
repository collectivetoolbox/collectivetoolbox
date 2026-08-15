;;; Patch for libepoxy to map propagated-inputs (mesa) through apply-patches.
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

(define-module (patches libepoxy)
  #:use-module (guix packages)
  #:use-module (gnu packages gl)
  #:use-module (ice-9 match)
  #:export (libepoxy-fixed-proc libepoxy-fixed))

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

(define (libepoxy-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (map-input-list (package-inputs pkg)))
    (propagated-inputs
     (map-input-list (package-propagated-inputs pkg)))))

(define libepoxy-fixed #f)

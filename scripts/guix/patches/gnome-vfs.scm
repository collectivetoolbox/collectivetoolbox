;;; Patch for gnome-vfs to provide bzip2, native gconf (gconftool-2), and disable tests during cross-compilation.
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

(define-module (patches gnome-vfs)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages compression)
  #:use-module (gnu packages gnome)
  #:use-module (ice-9 match)
  #:export (gnome-vfs-fixed-proc gnome-vfs-fixed))

(define (gnome-vfs-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (cons (list "bzip2" bzip2)
           (map (match-lambda
                  ((name (? package? p))
                   (list name ((@ (patches) apply-patches) p)))
                  ((name (? package? p) output)
                   (list name ((@ (patches) apply-patches) p) output))
                  (other other))
                (package-inputs pkg))))
    (native-inputs
     (cons `("gconf" ,gconf)
           (package-native-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)))))

(define gnome-vfs-fixed
  (gnome-vfs-fixed-proc gnome-vfs))

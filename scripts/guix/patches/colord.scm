;;; Patch for colord-minimal to ensure libraries are installed into lib/ instead of lib64/, disable introspection, and supply bash/polkit ITS rules when cross-compiling.
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

(define-module (patches colord)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages bash)
  #:use-module (gnu packages gnome)
  #:use-module (gnu packages polkit)
  #:use-module (ice-9 match)
  #:export (colord-fixed-proc colord-fixed))

(define (colord-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (cons (list "bash-minimal" bash-minimal)
           (map (match-lambda
                  ((name (? package? p))
                   (list name ((@ (patches) apply-patches) p)))
                  ((name (? package? p) output)
                   (list name ((@ (patches) apply-patches) p) output))
                  (other other))
                (package-inputs pkg))))
    (native-inputs
     (cons `("polkit" ,polkit)
           (package-native-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(append #$flags '("-Dlibdir=lib"
                            "-Dintrospection=false"
                            "-Dtests=false"
                            "-Dvapi=false")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'set-gettext-data-dirs
              (lambda* (#:key inputs native-inputs #:allow-other-keys)
                (let ((p (assoc-ref (or native-inputs inputs) "polkit")))
                  (when p
                    (setenv "GETTEXTDATADIRS"
                            (string-append p "/share/gettext/its:"
                                           (or (getenv "GETTEXTDATADIRS") "")))))))))))))

(define colord-fixed
  (colord-fixed-proc colord-minimal))

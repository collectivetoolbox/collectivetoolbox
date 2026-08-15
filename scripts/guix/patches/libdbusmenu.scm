;;; Patch for libdbusmenu to disable introspection/vala/gtk-doc and fix pkg-config during autogen.
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

(define-module (patches libdbusmenu)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gtk)
  #:use-module (ice-9 match)
  #:export (libdbusmenu-fixed-proc libdbusmenu-fixed))

(define (libdbusmenu-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")))
    (inputs
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
          (package-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  '("--enable-introspection=no"
                    "--disable-vala"
                    "--disable-gtk-doc")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'bootstrap 'symlink-pkg-config
              (lambda* (#:key target #:allow-other-keys)
                (when target
                  (let ((cross-pc (which (string-append target "-pkg-config"))))
                    (when cross-pc
                      (let ((bin (string-append (getcwd) "/bin-wrapper")))
                        (mkdir-p bin)
                        (symlink cross-pc (string-append bin "/pkg-config"))
                        (setenv "PATH" (string-append bin ":" (getenv "PATH")))))))))))))))

(define libdbusmenu-fixed #f)

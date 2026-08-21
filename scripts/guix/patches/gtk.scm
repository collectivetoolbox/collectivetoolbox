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

;;; Patch for gtk+ cross-compilation with native wayland-scanner and -Dlibdir=lib.

(define-module (patches gtk)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages freedesktop)
  #:use-module (gnu packages gtk)
  #:use-module (ice-9 match)
  #:export (gtk+-fixed-proc gtk+-fixed
            gtk-fixed-proc gtk-fixed))

(define (gtk-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")
       (append wayland)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append (delete "-Ddocumentation=true"
                          (delete "-Dman-pages=true" #$flags))
                  '("-Dlibdir=lib"
                    "-Dintrospection=disabled"
                    "-Ddocumentation=false"
                    "-Dman-pages=false"
                    "-Dbuild-tests=false"
                    "-Dbuild-testsuite=false")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'disable-cross-introspection
              (lambda* (#:key target #:allow-other-keys)
                (when target
                  (substitute* "meson_options.txt"
                    (("option\\('introspection', type: 'feature', value: 'auto'")
                     "option('introspection', type: 'feature', value: 'disabled'"))
                  (substitute* "meson.build"
                    (("build_gir = .*")
                     "build_gir = false\n")))))))))))

(define gtk-fixed #f)

(define (gtk+-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "colord-minimal" "librest")))
    (propagated-inputs
     (modify-inputs (package-propagated-inputs pkg)
       (delete "librsvg" "libcloudproviders-minimal")
       (append gdk-pixbuf)))
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")
       (append wayland)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append (delete "-Dcloudproviders=true"
                          (delete "-Dcolord=yes"
                                  (delete "-Dman=true" #$flags)))
                  '("-Dlibdir=lib"
                    "-Dcolord=no"
                    "-Dcloudproviders=false"
                    "-Dintrospection=false"
                    "-Dman=false"
                    "-Dgtk_doc=false")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'disable-cross-introspection
              (lambda* (#:key target #:allow-other-keys)
                (when target
                  (substitute* "meson_options.txt"
                    (("option\\('introspection', type: 'boolean', value: 'true'")
                     "option('introspection', type: 'boolean', value: 'false'"))
                  (substitute* "meson.build"
                    (("build_gir = .*")
                     "build_gir = false\n")))))))))))

(define gtk+-fixed #f)

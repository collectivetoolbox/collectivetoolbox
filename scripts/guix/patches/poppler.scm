;;; Patch for poppler cross-compilation.
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

(define-module (patches poppler)
  #:use-module (guix packages)
  #:use-module (guix gexp)
  #:use-module (guix utils)
  #:use-module (gnu packages pdf)
  #:export (poppler-fixed-proc poppler-fixed))

(define (poppler-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags ''())
        #~(list "-DENABLE_UNSTABLE_API_ABI_HEADERS=ON"
                "-DENABLE_ZLIB=ON"
                "-DENABLE_BOOST=OFF"
                "-DENABLE_GOBJECT_INTROSPECTION=OFF"
                "-DENABLE_QT5=OFF"
                "-DENABLE_QT6=OFF"
                "-DENABLE_NSS3=OFF"
                "-DCMAKE_DISABLE_FIND_PACKAGE_NSS3=TRUE"
                (string-append "-DCMAKE_INSTALL_LIBDIR=" #$output "/lib")
                (string-append "-DCMAKE_INSTALL_RPATH=" #$output "/lib")))
       ((#:phases _ #~%standard-phases)
        #~(modify-phases %standard-phases
            (add-after 'unpack 'set-PKG_CONFIG
              (lambda* (#:key target #:allow-other-keys)
                (let ((pkg-cfg (or (which (if target
                                              (string-append target "-pkg-config")
                                              "pkg-config"))
                                   (which "pkg-config"))))
                  (setenv "PKG_CONFIG" pkg-cfg)
                  (setenv "PKG_CONFIG_EXECUTABLE" pkg-cfg)
                  (substitute* "CMakeLists.txt"
                    (("project\\(poppler.*" match)
                     (string-append match "\nset(PKG_CONFIG_EXECUTABLE \""
                                    pkg-cfg
                                    "\" CACHE FILEPATH \"pkg-config\" FORCE)\n"))))))))))
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")))))

(define poppler-fixed
  (poppler-fixed-proc poppler))

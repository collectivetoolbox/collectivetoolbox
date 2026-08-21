;;; Patch for ibus to fix cross-compilation AC_CHECK_FILE, pre-generate compose tables, and disable ui/introspection/vala/docs.
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

(define-module (patches ibus)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages glib)
  #:use-module (gnu packages ibus)
  #:use-module (ice-9 match)
  #:export (ibus-fixed-proc ibus-fixed))

(define (ibus-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "gtk" "gtk+" "python" "python-dbus" "python-pygobject" "dconf" "libdbusmenu" "libnotify")))
    (native-inputs
     (modify-inputs (package-native-inputs pkg)
       (delete "gobject-introspection")
       (append glib)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append (delete "--enable-gtk-doc"
                          (delete "--enable-wayland" #$flags))
                  '("CC_FOR_BUILD=gcc"
                    "PKG_CONFIG_FOR_BUILD=pkg-config"
                    "--disable-introspection"
                    "--disable-vala"
                    "--disable-gtk-doc"
                    "--disable-ui"
                    "--disable-appindicator"
                    "--disable-emoji-dict"
                    "--disable-unicode-dict"
                    "--disable-tests")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (delete 'wrap-with-additional-paths)
            (add-after 'unpack 'patch-cross-makefile
              (lambda _
                (substitute* "configure"
                  (("test \"\\$cross_compiling\" = yes &&")
                   "false &&"))
                (substitute* "src/Makefile.in"
                  (("gen-internal-compose-table\\$\\(EXEEXT\\)")
                   "")
                  (("compose/sequences-\\$\\(ENDIAN\\)-endian:.*")
                   "compose/sequences-$(ENDIAN)-endian:\n\t@true\n"))))
            (add-before 'build 'generate-compose-tables
              (lambda* (#:key native-inputs inputs #:allow-other-keys)
                (let* ((build-inputs (or native-inputs inputs))
                       (glib-inc (search-input-directory build-inputs "include/glib-2.0"))
                       (glib-lib (dirname (search-input-file build-inputs "lib/libglib-2.0.so")))
                       (glib-libinc (string-append glib-lib "/glib-2.0/include"))
                       (x11-locale (search-input-directory build-inputs "share/X11/locale")))
                  (with-directory-excursion "src"
                    (invoke "gcc" "-I." "-I.."
                            (string-append "-I" glib-inc)
                            (string-append "-I" glib-libinc)
                            "-DIBUS_COMPILATION"
                            (string-append "-DX11_LOCALEDATADIR=\"" x11-locale "\"")
                            "gencomposetable.c" "ibuscomposetable.c" "ibuserror.c"
                            "ibuskeynames.c" "ibuskeyuni.c"
                            "-o" "gen-internal-compose-table-native"
                            (string-append "-L" glib-lib)
                            (string-append "-Wl,-rpath=" glib-lib)
                            "-lglib-2.0" "-lgobject-2.0" "-lgio-2.0")
                    (invoke "./gen-internal-compose-table-native")
                    (mkdir-p "compose")
                    (rename-file "sequences-big-endian" "compose/sequences-big-endian")
                    (rename-file "sequences-little-endian" "compose/sequences-little-endian")))))))))))

(define ibus-fixed #f)

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

;;; Patch for ibus to fix cross-compilation AC_CHECK_FILE, reuse compose tables, and disable ui/introspection/vala/docs/gtk/tools.

(define-module (patches ibus)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
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
       (delete "gobject-introspection")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append (delete "--enable-gtk-doc"
                          (delete "--enable-wayland" #$flags))
                  '("CC_FOR_BUILD=gcc"
                    "PKG_CONFIG_FOR_BUILD=pkg-config"
                    "--disable-gtk3"
                    "--disable-xim"
                    "--disable-dconf"
                    "--disable-setup"
                    "--disable-engine"
                    "--disable-libnotify"
                    "--disable-python-library"
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
            (delete 'patch-python-target-directories)
            (delete 'disable-dconf-update)
            (delete 'move-doc)
            (add-after 'unpack 'patch-build
              (lambda _
                (substitute* "tools/Makefile.am"
                  (("bin_PROGRAMS = ibus")
                   "bin_PROGRAMS ="))
                (substitute* "src/Makefile.am"
                  (("noinst_PROGRAMS = gen-internal-compose-table")
                   "noinst_PROGRAMS =")
                  (("compose/sequences-\\$\\(ENDIAN\\)-endian:.*")
                   "compose/sequences-$(ENDIAN)-endian:\n\t@true\n"))))))))))

(define ibus-fixed #f)

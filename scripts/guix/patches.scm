;;; Aggregator module for Guix package patches.
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

(define-module (patches)
  #:use-module (guix packages)
  #:use-module (patches abseil-cpp)
  #:use-module (patches colord)
  #:use-module (patches cups)
  #:use-module (patches dillo)
  #:use-module (patches ffmpeg)
  #:use-module (patches fltk)
  #:use-module (patches freeglut)
  #:use-module (patches gavl)
  #:use-module (patches gnome-vfs)
  #:use-module (patches gst-plugins-good)
  #:use-module (patches gstreamer)
  #:use-module (patches gtk)
  #:use-module (patches gnupg)
  #:use-module (patches gpgme)
  #:use-module (patches gusb)
  #:use-module (patches libaacs)
  #:use-module (patches libcddb)
  #:use-module (patches libgudev)
  #:use-module (patches libidl)
  #:use-module (patches libsoup)
  #:use-module (patches libx264)
  #:use-module (patches libxkbcommon)
  #:use-module (patches mesa)
  #:use-module (patches orbit2)
  #:use-module (patches qpdf)
  #:use-module (patches rest)
  #:use-module (patches shaderc)
  #:use-module (patches spice-gtk)
  #:use-module (patches talloc)
  #:use-module (patches x265)
  #:use-module (patches icecat)
  #:export (apply-patches))

(define package-patches
  `(("abseil-cpp" . ,abseil-cpp-fixed-proc)
    ("colord-minimal" . ,colord-fixed-proc)
    ("colord" . ,colord-fixed-proc)
    ("cups-filters" . ,cups-filters-fixed-proc)
    ("cups-minimal" . ,cups-minimal-fixed-proc)
    ("cups" . ,cups-fixed-proc)
    ("dillo" . ,dillo-fixed-proc)
    ("ffmpeg" . ,ffmpeg-fixed-proc)
    ("fltk" . ,fltk-fixed-proc)
    ("freeglut" . ,freeglut-fixed-proc)
    ("gavl" . ,gavl-fixed-proc)
    ("gnome-vfs" . ,gnome-vfs-fixed-proc)
    ("gnupg" . ,gnupg-fixed-proc)
    ("gpgme" . ,gpgme-fixed-proc)
    ("gst-plugins-good" . ,(const gst-plugins-good-no-tests))
    ("gstreamer" . ,gstreamer-fixed-proc)
    ("gtk+" . ,gtk+-fixed-proc)
    ("gusb-minimal" . ,gusb-fixed-proc)
    ("gusb" . ,gusb-fixed-proc)
    ("libaacs" . ,libaacs-fixed-proc)
    ("libbdplus" . ,libaacs-fixed-proc)
    ("libcddb" . ,libcddb-fixed-proc)
    ("libgudev" . ,libgudev-fixed-proc)
    ("libidl" . ,libidl-fixed-proc)
    ("libsoup-minimal" . ,libsoup-minimal-fixed-proc)
    ("libsoup-minimal-2" . ,libsoup-minimal-2-fixed-proc)
    ("libsoup" . ,libsoup-fixed-proc)
    ("libx264" . ,libx264-fixed-proc)
    ("libxkbcommon" . ,libxkbcommon-fixed-proc)
    ("ldb" . ,ldb-fixed-proc)
    ("mesa" . ,mesa-libclc-pkg-config-fixed-proc)
    ("orbit2" . ,orbit2-fixed-proc)
    ("qpdf" . ,qpdf-fixed-proc)
    ("rest" . ,rest-fixed-proc)
    ("samba" . ,identity)
    ("shaderc" . ,shaderc-fixed-proc)
    ("spice-gtk" . ,(const spice-gtk-fixed))
    ("talloc" . ,talloc-fixed-proc)
    ("tevent" . ,tevent-fixed-proc)
    ("x265" . ,x265-fixed-proc)
    ("icecat-minimal" . ,icecat-minimal-fixed-proc)
    ("icecat" . ,icecat-fixed-proc)))

(define apply-patches
  (package-input-rewriting/spec package-patches))

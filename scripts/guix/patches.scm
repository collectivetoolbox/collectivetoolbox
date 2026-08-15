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
  #:use-module (patches dillo)
  #:use-module (patches ffmpeg)
  #:use-module (patches fltk)
  #:use-module (patches freeglut)
  #:use-module (patches gavl)
  #:use-module (patches gst-plugins-good)
  #:use-module (patches gusb)
  #:use-module (patches libaacs)
  #:use-module (patches libcddb)
  #:use-module (patches libgudev)
  #:use-module (patches libidl)
  #:use-module (patches mesa)
  #:use-module (patches orbit2)
  #:use-module (patches spice-gtk)
  #:use-module (patches icecat)
  #:export (apply-patches))

(define package-patches
  `(("abseil-cpp" . ,abseil-cpp-fixed-proc)
    ("colord-minimal" . ,colord-fixed-proc)
    ("colord" . ,colord-fixed-proc)
    ("dillo" . ,dillo-fixed-proc)
    ("ffmpeg" . ,ffmpeg-fixed-proc)
    ("fltk" . ,fltk-fixed-proc)
    ("freeglut" . ,freeglut-fixed-proc)
    ("gavl" . ,gavl-fixed-proc)
    ("gst-plugins-good" . ,(const gst-plugins-good-no-tests))
    ("gusb-minimal" . ,gusb-fixed-proc)
    ("gusb" . ,gusb-fixed-proc)
    ("libaacs" . ,libaacs-fixed-proc)
    ("libbdplus" . ,libaacs-fixed-proc)
    ("libcddb" . ,libcddb-fixed-proc)
    ("libgudev" . ,libgudev-fixed-proc)
    ("libidl" . ,libidl-fixed-proc)
    ("mesa" . ,mesa-libclc-pkg-config-fixed-proc)
    ("orbit2" . ,orbit2-fixed-proc)
    ("spice-gtk" . ,(const spice-gtk-fixed))
    ("icecat-minimal" . ,icecat-minimal-fixed-proc)
    ("icecat" . ,icecat-fixed-proc)))

(define apply-patches
  (package-input-rewriting/spec package-patches))

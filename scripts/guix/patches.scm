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
  #:use-module (patches gst-plugins-good)
  #:use-module (patches mesa)
  #:use-module (patches spice-gtk)
  #:use-module (patches icecat)
  #:export (apply-patches))

(define package-patches
  `(("gst-plugins-good" . ,(const gst-plugins-good-no-tests))
    ("mesa" . ,(const mesa-libclc-pkg-config-fixed))
    ("spice-gtk" . ,(const spice-gtk-fixed))
    ("icecat" . ,(const icecat-patched))))

(define apply-patches
  (package-input-rewriting/spec package-patches))

;;; Patch for gsettings-desktop-schemas to remove gobject-introspection from target inputs.
;;; Copyright 2026 Collective Toolbox contributors
;;; This Scheme program is free software; you can redistribute it and/or modify it
;;; under the terms of the GNU General Public License as published by
;;; the Free Software Foundation; either version 3 of the License, or (at
;;; your option) any later version.

(define-module (patches gsettings-desktop-schemas)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnome)
  #:export (gsettings-desktop-schemas-fixed-proc
            gsettings-desktop-schemas-fixed))

(define (gsettings-desktop-schemas-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "gobject-introspection")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  '("-Dintrospection=false")))))))

(define gsettings-desktop-schemas-fixed
  (gsettings-desktop-schemas-fixed-proc gsettings-desktop-schemas))

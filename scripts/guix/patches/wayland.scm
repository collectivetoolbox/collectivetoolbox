;;; Patch for wayland to ensure libdir is lib.
;;; Copyright 2026 Collective Toolbox contributors
;;; This Scheme program is free software; you can redistribute it and/or modify it
;;; under the terms of the GNU General Public License as published by
;;; the Free Software Foundation; either version 3 of the License, or (at
;;; your option) any later version.

(define-module (patches wayland)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages freedesktop)
  #:export (wayland-fixed-proc wayland-fixed))

(define (wayland-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(cons "--libdir=lib" #$flags))))))

(define wayland-fixed
  (wayland-fixed-proc wayland))

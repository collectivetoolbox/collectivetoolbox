;; Aggregator module for Guix package patches.
;; Copyright (c) 2026 Collective Toolbox project.

(define-module (patches)
  #:use-module (guix packages)
  #:use-module (patches gst-plugins-good)
  #:use-module (patches spice-gtk)
  #:export (apply-patches))

(define package-patches
  `(("gst-plugins-good" . ,(const gst-plugins-good-no-tests))
    ("spice-gtk" . ,(const spice-gtk-fixed))))

(define apply-patches
  (package-input-rewriting/spec package-patches))

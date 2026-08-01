;; Patch for spice-gtk on i686-linux to ensure libraries are installed into lib/ instead of lib64/ and disable tests.
;; Copyright (c) 2026 Collective Toolbox project.

(define-module (patches spice-gtk)
  #:use-module (guix packages)
  #:use-module (guix gexp)
  #:use-module (gnu packages spice)
  #:export (spice-gtk-fixed))

(define spice-gtk-fixed
  (package
    (inherit spice-gtk)
    (arguments
      (cons* #:tests? #f
             #:configure-flags #~'("-Dlibdir=lib")
             (package-arguments spice-gtk)))))

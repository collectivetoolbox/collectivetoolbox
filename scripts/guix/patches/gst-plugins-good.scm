;; Patch for gst-plugins-good on i686-linux to disable hanging unit tests.
;; Copyright (c) 2026 Collective Toolbox project.

(define-module (patches gst-plugins-good)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (gnu packages gstreamer)
  #:export (gst-plugins-good-no-tests))

(define gst-plugins-good-no-tests
  (package
    (inherit gst-plugins-good)
    (arguments
      (substitute-keyword-arguments (package-arguments gst-plugins-good)
        ((#:tests? #f #f) #f)))))

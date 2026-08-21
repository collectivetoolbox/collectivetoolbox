;;; Patch for ffmpeg cross-compilation without rav1e and with cross-compile flags.
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

(define-module (patches ffmpeg)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (ice-9 match)
  #:use-module (gnu packages video)
  #:export (ffmpeg-fixed-proc ffmpeg-fixed ffmpeg-6-fixed))

(define (ffmpeg-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (modify-inputs (package-inputs pkg)
       (delete "rav1e" "sdl2")))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(cons "--disable-ffplay"
                (delete "--enable-librav1e" #$flags)))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (replace 'configure
              (lambda* (#:key outputs configure-flags target native-inputs inputs #:allow-other-keys)
                (let* ((out (assoc-ref outputs "out"))
                       (build-inputs (or native-inputs inputs))
                       (pkg-cfg (search-input-file build-inputs
                                                   (if target
                                                       (string-append "bin/" target "-pkg-config")
                                                       "bin/pkg-config"))))
                  (substitute* "configure"
                    (("#! */bin/sh") (string-append "#!" (which "sh"))))
                  (setenv "SHELL" (which "bash"))
                  (setenv "CONFIG_SHELL" (which "bash"))
                  (catch #t
                    (lambda ()
                      (apply invoke
                             "./configure"
                             (string-append "--prefix=" out)
                             (string-append "--extra-ldflags=-Wl,-rpath=" out "/lib")
                             (append (if target
                                         (list "--enable-cross-compile"
                                               (string-append "--cross-prefix=" target "-")
                                               "--target-os=linux"
                                               (string-append "--arch=" (if (string-prefix? "i686" target) "x86_32" target))
                                               (string-append "--cc=" target "-gcc")
                                               (string-append "--cxx=" target "-g++")
                                               (string-append "--pkg-config=" pkg-cfg))
                                         '())
                                     (delete "--enable-librav1e" configure-flags))))
                    (lambda (key . args)
                      (when (file-exists? "ffbuild/config.log")
                        (format #t "=== ffbuild/config.log tail ===~%")
                        (force-output)
                        (system* "tail" "-n" "100" "ffbuild/config.log"))
                      (apply throw key args))))))))))))

(define ffmpeg-fixed #f)
(define ffmpeg-6-fixed #f)

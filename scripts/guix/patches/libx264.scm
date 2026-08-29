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

;;; Patch for libx264 to supply config in native-inputs, fix 32-bit host detection, and --cross-prefix.

(define-module (patches libx264)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages autotools)
  #:use-module (gnu packages video)
  #:export (libx264-fixed-proc libx264-fixed))

(define (libx264-fixed-proc pkg)
  (package
    (inherit pkg)
    (native-inputs
     (cons `("config" ,config)
           (package-native-inputs pkg)))
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (replace 'configure
              (lambda* (#:key outputs configure-flags target system #:allow-other-keys)
                (let* ((out (assoc-ref outputs "out"))
                       (host-triplet (or target
                                         (if (and system (string-prefix? "i686" system))
                                             "i686-linux"
                                             #f))))
                  (catch #t
                    (lambda ()
                      (apply invoke
                             "./configure"
                             (string-append "--prefix=" out)
                             (append (if target
                                         (list (string-append "--cross-prefix=" target "-"))
                                         '())
                                     (if host-triplet
                                         (list (string-append "--host=" host-triplet))
                                         '())
                                     '("--enable-shared"
                                       "--disable-cli"
                                       "--enable-pic"))))
                    (lambda (key . args)
                      (when (file-exists? "config.log")
                        (format #t "=== config.log tail ===~%")
                        (force-output)
                        (system* "cat" "config.log"))
                      (apply throw key args))))))))))))

(define libx264-fixed
  (libx264-fixed-proc libx264))

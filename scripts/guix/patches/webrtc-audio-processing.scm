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

;;; Patch for webrtc-audio-processing to patch arch.h using substitute* and ensure libdir is lib.

(define-module (patches webrtc-audio-processing)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages audio)
  #:export (webrtc-audio-processing-fixed-proc webrtc-audio-processing-fixed))

(define (webrtc-audio-processing-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(cons* "-Dlibdir=lib" #$flags))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (add-before 'configure 'patch-arch
              (lambda _
                (substitute* "webrtc/rtc_base/system/arch.h"
                  (("elif defined\\(_M_IX86\\) \\|\\| defined\\(__i386__\\)")
                   "elif defined(__SSE__) && (defined(_M_IX86) || defined(__i386__))")
                  (("#error Please add support for your architecture in rtc_base/system/arch.h")
                   (string-append
                    "/* instead of failing, use typical unix defines... */\n"
                    "#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__\n"
                    "#define WEBRTC_ARCH_LITTLE_ENDIAN\n"
                    "#elif __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__\n"
                    "#define WEBRTC_ARCH_BIG_ENDIAN\n"
                    "#else\n"
                    "#error __BYTE_ORDER__ is not defined\n"
                    "#endif\n"
                    "#if defined(__LP64__)\n"
                    "#define WEBRTC_ARCH_64_BITS\n"
                    "#else\n"
                    "#define WEBRTC_ARCH_32_BITS\n"
                    "#endif\n")))))))))))

(define webrtc-audio-processing-fixed
  (webrtc-audio-processing-fixed-proc webrtc-audio-processing))

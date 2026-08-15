;;; Patch for ffmpeg cross-compilation without rav1e (working around Guix cargo-cross-build bug).
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
  #:use-module (ice-9 match)
  #:use-module (gnu packages video)
  #:export (ffmpeg-fixed-proc ffmpeg-fixed ffmpeg-6-fixed))

(define (ffmpeg-fixed-proc pkg)
  (package
    (inherit pkg)
    (inputs
     (map (match-lambda
            (("rav1e" . _) #f)
            ((name (? package? p))
             (list name ((@ (patches) apply-patches) p)))
            ((name (? package? p) output)
             (list name ((@ (patches) apply-patches) p) output))
            (other other))
          (filter (match-lambda (("rav1e" . _) #f) (_ #t))
                  (package-inputs pkg))))))

(define ffmpeg-fixed
  (ffmpeg-fixed-proc ffmpeg))

(define ffmpeg-6-fixed
  (ffmpeg-fixed-proc ffmpeg-6))

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

;;; Patch for xvfb-run to unpack debian patch directly without running out of memory.

(define-module (patches xvfb-run)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages xorg)
  #:export (xvfb-run-fixed-proc xvfb-run-fixed))

(define (xvfb-run-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (replace 'unpack
              (lambda* (#:key inputs #:allow-other-keys)
                (use-modules (ice-9 rdelim))
                (let* ((diff-gz (assoc-ref inputs "source"))
                       (extract-file
                        (lambda (diff-port target-path out-file)
                          (seek diff-port 0 SEEK_SET)
                          (let ((out (open-output-file out-file)))
                            (let loop ((in-target #f))
                              (let ((line (read-line diff-port)))
                                (cond
                                 ((eof-object? line)
                                  (close-port out))
                                 ((string-prefix? "+++ " line)
                                  (if (string-contains line target-path)
                                      (loop #t)
                                      (if in-target
                                          (close-port out)
                                          (loop #f))))
                                 ((and in-target (string-prefix? "--- " line))
                                  (close-port out))
                                 (in-target
                                  (cond
                                   ((string-prefix? "+" line)
                                    (display (substring line 1) out)
                                    (newline out))
                                   ((string-prefix? " " line)
                                    (display (substring line 1) out)
                                    (newline out)))
                                  (loop #t))
                                 (else
                                  (loop #f)))))))))
                  (mkdir-p "debian/local")
                  (let ((diff-file "diff.patch"))
                    (with-output-to-file diff-file
                      (lambda ()
                        (invoke "gzip" "-dc" diff-gz)))
                    (call-with-input-file diff-file
                      (lambda (port)
                        (extract-file port "debian/local/xvfb-run.1" "debian/local/xvfb-run.1")
                        (extract-file port "debian/local/xvfb-run" "debian/local/xvfb-run")))
                    (delete-file diff-file))
                  (chdir "debian/local"))))))))))

(define xvfb-run-fixed
  (xvfb-run-fixed-proc xvfb-run))

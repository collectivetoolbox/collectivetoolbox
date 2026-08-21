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

;;; Patch for libcddb to pass autoconf malloc/realloc nonnull checks when cross-compiling.

(define-module (patches libcddb)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages cdrom)
  #:export (libcddb-fixed-proc libcddb-fixed))

(define (libcddb-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:configure-flags flags #~'())
        #~(append #$flags '("ac_cv_func_malloc_0_nonnull=yes"
                            "ac_cv_func_realloc_0_nonnull=yes")))))))

(define libcddb-fixed
  (libcddb-fixed-proc libcddb))

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

;;; Patch for ORBit2 to supply cross-compilation alignment values.

(define-module (patches orbit2)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages gnome)
  #:use-module (ice-9 match)
  #:export (orbit2-fixed-proc orbit2-fixed))

(define (orbit2-fixed-proc pkg)
  (package
    (inherit pkg)

    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags '("ac_cv_alignof_CORBA_octet=1"
                            "ac_cv_alignof_CORBA_boolean=1"
                            "ac_cv_alignof_CORBA_char=1"
                            "ac_cv_alignof_CORBA_wchar=2"
                            "ac_cv_alignof_CORBA_short=2"
                            "ac_cv_alignof_CORBA_long=4"
                            "ac_cv_alignof_CORBA_long_long=4"
                            "ac_cv_alignof_CORBA_float=4"
                            "ac_cv_alignof_CORBA_double=4"
                            "ac_cv_alignof_CORBA_long_double=4"
                            "ac_cv_alignof_CORBA_struct=1"
                            "ac_cv_alignof_CORBA_pointer=4")))))))

(define orbit2-fixed #f)

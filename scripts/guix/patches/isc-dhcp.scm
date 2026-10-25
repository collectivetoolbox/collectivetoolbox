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

;;; Patch for isc-dhcp to disable zlib in bundled bind configure.

(define-module (patches isc-dhcp)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (gnu packages admin)
  #:export (isc-dhcp-fixed-proc isc-dhcp-fixed))

(define (isc-dhcp-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:phases phases)
        `(modify-phases ,phases
           (replace 'post-configure
             (lambda* (#:key outputs #:allow-other-keys)
               (substitute* "includes/dhcpd.h"
                 (("#define[[:blank:]]+_PATH_DHCLIENT_SCRIPT.*")
                  (let ((out (assoc-ref outputs "out")))
                    (string-append "#define _PATH_DHCLIENT_SCRIPT \""
                                   out "/libexec/dhclient-script\"\n"))))
               (with-directory-excursion "bind"
                 (substitute* "Makefile"
                   (("\\./configure ")
                    (let ((sh (which "sh")))
                      (string-append "./configure --without-zlib CONFIG_SHELL="
                                     sh " SHELL=" sh " "))))
                 (let ((bind-directory "bind-9.11.37"))
                   (invoke "tar" "xf" "bind.tar.gz")
                   (for-each patch-shebang (find-files bind-directory ".*"))
                   (substitute* (string-append bind-directory "/configure")
                     (("/usr/bin/file") (which "file")))
                   (invoke "tar" "cf" "bind.tar.gz" bind-directory
                           "--sort=name" "--mtime=@0" "--owner=root:0" "--group=root:0")))))))))))

(define isc-dhcp-fixed
  (isc-dhcp-fixed-proc isc-dhcp))


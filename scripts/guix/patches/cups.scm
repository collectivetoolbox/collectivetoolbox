;;; Patch for cups, cups-minimal, and cups-filters to disable tests and propagate patches.
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

(define-module (patches cups)
  #:use-module (guix packages)
  #:use-module (guix utils)
  #:use-module (guix gexp)
  #:use-module (gnu packages cups)
  #:export (cups-minimal-fixed-proc
            cups-minimal-fixed
            cups-fixed-proc
            cups-fixed
            cups-filters-fixed-proc
            cups-filters-fixed))

(define (cups-minimal-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)))))

(define cups-minimal-fixed
  (cups-minimal-fixed-proc cups-minimal))

(define (cups-filters-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)))))

(define cups-filters-fixed
  (cups-filters-fixed-proc cups-filters))

(define (cups-fixed-proc pkg)
  (package
    (inherit pkg)
    (arguments
     (substitute-keyword-arguments (package-arguments pkg)
       ((#:tests? _ #f) #f)
       ((#:configure-flags flags #~'())
        #~(append #$flags
                  (list "--with-languages=all")))
       ((#:phases phases #~%standard-phases)
        #~(modify-phases #$phases
            (delete 'check)
            (add-after 'install 'install-cups-filters
              (lambda* (#:key inputs outputs #:allow-other-keys)
                (let ((out (assoc-ref outputs "out"))
                      (filters (assoc-ref inputs "cups-filters")))
                  (when filters
                    ;; Filters.
                    (mkdir-p (string-append out "/lib/cups/filter"))
                    (for-each
                     (lambda (f)
                       (symlink f
                                (string-append out "/lib/cups/filter/"
                                               (basename f))))
                     (find-files (string-append filters "/lib/cups/filter")))

                    ;; Backends.
                    (mkdir-p (string-append out "/lib/cups/backend"))
                    (for-each
                     (lambda (f)
                       (symlink (string-append filters f)
                                (string-append out "/lib/cups/backend/"
                                               (basename f))))
                     '("/lib/cups/backend/parallel"
                       "/lib/cups/backend/serial"))

                    ;; Banners.
                    (let ((banners "/share/cups/banners"))
                      (delete-file-recursively (string-append out banners))
                      (symlink (string-append filters banners)
                               (string-append out banners)))

                    ;; Assorted data.
                    (let ((data "/share/cups/data"))
                      (delete-file-recursively (string-append out data))
                      (symlink (string-append filters data)
                               (string-append out data)))))))))))))

(define cups-fixed
  (cups-fixed-proc cups))

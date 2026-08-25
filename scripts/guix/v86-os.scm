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

;;; Minimal Guix Operating System configuration for 32-bit x86 (i686-linux) with GUI for v86.

(use-modules (gnu)
             (gnu system)
             (gnu packages base)
             (gnu packages xorg)
             (gnu packages wm)
             (gnu packages bash)
             (gnu packages virtualization)
             (gnu packages gnuzilla)
             (gnu packages web-browsers)
             (gnu packages tmux)
             (gnu services)
             (gnu services desktop)
             (gnu services xorg)
             (gnu services networking)
             (guix packages)
             (guix gexp)
             (srfi srfi-1)
             (patches))

(define raw-os-packages
  (append (list xorg-server
                xf86-video-vesa
                xf86-video-fbdev
                openbox
                xterm
                bash
                tmux
                dillo
                qemu)
          %base-packages))

(define os-packages
  (map (lambda (pkg)
         (if (package? pkg)
             (apply-patches pkg)
             pkg))
       raw-os-packages))

(define (all-transitive-sources packages)
  "Return a list of all source origins for PACKAGES and their transitive closure."
  (let* ((closure (package-closure (filter package? packages)))
         (sources (filter-map package-source closure)))
    (delete-duplicates
     (filter origin? sources)
     (lambda (a b)
       (equal? (origin-uri a) (origin-uri b))))))

(define (system-sources-service packages)
  "Create an etc-service entry exposing all source tarballs under /etc/sources,
ensuring all source origins are fetched and retained in the system store closure."
  (simple-service 'system-sources
                  etc-service-type
                  `(("sources"
                     ,(file-union
                       "system-sources"
                       (map (lambda (src)
                              (list (or (origin-actual-file-name src)
                                        (origin-file-name src)
                                        "source-tarball")
                                    src))
                            (all-transitive-sources packages)))))))

(operating-system
  (host-name "ctoolbox-v86")
  (timezone "UTC")
  (locale "en_US.utf8")

  (bootloader (bootloader-configuration
                (bootloader grub-bootloader)
                (targets '("/dev/sda"))))

  (file-systems (cons (file-system
                        (device "host9p")
                        (mount-point "/")
                        (type "9p")
                        (options "trans=virtio,cache=loose")
                        (needed-for-boot? #t))
                      %base-file-systems))

  (initrd-modules (append '("virtio" "virtio_pci" "9p" "9pnet" "9pnet_virtio")
                          %base-initrd-modules))

  (packages os-packages)

  (services (cons* (service static-networking-service-type '())
                   (service provenance-service-type)
                   (system-sources-service os-packages)
                   (simple-service 'v86-session-environment
                                   session-environment-service-type
                                   '(("GALLIUM_DRIVER" . "llvmpipe")
                                     ("MOZ_WEBRENDER" . "1")))
                   %base-services)))



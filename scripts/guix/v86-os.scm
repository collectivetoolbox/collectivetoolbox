;;; Minimal Guix Operating System configuration for 32-bit x86 (i686-linux) with GUI for v86.
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
             (patches))

(define qemu-patched (apply-patches qemu))

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

  (packages (cons* xorg-server
                   xf86-video-vesa
                   xf86-video-fbdev
                   openbox
                   xterm
                   bash
                   dillo
                   tmux
                   qemu-patched
                   %base-packages))

  (services (cons* (service static-networking-service-type '())
                   (simple-service 'v86-session-environment
                                   session-environment-service-type
                                   '(("GALLIUM_DRIVER" . "llvmpipe")
                                     ("MOZ_WEBRENDER" . "1")))
                   %base-services)))


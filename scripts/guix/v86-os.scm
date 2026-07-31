;; Minimal Guix Operating System configuration for 32-bit x86 (i686-linux) with GUI for v86.
;; Copyright (c) 2026 Collective Toolbox project.

(use-modules (gnu)
             (gnu system)
             (gnu packages base)
             (gnu packages xorg)
             (gnu packages wm)
             (gnu packages bash)
             (gnu packages virtualization)
             (gnu services desktop)
             (gnu services xorg)
             (gnu services networking))

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
                   qemu
                   %base-packages))

  (services (cons* (service static-networking-service-type '())
                   %base-services)))

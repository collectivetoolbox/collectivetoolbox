//! Minimal 32-bit POSIX Rust init process for v86 Guix bootloader.
//!
//! Mounts `/proc`, `/sys`, `/dev`, loads 9p and virtio kernel modules,
//! mounts 9pfs root filesystem, and executes X11 + Openbox inside chroot.

#![no_std]
#![no_main]
#![expect(unsafe_code, reason = "Bare-metal no_std init binary using raw Linux int 0x80 syscalls and C memory functions")]

use core::cell::UnsafeCell;
use core::ffi::CStr;
use core::panic::PanicInfo;

struct RawBuffer<const N: usize>(UnsafeCell<[u8; N]>);
// SAFETY: RawBuffer is only used as a static global buffer in single-threaded bare-metal init process where no concurrent access occurs.
unsafe impl<const N: usize> Sync for RawBuffer<N> {}

static MODULE_BUF: RawBuffer<{ 2048 * 1024 }> = RawBuffer(UnsafeCell::new([0; 2048 * 1024]));
static COPY_BUF: RawBuffer<{ 64 * 1024 }> = RawBuffer(UnsafeCell::new([0; 64 * 1024]));

/// C ABI memset implementation for bare-metal init.
///
/// # Safety
/// `s` must point to a valid writable buffer of at least `n` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i: usize = 0;
    let val = c.to_le_bytes()[0];
    while i < n {
        // SAFETY: Pointer offset is within valid buffer bounds [0, n).
        let ptr = unsafe { s.add(i) };
        // SAFETY: Writing byte value to valid pointer offset.
        unsafe {
            *ptr = val;
        }
        i = i.saturating_add(1);
    }
    s
}

/// C ABI memcpy implementation for bare-metal init.
///
/// # Safety
/// `dest` and `src` must point to valid buffers of at least `n` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i: usize = 0;
    while i < n {
        // SAFETY: Source pointer offset is within valid bounds [0, n).
        let src_ptr = unsafe { src.add(i) };
        // SAFETY: Reading byte from valid source pointer offset.
        let val = unsafe { *src_ptr };
        // SAFETY: Destination pointer offset is within valid bounds [0, n).
        let dest_ptr = unsafe { dest.add(i) };
        // SAFETY: Writing byte to valid destination pointer offset.
        unsafe {
            *dest_ptr = val;
        }
        i = i.saturating_add(1);
    }
    dest
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        #[cfg(target_arch = "x86")]
        // SAFETY: Executing CPU pause instruction in infinite panic loop.
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "x86"))]
        {
            core::hint::spin_loop();
        }
    }
}

#[inline]
unsafe fn syscall1(n: usize, a1: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    // SAFETY: Executing x86 int 0x80 Linux syscall interface.
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inout("eax") n => ret,
            in("ebx") a1,
            options(nostack, preserves_flags)
        );
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (n, a1);
        ret = -1;
    }
    ret
}

#[inline]
unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    // SAFETY: Executing x86 int 0x80 Linux syscall interface.
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inout("eax") n => ret,
            in("ebx") a1,
            in("ecx") a2,
            options(nostack, preserves_flags)
        );
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (n, a1, a2);
        ret = -1;
    }
    ret
}

#[inline]
unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    // SAFETY: Executing x86 int 0x80 Linux syscall interface.
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inout("eax") n => ret,
            in("ebx") a1,
            in("ecx") a2,
            in("edx") a3,
            options(nostack, preserves_flags)
        );
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (n, a1, a2, a3);
        ret = -1;
    }
    ret
}

#[inline]
unsafe fn syscall5(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    // SAFETY: Executing x86 int 0x80 Linux syscall interface with saved/restored ESI register.
    unsafe {
        core::arch::asm!(
            "push esi",
            "mov esi, {a4}",
            "int 0x80",
            "pop esi",
            a4 = in(reg) a4,
            inout("eax") n => ret,
            in("ebx") a1,
            in("ecx") a2,
            in("edx") a3,
            in("edi") a5,
            options(preserves_flags)
        );
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (n, a1, a2, a3, a4, a5);
        ret = -1;
    }
    ret
}

#[inline]
unsafe fn sys_open(path: *const u8, flags: usize, mode: usize) -> isize {
    // SAFETY: Executing open syscall (5).
    unsafe { syscall3(5, path.addr(), flags, mode) }
}

#[inline]
unsafe fn sys_read(fd: usize, buf: *mut u8, count: usize) -> isize {
    // SAFETY: Executing read syscall (3).
    unsafe { syscall3(3, fd, buf.addr(), count) }
}

#[inline]
unsafe fn sys_write(fd: usize, buf: *const u8, count: usize) -> isize {
    // SAFETY: Executing write syscall (4).
    unsafe { syscall3(4, fd, buf.addr(), count) }
}

#[inline]
unsafe fn sys_close(fd: usize) -> isize {
    // SAFETY: Executing close syscall (6).
    unsafe { syscall1(6, fd) }
}

#[inline]
unsafe fn sys_dup2(oldfd: usize, newfd: usize) -> isize {
    // SAFETY: Executing dup2 syscall (63).
    unsafe { syscall2(63, oldfd, newfd) }
}

#[inline]
unsafe fn sys_mkdir(path: *const u8, mode: usize) -> isize {
    // SAFETY: Executing mkdir syscall (39).
    unsafe { syscall2(39, path.addr(), mode) }
}

#[inline]
unsafe fn sys_mount(
    dev: *const u8,
    dir: *const u8,
    fstype: *const u8,
    flags: usize,
    data: *const u8,
) -> isize {
    // SAFETY: Executing mount syscall (21).
    unsafe { syscall5(21, dev.addr(), dir.addr(), fstype.addr(), flags, data.addr()) }
}

#[inline]
unsafe fn sys_mknod(path: *const u8, mode: usize, dev: usize) -> isize {
    // SAFETY: Executing mknod syscall (14).
    unsafe { syscall3(14, path.addr(), mode, dev) }
}

#[inline]
unsafe fn sys_chroot(path: *const u8) -> isize {
    // SAFETY: Executing chroot syscall (61).
    unsafe { syscall1(61, path.addr()) }
}

#[inline]
unsafe fn sys_chdir(path: *const u8) -> isize {
    // SAFETY: Executing chdir syscall (12).
    unsafe { syscall1(12, path.addr()) }
}

#[inline]
unsafe fn sys_execve(
    file: *const u8,
    argv: *const *const u8,
    envp: *const *const u8,
) -> isize {
    // SAFETY: Executing execve syscall (11).
    unsafe { syscall3(11, file.addr(), argv.addr(), envp.addr()) }
}

#[inline]
unsafe fn sys_init_module(image: *const u8, len: usize, params: *const u8) -> isize {
    // SAFETY: Executing init_module syscall (128).
    unsafe { syscall3(128, image.addr(), len, params.addr()) }
}

#[inline]
unsafe fn sys_ioctl(fd: usize, cmd: usize, arg: *mut u8) -> isize {
    // SAFETY: Executing ioctl syscall (54).
    unsafe { syscall3(54, fd, cmd, arg.addr()) }
}

#[repr(C)]
#[expect(clippy::struct_field_names, reason = "Matching C ABI termios struct layout in Linux kernel")]
struct Termios {
    c_iflag: u32,
    c_oflag: u32,
    c_cflag: u32,
    c_lflag: u32,
    c_line: u8,
    c_cc: [u8; 32],
    c_ispeed: u32,
    c_ospeed: u32,
}

unsafe fn disable_echo(fd: usize) {
    let mut t = Termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c_ispeed: 0,
        c_ospeed: 0,
    };
    let t_ptr: *mut Termios = &raw mut t;
    // SAFETY: Querying terminal attributes via TCGETS ioctl.
    if unsafe { sys_ioctl(fd, 0x5401, t_ptr.cast()) } == 0 {
        // SAFETY: Setting terminal attributes via TCSETS ioctl.
        let _ = unsafe { sys_ioctl(fd, 0x5402, t_ptr.cast()) };
    }
}

fn print(s: &[u8]) {
    // SAFETY: Writing string slice to standard output (file descriptor 1).
    unsafe {
        sys_write(1, s.as_ptr(), s.len());
    }
}

fn print_num(mut val: isize) {
    let mut buf = [0_u8; 32];
    let mut i: usize = 0;
    if val < 0 {
        print(b"-");
        val = val.saturating_neg();
    }
    if val == 0 {
        print(b"0");
        return;
    }
    while val > 0 && i < 30 {
        let digit = u8::try_from(val.rem_euclid(10)).unwrap_or(0);
        if let Some(slot) = buf.get_mut(i) {
            *slot = b'0'.saturating_add(digit);
        }
        val = val.div_euclid(10);
        i = i.saturating_add(1);
    }
    while i > 0 {
        i = i.saturating_sub(1);
        if let Some(&b) = buf.get(i) {
            print(&[b]);
        }
    }
}

fn load_module(path: &CStr) {
    // SAFETY: Opening module file descriptor.
    let fd = unsafe { sys_open(path.as_ptr().cast(), 0, 0) };
    if fd < 0 {
        print(b"[INIT] Mod open failed: ");
        print(path.to_bytes());
        print(b"\n");
        return;
    }
    let fd_u = usize::try_from(fd).unwrap_or(0);
    let mod_buf_ptr = MODULE_BUF.0.get().cast::<u8>();
    let mod_cap = 2048_usize.saturating_mul(1024);
    let mut total: usize = 0;
    while total < mod_cap {
        let remain = mod_cap.saturating_sub(total);
        // SAFETY: Offset is within MODULE_BUF allocated capacity.
        let buf_ptr = unsafe { mod_buf_ptr.add(total) };
        // SAFETY: Reading up to remain bytes into MODULE_BUF.
        let n = unsafe { sys_read(fd_u, buf_ptr, remain) };
        if n <= 0 {
            break;
        }
        let n_u = usize::try_from(n).unwrap_or(0);
        total = total.saturating_add(n_u);
    }
    // SAFETY: Closing module file descriptor.
    unsafe {
        sys_close(fd_u);
    }
    if total > 0 {
        // SAFETY: Calling init_module syscall with loaded module image bytes.
        let res = unsafe { sys_init_module(mod_buf_ptr, total, c"".as_ptr().cast()) };
        if res != 0 {
            print(b"[INIT] init_module ");
            print(path.to_bytes());
            print(b" returned ");
            print_num(res);
            print(b"\n");
        } else {
            print(b"[INIT] Loaded ");
            print(path.to_bytes());
            print(b"\n");
        }
    }
}

/// Entry point for 32-bit v86 POSIX Rust init process.
#[unsafe(no_mangle)]
#[expect(clippy::too_many_lines, reason = "Sequential bare-metal init process sequence")]
pub extern "C" fn _start() -> ! {
    // SAFETY: Creating /dev directory for tty device setup.
    unsafe {
        sys_mkdir(c"/dev".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating /dev/ttyS0 device node with major 4 minor 64.
    unsafe {
        sys_mknod(c"/dev/ttyS0".as_ptr().cast(), 0o020_666, 4_usize.checked_shl(8).unwrap_or(0) | 64_usize);
    }

    // SAFETY: Opening serial console ttyS0.
    let fd = unsafe { sys_open(c"/dev/ttyS0".as_ptr().cast(), 2, 0) };
    if fd >= 0 {
        let fd_u = usize::try_from(fd).unwrap_or(0);
        if fd_u != 0 {
            // SAFETY: Duplicating console file descriptor to stdin.
            unsafe {
                sys_dup2(fd_u, 0);
            }
        }
        // SAFETY: Duplicating stdin to stdout.
        unsafe {
            sys_dup2(0, 1);
        }
        // SAFETY: Duplicating stdin to stderr.
        unsafe {
            sys_dup2(0, 2);
        }
        // SAFETY: Disabling terminal echo on console.
        unsafe {
            disable_echo(0);
        }
    }

    print(b"\n[INIT] Starting 32-bit POSIX Rust init for v86 Guix...\n");

    // SAFETY: Creating system mount point directories.
    unsafe {
        sys_mkdir(c"/proc".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating sysfs mount point directory.
    unsafe {
        sys_mkdir(c"/sys".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating tmpfs mount point directory.
    unsafe {
        sys_mkdir(c"/tmp".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating root mount point directory.
    unsafe {
        sys_mkdir(c"/root".as_ptr().cast(), 0o755);
    }

    // SAFETY: Mounting procfs on /proc.
    unsafe {
        sys_mount(c"proc".as_ptr().cast(), c"/proc".as_ptr().cast(), c"proc".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting sysfs on /sys.
    unsafe {
        sys_mount(c"sysfs".as_ptr().cast(), c"/sys".as_ptr().cast(), c"sysfs".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting devtmpfs on /dev.
    unsafe {
        sys_mount(c"devtmpfs".as_ptr().cast(), c"/dev".as_ptr().cast(), c"devtmpfs".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting tmpfs on /tmp.
    unsafe {
        sys_mount(c"tmpfs".as_ptr().cast(), c"/tmp".as_ptr().cast(), c"tmpfs".as_ptr().cast(), 0, core::ptr::null());
    }

    // SAFETY: Creating devpts mount point directory.
    unsafe {
        sys_mkdir(c"/dev/pts".as_ptr().cast(), 0o755);
    }
    // SAFETY: Mounting devpts on /dev/pts.
    unsafe {
        sys_mount(c"devpts".as_ptr().cast(), c"/dev/pts".as_ptr().cast(), c"devpts".as_ptr().cast(), 0, core::ptr::null());
    }

    print(b"[INIT] Loading kernel modules...\n");
    load_module(c"/lib/modules/virtio_ring.ko");
    load_module(c"/lib/modules/virtio.ko");
    load_module(c"/lib/modules/virtio_pci_legacy_dev.ko");
    load_module(c"/lib/modules/virtio_pci_modern_dev.ko");
    load_module(c"/lib/modules/virtio_pci.ko");
    load_module(c"/lib/modules/fscache.ko");
    load_module(c"/lib/modules/netfs.ko");
    load_module(c"/lib/modules/9pnet.ko");
    load_module(c"/lib/modules/9pnet_virtio.ko");
    load_module(c"/lib/modules/9p.ko");
    load_module(c"/lib/modules/fb_sys_fops.ko");
    load_module(c"/lib/modules/sysfillrect.ko");
    load_module(c"/lib/modules/syscopyarea.ko");
    load_module(c"/lib/modules/sysimgblt.ko");
    load_module(c"/lib/modules/cirrusfb.ko");
    load_module(c"/lib/modules/cec.ko");
    load_module(c"/lib/modules/drm.ko");
    load_module(c"/lib/modules/drm_display_helper.ko");
    load_module(c"/lib/modules/drm_kms_helper.ko");
    load_module(c"/lib/modules/drm_client_lib.ko");
    load_module(c"/lib/modules/ttm.ko");
    load_module(c"/lib/modules/drm_ttm_helper.ko");
    load_module(c"/lib/modules/drm_shmem_helper.ko");
    load_module(c"/lib/modules/drm_vram_helper.ko");
    load_module(c"/lib/modules/bochs.ko");
    load_module(c"/lib/modules/uvesafb.ko");

    print(b"[INIT] Mounting 9p host9p on /root...\n");
    // SAFETY: Mounting 9p host filesystem on /root.
    let mres = unsafe {
        sys_mount(
            c"host9p".as_ptr().cast(),
            c"/root".as_ptr().cast(),
            c"9p".as_ptr().cast(),
            0,
            c"trans=virtio,cache=loose".as_ptr().cast(),
        )
    };
    if mres != 0 {
        print(b"[INIT] Warning: 9p mount returned ");
        print_num(mres);
        print(b"\n");
    }

    // SAFETY: Creating chroot target directories.
    unsafe {
        sys_mkdir(c"/root/proc".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating /root/sys directory.
    unsafe {
        sys_mkdir(c"/root/sys".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating /root/dev directory.
    unsafe {
        sys_mkdir(c"/root/dev".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating /root/tmp directory.
    unsafe {
        sys_mkdir(c"/root/tmp".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating /root/var directory.
    unsafe {
        sys_mkdir(c"/root/var".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating /root/var/log directory.
    unsafe {
        sys_mkdir(c"/root/var/log".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating /root/var/run directory.
    unsafe {
        sys_mkdir(c"/root/var/run".as_ptr().cast(), 0o755);
    }

    // SAFETY: Mounting procfs on /root/proc.
    unsafe {
        sys_mount(c"proc".as_ptr().cast(), c"/root/proc".as_ptr().cast(), c"proc".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting sysfs on /root/sys.
    unsafe {
        sys_mount(c"sysfs".as_ptr().cast(), c"/root/sys".as_ptr().cast(), c"sysfs".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting devtmpfs on /root/dev.
    unsafe {
        sys_mount(c"devtmpfs".as_ptr().cast(), c"/root/dev".as_ptr().cast(), c"devtmpfs".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting tmpfs on /root/tmp.
    unsafe {
        sys_mount(c"tmpfs".as_ptr().cast(), c"/root/tmp".as_ptr().cast(), c"tmpfs".as_ptr().cast(), 0, core::ptr::null());
    }

    // SAFETY: Creating /root/dev/pts directory.
    unsafe {
        sys_mkdir(c"/root/dev/pts".as_ptr().cast(), 0o755);
    }
    // SAFETY: Mounting devpts on /root/dev/pts.
    unsafe {
        sys_mount(c"devpts".as_ptr().cast(), c"/root/dev/pts".as_ptr().cast(), c"devpts".as_ptr().cast(), 0, core::ptr::null());
    }

    // SAFETY: Creating /root/dev/fb0 device node (29, 0).
    unsafe {
        sys_mknod(c"/root/dev/fb0".as_ptr().cast(), 0o020_666, 29_usize.checked_shl(8).unwrap_or(0));
    }
    // SAFETY: Creating /root/dev/tty0 device node (4, 0).
    unsafe {
        sys_mknod(c"/root/dev/tty0".as_ptr().cast(), 0o020_666, 4_usize.checked_shl(8).unwrap_or(0));
    }
    // SAFETY: Creating /root/dev/tty1 device node (4, 1).
    unsafe {
        sys_mknod(c"/root/dev/tty1".as_ptr().cast(), 0o020_666, 4_usize.checked_shl(8).unwrap_or(0) | 1_usize);
    }
    // SAFETY: Creating /root/dev/ttyS0 device node (4, 64).
    unsafe {
        sys_mknod(c"/root/dev/ttyS0".as_ptr().cast(), 0o020_666, 4_usize.checked_shl(8).unwrap_or(0) | 64_usize);
    }
    // SAFETY: Creating /root/dev/zero device node (1, 5).
    unsafe {
        sys_mknod(c"/root/dev/zero".as_ptr().cast(), 0o020_666, 1_usize.checked_shl(8).unwrap_or(0) | 5_usize);
    }
    // SAFETY: Creating /root/dev/null device node (1, 3).
    unsafe {
        sys_mknod(c"/root/dev/null".as_ptr().cast(), 0o020_666, 1_usize.checked_shl(8).unwrap_or(0) | 3_usize);
    }
    // SAFETY: Creating /root/dev/mem device node (1, 1).
    unsafe {
        sys_mknod(c"/root/dev/mem".as_ptr().cast(), 0o020_666, 1_usize.checked_shl(8).unwrap_or(0) | 1_usize);
    }
    // SAFETY: Creating /root/dev/port device node (1, 4).
    unsafe {
        sys_mknod(c"/root/dev/port".as_ptr().cast(), 0o020_666, 1_usize.checked_shl(8).unwrap_or(0) | 4_usize);
    }
    // SAFETY: Creating /root/dev/tty device node (5, 0).
    unsafe {
        sys_mknod(c"/root/dev/tty".as_ptr().cast(), 0o020_666, 5_usize.checked_shl(8).unwrap_or(0));
    }
    // SAFETY: Creating /root/dev/console device node (5, 1).
    unsafe {
        sys_mknod(c"/root/dev/console".as_ptr().cast(), 0o020_666, 5_usize.checked_shl(8).unwrap_or(0) | 1_usize);
    }
    // SAFETY: Creating /root/dev/ptmx device node (5, 2).
    unsafe {
        sys_mknod(c"/root/dev/ptmx".as_ptr().cast(), 0o020_666, 5_usize.checked_shl(8).unwrap_or(0) | 2_usize);
    }

    print(b"[INIT] Copying static shell to /root/tmp/sh...\n");
    // SAFETY: Opening source shell binary /bin/sh.
    let fsin = unsafe { sys_open(c"/bin/sh".as_ptr().cast(), 0, 0) };
    // SAFETY: Opening destination shell path /root/tmp/sh for writing.
    let fsout = unsafe { sys_open(c"/root/tmp/sh".as_ptr().cast(), 65 | 512, 0o755) };
    if fsin >= 0 && fsout >= 0 {
        let fsin_u = usize::try_from(fsin).unwrap_or(0);
        let fsout_u = usize::try_from(fsout).unwrap_or(0);
        let copy_buf_ptr = COPY_BUF.0.get().cast::<u8>();
        let copy_cap = 64_usize.saturating_mul(1024);
        loop {
            // SAFETY: Reading up to copy_cap bytes from fsin.
            let n = unsafe { sys_read(fsin_u, copy_buf_ptr, copy_cap) };
            if n <= 0 {
                break;
            }
            let n_u = usize::try_from(n).unwrap_or(0);
            // SAFETY: Writing n_u bytes to fsout.
            unsafe {
                sys_write(fsout_u, copy_buf_ptr, n_u);
            }
        }
        // SAFETY: Closing source file descriptor.
        unsafe {
            sys_close(fsin_u);
        }
        // SAFETY: Closing destination file descriptor.
        unsafe {
            sys_close(fsout_u);
        }
    } else {
        print(b"[INIT] Warning: Copying /bin/sh to /root/tmp/sh failed!\n");
    }

    // Read profile path from `/guix_profile` (written into initrd by asset packer)
    let mut profile_buf = [0_u8; 256];
    // SAFETY: Opening /guix_profile path file descriptor.
    let pfd = unsafe { sys_open(c"/guix_profile".as_ptr().cast(), 0, 0) };
    if pfd >= 0 {
        let pfd_u = usize::try_from(pfd).unwrap_or(0);
        // SAFETY: Reading profile path into profile_buf.
        let pn = unsafe { sys_read(pfd_u, profile_buf.as_mut_ptr(), profile_buf.len()) };
        // SAFETY: Closing profile path file descriptor.
        unsafe {
            sys_close(pfd_u);
        }
        if pn > 0 {
            let mut p_len = usize::try_from(pn).unwrap_or(0);
            while p_len > 0 {
                if let Some(&last) = profile_buf.get(p_len.saturating_sub(1)) {
                    if last == b'\n' || last == b'\r' || last == b' ' || last == b'\0' {
                        p_len = p_len.saturating_sub(1);
                        continue;
                    }
                }
                break;
            }
        }
    }

    // SAFETY: Creating proc mount point in chroot.
    unsafe {
        sys_mkdir(c"/root/proc".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating sys mount point in chroot.
    unsafe {
        sys_mkdir(c"/root/sys".as_ptr().cast(), 0o755);
    }
    // SAFETY: Creating dev mount point in chroot.
    unsafe {
        sys_mkdir(c"/root/dev".as_ptr().cast(), 0o755);
    }
    // SAFETY: Mounting procfs in chroot.
    unsafe {
        sys_mount(c"proc".as_ptr().cast(), c"/root/proc".as_ptr().cast(), c"proc".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting sysfs in chroot.
    unsafe {
        sys_mount(c"sysfs".as_ptr().cast(), c"/root/sys".as_ptr().cast(), c"sysfs".as_ptr().cast(), 0, core::ptr::null());
    }
    // SAFETY: Mounting devtmpfs in chroot.
    unsafe {
        sys_mount(c"devtmpfs".as_ptr().cast(), c"/root/dev".as_ptr().cast(), c"devtmpfs".as_ptr().cast(), 0, core::ptr::null());
    }

    print(b"[INIT] Chrooting into /root...\n");
    // SAFETY: Changing root directory to /root.
    unsafe {
        sys_chroot(c"/root".as_ptr().cast());
    }
    // SAFETY: Changing current directory to /.
    unsafe {
        sys_chdir(c"/".as_ptr().cast());
    }

    let cmd_str = b"\
mkdir -p /bin /usr/bin /etc /tmp/.X11-unix /var/log /var/run
chmod 1777 /tmp /tmp/.X11-unix

for p in /gnu/store/*-profile/bin/* /gnu/store/*-profile/sbin/*; do
    ln -sf \"$p\" /bin/ 2>/dev/null
    ln -sf \"$p\" /usr/bin/ 2>/dev/null
done

for p in /gnu/store/*-profile; do
    export PATH=\"$p/bin:$p/sbin:$PATH\"
done
export PATH=\"/bin:/usr/bin:$PATH\"

echo \"export PATH=$PATH\" > /etc/profile
export DISPLAY=:0;

MP=$(
    echo /gnu/store/*-profile/lib/xorg/modules \\
         /gnu/store/*xorg-server*/lib/xorg/modules | tr ' ' ','
)

cat > /tmp/xorg.conf <<'EOF'
Section \"Device\"
    Identifier \"Card0\"
    Driver \"fbdev\"
    Option \"fbdev\" \"/dev/fb0\"
    Option \"ShadowFB\" \"true\"
EndSection

Section \"Screen\"
    Identifier \"Screen0\"
    Device \"Card0\"
EndSection
EOF

Xorg :0 -modulepath \"${MP}\" -config /tmp/xorg.conf -ac > /tmp/x.log 2>&1 &

for i in $(seq 1 30); do
    if obxprop -display :0 >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

sleep 2
cat /var/log/Xorg.0.log

openbox --display :0 > /tmp/ob.log 2>&1 &

sleep 2
ps aux

exec /tmp/sh -i
\0";

    let path_env = c"PATH=/bin:/usr/bin";
    let home_env = c"HOME=/root";
    let term_env = c"TERM=vt100";
    let display_env = c"DISPLAY=:0";

    let envp: [*const u8; 5] = [
        path_env.as_ptr().cast(),
        home_env.as_ptr().cast(),
        term_env.as_ptr().cast(),
        display_env.as_ptr().cast(),
        core::ptr::null(),
    ];

    let sh_target = c"/tmp/sh";
    let bin_sh_target = c"/bin/sh";
    let bin_bash_target = c"/bin/bash";
    let usr_bin_bash = c"/usr/bin/bash";
    let usr_bin_sh = c"/usr/bin/sh";

    let c_flag = c"-c";

    let targets: [*const u8; 6] = [
        sh_target.as_ptr().cast(),
        bin_sh_target.as_ptr().cast(),
        bin_bash_target.as_ptr().cast(),
        usr_bin_bash.as_ptr().cast(),
        usr_bin_sh.as_ptr().cast(),
        core::ptr::null(),
    ];

    print(b"[INIT] Executing shell inside chroot...\n");
    let mut t: usize = 0;
    while let Some(&target) = targets.get(t) {
        if target.is_null() {
            break;
        }
        let argv: [*const u8; 4] = [target, c_flag.as_ptr().cast(), cmd_str.as_ptr(), core::ptr::null()];
        // SAFETY: Executing shell target binary via execve syscall.
        let res = unsafe { sys_execve(target, argv.as_ptr(), envp.as_ptr()) };
        print(b"[INIT] execve returned ");
        print_num(res);
        print(b"\n");
        t = t.saturating_add(1);
    }

    print(b"[INIT] Error: All sys_execve target attempts failed!\n");
    loop {
        #[cfg(target_arch = "x86")]
        // SAFETY: Executing CPU pause instruction in final halt loop.
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "x86"))]
        {
            core::hint::spin_loop();
        }
    }
}

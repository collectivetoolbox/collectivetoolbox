//! Minimal 32-bit POSIX Rust init process for v86 Guix bootloader.
//!
//! Mounts `/proc`, `/sys`, `/dev`, loads 9p and virtio kernel modules,
//! mounts 9pfs root filesystem, and executes X11 + Openbox inside chroot.

#![no_std]
#![no_main]

#[allow(unused_imports)]
use core::arch::asm;
use core::cell::UnsafeCell;
use core::convert::TryFrom;
use core::panic::PanicInfo;

struct RawBuffer<const N: usize>(UnsafeCell<[u8; N]>);
#[allow(unsafe_code)]
unsafe impl<const N: usize> Sync for RawBuffer<N> {}

static MODULE_BUF: RawBuffer<{ 2048 * 1024 }> = RawBuffer(UnsafeCell::new([0; 2048 * 1024]));
static COPY_BUF: RawBuffer<{ 64 * 1024 }> = RawBuffer(UnsafeCell::new([0; 64 * 1024]));

#[unsafe(no_mangle)]
#[allow(unsafe_code, clippy::as_conversions)]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i: usize = 0;
    let val = c as u8;
    while i < n {
        unsafe {
            *s.add(i) = val;
        }
        i = i.saturating_add(1);
    }
    s
}

#[unsafe(no_mangle)]
#[allow(unsafe_code)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i: usize = 0;
    while i < n {
        unsafe {
            *dest.add(i) = *src.add(i);
        }
        i = i.saturating_add(1);
    }
    dest
}

#[panic_handler]
#[allow(clippy::empty_loop, unsafe_code)]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!("pause", options(nomem, nostack, preserves_flags));
        }
    }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::needless_pass_by_value)]
unsafe fn syscall1(n: usize, a1: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!(
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

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::needless_pass_by_value)]
unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!(
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

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::needless_pass_by_value)]
unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!(
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

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::needless_pass_by_value)]
unsafe fn syscall5(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
    let ret: isize;
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!(
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

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_open(path: *const u8, flags: usize, mode: usize) -> isize {
    unsafe { syscall3(5, path as usize, flags, mode) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_read(fd: usize, buf: *mut u8, count: usize) -> isize {
    unsafe { syscall3(3, fd, buf as usize, count) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_write(fd: usize, buf: *const u8, count: usize) -> isize {
    unsafe { syscall3(4, fd, buf as usize, count) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_close(fd: usize) -> isize {
    unsafe { syscall1(6, fd) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_dup2(oldfd: usize, newfd: usize) -> isize {
    unsafe { syscall2(63, oldfd, newfd) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_mkdir(path: *const u8, mode: usize) -> isize {
    unsafe { syscall2(39, path as usize, mode) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_mount(
    dev: *const u8,
    dir: *const u8,
    fstype: *const u8,
    flags: usize,
    data: *const u8,
) -> isize {
    unsafe { syscall5(21, dev as usize, dir as usize, fstype as usize, flags, data as usize) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_mknod(path: *const u8, mode: usize, dev: usize) -> isize {
    unsafe { syscall3(14, path as usize, mode, dev) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_chroot(path: *const u8) -> isize {
    unsafe { syscall1(61, path as usize) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_chdir(path: *const u8) -> isize {
    unsafe { syscall1(12, path as usize) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_execve(
    file: *const u8,
    argv: *const *const u8,
    envp: *const *const u8,
) -> isize {
    unsafe { syscall3(11, file as usize, argv as usize, envp as usize) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_init_module(image: *const u8, len: usize, params: *const u8) -> isize {
    unsafe { syscall3(128, image as usize, len, params as usize) }
}

#[inline(always)]
#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
unsafe fn sys_ioctl(fd: usize, cmd: usize, arg: *mut u8) -> isize {
    unsafe { syscall3(54, fd, cmd, arg as usize) }
}

#[repr(C)]
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

#[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
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
    let t_ptr: *mut Termios = &mut t;
    if unsafe { sys_ioctl(fd, 0x5401, t_ptr.cast()) } == 0 {
        let _ = unsafe { sys_ioctl(fd, 0x5402, t_ptr.cast()) };
    }
}

#[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
fn print(s: &[u8]) {
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

#[allow(unsafe_code, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
fn load_module(path: &[u8]) {
    let fd = unsafe { sys_open(path.as_ptr(), 0, 0) };
    if fd < 0 {
        print(b"[INIT] Mod open failed: ");
        print(path);
        print(b"\n");
        return;
    }
    let fd_u = usize::try_from(fd).unwrap_or(0);
    let mod_buf_ptr = MODULE_BUF.0.get() as *mut u8;
    let mod_cap = 2048_usize.saturating_mul(1024);
    let mut total: usize = 0;
    unsafe {
        while total < mod_cap {
            let remain = mod_cap.saturating_sub(total);
            let buf_ptr = mod_buf_ptr.add(total);
            let n = sys_read(fd_u, buf_ptr, remain);
            if n <= 0 {
                break;
            }
            let n_u = usize::try_from(n).unwrap_or(0);
            total = total.saturating_add(n_u);
        }
        sys_close(fd_u);
        if total > 0 {
            let res = sys_init_module(mod_buf_ptr, total, b"\0".as_ptr());
            if res != 0 {
                print(b"[INIT] init_module ");
                print(path);
                print(b" returned ");
                print_num(res);
                print(b"\n");
            } else {
                print(b"[INIT] Loaded ");
                print(path);
                print(b"\n");
            }
        }
    }
}

/// Entry point for 32-bit v86 POSIX Rust init process.
#[unsafe(no_mangle)]
#[allow(unsafe_code, clippy::cognitive_complexity, clippy::too_many_lines, clippy::as_conversions, clippy::undocumented_unsafe_blocks)]
pub extern "C" fn _start() -> ! {
    unsafe {
        sys_mkdir(b"/dev\0".as_ptr(), 0o755);
        sys_mknod(b"/dev/ttyS0\0".as_ptr(), 0o020666, (4 << 8) | 64);

        let fd = sys_open(b"/dev/ttyS0\0".as_ptr(), 2, 0);
        if fd >= 0 {
            let fd_u = usize::try_from(fd).unwrap_or(0);
            if fd_u != 0 {
                sys_dup2(fd_u, 0);
            }
            sys_dup2(0, 1);
            sys_dup2(0, 2);
            disable_echo(0);
        }
    }

    print(b"\n[INIT] Starting 32-bit POSIX Rust init for v86 Guix...\n");

    unsafe {
        sys_mkdir(b"/proc\0".as_ptr(), 0o755);
        sys_mkdir(b"/sys\0".as_ptr(), 0o755);
        sys_mkdir(b"/tmp\0".as_ptr(), 0o755);
        sys_mkdir(b"/root\0".as_ptr(), 0o755);

        sys_mount(b"proc\0".as_ptr(), b"/proc\0".as_ptr(), b"proc\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"sysfs\0".as_ptr(), b"/sys\0".as_ptr(), b"sysfs\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"devtmpfs\0".as_ptr(), b"/dev\0".as_ptr(), b"devtmpfs\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"tmpfs\0".as_ptr(), b"/tmp\0".as_ptr(), b"tmpfs\0".as_ptr(), 0, core::ptr::null());

        sys_mkdir(b"/dev/pts\0".as_ptr(), 0o755);
        sys_mount(b"devpts\0".as_ptr(), b"/dev/pts\0".as_ptr(), b"devpts\0".as_ptr(), 0, core::ptr::null());
    }

    print(b"[INIT] Loading kernel modules...\n");
    load_module(b"/lib/modules/virtio_ring.ko\0");
    load_module(b"/lib/modules/virtio.ko\0");
    load_module(b"/lib/modules/virtio_pci_legacy_dev.ko\0");
    load_module(b"/lib/modules/virtio_pci_modern_dev.ko\0");
    load_module(b"/lib/modules/virtio_pci.ko\0");
    load_module(b"/lib/modules/fscache.ko\0");
    load_module(b"/lib/modules/netfs.ko\0");
    load_module(b"/lib/modules/9pnet.ko\0");
    load_module(b"/lib/modules/9pnet_virtio.ko\0");
    load_module(b"/lib/modules/9p.ko\0");
    load_module(b"/lib/modules/fb_sys_fops.ko\0");
    load_module(b"/lib/modules/sysfillrect.ko\0");
    load_module(b"/lib/modules/syscopyarea.ko\0");
    load_module(b"/lib/modules/sysimgblt.ko\0");
    load_module(b"/lib/modules/cirrusfb.ko\0");
    load_module(b"/lib/modules/cec.ko\0");
    load_module(b"/lib/modules/drm.ko\0");
    load_module(b"/lib/modules/drm_display_helper.ko\0");
    load_module(b"/lib/modules/drm_kms_helper.ko\0");
    load_module(b"/lib/modules/drm_client_lib.ko\0");
    load_module(b"/lib/modules/ttm.ko\0");
    load_module(b"/lib/modules/drm_ttm_helper.ko\0");
    load_module(b"/lib/modules/drm_shmem_helper.ko\0");
    load_module(b"/lib/modules/drm_vram_helper.ko\0");
    load_module(b"/lib/modules/bochs.ko\0");
    load_module(b"/lib/modules/uvesafb.ko\0");

    print(b"[INIT] Mounting 9p host9p on /root...\n");
    let mres = unsafe {
        sys_mount(
            b"host9p\0".as_ptr(),
            b"/root\0".as_ptr(),
            b"9p\0".as_ptr(),
            0,
            b"trans=virtio,cache=loose\0".as_ptr(),
        )
    };
    if mres != 0 {
        print(b"[INIT] Warning: 9p mount returned ");
        print_num(mres);
        print(b"\n");
    }

    unsafe {
        sys_mkdir(b"/root/proc\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/sys\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/dev\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/tmp\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/var\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/var/log\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/var/run\0".as_ptr(), 0o755);

        sys_mount(b"proc\0".as_ptr(), b"/root/proc\0".as_ptr(), b"proc\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"sysfs\0".as_ptr(), b"/root/sys\0".as_ptr(), b"sysfs\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"devtmpfs\0".as_ptr(), b"/root/dev\0".as_ptr(), b"devtmpfs\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"tmpfs\0".as_ptr(), b"/root/tmp\0".as_ptr(), b"tmpfs\0".as_ptr(), 0, core::ptr::null());

        sys_mkdir(b"/root/dev/pts\0".as_ptr(), 0o755);
        sys_mount(b"devpts\0".as_ptr(), b"/root/dev/pts\0".as_ptr(), b"devpts\0".as_ptr(), 0, core::ptr::null());

        sys_mknod(b"/root/dev/fb0\0".as_ptr(), 0o020666, (29 << 8) | 0);
        sys_mknod(b"/root/dev/tty0\0".as_ptr(), 0o020666, (4 << 8) | 0);
        sys_mknod(b"/root/dev/tty1\0".as_ptr(), 0o020666, (4 << 8) | 1);
        sys_mknod(b"/root/dev/ttyS0\0".as_ptr(), 0o020666, (4 << 8) | 64);
        sys_mknod(b"/root/dev/zero\0".as_ptr(), 0o020666, (1 << 8) | 5);
        sys_mknod(b"/root/dev/null\0".as_ptr(), 0o020666, (1 << 8) | 3);
        sys_mknod(b"/root/dev/mem\0".as_ptr(), 0o020666, (1 << 8) | 1);
        sys_mknod(b"/root/dev/port\0".as_ptr(), 0o020666, (1 << 8) | 4);
        sys_mknod(b"/root/dev/tty\0".as_ptr(), 0o020666, (5 << 8) | 0);
        sys_mknod(b"/root/dev/console\0".as_ptr(), 0o020666, (5 << 8) | 1);
        sys_mknod(b"/root/dev/ptmx\0".as_ptr(), 0o020666, (5 << 8) | 2);
    }

    print(b"[INIT] Copying static shell to /root/tmp/sh...\n");
    let fsin = unsafe { sys_open(b"/bin/sh\0".as_ptr(), 0, 0) };
    let fsout = unsafe { sys_open(b"/root/tmp/sh\0".as_ptr(), 65 | 512, 0o755) };
    if fsin >= 0 && fsout >= 0 {
        let fsin_u = usize::try_from(fsin).unwrap_or(0);
        let fsout_u = usize::try_from(fsout).unwrap_or(0);
        let copy_buf_ptr = COPY_BUF.0.get() as *mut u8;
        let copy_cap = 64_usize.saturating_mul(1024);
        unsafe {
            loop {
                let n = sys_read(fsin_u, copy_buf_ptr, copy_cap);
                if n <= 0 {
                    break;
                }
                let n_u = usize::try_from(n).unwrap_or(0);
                sys_write(fsout_u, copy_buf_ptr as *const u8, n_u);
            }
            sys_close(fsin_u);
            sys_close(fsout_u);
        }
    } else {
        print(b"[INIT] Warning: Copying /bin/sh to /root/tmp/sh failed!\n");
    }

    // Read profile path from `/guix_profile` (written into initrd by asset packer)
    let mut profile_buf = [0_u8; 256];
    let pfd = unsafe { sys_open(b"/guix_profile\0".as_ptr(), 0, 0) };
    if pfd >= 0 {
        let pfd_u = usize::try_from(pfd).unwrap_or(0);
        let pn = unsafe { sys_read(pfd_u, profile_buf.as_mut_ptr(), profile_buf.len()) };
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

    unsafe {
        sys_mkdir(b"/root/proc\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/sys\0".as_ptr(), 0o755);
        sys_mkdir(b"/root/dev\0".as_ptr(), 0o755);
        sys_mount(b"proc\0".as_ptr(), b"/root/proc\0".as_ptr(), b"proc\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"sysfs\0".as_ptr(), b"/root/sys\0".as_ptr(), b"sysfs\0".as_ptr(), 0, core::ptr::null());
        sys_mount(b"devtmpfs\0".as_ptr(), b"/root/dev\0".as_ptr(), b"devtmpfs\0".as_ptr(), 0, core::ptr::null());
    }

    print(b"[INIT] Chrooting into /root...\n");
    unsafe {
        sys_chroot(b"/root\0".as_ptr());
        sys_chdir(b"/\0".as_ptr());
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

    let env0 = b"PATH=/bin:/usr/bin\0";
    let env1 = b"HOME=/root\0";
    let env2 = b"TERM=vt100\0";
    let env3 = b"DISPLAY=:0\0";

    let envp: [*const u8; 5] = [
        env0.as_ptr(),
        env1.as_ptr(),
        env2.as_ptr(),
        env3.as_ptr(),
        core::ptr::null(),
    ];

    let sh_target = b"/tmp/sh\0";
    let bin_sh_target = b"/bin/sh\0";
    let bin_bash_target = b"/bin/bash\0";
    let usr_bin_bash = b"/usr/bin/bash\0";
    let usr_bin_sh = b"/usr/bin/sh\0";

    let c_flag = b"-c\0";

    let targets: [*const u8; 6] = [
        sh_target.as_ptr(),
        bin_sh_target.as_ptr(),
        bin_bash_target.as_ptr(),
        usr_bin_bash.as_ptr(),
        usr_bin_sh.as_ptr(),
        core::ptr::null(),
    ];

    print(b"[INIT] Executing shell inside chroot...\n");
    let mut t: usize = 0;
    while let Some(&target) = targets.get(t) {
        if target.is_null() {
            break;
        }
        let argv: [*const u8; 4] = [target, c_flag.as_ptr(), cmd_str.as_ptr(), core::ptr::null()];
        let res = unsafe { sys_execve(target, argv.as_ptr(), envp.as_ptr()) };
        print(b"[INIT] execve returned ");
        print_num(res);
        print(b"\n");
        t = t.saturating_add(1);
    }

    print(b"[INIT] Error: All sys_execve target attempts failed!\n");
    loop {
        #[cfg(target_arch = "x86")]
        unsafe {
            asm!("pause", options(nomem, nostack, preserves_flags));
        }
    }
}

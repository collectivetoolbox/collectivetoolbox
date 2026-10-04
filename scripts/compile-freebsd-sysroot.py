#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# This file is part of Collective Toolbox, a database and document workspace and utilities.
# Copyright (C) 2026 Collective Toolbox Developers
# Contact: info@collectivetoolbox.com
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU Affero General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option) any
# later version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
# A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along
# with this program.  If not, see <https://www.gnu.org/licenses/>.

import os
import re
import shutil
import subprocess
import sys

script_dir = os.path.dirname(os.path.realpath(__file__))
workspace_root = os.path.realpath(os.path.join(script_dir, ".."))
os.chdir(workspace_root)

tmp_dir = os.path.join(workspace_root, "built/freebsd_sysroot_tmp")
source_sysroot = os.path.join(workspace_root, "vendor/freebsd-sysroot")
sysroot = os.path.join(workspace_root, "built/freebsd-sysroot")

if os.path.isdir(source_sysroot) and os.path.realpath(source_sysroot) != os.path.realpath(sysroot):
    if os.path.lexists(sysroot):
        print(f"Cleaning existing sysroot at {sysroot}...")
        if os.path.islink(sysroot) or not os.path.isdir(sysroot):
            os.unlink(sysroot)
        else:
            shutil.rmtree(sysroot)
    print(f"Copying sysroot from {source_sysroot} to {sysroot}...")
    os.makedirs(os.path.dirname(sysroot), exist_ok=True)
    shutil.copytree(
        source_sysroot,
        sysroot,
        dirs_exist_ok=True,
        symlinks=True,
    )
elif not os.path.isdir(sysroot):
    sys.exit(
        f"error: source FreeBSD sysroot not found in vendor/ and target sysroot {sysroot} does not exist"
    )

src = os.path.join(tmp_dir, "freebsd")
if not os.path.isdir(src):
    alt_src = os.path.join(sysroot, "src/freebsd")
    if os.path.isdir(alt_src):
        src = alt_src
lib_dir = os.path.join(sysroot, "usr/lib")
os.makedirs(lib_dir, exist_ok=True)

print(
    f"1. Compiling startup objects from audited FreeBSD CSU source into {lib_dir}..."
)

csu_amd64 = os.path.join(src, 'lib/csu/amd64')
csu_common = os.path.join(src, 'lib/csu/common')
libc_inc = os.path.join(src, 'lib/libc/include')

# Compile crti.S and crtn.S
for asm_file, out_name in [('crti.S', 'crti.o'), ('crtn.S', 'crtn.o')]:
    asm_path = os.path.join(csu_amd64, asm_file)
    out_path = os.path.join(lib_dir, out_name)
    subprocess.run([
        'clang', '--target=x86_64-unknown-freebsd',
        f'--sysroot={sysroot}',
        '-c', asm_path, '-o', out_path
    ], check=True)

# Compile crtbegin.c and crtend.c (both non-PIE and PIE)
for c_file, out_name, out_pic_name in [('crtbegin.c', 'crtbegin.o', 'crtbeginS.o'), ('crtend.c', 'crtend.o', 'crtendS.o')]:
    c_path = os.path.join(csu_common, c_file)
    subprocess.run([
        'clang', '--target=x86_64-unknown-freebsd',
        f'--sysroot={sysroot}',
        f'-I{csu_amd64}',
        '-c', c_path, '-o', os.path.join(lib_dir, out_name)
    ], check=True)
    subprocess.run([
        'clang', '--target=x86_64-unknown-freebsd',
        f'--sysroot={sysroot}',
        f'-I{csu_amd64}',
        '-fPIC', '-DPIC',
        '-c', c_path, '-o', os.path.join(lib_dir, out_pic_name)
    ], check=True)

# Compile crt1_c.c -> crt1.o (non-PIE) and Scrt1.o (PIE)
crt1_src = os.path.join(csu_amd64, 'crt1_c.c')
subprocess.run([
    'clang', '--target=x86_64-unknown-freebsd',
    f'--sysroot={sysroot}',
    f'-I{csu_amd64}',
    f'-I{csu_common}',
    f'-I{libc_inc}',
    '-DCRT_IRELOC_RELA',
    '-c', crt1_src, '-o', os.path.join(lib_dir, 'crt1.o')
], check=True)
subprocess.run([
    'clang', '--target=x86_64-unknown-freebsd',
    f'--sysroot={sysroot}',
    f'-I{csu_amd64}',
    f'-I{csu_common}',
    f'-I{libc_inc}',
    '-fPIC', '-DPIC',
    '-DCRT_IRELOC_RELA',
    '-c', crt1_src, '-o', os.path.join(lib_dir, 'Scrt1.o')
], check=True)

print('Compiled crt1.o, Scrt1.o, crti.o, crtn.o, crtbegin.o, crtbeginS.o, crtend.o, crtendS.o')


print(
    f"2. Generating dynamic library linker stubs from audited sources into {lib_dir}..."
)

def extract_license_block(file_path):
    if not os.path.isfile(file_path):
        return ''
    with open(file_path, 'r', encoding='latin1', errors='replace') as fp:
        txt = fp.read()
    m = re.search(r'(/\*.*?\*/|//(?:[^\n]*\n)+)', txt, re.DOTALL)
    if m:
        return m.group(1).strip()
    return ''

libraries = [
    ('libc.so.7', 'libc.so', 'lib/libc/gen/getgrent.c'),
    ('libthr.so.3', 'libthr.so', 'lib/libthr/thread/thr_create.c'),
    ('libm.so.5', 'libm.so', 'lib/msun/amd64/fenv.c'),
    ('libprocstat.so.1', 'libprocstat.so', 'lib/libprocstat/core.c'),
    ('libkvm.so.7', 'libkvm.so', 'lib/libkvm/kvm.c'),
    ('libutil.so', 'libutil.so', 'lib/libutil/auth.c'),
    ('libdevstat.so.7', 'libdevstat.so', 'lib/libdevstat/devstat.c'),
    ('libmemstat.so.3', 'libmemstat.so', 'lib/libmemstat/memstat.c'),
    ('libgeom.so.5', 'libgeom.so', 'lib/libgeom'),
    ('libexecinfo.so.1', 'libexecinfo.so', 'contrib/libexecinfo/backtrace.c'),
    ('libgcc_s.so.1', 'libgcc_s.so', 'lib/libc/gen/getgrent.c'),
]

for soname, link_name, src_target in libraries:
    target_full = os.path.join(src, src_target)
    if os.path.isdir(target_full):
        c_candidates = [os.path.join(src_target, f) for f in sorted(os.listdir(target_full)) if f.endswith('.c')]
        src_path = os.path.join(src, c_candidates[0]) if c_candidates else ''
    else:
        src_path = target_full
    lic_header = extract_license_block(src_path)
    c_stub = os.path.join(lib_dir, f'stub_{soname}.c')
    extra_ld_args = []
    version_map = os.path.join(lib_dir, f'{soname}.map')
    if soname == 'libc.so.7':
        compat_h = os.path.join(src, 'lib/libc/include/compat.h')
        version_syms = {
            'FBSD_1.0': [
                'fstat', 'stat', 'lstat', 'readdir', 'mknod', 'rand', 'srand',
                'glob', 'globfree', 'wait4', 'kevent', 'qsort_r', 'statfs',
                'fstatfs', 'getmntinfo',
            ],
            'FBSD_1.1': ['fstatat', 'mknodat'],
            'FBSD_1.5': ['readdir_r'],
        }
        compat_lic = ''
        if os.path.exists(compat_h):
            with open(compat_h) as fp:
                compat_txt = fp.read()
            compat_lic = extract_license_block(compat_h)
            for m in re.finditer(r'__sym_compat\s*\(\s*([a-zA-Z0-9_]+),\s*[a-zA-Z0-9_]+,\s*([a-zA-Z0-9_.]+)\)', compat_txt):
                sym, ver = m.group(1), m.group(2)
                version_syms.setdefault(ver, []).append(sym)

        with open(version_map, 'w') as fp:
            prev_ver = None
            for ver in ['FBSD_1.0', 'FBSD_1.1', 'FBSD_1.2', 'FBSD_1.3', 'FBSD_1.4', 'FBSD_1.5', 'FBSD_1.6', 'FBSD_1.7']:
                syms = sorted(set(version_syms.get(ver, [])))
                if not syms and not prev_ver:
                    continue
                inherit = f' {prev_ver};' if prev_ver else ''
                fp.write(f'{ver} {{\n    global:\n')
                for s in syms:
                    fp.write(f'        {s};\n')
                if inherit:
                    fp.write(f'}}{inherit}\n')
                else:
                    fp.write('};\n')
                prev_ver = ver

        all_syms = sorted(set(s for sym_list in version_syms.values() for s in sym_list))
        with open(c_stub, 'w') as fp:
            if compat_lic:
                fp.write(compat_lic + '\n')
            for s in all_syms:
                fp.write(f'void {s}(void) {{}}\n')
        extra_ld_args.append(f'-Wl,--version-script={version_map}')
    else:
        with open(c_stub, 'w') as fp:
            if lic_header:
                fp.write(lic_header + '\n')

    out_so = os.path.join(lib_dir, soname)
    if os.path.islink(out_so) or os.path.exists(out_so):
        os.remove(out_so)
    cmd = [
        'clang', '--target=x86_64-unknown-freebsd',
        '-fuse-ld=lld', '-nostdlib', '-shared',
        f'-Wl,-soname={soname}'
    ] + extra_ld_args + ['-o', out_so, c_stub]
    subprocess.run(cmd, check=True)
    os.remove(c_stub)
    if os.path.exists(version_map):
        os.remove(version_map)

    if link_name and link_name != soname:
        symlink_path = os.path.join(lib_dir, link_name)
        if os.path.islink(symlink_path) or os.path.exists(symlink_path):
            os.remove(symlink_path)
        os.symlink(soname, symlink_path)

for alias, target in [('libpthread.so', 'libthr.so.3'), ('librt.so', 'libc.so.7')]:
    alias_path = os.path.join(lib_dir, alias)
    if os.path.islink(alias_path) or os.path.exists(alias_path):
        os.remove(alias_path)
    os.symlink(target, alias_path)

libgcc_a = os.path.join(lib_dir, 'libgcc.a')
if os.path.exists(libgcc_a):
    os.remove(libgcc_a)
subprocess.run(['ar', 'cr', libgcc_a], check=True)

print('Compiled dynamic library stubs into usr/lib')


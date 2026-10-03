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

"""Prepare, review, and generate a source-only FreeBSD/LLVM sysroot."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile
import unicodedata


WORKSPACE_ROOT = Path(__file__).resolve().parents[1]
REPOSITORIES = {
    "freebsd": (
        "https://github.com/freebsd/freebsd-src.git",
        "c876eaf40915f213dc07f74e730ee6a2a9de7ba1",
    ),
    "llvm": (
        "https://github.com/llvm/llvm-project.git",
        "92ee37966563310901951b1b5a89bcbafbcea767",
    ),
}
OUTPUTS = (
    WORKSPACE_ROOT / "vendor/upstream-for-reference/freebsd-sysroot",
    WORKSPACE_ROOT / "vendor/freebsd-sysroot",
)
ALLOWED_LICENSES = frozenset({
    "BSD-1-Clause", "BSD-2-Clause", "BSD-3-Clause", "0BSD", "ISC", "MIT",
    "MIT-CMU", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "Zlib",
    "CC0-1.0", "Unlicense", "GPL-3.0-or-later", "AGPL-3.0-or-later",
})
SOURCE_SUFFIXES = frozenset({".h", ".c", ".S", ".s", ".inc", ".def", ".yaml"})
BINARY_SIGNATURES = (
    b"\x7fELF", b"!<arch>\n", b"!<thin>\n", b"MZ", b"\x89PNG", b"GIF87a",
    b"GIF89a", b"%PDF-", b"PK\x03\x04", b"\x1f\x8b", b"\xfd7zXZ",
    b"\xfe\xed\xfa\xce", b"\xce\xfa\xed\xfe", b"\xfe\xed\xfa\xcf",
    b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe",
)
FREEBSD_DIRECTORIES = (
    "include", "sys/sys", "sys/amd64/include", "sys/x86/include", "sys/arm64/include",
    "sys/net", "sys/netinet", "sys/netinet6", "sys/vm", "sys/ufs/ufs",
    "sys/fs/devfs", "sys/fs/fdescfs", "sys/fs/nullfs", "sys/fs/procfs",
    "contrib/libc-vis", "sys/kern", "lib/csu", "lib/libc", "lib/libthr", "lib/msun",
    "lib/libprocstat", "lib/libutil", "lib/libkvm", "lib/libmemstat", "lib/libdevstat",
    "lib/libgeom", "contrib/libexecinfo",
)


def generate_headers(source_root, destination, notices):
    TMP_DIR = str(source_root)
    TARGET_SYSROOT_DIR = str(destination)
    print(f"3. Copying audited FreeBSD headers into {TARGET_SYSROOT_DIR}/usr/include...")
    import os, shutil

    src = os.path.join(TMP_DIR, 'freebsd')
    dst = os.path.join(TARGET_SYSROOT_DIR, 'usr/include')

    mappings = [
        ('include', ''),
        ('include/arpa', 'arpa'),
        ('include/protocols', 'protocols'),
        ('include/rpc', 'rpc'),
        ('include/rpcsvc', 'rpcsvc'),
        ('include/ssp', 'ssp'),
        ('include/xlocale', 'xlocale'),
        ('sys/sys', 'sys'),
        ('sys/amd64/include', 'machine'),
        ('sys/x86/include', 'x86'),
        ('sys/net', 'net'),
        ('sys/netinet', 'netinet'),
        ('sys/netinet6', 'netinet6'),
        ('sys/vm', 'vm'),
        ('sys/ufs/ufs', 'ufs/ufs'),
        ('sys/fs/devfs', 'fs/devfs'),
        ('sys/fs/fdescfs', 'fs/fdescfs'),
        ('sys/fs/nullfs', 'fs/nullfs'),
        ('sys/fs/procfs', 'fs/procfs'),
        ('lib/libgeom', ''),
    ]

    for src_sub, dst_sub in mappings:
        src_dir = os.path.join(src, src_sub)
        target_dir = os.path.join(dst, dst_sub) if dst_sub else dst
        if not os.path.isdir(src_dir):
            continue
        os.makedirs(target_dir, exist_ok=True)
        for entry in os.listdir(src_dir):
            full_src = os.path.join(src_dir, entry)
            if os.path.isfile(full_src) and entry.endswith('.h'):
                shutil.copy2(full_src, os.path.join(target_dir, entry))

    vis_src = os.path.join(src, 'contrib/libc-vis/vis.h')
    if os.path.isfile(vis_src):
        shutil.copy2(vis_src, os.path.join(dst, 'vis.h'))

    print("4. Generating derived headers from audited sources through string/file manipulation...")
    import os, re, shutil

    freebsd_src = os.path.join(TMP_DIR, 'freebsd')
    llvm_src = os.path.join(TMP_DIR, 'llvm')
    dst = os.path.join(TARGET_SYSROOT_DIR, 'usr/include')

    os.makedirs(os.path.join(dst, 'x86'), exist_ok=True)
    os.makedirs(os.path.join(dst, 'machine'), exist_ok=True)
    os.makedirs(os.path.join(dst, 'sys'), exist_ok=True)
    os.makedirs(os.path.join(dst, 'xlocale'), exist_ok=True)

    def extract_license_block(file_path):
        return notices(Path(file_path))

    def combine_licenses(*blocks):
        seen = set()
        res = []
        for b in blocks:
            b = b.strip()
            if b and b not in seen:
                seen.add(b)
                res.append(b)
        return '\n\n'.join(res)

    def extract_signatures_from_c(src_text):
        sigs = []
        pattern = re.compile(r'^(int|void|pthread_t|void\s*\*)\s*\n([_a-zA-Z][_a-zA-Z0-9]*)\s*\(', re.MULTILINE)
        for m in pattern.finditer(src_text):
            ret_type = m.group(1).strip()
            fn_name = m.group(2).strip()
            start = m.end() - 1
            depth = 0
            end = -1
            for i in range(start, len(src_text)):
                if src_text[i] == '(':
                    depth += 1
                elif src_text[i] == ')':
                    depth -= 1
                    if depth == 0:
                        end = i + 1
                        break
            if end != -1:
                params = src_text[start+1:end-1].strip()
                params = ' '.join(params.split())
                name = fn_name.lstrip('_')
                sigs.append(f'{ret_type}\t{name}({params});')
        return sigs

    def parse_yaml_functions(yaml_text):
        funcs = []
        blocks = yaml_text.split('  - name: ')
        for b in blocks[1:]:
            lines = b.strip().split('\n')
            name = lines[0].strip()
            ret_m = re.search(r'return_type:\s*(.+)', b)
            ret = ret_m.group(1).strip() if ret_m else 'int'
            args = []
            in_args = False
            for l in lines:
                if 'arguments:' in l:
                    in_args = True
                    continue
                if in_args:
                    if l.strip().startswith('- type:'):
                        arg_type = l.split('- type:')[1].strip().strip('\"\'')
                        args.append(arg_type)
                    elif not l.startswith('      ') and not l.startswith('    -'):
                        break
            arg_str = ', '.join(args) if args else 'void'
            funcs.append(f'{ret}\t{name}({arg_str});')
        return funcs

    # A. Forwarding headers in machine/ -> x86/
    x86_dir = os.path.join(dst, 'x86')
    machine_dir = os.path.join(dst, 'machine')
    if os.path.isdir(x86_dir):
        for fname in os.listdir(x86_dir):
            if fname.endswith('.h'):
                target_f = os.path.join(x86_dir, fname)
                mpath = os.path.join(machine_dir, fname)
                if not os.path.exists(mpath):
                    lic = extract_license_block(target_f)
                    with open(mpath, 'w', encoding='utf-8') as fp:
                        if lic:
                            fp.write(lic + '\n\n')
                        fp.write(f'#pragma once\n#include <x86/{fname}>\n')

    # B. Forwarding headers in usr/include for sys/ and machine/
    sys_dir = os.path.join(dst, 'sys')
    sys_links = [
        'aio.h', 'errno.h', 'fcntl.h', 'linker_set.h', 'poll.h',
        'stdatomic.h', 'stdint.h', 'syslog.h', 'ucontext.h', '_semaphore.h',
        'param.h', 'signal.h',
    ]
    for name in sys_links:
        path = os.path.join(dst, name)
        target = os.path.join(sys_dir, name)
        if not os.path.exists(path) and os.path.isfile(target):
            lic = extract_license_block(target)
            with open(path, 'w', encoding='utf-8') as fp:
                if lic:
                    fp.write(lic + '\n\n')
                fp.write(f'#pragma once\n#include <sys/{name}>\n')

    mach_links = ['float.h', 'floatingpoint.h', 'stdarg.h']
    for name in mach_links:
        path = os.path.join(dst, name)
        target = os.path.join(machine_dir, name)
        if not os.path.exists(path) and os.path.isfile(target):
            lic = extract_license_block(target)
            with open(path, 'w', encoding='utf-8') as fp:
                if lic:
                    fp.write(lic + '\n\n')
                fp.write(f'#pragma once\n#include <machine/{name}>\n')

    # C. osreldate.h: derived through line extraction from sys/sys/param.h
    param_h_path = os.path.join(freebsd_src, 'sys/sys/param.h')
    lic_param = extract_license_block(param_h_path)
    version_lines = []
    if os.path.isfile(param_h_path):
        with open(param_h_path, 'r', encoding='utf-8') as fp:
            for line in fp:
                if line.startswith('#define') and '__FreeBSD_version' in line:
                    version_lines.append(line.strip())

    with open(os.path.join(dst, 'osreldate.h'), 'w', encoding='utf-8') as fp:
        if lic_param:
            fp.write(lic_param + '\n\n')
        fp.write('#pragma once\n#include <sys/cdefs.h>\n\n')
        for vl in version_lines:
            fp.write(f'#ifndef __FreeBSD_version\n{vl}\n#endif\n')

    # D. sys/syscall.h and syscall.h: derived through line extraction from LLVM libc sys/syscall.h.def
    llvm_syscall = os.path.join(llvm_src, 'libc/include/sys/syscall.h.def')
    lic_syscalls = extract_license_block(llvm_syscall)
    syscall_lines = []
    if os.path.isfile(llvm_syscall):
        with open(llvm_syscall, 'r', encoding='utf-8') as fp:
            for line in fp:
                if line.startswith('#define SYS_'):
                    syscall_lines.append(line.strip())

    with open(os.path.join(dst, 'sys/syscall.h'), 'w', encoding='utf-8') as fp:
        if lic_syscalls:
            fp.write(lic_syscalls + '\n\n')
        fp.write('#pragma once\n\n')
        for sline in syscall_lines:
            fp.write(sline + '\n')

    with open(os.path.join(dst, 'syscall.h'), 'w', encoding='utf-8') as fp:
        if lic_syscalls:
            fp.write(lic_syscalls + '\n\n')
        fp.write('#pragma once\n#include <sys/syscall.h>\n')

    # E. Copy LLVM libc types and macros into sysroot for clean POSIX compliance
    for subdir in ['llvm-libc-macros', 'llvm-libc-types']:
        sdir = os.path.join(llvm_src, 'libc/include', subdir)
        ddir = os.path.join(dst, subdir)
        if os.path.isdir(sdir):
            os.makedirs(ddir, exist_ok=True)
            for root, dirs, files in os.walk(sdir):
                for f in files:
                    if f.endswith('.h'):
                        sf = os.path.join(root, f)
                        rel = os.path.relpath(sf, sdir)
                        tf = os.path.join(ddir, rel)
                        os.makedirs(os.path.dirname(tf), exist_ok=True)
                        shutil.copy2(sf, tf)

    # F. sys/_pthreadtypes.h: derived through regex extraction from thr_private.h
    thr_private_f = os.path.join(freebsd_src, 'lib/libthr/thread/thr_private.h')
    lic_thr_priv = extract_license_block(thr_private_f)
    struct_names = []
    if os.path.isfile(thr_private_f):
        with open(thr_private_f, 'r', encoding='utf-8') as fp:
            for line in fp:
                m = re.match(r'^\s*struct\s+(pthread\w*)\s*[{;]', line)
                if m:
                    sname = m.group(1)
                    if sname not in struct_names and sname != 'pthread_key':
                        struct_names.append(sname)

    pthread_type_lines = []
    for sname in struct_names:
        pthread_type_lines.append(f'struct {sname};')
        tname = sname + '_t'
        if sname == 'pthread':
            tname = 'pthread_t'
        elif sname == 'pthread_mutex_attr':
            tname = 'pthread_mutexattr_t'
        elif sname == 'pthread_cond_attr':
            tname = 'pthread_condattr_t'
        pthread_type_lines.append(f'typedef struct {sname} *{tname};')

    pthread_type_lines.append('typedef int pthread_key_t;')
    pthread_type_lines.append('struct pthread_once { int state; void *mutex; };')
    pthread_type_lines.append('typedef struct pthread_once pthread_once_t;')

    with open(os.path.join(dst, 'sys/_pthreadtypes.h'), 'w', encoding='utf-8') as fp:
        if lic_thr_priv:
            fp.write(lic_thr_priv + '\n\n')
        fp.write('#pragma once\n\n' + '\n'.join(pthread_type_lines) + '\n')

    # G. pthread.h: derived through string extraction from LLVM macros, types, and libthr thr_*.c
    llvm_macros = os.path.join(llvm_src, 'libc/include/llvm-libc-macros/pthread-macros.h')
    lic_llvm_macros = extract_license_block(llvm_macros)
    pthread_macros = []
    if os.path.isfile(llvm_macros):
        with open(llvm_macros, 'r', encoding='utf-8') as fp:
            for line in fp:
                if line.startswith('#define PTHREAD_'):
                    pthread_macros.append(line.strip())

    thr_create_f = os.path.join(freebsd_src, 'lib/libthr/thread/thr_create.c')
    lic_thr = extract_license_block(thr_create_f)
    pthread_licenses = [lic_llvm_macros, lic_thr]

    thr_prototypes = set()
    yaml_pthread = os.path.join(llvm_src, 'libc/include/pthread.yaml')
    if os.path.isfile(yaml_pthread):
        pthread_licenses.append(extract_license_block(yaml_pthread))
        with open(yaml_pthread, 'r', encoding='utf-8') as fp:
            for fn in parse_yaml_functions(fp.read()):
                if 'pthread_' in fn:
                    thr_prototypes.add(fn)

    thr_dir = os.path.join(freebsd_src, 'lib/libthr/thread')
    if os.path.isdir(thr_dir):
        for f in sorted(os.listdir(thr_dir)):
            if f.startswith('thr_') and f.endswith('.c'):
                p = os.path.join(thr_dir, f)
                contributes = False
                with open(p, 'r', encoding='utf-8') as fp:
                    for sig in extract_signatures_from_c(fp.read()):
                        if '\tpthread_' in sig:
                            thr_prototypes.add(sig)
                            contributes = True
                if contributes:
                    pthread_licenses.append(extract_license_block(p))

    pthread_all_lic = combine_licenses(*pthread_licenses)

    with open(os.path.join(dst, 'pthread.h'), 'w', encoding='utf-8') as fp:
        if pthread_all_lic:
            fp.write(pthread_all_lic + '\n\n')
        fp.write('#pragma once\n\n')
        fp.write('#include <sys/cdefs.h>\n')
        fp.write('#include <time.h>\n')
        fp.write('#include <sched.h>\n')
        fp.write('#include <sys/_pthreadtypes.h>\n\n')
        for mline in pthread_macros:
            fp.write(mline + '\n')
        fp.write('\n__BEGIN_DECLS\n')
        for proto in sorted(thr_prototypes):
            fp.write(proto + '\n')
        fp.write('__END_DECLS\n')

    # H. sched.h & sys/sched.h: derived through string extraction from LLVM sched macros and types
    llvm_sched = os.path.join(llvm_src, 'libc/include/llvm-libc-macros/sched-macros.h')
    lic_llvm_sched = extract_license_block(llvm_sched)
    sched_macros = []
    if os.path.isfile(llvm_sched):
        with open(llvm_sched, 'r', encoding='utf-8') as fp:
            for line in fp:
                if line.startswith('#define SCHED_'):
                    sched_macros.append(line.strip())

    sched_param_f = os.path.join(llvm_src, 'libc/include/llvm-libc-types/struct_sched_param.h')
    lic_sched_param = extract_license_block(sched_param_f)
    sched_param_lines = []
    if os.path.isfile(sched_param_f):
        with open(sched_param_f, 'r', encoding='utf-8') as fp:
            copy = False
            for line in fp:
                if 'struct sched_param {' in line:
                    copy = True
                if copy:
                    sched_param_lines.append(line)
                    if line.strip().startswith('};'):
                        break

    sched_functions = set()
    lic_sched_yaml = ''
    yaml_sched = os.path.join(llvm_src, 'libc/include/sched.yaml')
    if os.path.isfile(yaml_sched):
        lic_sched_yaml = extract_license_block(yaml_sched)
        with open(yaml_sched, 'r', encoding='utf-8') as fp:
            for fn in parse_yaml_functions(fp.read()):
                if 'sched_' in fn:
                    sched_functions.add(fn)

    sched_all_lic = combine_licenses(lic_llvm_sched, lic_sched_param, lic_sched_yaml)

    with open(os.path.join(dst, 'sys/sched.h'), 'w', encoding='utf-8') as fp:
        if sched_all_lic:
            fp.write(sched_all_lic + '\n\n')
        fp.write('#pragma once\n\n')
        for sline in sched_macros:
            fp.write(sline + '\n')
        fp.write('\n' + ''.join(sched_param_lines) + '\n')
        fp.write('#include <sys/cdefs.h>\n')
        fp.write('#include <sys/_types.h>\n')
        fp.write('#include <sys/_timespec.h>\n')
        fp.write('\n__BEGIN_DECLS\n')
        for sfn in sorted(sched_functions):
            fp.write(sfn + '\n')
        fp.write('__END_DECLS\n')

    with open(os.path.join(dst, 'sched.h'), 'w', encoding='utf-8') as fp:
        if sched_all_lic:
            fp.write(sched_all_lic + '\n\n')
        fp.write('#pragma once\n#include <sys/sched.h>\n')

    # I. x86/_types.h, machine/_types.h: derived through string/regex extraction from clang stdint.h
    stdint_h = os.path.join(llvm_src, 'clang/lib/Headers/stdint.h')
    sys_types_h = os.path.join(freebsd_src, 'sys/sys/types.h')

    lic_clang = extract_license_block(stdint_h)
    lic_sys_types = extract_license_block(sys_types_h)
    types_all_lic = combine_licenses(lic_clang, lic_sys_types)

    extracted_types = []
    if os.path.isfile(stdint_h):
        with open(stdint_h, 'r', encoding='utf-8') as fp:
            for line in fp:
                line = line.strip()
                if line.startswith('typedef ') and line.endswith(';'):
                    m = re.match(r'typedef\s+(.+)\s+([u]?int[^;\s]*_t);', line)
                    if m:
                        base_type = m.group(1).strip()
                        tname = '__' + m.group(2).strip()
                        if any(w in tname for w in ['8', '16', '32', '64', 'ptr', 'max']):
                            if not any(w in tname for w in ['24', '40', '48', '56']):
                                if '__int_least' in base_type or '__uint_least' in base_type:
                                    w_m = re.search(r'\d+', base_type)
                                    if w_m:
                                        width = w_m.group(0)
                                        sign = 'UINT' if 'uint' in base_type else 'INT'
                                        base_type = f'__{sign}{width}_TYPE__'
                                extracted_types.append(f'typedef {base_type}\t{tname};')

    extra_types = [
        'typedef __INT32_TYPE__\t__clock_t;',
        'typedef __INT64_TYPE__\t__critical_t;',
        'typedef double\t__double_t;',
        'typedef float\t__float_t;',
        'typedef __INTPTR_TYPE__\t__intfptr_t;',
        'typedef __UINTPTR_TYPE__\t__uintfptr_t;',
        'typedef __PTRDIFF_TYPE__\t__ptrdiff_t;',
        'typedef __INT64_TYPE__\t__register_t;',
        'typedef __INT64_TYPE__\t__segsz_t;',
        'typedef __SIZE_TYPE__\t__size_t;',
        'typedef __INT64_TYPE__\t__ssize_t;',
        'typedef __INT64_TYPE__\t__time_t;',
        'typedef __UINT64_TYPE__\t__u_register_t;',
        'typedef __UINT64_TYPE__\t__vm_offset_t;',
        'typedef __UINT64_TYPE__\t__vm_paddr_t;',
        'typedef __UINT64_TYPE__\t__vm_size_t;',
        'typedef int\t___wchar_t;',
        '#define __WCHAR_MIN 0',
        '#define __WCHAR_MAX __INT_MAX',
    ]
    extracted_types.extend(extra_types)

    with open(os.path.join(dst, 'x86/_types.h'), 'w', encoding='utf-8') as fp:
        if types_all_lic:
            fp.write(types_all_lic + '\n\n')
        fp.write('#pragma once\n#include <machine/_limits.h>\n\n' + '\n'.join(extracted_types) + '\n')

    with open(os.path.join(dst, 'machine/_types.h'), 'w', encoding='utf-8') as fp:
        if types_all_lic:
            fp.write(types_all_lic + '\n\n')
        fp.write('#pragma once\n#include <x86/_types.h>\n')

    # J. x86/_stdint.h, machine/_stdint.h: derived through forwarding to clean LLVM libc stdint-macros.h
    llvm_stdint_macros = os.path.join(llvm_src, 'libc/include/llvm-libc-macros/stdint-macros.h')
    lic_stdint = extract_license_block(llvm_stdint_macros)

    with open(os.path.join(dst, 'x86/_stdint.h'), 'w', encoding='utf-8') as fp:
        if lic_stdint:
            fp.write(lic_stdint + '\n\n')
        fp.write('#pragma once\n#include <machine/_limits.h>\n#include <llvm-libc-macros/stdint-macros.h>\n')

    with open(os.path.join(dst, 'machine/_stdint.h'), 'w', encoding='utf-8') as fp:
        if lic_stdint:
            fp.write(lic_stdint + '\n\n')
        fp.write('#pragma once\n#include <x86/_stdint.h>\n')

    # K. machine/_align.h, machine/param.h: derived through string substitution from sys/arm64 clean 2-clause BSD sources
    arm64_align = os.path.join(freebsd_src, 'sys/arm64/include/_align.h')
    arm64_param = os.path.join(freebsd_src, 'sys/arm64/include/param.h')

    if os.path.isfile(arm64_align):
        with open(arm64_align, 'r', encoding='utf-8') as fp:
            align_content = fp.read()
        lic_align = extract_license_block(arm64_align)
        with open(os.path.join(dst, 'machine/_align.h'), 'w', encoding='utf-8') as fp:
            if lic_align:
                fp.write(lic_align + '\n\n')
            for line in align_content.split('\n'):
                if line.startswith('#define') and ('_ALIGN' in line or '_ALIGNBYTES' in line):
                    fp.write(line + '\n')

        with open(os.path.join(dst, 'x86/_align.h'), 'w', encoding='utf-8') as fp:
            if lic_align:
                fp.write(lic_align + '\n\n')
            fp.write('#pragma once\n#include <machine/_align.h>\n')

    if os.path.isfile(arm64_param):
        with open(arm64_param, 'r', encoding='utf-8') as fp:
            param_content = fp.read()
        lic_param = extract_license_block(arm64_param)
        parts = re.split(
            r"(\"(?:\\.|[^\"\\])*\"|'(?:\\.|[^'\\])*'|/\*.*?\*/|//[^\r\n]*)",
            param_content, flags=re.DOTALL,
        )
        param_adapted = ''.join(
            part if part.startswith(('/*', '//')) else
            part.replace('arm64', 'amd64').replace('aarch64', 'amd64').replace('armv7', 'i386')
            for part in parts
        )
        with open(os.path.join(dst, 'machine/param.h'), 'w', encoding='utf-8') as fp:
            if lic_param:
                fp.write(lic_param + '\n\n')
            fp.write(param_adapted)

    # L. xlocale/_strings.h: extract from FreeBSD include/xlocale/_strings.h
    xstrings_src = os.path.join(freebsd_src, 'include/xlocale/_strings.h')
    if os.path.isfile(xstrings_src):
        lic_xstrings = extract_license_block(xstrings_src)
        with open(xstrings_src, 'r', encoding='utf-8') as fp:
            xstrings_content = fp.read()
        with open(os.path.join(dst, 'xlocale/_strings.h'), 'w', encoding='utf-8') as fp:
            if lic_xstrings:
                fp.write(lic_xstrings + '\n\n')
            body = re.sub(r'/\*.*?\*/', '', xstrings_content, flags=re.DOTALL).strip()
            fp.write(body + '\n')

    print('Derived headers successfully generated from audited sources.')


def relative_path(value):
    if not isinstance(value, str) or not value:
        raise ValueError("Expected a nonempty relative POSIX path")
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in {".", ".."} for part in value.split("/")):
        raise ValueError(f"Unsafe path: {value!r}")
    if str(path) != value or "\\" in value or any(ord(char) < 32 for char in value):
        raise ValueError(f"Noncanonical path: {value!r}")
    return path


def unique_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def load_manifest(path):
    manifest = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_keys)
    if not isinstance(manifest, dict) or set(manifest) != {"version", "commits", "files"}:
        raise ValueError("Manifest must contain exactly version, commits, and files")
    if type(manifest["version"]) is not int or manifest["version"] != 1:
        raise ValueError("Unsupported manifest version")
    if manifest["commits"] != {name: commit for name, (_, commit) in REPOSITORIES.items()}:
        raise ValueError("Manifest must match both pinned upstream commits")
    entries = manifest["files"]
    if not isinstance(entries, list) or not entries:
        raise ValueError("No reviewed files: refusing to generate an empty sysroot")
    approved = []
    for entry in entries:
        if not isinstance(entry, dict):
            raise ValueError("Each file entry must be an object")
        decision = entry.get("decision", "approved")
        if decision == "pending":
            raise ValueError(f"Pending review: {entry.get('path')}; run --review-manifest {path}")
        if decision not in {"approved", "rejected"}:
            raise ValueError(f"Unknown review decision: {decision!r}")
        if decision == "approved":
            approved.append(entry)
    entries = approved
    if not entries:
        raise ValueError("No approved source files in the manifest")
    destinations = set()
    required_keys = {
        "repository", "path", "destination", "sha256", "kind", "licenses",
        "review", "source_only_reviewed", "notices",
    }
    for entry in entries:
        if set(entry) not in (required_keys, required_keys | {"decision"}):
            raise ValueError(f"File entry must contain exactly {sorted(required_keys)}")
        if not isinstance(entry["repository"], str) or entry["repository"] not in REPOSITORIES:
            raise ValueError("Unknown upstream repository")
        relative_path(entry["path"])
        destination = relative_path(entry["destination"])
        if destination in destinations:
            raise ValueError(f"Duplicate destination: {destination}")
        if any(other in destination.parents or destination in other.parents for other in destinations):
            raise ValueError(f"Overlapping destinations: {destination}")
        destinations.add(destination)
        if not isinstance(entry["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]):
            raise ValueError("Each file requires its reviewed SHA-256 digest")
        licenses = entry["licenses"]
        if not isinstance(licenses, list) or not licenses or any(
            not isinstance(license_id, str) or license_id not in ALLOWED_LICENSES
            for license_id in licenses
        ):
            raise ValueError(f"Unknown or incompatible reviewed license: {licenses!r}")
        if not isinstance(entry["review"], str) or not entry["review"].strip():
            raise ValueError("Each file requires an explicit license review reference")
        if entry["source_only_reviewed"] is not True:
            raise ValueError("Each file must be reviewed for embedded binary blobs")
        if entry["kind"] == "source":
            if destination.suffix not in SOURCE_SUFFIXES:
                raise ValueError(f"Not a supported source filename: {destination}")
            if not (destination.is_relative_to("usr/include") or destination.is_relative_to("src")):
                raise ValueError("Source destinations must be under usr/include or src")
            if destination.is_relative_to("usr/include") and destination.suffix != ".h":
                raise ValueError("Only headers may be installed under usr/include")
            if destination.is_relative_to("src") and str(destination) != (
                f"src/{entry['repository']}/{entry['path']}"
            ):
                raise ValueError("Source destinations under src must preserve repository paths")
        elif entry["kind"] == "notice":
            if not destination.is_relative_to("share/licenses"):
                raise ValueError("License notices must be installed under share/licenses")
        else:
            raise ValueError("Only source and notice files are permitted")
        if not isinstance(entry["notices"], list):
            raise ValueError("notices must list all required companion notice files")
        for notice in entry["notices"]:
            relative_path(notice)
    notices = {
        (entry["repository"], entry["path"])
        for entry in entries if entry["kind"] == "notice"
    }
    for entry in entries:
        if any((entry["repository"], path) not in notices for path in entry["notices"]):
            raise ValueError(f"Missing reviewed companion notice for {entry['path']}")
    return dict(manifest, files=entries)


def validate_bytes(entry, raw):
    if hashlib.sha256(raw).hexdigest() != entry["sha256"]:
        raise ValueError(f"Reviewed digest mismatch: {entry['path']}")
    text = validate_text(raw, entry["path"])
    for expression in re.findall(r"SPDX-License-Identifier:[ \t]*([^\r\n]*)", text):
        expression = expression.strip()
        if expression.endswith("*/"):
            expression = expression[:-2].rstrip()
        terms = spdx_terms(expression)
        if any(term not in entry["licenses"] for term in terms):
            raise ValueError(f"Unreviewed SPDX expression in {entry['path']}: {expression}")


def spdx_terms(expression):
    token_pattern = re.compile(
        '(?:' + '|'.join(re.escape(value) for value in sorted(ALLOWED_LICENSES, key=len, reverse=True))
        + r'|AND)(?=\s|[()]|$)|[()]'
    )
    tokens = []
    position = 0
    while position < len(expression):
        if expression[position].isspace():
            position += 1
            continue
        match = token_pattern.match(expression, position)
        if not match:
            raise ValueError(f"Unsupported or prohibited SPDX expression: {expression}")
        tokens.append(match.group())
        position = match.end()
    position = 0

    def parse_group(depth):
        nonlocal position
        if depth > 32:
            raise ValueError("SPDX expression nested too deeply")
        terms = set()
        need_term = True
        while position < len(tokens):
            token = tokens[position]
            if token == ')':
                break
            if need_term:
                if token == '(':
                    position += 1
                    terms.update(parse_group(depth + 1))
                    if position >= len(tokens) or tokens[position] != ')':
                        raise ValueError(f"Unbalanced SPDX expression: {expression}")
                elif token in ALLOWED_LICENSES:
                    terms.add(token)
                else:
                    raise ValueError(f"Malformed SPDX expression: {expression}")
            elif token != 'AND':
                raise ValueError(f"Malformed SPDX expression: {expression}")
            need_term = not need_term
            position += 1
        if need_term:
            raise ValueError(f"Incomplete SPDX expression: {expression}")
        return terms

    terms = parse_group(0)
    if position != len(tokens):
        raise ValueError(f"Unbalanced SPDX expression: {expression}")
    return terms


def validate_text(raw, path):
    if raw.startswith(BINARY_SIGNATURES):
        raise ValueError(f"Binary file: {path}")
    text = raw.decode("utf-8", errors="strict")
    if not text.strip() or any(
        unicodedata.category(char) == "Cc" and char not in "\t\r\n\f"
        for char in text
    ):
        raise ValueError(f"Empty or non-text file: {path}")
    return text


def git(repository, *arguments):
    return subprocess.run(
        ["git", "--git-dir", str(repository), *arguments],
        check=True, stdout=subprocess.PIPE,
    ).stdout


def read_blob(repository, commit, path):
    records = git(repository, "ls-tree", "-z", commit, "--", f":(literal){path}").split(b"\0")
    records = [record for record in records if record]
    if len(records) != 1:
        raise ValueError(f"Expected one tracked file: {path}")
    metadata, filename = records[0].split(b"\t", 1)
    mode, kind, object_id = metadata.split()
    if mode != b"100644" or kind != b"blob" or filename.decode("utf-8") != path:
        raise ValueError(f"Not a regular, non-executable upstream source file: {path}")
    return git(repository, "cat-file", "blob", object_id.decode("ascii"))


def stage_sysroot(manifest, repositories, destination):
    for entry in manifest["files"]:
        commit = REPOSITORIES[entry["repository"]][1]
        raw = read_blob(repositories[entry["repository"]], commit, entry["path"])
        validate_bytes(entry, raw)
        target = destination / entry["destination"]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
        if entry["kind"] == "source":
            source = destination / "src" / entry["repository"] / entry["path"]
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_bytes(raw)
    (destination / "reviewed-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8",
    )


def generate_reviewed_headers(manifest, destination):
    entries = {(entry["repository"], entry["path"]): entry for entry in manifest["files"]}
    by_hash = {}
    raw_files = {}
    blocks = {}
    for key, entry in entries.items():
        raw = (destination / entry["destination"]).read_bytes()
        validate_bytes(entry, raw)
        raw_files[key] = raw
        by_hash.setdefault(entry["sha256"], []).append(key)

    def archive(key, visited=None):
        if visited is None:
            visited = set()
        if key in visited:
            return ''
        visited.add(key)
        entry = entries[key]
        text = raw_files[key].decode("utf-8")
        lines = ''.join('// ' + line + '\n' for line in text.splitlines())
        result = f"// Provenance source: {key[0]}/{key[1]} sha256={entry['sha256']}\n"
        result += lines + '// End provenance source\n'
        for path in entry["notices"]:
            result += archive((key[0], path), visited)
        return result

    for key in entries:
        blocks[key] = archive(key)

    def notices(path):
        if not path.is_file():
            raise ValueError(f"Required generation input missing or not approved: {path}")
        raw = path.read_bytes()
        digest = hashlib.sha256(raw).hexdigest()
        keys = by_hash.get(digest)
        if not keys:
            raise ValueError(f"Unreviewed generation input: {path}")
        return '\n'.join(blocks[key] for key in keys)

    source_root = destination / "src"
    for repository, path in (
        ("llvm", "libc/include/pthread.yaml"),
        ("llvm", "libc/include/sched.yaml"),
        ("freebsd", "sys/arm64/include/_align.h"),
        ("freebsd", "sys/arm64/include/param.h"),
        ("freebsd", "include/xlocale/_strings.h"),
    ):
        if (repository, path) not in entries:
            raise ValueError(f"Required generation input missing or not approved: {repository}/{path}")
    generate_headers(source_root, destination, notices)
    provenance = {}
    tool_notice = '\n'.join(
        '// ' + line for line in Path(__file__).read_text(encoding="utf-8").splitlines()[1:18]
    ) + '\n\n'
    for path in sorted((destination / "usr/include").rglob("*")):
        if not path.is_file():
            continue
        raw = path.read_bytes()
        text = validate_text(raw, str(path))
        keys = by_hash.get(hashlib.sha256(raw).hexdigest())
        if not keys:
            keys = []
            for key, block in blocks.items():
                marker = f"// Provenance source: {key[0]}/{key[1]} sha256={entries[key]['sha256']}"
                if marker in text:
                    if block.strip() not in text:
                        raise ValueError(f"Incomplete contributor provenance: {path}, {key}")
                    keys.append(key)
            if not keys:
                raise ValueError(f"Generated header has no reviewed contributors: {path}")
            path.write_bytes(tool_notice.encode("utf-8") + raw)
        provenance[str(path.relative_to(destination))] = [
            {"repository": key[0], "path": key[1], "sha256": entries[key]["sha256"]}
            for key in keys
        ]
    (destination / "header-provenance.json").write_text(
        json.dumps(provenance, indent=2, sort_keys=True) + '\n', encoding="utf-8",
    )


def publish(staged, outputs):
    for output in outputs:
        if output.is_symlink() or (output.exists() and not output.is_dir()):
            raise ValueError(f"Refusing to replace a non-directory output: {output}")
        if any(parent.is_symlink() for parent in output.parents):
            raise ValueError(f"Refusing a symlinked output parent: {output}")
    for output in outputs:
        output.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix=".freebsd-sysroot-", dir=output.parent) as temporary:
            replacement = Path(temporary) / "sysroot"
            shutil.copytree(staged, replacement)
            backup = Path(temporary) / "previous"
            if output.exists():
                output.rename(backup)
            try:
                replacement.rename(output)
            except OSError:
                if backup.exists():
                    backup.rename(output)
                raise


def download_repositories(directory):
    directory.mkdir(parents=True, exist_ok=True)
    repositories = {}
    for name, (url, commit) in REPOSITORIES.items():
        repository = directory / name
        if not repository.exists():
            git(repository, "init", "--bare", "--quiet")
            git(repository, "remote", "add", "origin", url)
            git(repository, "config", "remote.origin.promisor", "true")
            git(repository, "config", "remote.origin.partialclonefilter", "blob:none")
        actual_url = git(repository, "remote", "get-url", "origin").decode("utf-8").strip()
        if actual_url != url:
            raise ValueError(f"Unexpected cached upstream URL for {name}")
        git(repository, "fetch", "--quiet", "--depth=1", "--filter=blob:none", "origin", commit)
        resolved = git(repository, "rev-parse", "FETCH_HEAD").decode("ascii").strip()
        if resolved != commit:
            raise ValueError(f"Upstream commit mismatch: {name}")
        repositories[name] = repository
    return repositories


def save_manifest(path, manifest):
    if path.is_symlink():
        raise ValueError(f"Refusing to write a symlinked manifest: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent, delete=False) as handle:
        temporary = Path(handle.name)
        try:
            json.dump(manifest, handle, indent=2, sort_keys=True)
            handle.write('\n')
            handle.flush()
            os.fsync(handle.fileno())
            temporary.replace(path)
        finally:
            if temporary.exists():
                temporary.unlink()


def draft_entry(repository, path, raw, kind):
    destination = f"src/{repository}/{path}" if kind == "source" else f"share/licenses/{repository}/{path}"
    entry = {
        "repository": repository, "path": path, "destination": destination,
        "sha256": hashlib.sha256(raw).hexdigest(), "kind": kind, "licenses": [],
        "review": "", "source_only_reviewed": False, "notices": [], "decision": "pending",
    }
    try:
        validate_text(raw, path)
    except (ValueError, UnicodeError) as error:
        entry["decision"] = "rejected"
        entry["review"] = f"Automatically excluded as non-text/binary: {error}"
    return entry


def prepare_manifest(path, repositories):
    if path.exists() or path.is_symlink():
        raise ValueError(f"Manifest already exists: {path}; use --review-manifest to resume")
    entries = []
    for name, repository in repositories.items():
        commit = REPOSITORIES[name][1]
        records = []
        for record in git(repository, "ls-tree", "-r", "-z", commit).split(b'\0'):
            if not record:
                continue
            metadata, filename = record.split(b'\t', 1)
            mode, kind, object_id = metadata.split()
            if mode == b"100644" and kind == b"blob":
                filename = filename.decode("utf-8")
                relative_path(filename)
                records.append((filename, object_id.decode("ascii")))
        selected = {}
        ancestors = {PurePosixPath('.')}
        for filename, object_id in records:
            source_path = PurePosixPath(filename)
            if name == "freebsd":
                included = any(source_path.is_relative_to(directory) for directory in FREEBSD_DIRECTORIES)
            else:
                included = source_path.is_relative_to("libc/include") or filename == "clang/lib/Headers/stdint.h"
            if included and source_path.suffix in SOURCE_SUFFIXES:
                selected[filename] = (object_id, "source")
                ancestors.update(source_path.parents)
        for filename, object_id in records:
            notice_path = PurePosixPath(filename)
            if notice_path.parent in ancestors and re.fullmatch(
                r"(?:LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE)(?:[._-].*)?", notice_path.name.upper(),
            ):
                selected[filename] = (object_id, "notice")
        object_ids = sorted({object_id for object_id, _ in selected.values()})
        for offset in range(0, len(object_ids), 500):
            git(repository, "fetch", "--quiet", "--no-write-fetch-head", "origin", *object_ids[offset:offset + 500])
        for filename, (object_id, kind) in sorted(selected.items()):
            raw = git(repository, "cat-file", "blob", object_id)
            entries.append(draft_entry(name, filename, raw, kind))
    manifest = {
        "version": 1,
        "commits": {name: commit for name, (_, commit) in REPOSITORIES.items()},
        "files": entries,
    }
    save_manifest(path, manifest)
    pending = sum(entry["decision"] == "pending" for entry in entries)
    print(f"Prepared {path}: {pending} files await review; nothing has been approved.")
    print(f"Next: {sys.executable} -B {__file__} --review-manifest {path}")


def read_review_manifest(path):
    manifest = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_keys)
    if not isinstance(manifest, dict) or set(manifest) != {"version", "commits", "files"}:
        raise ValueError("Manifest must contain exactly version, commits, and files")
    if type(manifest["version"]) is not int or manifest["version"] != 1:
        raise ValueError("Unsupported manifest version")
    if manifest["commits"] != {name: commit for name, (_, commit) in REPOSITORIES.items()}:
        raise ValueError("Manifest must match the pinned upstream commits")
    if not isinstance(manifest["files"], list) or not manifest["files"]:
        raise ValueError("No candidate files in manifest")
    seen = set()
    for entry in manifest["files"]:
        if not isinstance(entry, dict) or entry.get("repository") not in REPOSITORIES:
            raise ValueError("Invalid review entry")
        relative_path(entry.get("path"))
        relative_path(entry.get("destination"))
        key = (entry["repository"], entry["path"])
        if key in seen:
            raise ValueError(f"Duplicate source review: {key}")
        seen.add(key)
        if entry.get("decision", "approved") not in {"pending", "approved", "rejected"}:
            raise ValueError(f"Unknown review decision for {key}")
    return manifest


def review_manifest(path, repositories, ask=None):
    if ask is None:
        ask = input
    manifest = read_review_manifest(path)
    entries = manifest["files"]
    print("Review the entire file and applicable companion terms. SPDX is a hint, not approval.")
    for index, entry in enumerate(entries):
        if entry.get("decision", "approved") != "pending":
            continue
        name = entry["repository"]
        raw = read_blob(repositories[name], REPOSITORIES[name][1], entry["path"])
        if hashlib.sha256(raw).hexdigest() != entry["sha256"]:
            raise ValueError(f"Candidate digest mismatch: {entry['path']}; prepare a new manifest")
        text = validate_text(raw, entry["path"])
        print(f"\n[{index + 1}/{len(entries)}] {name}/{entry['path']} ({entry['kind']})")
        hints = re.findall(r"SPDX-License-Identifier:[ \t]*([^\r\n]*)", text)
        print(f"SPDX hints (not verified): {', '.join(hints) if hints else 'none'}")
        while True:
            action = ask("[v]iew entire file, [a]pprove, [r]eject, [s]kip, [q]uit [q]: ").strip().lower()
            if action in {"", "q"}:
                save_manifest(path, manifest)
                print(f"Saved progress to {path}. Resume with --review-manifest {path}")
                return
            if action == "v":
                print(text)
                continue
            if action == "s":
                break
            if action == "r":
                reason = ask("Reason for rejection: ").strip()
                if not reason:
                    print("A rejection reason is required.")
                    continue
                entry["decision"] = "rejected"
                entry["review"] = reason
                save_manifest(path, manifest)
                break
            if action != "a":
                print("Choose v, a, r, s, or q.")
                continue
            licenses = [value.strip() for value in ask(
                "All applicable compatible license IDs (comma-separated): "
            ).split(',') if value.strip()]
            if not licenses or any(value not in ALLOWED_LICENSES for value in licenses):
                print("Unknown/prohibited license. No approval saved.")
                continue
            evidence = ask("Your name and license-review evidence/reference: ").strip()
            if not evidence:
                print("A review reference is required.")
                continue
            if ask("Entire file checked: genuine source/plain-text notice, no encoded binary blobs? Type yes: ").strip() != "yes":
                print("No approval saved.")
                continue
            notice_paths = [value.strip() for value in ask(
                "Required companion notice paths in this repository (comma-separated; empty means none): "
            ).split(',') if value.strip()]
            additional = []
            for notice_path in notice_paths:
                relative_path(notice_path)
                matches = [item for item in entries if item["repository"] == name and item["path"] == notice_path]
                if matches:
                    if matches[0]["kind"] != "notice" or matches[0].get("decision") == "rejected":
                        raise ValueError(f"Required notice is not an eligible notice entry: {notice_path}")
                else:
                    notice_raw = read_blob(repositories[name], REPOSITORIES[name][1], notice_path)
                    additional.append(draft_entry(name, notice_path, notice_raw, "notice"))
            candidate = dict(
                entry, licenses=licenses, review=evidence, source_only_reviewed=True,
                notices=notice_paths, decision="approved",
            )
            try:
                validate_bytes(candidate, raw)
            except ValueError as error:
                print(f"No approval saved: {error}")
                continue
            entry.update(candidate)
            entries.extend(additional)
            save_manifest(path, manifest)
            break
    pending = sum(entry.get("decision") == "pending" for entry in entries)
    if pending:
        print(f"{pending} files still pending. Resume with --review-manifest {path}")
    else:
        load_manifest(path)
        print(f"Review decisions saved. Generate with --manifest {path}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_mutually_exclusive_group(required=True)
    commands.add_argument("--prepare-manifest", type=Path, metavar="FILE", help="download candidates and create an unapproved review list")
    commands.add_argument("--review-manifest", type=Path, metavar="FILE", help="review files one by one; saves progress and resumes")
    commands.add_argument("--manifest", type=Path, metavar="FILE", help="generate sysroot using completed review decisions")
    arguments = parser.parse_args(argv)
    manifest = None
    if arguments.manifest:
        manifest = load_manifest(arguments.manifest)
    if arguments.prepare_manifest and (arguments.prepare_manifest.exists() or arguments.prepare_manifest.is_symlink()):
        raise ValueError("Manifest already exists; use --review-manifest to resume")
    if arguments.review_manifest:
        read_review_manifest(arguments.review_manifest)
    if not shutil.which("git"):
        raise ValueError("git is required to download reviewed upstream sources")
    build_dir = WORKSPACE_ROOT / "built"
    build_dir.mkdir(exist_ok=True)
    if arguments.prepare_manifest or arguments.review_manifest:
        repositories = download_repositories(build_dir / "freebsd-sysroot-review")
        if arguments.prepare_manifest:
            prepare_manifest(arguments.prepare_manifest, repositories)
        else:
            review_manifest(arguments.review_manifest, repositories)
        return
    with tempfile.TemporaryDirectory(prefix="freebsd-sysroot-", dir=build_dir) as temporary:
        temporary = Path(temporary)
        repositories = download_repositories(temporary / "repositories")
        staged = temporary / "sysroot"
        staged.mkdir()
        stage_sysroot(manifest, repositories, staged)
        generate_reviewed_headers(manifest, staged)
        publish(staged, OUTPUTS)
    print("FreeBSD/LLVM headers generated from reviewed sources with contributor notices; no binaries compiled.")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"error: {error}") from error
#!/usr/bin/env python3
"""Independent raw-stream codecs for compression fixture interoperability tests."""

import argparse
import ctypes
import ctypes.util
import sys
import zlib


def transform(mode, codec, data, raw_size):
    if codec in ("deflate", "zlib"):
        window = -15 if codec == "deflate" else 15
        if mode == "compress":
            encoder = zlib.compressobj(wbits=window)
            return encoder.compress(data) + encoder.flush()
        decoder = zlib.decompressobj(wbits=window)
        result = decoder.decompress(data) + decoder.flush()
        if not decoder.eof or decoder.unused_data or decoder.unconsumed_tail:
            raise ValueError("Incomplete stream or trailing compressed data")
        return result

    library = ctypes.util.find_library("lzo2")
    if library is None:
        raise RuntimeError("The test oracle requires liblzo2")
    lzo = ctypes.CDLL(library)
    operation = lzo.lzo1x_1_compress if mode == "compress" else lzo.lzo1x_decompress_safe
    operation.argtypes = [
        ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p,
        ctypes.POINTER(ctypes.c_size_t), ctypes.c_void_p,
    ]
    operation.restype = ctypes.c_int
    if mode == "decompress" and (raw_size is None or raw_size < 0):
        raise ValueError("Raw LZO decompression requires a nonnegative --raw-size")
    capacity = len(data) + len(data) // 16 + 64 + 3 if mode == "compress" else raw_size
    output = ctypes.create_string_buffer(max(1, capacity))
    output_size = ctypes.c_size_t(capacity)
    workspace = ctypes.create_string_buffer(16384 * ctypes.sizeof(ctypes.c_void_p))
    status = operation(data, len(data), output, ctypes.byref(output_size), workspace)
    if status != 0:
        raise ValueError(f"liblzo2 returned {status}")
    return output.raw[:output_size.value]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("compress", "decompress"))
    parser.add_argument("codec", choices=("deflate", "zlib", "lzo"))
    parser.add_argument("--raw-size", type=int)
    args = parser.parse_args()
    sys.stdout.buffer.write(transform(args.mode, args.codec, sys.stdin.buffer.read(), args.raw_size))


if __name__ == "__main__":
    main()
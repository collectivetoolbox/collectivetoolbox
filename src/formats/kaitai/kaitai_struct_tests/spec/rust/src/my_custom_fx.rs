// SPDX-License-Identifier: MIT
// license-linter:allow-non-AGPL

/*
== This file is derived from kaitai_struct_tests. License, from https://raw.githubusercontent.com/kaitai-io/kaitai_struct_tests/59afee013e1a8e5fb894ca99838f55ef7b329cb3/LICENSE :

MIT License

Copyright (c) 2019 Kaitai Project

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


*/

extern crate kaitai;
use kaitai::CustomDecoder;

pub struct MyCustomFx {
    key: i32,
}

impl MyCustomFx {
    pub fn new(p_key: u8, p_flag: bool, _p_some_bytes: &[u8]) -> Self {
        if p_flag {
            Self { key: p_key as i32 }
        } else {
            Self {
                key: -(p_key as i32),
            }
        }
    }
}

impl CustomDecoder for MyCustomFx {
    fn decode(&self, bytes: &[u8]) -> Result<Vec<u8>, String> {
        let mut res = bytes.to_vec();
        for i in res.iter_mut() {
            *i = (*i as i32 + self.key) as u8;
        }
        Ok(res)
    }
}

;;; GNU Guix --- Functional package management for GNU
;;; Copyright © 2025 Hilton Chain <hako@ultrarare.space>
;;;
;;; This file is part of GNU Guix.
;;;
;;; GNU Guix is free software; you can redistribute it and/or modify it
;;; under the terms of the GNU General Public License as published by
;;; the Free Software Foundation; either version 3 of the License, or (at
;;; your option) any later version.
;;;
;;; GNU Guix is distributed in the hope that it will be useful, but
;;; WITHOUT ANY WARRANTY; without even the implied warranty of
;;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;;; GNU General Public License for more details.
;;;
;;; You should have received a copy of the GNU General Public License
;;; along with GNU Guix.  If not, see <http://www.gnu.org/licenses/>.

;; Based on https://cgit.git.savannah.gnu.org/cgit/guix.git/plain/etc/teams/rust/rust-crates.tmpl?h=rust-team

(define-module (ctoolbox-rust-crates)
  #:use-module (guix gexp)
  #:use-module (guix packages)
  #:use-module (guix download)
  #:use-module (guix git-download)
  #:use-module (guix build-system cargo)
  #:use-module (gnu packages rust-sources)
  #:export (lookup-cargo-inputs))

;;;
;;; This file is managed by ‘guix import’.  Do NOT add definitions manually.
;;;

;;;
;;; Rust libraries fetched from crates.io and non-workspace development
;;; snapshots.
;;;

(define qqqq-separator 'begin-of-crates)

(define rust-addr2line-0.24.2
  (crate-source "addr2line" "0.24.2"
                "1hd1i57zxgz08j6h5qrhsnm2fi0bcqvsh389fw400xm3arz2ggnz"))

(define rust-adler2-2.0.1
  (crate-source "adler2" "2.0.1"
                "1ymy18s9hs7ya1pjc9864l30wk8p2qfqdi7mhhcc5nfakxbij09j"))

(define rust-adler32-1.2.0
  (crate-source "adler32" "1.2.0"
                "0d7jq7jsjyhsgbhnfq5fvrlh9j0i9g1fqrl2735ibv5f75yjgqda"))

(define rust-aead-0.5.2
  (crate-source "aead" "0.5.2"
                "1c32aviraqag7926xcb9sybdm36v5vh9gnxpn4pxdwjc50zl28ni"))

(define rust-aes-0.8.4
  (crate-source "aes" "0.8.4"
                "1853796anlwp4kqim0s6wm1srl4ib621nm0cl2h3c8klsjkgfsdi"))

(define rust-aes-gcm-0.10.3
  (crate-source "aes-gcm" "0.10.3"
                "1lgaqgg1gh9crg435509lqdhajg1m2vgma6f7fdj1qa2yyh10443"))

(define rust-age-0.11.1
  (crate-source "age" "0.11.1"
                "12k0rkmsy31p5gn33ai1sh607kszln0qy227gs411ykl90gigz2p"))

(define rust-age-core-0.11.0
  (crate-source "age-core" "0.11.0"
                "16fgb96fxgjkn81b150a7db01lp177df5v0k162rvjl4r64nmgz2"))

(define rust-aho-corasick-1.1.3
  (crate-source "aho-corasick" "1.1.3"
                "05mrpkvdgp5d20y2p989f187ry9diliijgwrs254fs9s1m1x6q4f"))

(define rust-alloc-no-stdlib-2.0.4
  (crate-source "alloc-no-stdlib" "2.0.4"
                "1cy6r2sfv5y5cigv86vms7n5nlwhx1rbyxwcraqnmm1rxiib2yyc"))

(define rust-alloc-stdlib-0.2.2
  (crate-source "alloc-stdlib" "0.2.2"
                "1kkfbld20ab4165p29v172h8g0wvq8i06z8vnng14whw0isq5ywl"))

(define rust-ammonia-4.1.1
  (crate-source "ammonia" "4.1.1"
                "13wpycwpx0c5qxqlf6mvd7pbqr9hw2gqkgwavq2li0fh9mv4dcyn"))

(define rust-android-system-properties-0.1.5
  (crate-source "android_system_properties" "0.1.5"
                "04b3wrz12837j7mdczqd95b732gw5q7q66cv4yn4646lvccp57l1"))

(define rust-android-tzdata-0.1.1
  (crate-source "android-tzdata" "0.1.1"
                "1w7ynjxrfs97xg3qlcdns4kgfpwcdv824g611fq32cag4cdr96g9"))

(define rust-anstream-0.6.20
  (crate-source "anstream" "0.6.20"
                "14k1iqdf3dx7hdjllmql0j9sjxkwr1lfdddi3adzff0r7mjn7r9s"))

(define rust-anstyle-1.0.11
  (crate-source "anstyle" "1.0.11"
                "1gbbzi0zbgff405q14v8hhpi1kz2drzl9a75r3qhks47lindjbl6"))

(define rust-anstyle-parse-0.2.7
  (crate-source "anstyle-parse" "0.2.7"
                "1hhmkkfr95d462b3zf6yl2vfzdqfy5726ya572wwg8ha9y148xjf"))

(define rust-anstyle-query-1.1.4
  (crate-source "anstyle-query" "1.1.4"
                "1qir6d6fl5a4y2gmmw9a5w93ckwx6xn51aryd83p26zn6ihiy8wy"))

(define rust-anstyle-wincon-3.0.10
  (crate-source "anstyle-wincon" "3.0.10"
                "0ajz9wsf46a2l3pds7v62xbhq2cffj7wrilamkx2z8r28m0k61iy"))

(define rust-anyhow-1.0.99
  (crate-source "anyhow" "1.0.99"
                "001icqvkfl28rxxmk99rm4gvdzxqngj5v50yg2bh3dzcvqfllrxh"))

(define rust-arc-swap-1.7.1
  (crate-source "arc-swap" "1.7.1"
                "0mrl9a9r9p9bln74q6aszvf22q1ijiw089jkrmabfqkbj31zixv9"))

(define rust-ascii-1.1.0
  (crate-source "ascii" "1.1.0"
                "05nyyp39x4wzc1959kv7ckwqpkdzjd9dw4slzyjh73qbhjcfqayr"))

(define rust-asn1-rs-0.6.2
  (crate-source "asn1-rs" "0.6.2"
                "0j5h437ycgih5hnrma6kmaxi4zb8csynnd66h9rzvxxcvfzc74sl"))

(define rust-asn1-rs-derive-0.5.1
  (crate-source "asn1-rs-derive" "0.5.1"
                "140ldl0vp1d0090bpm0w9j8g80dwc03wp928w5kv5diwwlrjsp4n"))

(define rust-asn1-rs-impl-0.2.0
  (crate-source "asn1-rs-impl" "0.2.0"
                "1xv56m0wrwix4av3w86sih1nsa5g1dgfz135lz1qdznn5h60a63v"))

(define rust-async-channel-1.9.0
  (crate-source "async-channel" "1.9.0"
                "0dbdlkzlncbibd3ij6y6jmvjd0cmdn48ydcfdpfhw09njd93r5c1"))

(define rust-async-channel-2.5.0
  (crate-source "async-channel" "2.5.0"
                "1ljq24ig8lgs2555myrrjighycpx2mbjgrm3q7lpa6rdsmnxjklj"))

(define rust-async-compat-0.2.5
  (crate-source "async-compat" "0.2.5"
                "14550kssrwr6k8537gk2j37z8wyn37hrfvdm53vwnka6any8bfm1"))

(define rust-async-executor-1.13.3
  (crate-source "async-executor" "1.13.3"
                "1f3za9v8wkqzv6rz69g0qzvdcmghwbixijwzldwjm9w3zph00z29"))

(define rust-async-global-executor-2.4.1
  (crate-source "async-global-executor" "2.4.1"
                "1762s45cc134d38rrv0hyp41hv4iv6nmx59vswid2p0il8rvdc85"))

(define rust-async-io-2.5.0
  (crate-source "async-io" "2.5.0"
                "1ji3y970jdnc6xa3905zbhlln62wrrl13lwzy0hg57h16rilsqqr"))

(define rust-async-io-stream-0.3.3
  (crate-source "async_io_stream" "0.3.3"
                "0k5rv51935p3il74q59hwaaid6sy9kv05vz3lw48jpgkrpgbkmxn"))

(define rust-async-lock-3.4.1
  (crate-source "async-lock" "3.4.1"
                "1p6i1sw3mwv1msdx9jqkr0h0a2jlrp3717yyx5n9pvkw0h23dl2z"))

(define rust-async-std-1.13.2
  (crate-source "async-std" "1.13.2"
                "0fzrg0ainb5h87z0q09adnrnspc13132wqq3fhmyaymn9ad0g3ic"))

(define rust-async-task-4.7.1
  (crate-source "async-task" "4.7.1"
                "1pp3avr4ri2nbh7s6y9ws0397nkx1zymmcr14sq761ljarh3axcb"))

(define rust-async-tls-0.13.0
  (crate-source "async-tls" "0.13.0"
                "0plsx2ysd8rbmzfllxib7pkbp3zp7m1yl7gywjh75m49pag3rbmj"))

(define rust-async-trait-0.1.89
  (crate-source "async-trait" "0.1.89"
                "1fsxxmz3rzx1prn1h3rs7kyjhkap60i7xvi0ldapkvbb14nssdch"))

(define rust-async-tungstenite-0.29.1
  (crate-source "async-tungstenite" "0.29.1"
                "11lq0xgny2svj97ad9rclly4fcgxxijjkxvhc6rdjmxcvvz7w3zg"))

(define rust-atomic-waker-1.1.2
  (crate-source "atomic-waker" "1.1.2"
                "1h5av1lw56m0jf0fd3bchxq8a30xv0b4wv8s4zkp4s0i7mfvs18m"))

(define rust-autocfg-1.5.0
  (crate-source "autocfg" "1.5.0"
                "1s77f98id9l4af4alklmzq46f21c980v13z2r1pcxx6bqgw0d1n0"))

(define rust-backtrace-0.3.75
  (crate-source "backtrace" "0.3.75"
                "00hhizz29mvd7cdqyz5wrj98vqkihgcxmv2vl7z0d0f53qrac1k8"))

(define rust-base16ct-0.2.0
  (crate-source "base16ct" "0.2.0"
                "1kylrjhdzk7qpknrvlphw8ywdnvvg39dizw9622w3wk5xba04zsc"))

(define rust-base64-0.21.7
  (crate-source "base64" "0.21.7"
                "0rw52yvsk75kar9wgqfwgb414kvil1gn7mqkrhn9zf1537mpsacx"))

(define rust-base64-0.22.1
  (crate-source "base64" "0.22.1"
                "1imqzgh7bxcikp5vx3shqvw9j09g9ly0xr0jma0q66i52r7jbcvj"))

(define rust-base64ct-1.8.0
  (crate-source "base64ct" "1.8.0"
                "1fj4vc6ghy3j1120r7dwn4xw90crfy46b448g5pm9w6an13qn92m"))

(define rust-basic-toml-0.1.10
  (crate-source "basic-toml" "0.1.10"
                "12hp59jl28kk229q4sqx6v4fc9p66v8i2byi0vlc9922h9g6fqms"))

(define rust-bcrypt-0.15.1
  (crate-source "bcrypt" "0.15.1"
                "1iv2fvy5yywkx4kijqyy59bq92gldv3nqd4bry97vx4f0pnkhng6"))

(define rust-bech32-0.9.1
  (crate-source "bech32" "0.9.1"
                "0igl565rfpxwbh0g36cb7469sjkiap8yd21kcr0ppi2jfbwr6syq"))

(define rust-bincode-1.3.3
  (crate-source "bincode" "1.3.3"
                "1bfw3mnwzx5g1465kiqllp5n4r10qrqy88kdlp3jfwnq2ya5xx5i"))

(define rust-bitflags-1.3.2
  (crate-source "bitflags" "1.3.2"
                "12ki6w8gn1ldq7yz9y680llwk5gmrhrzszaa17g1sbrw2r2qvwxy"))

(define rust-bitflags-2.9.3
  (crate-source "bitflags" "2.9.3"
                "0pgjwsd9qgdmsmwpvg47p9ccrsc26yfjqawbhsi9qds5sg6brvrl"))

(define rust-block-buffer-0.10.4
  (crate-source "block-buffer" "0.10.4"
                "0w9sa2ypmrsqqvc20nhwr75wbb5cjr4kkyhpjm1z1lv2kdicfy1h"))

(define rust-block-padding-0.3.3
  (crate-source "block-padding" "0.3.3"
                "14wdad0r1qk5gmszxqd8cky6vx8qg7c153jv981mixzrpzmlz2d8"))

(define rust-blocking-1.6.2
  (crate-source "blocking" "1.6.2"
                "08bz3f9agqlp3102snkvsll6wc9ag7x5m1xy45ak2rv9pq18sgz8"))

(define rust-blowfish-0.9.1
  (crate-source "blowfish" "0.9.1"
                "1mw7bvj3bg5w8vh9xw9xawqh7ixk2xwsxkj34ph96b9b1z6y44p4"))

(define rust-brotli-3.5.0
  (crate-source "brotli" "3.5.0"
                "14f34ml3i8qbnh4hhlv5r6j10bkx420gspsl1cgznl1wqrdx4h6n"))

(define rust-brotli-decompressor-2.5.1
  (crate-source "brotli-decompressor" "2.5.1"
                "0kyyh9701dwqzwvn2frff4ww0zibikqd1s1xvl7n1pfpc3z4lbjf"))

(define rust-buffer-redux-1.0.2
  (crate-source "buffer-redux" "1.0.2"
                "1waq39blrj7j6qp1sp2fvplwmq10yhks7fgbsdy8kxdrqn3wz2jf"))

(define rust-bumpalo-3.19.0
  (crate-source "bumpalo" "3.19.0"
                "0hsdndvcpqbjb85ghrhska2qxvp9i75q2vb70hma9fxqawdy9ia6"))

(define rust-byteorder-1.5.0
  (crate-source "byteorder" "1.5.0"
                "0jzncxyf404mwqdbspihyzpkndfgda450l0893pz5xj685cg5l0z"))

(define rust-bytes-1.10.1
  (crate-source "bytes" "1.10.1"
                "0smd4wi2yrhp5pmq571yiaqx84bjqlm1ixqhnvfwzzc6pqkn26yp"))

(define rust-cbc-0.1.2
  (crate-source "cbc" "0.1.2"
                "19l9y9ccv1ffg6876hshd123f2f8v7zbkc4nkckqycxf8fajmd96"))

(define rust-cc-1.2.34
  (crate-source "cc" "1.2.34"
                "1p5ycww65h7xca03lwdp264qalw8v357rg5h17s7naq3h3m4mg22"))

(define rust-ccm-0.5.0
  (crate-source "ccm" "0.5.0"
                "0irqnk2lqcc730bgsvrynsd6jcwqw9qca4k2fmklf8sm8cpciqws"))

(define rust-cesu8-1.1.0
  (crate-source "cesu8" "1.1.0"
                "0g6q58wa7khxrxcxgnqyi9s1z2cjywwwd3hzr5c55wskhx6s0hvd"))

(define rust-cfg-aliases-0.2.1
  (crate-source "cfg_aliases" "0.2.1"
                "092pxdc1dbgjb6qvh83gk56rkic2n2ybm4yvy76cgynmzi3zwfk1"))

(define rust-cfg-if-1.0.3
  (crate-source "cfg-if" "1.0.3"
                "1afg7146gbxjvkbjx7i5sdrpqp9q5akmk9004fr8rsm90jf2il9g"))

(define rust-chacha20-0.9.1
  (crate-source "chacha20" "0.9.1"
                "0678wipx6kghp71hpzhl2qvx80q7caz3vm8vsvd07b1fpms3yqf3"))

(define rust-chacha20poly1305-0.10.1
  (crate-source "chacha20poly1305" "0.10.1"
                "0dfwq9ag7x7lnd0znafpcn8h7k4nfr9gkzm0w7sc1lcj451pkk8h"))

(define rust-chrono-0.4.41
  (crate-source "chrono" "0.4.41"
                "0k8wy2mph0mgipq28vv3wirivhb31pqs7jyid0dzjivz0i9djsf4"))

(define rust-chunked-transfer-1.5.0
  (crate-source "chunked_transfer" "1.5.0"
                "00a9h3csr1xwkqrzpz5kag4h92zdkrnxq4ppxidrhrx29syf6kbf"))

(define rust-cipher-0.4.4
  (crate-source "cipher" "0.4.4"
                "1b9x9agg67xq5nq879z66ni4l08m6m3hqcshk37d4is4ysd3ngvp"))

(define rust-colorchoice-1.0.4
  (crate-source "colorchoice" "1.0.4"
                "0x8ymkz1xr77rcj1cfanhf416pc4v681gmkc9dzb3jqja7f62nxh"))

(define rust-combine-4.6.7
  (crate-source "combine" "4.6.7"
                "1z8rh8wp59gf8k23ar010phgs0wgf5i8cx4fg01gwcnzfn5k0nms"))

(define rust-concurrent-queue-2.5.0
  (crate-source "concurrent-queue" "2.5.0"
                "0wrr3mzq2ijdkxwndhf79k952cp4zkz35ray8hvsxl96xrx1k82c"))

(define rust-configparser-3.1.0
  (crate-source "configparser" "3.1.0"
                "16v47b7lknb35ragwhj9gzgwfpxs34vn2b97hhaky30ry1r34zp5"))

(define rust-const-default-1.0.0
  (crate-source "const-default" "1.0.0"
                "1apcnxfrz5xsfxaxbv1n9c5sdfqlmrk81v0q29z5amflfqgnsf8b"))

(define rust-const-default-derive-0.2.0
  (crate-source "const-default-derive" "0.2.0"
                "1nh3iwba073s9vsyhr5ci0pgbnc6zavmfs7za4vj64mqrgc4v08g"))

(define rust-const-oid-0.9.6
  (crate-source "const-oid" "0.9.6"
                "1y0jnqaq7p2wvspnx7qj76m7hjcqpz73qzvr9l2p9n2s51vr6if2"))

(define rust-cookie-factory-0.3.3
  (crate-source "cookie-factory" "0.3.3"
                "18mka6fk3843qq3jw1fdfvzyv05kx7kcmirfbs2vg2kbw9qzm1cq"))

(define rust-core-foundation-0.10.1
  (crate-source "core-foundation" "0.10.1"
                "1xjns6dqf36rni2x9f47b65grxwdm20kwdg9lhmzdrrkwadcv9mj"))

(define rust-core-foundation-sys-0.8.7
  ;; TODO: Check bundled sources.
  (crate-source "core-foundation-sys" "0.8.7"
                "12w8j73lazxmr1z0h98hf3z623kl8ms7g07jch7n4p8f9nwlhdkp"))

(define rust-cpufeatures-0.2.17
  (crate-source "cpufeatures" "0.2.17"
                "10023dnnaghhdl70xcds12fsx2b966sxbxjq5sxs49mvxqw5ivar"))

(define rust-crc-3.3.0
  (crate-source "crc" "3.3.0"
                "0xg6yg57lbyzf69y8znq5gjb333w1fnlis2gnjg38blwffrx644p"))

(define rust-crc-catalog-2.4.0
  (crate-source "crc-catalog" "2.4.0"
                "1xg7sz82w3nxp1jfn425fvn1clvbzb3zgblmxsyqpys0dckp9lqr"))

(define rust-crc32fast-1.5.0
  (crate-source "crc32fast" "1.5.0"
                "04d51liy8rbssra92p0qnwjw8i9rm9c4m3bwy19wjamz1k4w30cl"))

(define rust-crossbeam-utils-0.8.21
  (crate-source "crossbeam-utils" "0.8.21"
                "0a3aa2bmc8q35fb67432w16wvi54sfmb69rk9h5bhd18vw0c99fh"))

(define rust-crypto-bigint-0.5.5
  (crate-source "crypto-bigint" "0.5.5"
                "0xmbdff3g6ii5sbxjxc31xfkv9lrmyril4arh3dzckd4gjsjzj8d"))

(define rust-crypto-common-0.1.6
  (crate-source "crypto-common" "0.1.6"
                "1cvby95a6xg7kxdz5ln3rl9xh66nz66w46mm3g56ri1z5x815yqv"))

(define rust-cssparser-0.35.0
  (crate-source "cssparser" "0.35.0"
                "1am2mj4rddlbmi08drk7gv9m8vw47zgicld48kwp451sfgfix42f"))

(define rust-cssparser-macros-0.6.1
  (crate-source "cssparser-macros" "0.6.1"
                "0cfkzj60avrnskdmaf7f8zw6pp3di4ylplk455zrzaf19ax8id8k"))

(define rust-csv-1.3.1
  (crate-source "csv" "1.3.1"
                "1bzxgbbhy27flcyafxbj7f1hbn7b8wac04ijfgj34ry9m61lip5c"))

(define rust-csv-core-0.1.12
  (crate-source "csv-core" "0.1.12"
                "0gfrjjlfagarhyclxrqv6b14iaxgvgc8kmwwdvw08racvaqg60kx"))

(define rust-ctr-0.9.2
  (crate-source "ctr" "0.9.2"
                "0d88b73waamgpfjdml78icxz45d95q7vi2aqa604b0visqdfws83"))

(define rust-ctrlc-3.4.7
  (crate-source "ctrlc" "3.4.7"
                "0wvf4w2wbpdnhp828jqw435x5ly4k7k1y1vzxxbdddsrlj03gya6"))

(define rust-curve25519-dalek-4.1.3
  (crate-source "curve25519-dalek" "4.1.3"
                "1gmjb9dsknrr8lypmhkyjd67p1arb8mbfamlwxm7vph38my8pywp"))

(define rust-curve25519-dalek-derive-0.1.1
  (crate-source "curve25519-dalek-derive" "0.1.1"
                "1cry71xxrr0mcy5my3fb502cwfxy6822k4pm19cwrilrg7hq4s7l"))

(define rust-darling-0.20.11
  (crate-source "darling" "0.20.11"
                "1vmlphlrlw4f50z16p4bc9p5qwdni1ba95qmxfrrmzs6dh8lczzw"))

(define rust-darling-core-0.20.11
  (crate-source "darling_core" "0.20.11"
                "0bj1af6xl4ablnqbgn827m43b8fiicgv180749f5cphqdmcvj00d"))

(define rust-darling-macro-0.20.11
  (crate-source "darling_macro" "0.20.11"
                "1bbfbc2px6sj1pqqq97bgqn6c8xdnb2fmz66f7f40nrqrcybjd7w"))

(define rust-data-encoding-2.9.0
  (crate-source "data-encoding" "2.9.0"
                "0xm46371aw613ghc12ay4vsnn49hpcmcwlijnqy8lbp2bpd308ra"))

(define rust-deflate-1.0.0
  (crate-source "deflate" "1.0.0"
                "0bs319wa9wl7pn9j6jrrxg1gaqbak581rkx210cbix0qyljpwvy8"))

(define rust-der-0.7.10
  (crate-source "der" "0.7.10"
                "1jyxacyxdx6mxbkfw99jz59dzvcd9k17rq01a7xvn1dr6wl87hg7"))

(define rust-der-parser-9.0.0
  (crate-source "der-parser" "9.0.0"
                "0lxmykajggvaq5mvpm2avgzwib4n9nyxii0kqaz2d5k88g3abl2w"))

(define rust-deranged-0.4.0
  (crate-source "deranged" "0.4.0"
                "13h6skwk411wzhf1l9l7d3yz5y6vg9d7s3dwhhb4a942r88nm7lw"))

(define rust-derive-builder-0.20.2
  (crate-source "derive_builder" "0.20.2"
                "0is9z7v3kznziqsxa5jqji3ja6ay9wzravppzhcaczwbx84znzah"))

(define rust-derive-builder-core-0.20.2
  (crate-source "derive_builder_core" "0.20.2"
                "1s640r6q46c2iiz25sgvxw3lk6b6v5y8hwylng7kas2d09xwynrd"))

(define rust-derive-builder-macro-0.20.2
  (crate-source "derive_builder_macro" "0.20.2"
                "0g1zznpqrmvjlp2w7p0jzsjvpmw5rvdag0rfyypjhnadpzib0qxb"))

(define rust-derive-more-2.0.1
  (crate-source "derive_more" "2.0.1"
                "0y3n97cc7rsvgnj211p92y1ppzh6jzvq5kvk6340ghkhfp7l4ch9"))

(define rust-derive-more-impl-2.0.1
  (crate-source "derive_more-impl" "2.0.1"
                "1wqxcb7d5lzvpplz9szp4rwy1r23f5wmixz0zd2vcjscqknji9mx"))

(define rust-digest-0.10.7
  (crate-source "digest" "0.10.7"
                "14p2n6ih29x81akj097lvz7wi9b6b9hvls0lwrv7b6xwyy0s5ncy"))

(define rust-directories-6.0.0
  (crate-source "directories" "6.0.0"
                "0zgy2w088v8w865c11dmc3dih899fgrhvrfp7g83h6v6ai60kx8n"))

(define rust-dirs-sys-0.5.0
  ;; TODO: Check bundled sources.
  (crate-source "dirs-sys" "0.5.0"
                "1aqzpgq6ampza6v012gm2dppx9k35cdycbj54808ksbys9k366p0"))

(define rust-displaydoc-0.2.5
  (crate-source "displaydoc" "0.2.5"
                "1q0alair462j21iiqwrr21iabkfnb13d6x5w95lkdg21q2xrqdlp"))

(define rust-dtoa-1.0.10
  (crate-source "dtoa" "1.0.10"
                "016gid01rarcdv57h049d7nr9daxc2hc2gqzx0mji57krywd7bfn"))

(define rust-dtoa-short-0.3.5
  (crate-source "dtoa-short" "0.3.5"
                "11rwnkgql5jilsmwxpx6hjzkgyrbdmx1d71s0jyrjqm5nski25fd"))

(define rust-dyn-clone-1.0.20
  (crate-source "dyn-clone" "1.0.20"
                "0m956cxcg8v2n8kmz6xs5zl13k2fak3zkapzfzzp7pxih6hix26h"))

(define rust-ecdsa-0.16.9
  (crate-source "ecdsa" "0.16.9"
                "1jhb0bcbkaz4001sdmfyv8ajrv8a1cg7z7aa5myrd4jjbhmz69zf"))

(define rust-elliptic-curve-0.13.8
  (crate-source "elliptic-curve" "0.13.8"
                "0ixx4brgnzi61z29r3g1606nh2za88hzyz8c5r3p6ydzhqq09rmm"))

(define rust-encre-css-0.17.1
  (crate-source "encre-css" "0.17.1"
                "1lfbskxqi0alvy7mmc33phj6pk183dy79l0f1wka0hmahb724lih"))

(define rust-env-filter-0.1.3
  (crate-source "env_filter" "0.1.3"
                "1l4p6f845cylripc3zkxa0lklk8rn2q86fqm522p6l2cknjhavhq"))

(define rust-env-logger-0.11.8
  (crate-source "env_logger" "0.11.8"
                "17q6zbjam4wq75fa3m4gvvmv3rj3ch25abwbm84b28a0j3q67j0k"))

(define rust-equivalent-1.0.2
  (crate-source "equivalent" "1.0.2"
                "03swzqznragy8n0x31lqc78g2af054jwivp7lkrbrc0khz74lyl7"))

(define rust-errno-0.3.13
  (crate-source "errno" "0.3.13"
                "1bd5g3srn66zr3bspac0150bvpg1s7zi6zwhwhlayivciz12m3kp"))

(define rust-event-listener-2.5.3
  (crate-source "event-listener" "2.5.3"
                "1q4w3pndc518crld6zsqvvpy9lkzwahp2zgza9kbzmmqh9gif1h2"))

(define rust-event-listener-5.4.1
  (crate-source "event-listener" "5.4.1"
                "1asnp3agbr8shcl001yd935m167ammyi8hnvl0q1ycajryn6cfz1"))

(define rust-event-listener-strategy-0.5.4
  (crate-source "event-listener-strategy" "0.5.4"
                "14rv18av8s7n8yixg38bxp5vg2qs394rl1w052by5npzmbgz7scb"))

(define rust-fastrand-2.3.0
  (crate-source "fastrand" "2.3.0"
                "1ghiahsw1jd68df895cy5h3gzwk30hndidn3b682zmshpgmrx41p"))

(define rust-fern-0.7.1
  (crate-source "fern" "0.7.1"
                "0a9v59vcq2fgd6bwgbfl7q6b0zzgxn85y6g384z728wvf1gih5j3"))

(define rust-ff-0.13.1
  (crate-source "ff" "0.13.1"
                "14v3bc6q24gbcjnxjfbq2dddgf4as2z2gd4mj35gjlrncpxhpdf0"))

(define rust-fiat-crypto-0.2.9
  (crate-source "fiat-crypto" "0.2.9"
                "07c1vknddv3ak7w89n85ik0g34nzzpms6yb845vrjnv9m4csbpi8"))

(define rust-filetime-0.2.26
  (crate-source "filetime" "0.2.26"
                "1vb3vz83saxr084wjf2032hspx7wfc5ggggnhc15i9kg3g6ha1dw"))

(define rust-find-crate-0.6.3
  (crate-source "find-crate" "0.6.3"
                "1ljpkh11gj7940xwz47xjhsvfbl93c2q0ql7l2v0w77amjx8paar"))

(define rust-flate2-1.1.2
  (crate-source "flate2" "1.1.2"
                "07abz7v50lkdr5fjw8zaw2v8gm2vbppc0f7nqm8x3v3gb6wpsgaa"))

(define rust-fluent-0.16.1
  (crate-source "fluent" "0.16.1"
                "0njmdpwz52yjzyp55iik9k6vrixqiy7190d98pk0rgdy0x3n6x5v"))

(define rust-fluent-bundle-0.15.3
  (crate-source "fluent-bundle" "0.15.3"
                "14zl0cjn361is69pb1zry4k2zzh5nzsfv0iz05wccl00x0ga5q3z"))

(define rust-fluent-langneg-0.13.0
  (crate-source "fluent-langneg" "0.13.0"
                "152yxplc11vmxkslvmaqak9x86xnavnhdqyhrh38ym37jscd0jic"))

(define rust-fluent-syntax-0.11.1
  (crate-source "fluent-syntax" "0.11.1"
                "0gd3cdvsx9ymbb8hijcsc9wyf8h1pbcbpsafg4ldba56ji30qlra"))

(define rust-fnv-1.0.7
  (crate-source "fnv" "1.0.7"
                "1hc2mcqha06aibcaza94vbi81j6pr9a1bbxrxjfhc91zin8yr7iz"))

(define rust-foldhash-0.1.5
  (crate-source "foldhash" "0.1.5"
                "1wisr1xlc2bj7hk4rgkcjkz3j2x4dhd1h9lwk7mj8p71qpdgbi6r"))

(define rust-form-urlencoded-1.2.2
  (crate-source "form_urlencoded" "1.2.2"
                "1kqzb2qn608rxl3dws04zahcklpplkd5r1vpabwga5l50d2v4k6b"))

(define rust-fs2-0.4.3
  (crate-source "fs2" "0.4.3"
                "04v2hwk7035c088f19mfl5b1lz84gnvv2hv6m935n0hmirszqr4m"))

(define rust-futf-0.1.5
  (crate-source "futf" "0.1.5"
                "0hvqk2r7v4fnc34hvc3vkri89gn52d5m9ihygmwn75l1hhp0whnz"))

(define rust-futures-0.3.31
  (crate-source "futures" "0.3.31"
                "0xh8ddbkm9jy8kc5gbvjp9a4b6rqqxvc8471yb2qaz5wm2qhgg35"))

(define rust-futures-channel-0.3.31
  (crate-source "futures-channel" "0.3.31"
                "040vpqpqlbk099razq8lyn74m0f161zd0rp36hciqrwcg2zibzrd"))

(define rust-futures-core-0.3.31
  (crate-source "futures-core" "0.3.31"
                "0gk6yrxgi5ihfanm2y431jadrll00n5ifhnpx090c2f2q1cr1wh5"))

(define rust-futures-executor-0.3.31
  (crate-source "futures-executor" "0.3.31"
                "17vcci6mdfzx4gbk0wx64chr2f13wwwpvyf3xd5fb1gmjzcx2a0y"))

(define rust-futures-io-0.3.31
  (crate-source "futures-io" "0.3.31"
                "1ikmw1yfbgvsychmsihdkwa8a1knank2d9a8dk01mbjar9w1np4y"))

(define rust-futures-lite-2.6.1
  (crate-source "futures-lite" "2.6.1"
                "1ba4dg26sc168vf60b1a23dv1d8rcf3v3ykz2psb7q70kxh113pp"))

(define rust-futures-macro-0.3.31
  (crate-source "futures-macro" "0.3.31"
                "0l1n7kqzwwmgiznn0ywdc5i24z72zvh9q1dwps54mimppi7f6bhn"))

(define rust-futures-sink-0.3.31
  (crate-source "futures-sink" "0.3.31"
                "1xyly6naq6aqm52d5rh236snm08kw8zadydwqz8bip70s6vzlxg5"))

(define rust-futures-task-0.3.31
  (crate-source "futures-task" "0.3.31"
                "124rv4n90f5xwfsm9qw6y99755y021cmi5dhzh253s920z77s3zr"))

(define rust-futures-timer-3.0.3
  (crate-source "futures-timer" "3.0.3"
                "094vw8k37djpbwv74bwf2qb7n6v6ghif4myss6smd6hgyajb127j"))

(define rust-futures-util-0.3.31
  (crate-source "futures-util" "0.3.31"
                "10aa1ar8bgkgbr4wzxlidkqkcxf77gffyj8j7768h831pcaq784z"))

(define rust-generic-array-0.14.7
  (crate-source "generic-array" "0.14.7"
                "16lyyrzrljfq424c3n8kfwkqihlimmsg5nhshbbp48np3yjrqr45"))

(define rust-getrandom-0.2.16
  (crate-source "getrandom" "0.2.16"
                "14l5aaia20cc6cc08xdlhrzmfcylmrnprwnna20lqf746pqzjprk"))

(define rust-getrandom-0.3.3
  (crate-source "getrandom" "0.3.3"
                "1x6jl875zp6b2b6qp9ghc84b0l76bvng2lvm8zfcmwjl7rb5w516"))

(define rust-ghash-0.5.1
  (crate-source "ghash" "0.5.1"
                "1wbg4vdgzwhkpkclz1g6bs4r5x984w5gnlsj4q5wnafb5hva9n7h"))

(define rust-gimli-0.31.1
  (crate-source "gimli" "0.31.1"
                "0gvqc0ramx8szv76jhfd4dms0zyamvlg4whhiz11j34hh3dqxqh7"))

(define rust-glob-0.3.3
  (crate-source "glob" "0.3.3"
                "106jpd3syfzjfj2k70mwm0v436qbx96wig98m4q8x071yrq35hhc"))

(define rust-gloo-timers-0.2.6
  (crate-source "gloo-timers" "0.2.6"
                "0p2yqcxw0q9kclhwpgshq1r4ijns07nmmagll3lvrgl7pdk5m6cv"))

(define rust-gloo-timers-0.3.0
  (crate-source "gloo-timers" "0.3.0"
                "1519157n7xppkk6pdw5w52vy1llzn5iljkqd7q1h5609jv7l7cdv"))

(define rust-group-0.13.0
  (crate-source "group" "0.13.0"
                "0qqs2p5vqnv3zvq9mfjkmw3qlvgqb0c3cm6p33srkh7pc9sfzygh"))

(define rust-gzip-header-1.0.0
  (crate-source "gzip-header" "1.0.0"
                "18lm2y96mahkmcd76pzyam2sl3v6lsl9mn8ajri9l0p6j9xm5k4m"))

(define rust-handlebars-6.3.2
  (crate-source "handlebars" "6.3.2"
                "1630l9p8ag2f6lzj3gzwfjzvav46897kkv68j08wp1rjx9d2v7km"))

(define rust-hashbrown-0.15.5
  (crate-source "hashbrown" "0.15.5"
                "189qaczmjxnikm9db748xyhiw04kpmhm9xj9k9hg0sgx7pjwyacj"))

(define rust-hermit-abi-0.5.2
  (crate-source "hermit-abi" "0.5.2"
                "1744vaqkczpwncfy960j2hxrbjl1q01csm84jpd9dajbdr2yy3zw"))

(define rust-hex-0.4.3
  (crate-source "hex" "0.4.3"
                "0w1a4davm1lgzpamwnba907aysmlrnygbqmfis2mqjx5m552a93z"))

(define rust-hifijson-0.2.3
  (crate-source "hifijson" "0.2.3"
                "038g1xbdrrsc1dcd2mb41mj6qrz7jyqrmgwqwrclz8m8ifwn6xqa"))

(define rust-hkdf-0.12.4
  (crate-source "hkdf" "0.12.4"
                "1xxxzcarz151p1b858yn5skmhyrvn8fs4ivx5km3i1kjmnr8wpvv"))

(define rust-hmac-0.12.1
  (crate-source "hmac" "0.12.1"
                "0pmbr069sfg76z7wsssfk5ddcqd9ncp79fyz6zcm6yn115yc6jbc"))

(define rust-html5ever-0.35.0
  (crate-source "html5ever" "0.35.0"
                "1m4yajw7slxqn0x3zdh3i9qlhb03vgdf2pq3la3l8rjbyz15inam"))

(define rust-http-1.3.1
  (crate-source "http" "1.3.1"
                "0r95i5h7dr1xadp1ac9453w0s62s27hzkam356nyx2d9mqqmva7l"))

(define rust-httparse-1.10.1
  (crate-source "httparse" "1.10.1"
                "11ycd554bw2dkgw0q61xsa7a4jn1wb1xbfacmf3dbwsikvkkvgvd"))

(define rust-httpdate-1.0.3
  (crate-source "httpdate" "1.0.3"
                "1aa9rd2sac0zhjqh24c9xvir96g188zldkx0hr6dnnlx5904cfyz"))

(define rust-humantime-2.2.0
  (crate-source "humantime" "2.2.0"
                "17rz8jhh1mcv4b03wnknhv1shwq2v9vhkhlfg884pprsig62l4cv"))

(define rust-i18n-config-0.4.8
  (crate-source "i18n-config" "0.4.8"
                "1vv31hz9zpzqz1ddpisxm2iz6c2swchlnd4l7hh2w98di86bj1iy"))

(define rust-i18n-embed-0.15.4
  (crate-source "i18n-embed" "0.15.4"
                "1i04hjbwg1y0sgvqbfvq54sf70k7rngrkgnx0vgnszprjcngr7v6"))

(define rust-i18n-embed-fl-0.9.4
  (crate-source "i18n-embed-fl" "0.9.4"
                "0b9wqnp8zy531xqjsr3a7ss3483j3561hdf5fqvi9iiz1ffrdch4"))

(define rust-i18n-embed-impl-0.8.4
  (crate-source "i18n-embed-impl" "0.8.4"
                "1hmnimlv310cirg8nx77nf8q1si4hq1yarkg5kyfc7rxabhc0b0g"))

(define rust-iana-time-zone-0.1.63
  (crate-source "iana-time-zone" "0.1.63"
                "1n171f5lbc7bryzmp1h30zw86zbvl5480aq02z92lcdwvvjikjdh"))

(define rust-iana-time-zone-haiku-0.1.2
  (crate-source "iana-time-zone-haiku" "0.1.2"
                "17r6jmj31chn7xs9698r122mapq85mfnv98bb4pg6spm0si2f67k"))

(define rust-icu-collections-2.0.0
  (crate-source "icu_collections" "2.0.0"
                "0izfgypv1hsxlz1h8fc2aak641iyvkak16aaz5b4aqg3s3sp4010"))

(define rust-icu-locale-core-2.0.0
  (crate-source "icu_locale_core" "2.0.0"
                "02phv7vwhyx6vmaqgwkh2p4kc2kciykv2px6g4h8glxfrh02gphc"))

(define rust-icu-normalizer-2.0.0
  (crate-source "icu_normalizer" "2.0.0"
                "0ybrnfnxx4sf09gsrxri8p48qifn54il6n3dq2xxgx4dw7l80s23"))

(define rust-icu-normalizer-data-2.0.0
  (crate-source "icu_normalizer_data" "2.0.0"
                "1lvjpzxndyhhjyzd1f6vi961gvzhj244nribfpdqxjdgjdl0s880"))

(define rust-icu-properties-2.0.1
  (crate-source "icu_properties" "2.0.1"
                "0az349pjg8f18lrjbdmxcpg676a7iz2ibc09d2wfz57b3sf62v01"))

(define rust-icu-properties-data-2.0.1
  (crate-source "icu_properties_data" "2.0.1"
                "0cnn3fkq6k88w7p86w7hsd1254s4sl783rpz4p6hlccq74a5k119"))

(define rust-icu-provider-2.0.0
  (crate-source "icu_provider" "2.0.0"
                "1bz5v02gxv1i06yhdhs2kbwxkw3ny9r2vvj9j288fhazgfi0vj03"))

(define rust-ident-case-1.0.1
  (crate-source "ident_case" "1.0.1"
                "0fac21q6pwns8gh1hz3nbq15j8fi441ncl6w4vlnd1cmc55kiq5r"))

(define rust-idna-1.1.0
  (crate-source "idna" "1.1.0"
                "1pp4n7hppm480zcx411dsv9wfibai00wbpgnjj4qj0xa7kr7a21v"))

(define rust-idna-adapter-1.2.1
  (crate-source "idna_adapter" "1.2.1"
                "0i0339pxig6mv786nkqcxnwqa87v4m94b2653f6k3aj0jmhfkjis"))

(define rust-include-dir-0.7.4
  (crate-source "include_dir" "0.7.4"
                "1pfh3g45z88kwq93skng0n6g3r7zkhq9ldqs9y8rvr7i11s12gcj"))

(define rust-include-dir-macros-0.7.4
  (crate-source "include_dir_macros" "0.7.4"
                "0x8smnf6knd86g69p19z5lpfsaqp8w0nx14kdpkz1m8bxnkqbavw"))

(define rust-indexmap-2.11.0
  (crate-source "indexmap" "2.11.0"
                "1sb3nmhisf9pdwfcxzqlvx97xifcvlh5g0rqj9j7i7qg8f01jj7j"))

(define rust-inout-0.1.4
  (crate-source "inout" "0.1.4"
                "008xfl1jn9rxsq19phnhbimccf4p64880jmnpg59wqi07kk117w7"))

(define rust-interceptor-0.14.0
  (crate-source "interceptor" "0.14.0"
                "1y6frnchg3hdb65fdq4shv5sy1q6xy4y6wip26aj0q2xh8f7ih0s"))

(define rust-intl-memoizer-0.5.3
  (crate-source "intl-memoizer" "0.5.3"
                "0gqn5wwhzacvj0z25r5r3l2pajg9c8i1ivh7g8g8dszm8pis439i"))

(define rust-intl-pluralrules-7.0.2
  (crate-source "intl_pluralrules" "7.0.2"
                "0wprd3h6h8nfj62d8xk71h178q7zfn3srxm787w4sawsqavsg3h7"))

(define rust-io-tee-0.1.1
  (crate-source "io_tee" "0.1.1"
                "013ka85akdcsj9rr92jrkm4jia9s8ihirpqi0ncqc6156kppqgsb"))

(define rust-io-uring-0.7.10
  (crate-source "io-uring" "0.7.10"
                "0yvjyygwdcqjcgw8zp254hvjbm7as1c075dl50spdshas3aa4vq4"))

(define rust-ipnet-2.11.0
  (crate-source "ipnet" "2.11.0"
                "0c5i9sfi2asai28m8xp48k5gvwkqrg5ffpi767py6mzsrswv17s6"))

(define rust-is-terminal-polyfill-1.70.1
  (crate-source "is_terminal_polyfill" "1.70.1"
                "1kwfgglh91z33kl0w5i338mfpa3zs0hidq5j4ny4rmjwrikchhvr"))

(define rust-itoa-1.0.15
  (crate-source "itoa" "1.0.15"
                "0b4fj9kz54dr3wam0vprjwgygvycyw8r0qwg7vp19ly8b2w16psa"))

(define rust-jaq-core-2.2.1
  (crate-source "jaq-core" "2.2.1"
                "1670g3ldack5w5pma00fnhfcpgwvajk6f5qlzlljqhbrxdr6llkp"))

(define rust-jaq-json-1.1.3
  (crate-source "jaq-json" "1.1.3"
                "1g3j27lf205zfyzl9g1vjp6jwgnr8ivwws5cmc1q8vh7gg8dpnq1"))

(define rust-jaq-std-2.1.2
  (crate-source "jaq-std" "2.1.2"
                "0zfnpm3y31g8gzjm1s9alpnvk0h30bz1n7y7frcp10f9jzily9ic"))

(define rust-jni-0.21.1
  (crate-source "jni" "0.21.1"
                "15wczfkr2r45slsljby12ymf2hij8wi5b104ghck9byjnwmsm1qs"))

(define rust-jni-sys-0.3.0
  ;; TODO: Check bundled sources.
  (crate-source "jni-sys" "0.3.0"
                "0c01zb9ygvwg9wdx2fii2d39myzprnpqqhy7yizxvjqp5p04pbwf"))

(define rust-js-sys-0.3.77
  ;; TODO: Check bundled sources.
  (crate-source "js-sys" "0.3.77"
                "13x2qcky5l22z4xgivi59xhjjx4kxir1zg7gcj0f1ijzd4yg7yhw"))

(define rust-kv-log-macro-1.0.7
  (crate-source "kv-log-macro" "1.0.7"
                "0zwp4bxkkp87rl7xy2dain77z977rvcry1gmr5bssdbn541v7s0d"))

(define rust-lazy-static-1.5.0
  (crate-source "lazy_static" "1.5.0"
                "1zk6dqqni0193xg6iijh7i3i44sryglwgvx20spdvwk3r6sbrlmv"))

(define rust-libc-0.2.175
  (crate-source "libc" "0.2.175"
                "0hw5sb3gjr0ivah7s3fmavlpvspjpd4mr009abmam2sr7r4sx0ka"))

(define rust-libm-0.2.15
  (crate-source "libm" "0.2.15"
                "1plpzf0p829viazdj57yw5dhmlr8ywf3apayxc2f2bq5a6mvryzr"))

(define rust-libredox-0.1.9
  (crate-source "libredox" "0.1.9"
                "1qqczzfqcc3sw3bl7la6qv7i9hy1s7sxhxmdvpxkfgdd3c9904ir"))

(define rust-linux-raw-sys-0.9.4
  ;; TODO: Check bundled sources.
  (crate-source "linux-raw-sys" "0.9.4"
                "04kyjdrq79lz9ibrf7czk6cv9d3jl597pb9738vzbsbzy1j5i56d"))

(define rust-litemap-0.8.0
  (crate-source "litemap" "0.8.0"
                "0mlrlskwwhirxk3wsz9psh6nxcy491n0dh8zl02qgj0jzpssw7i4"))

(define rust-lock-api-0.4.13
  (crate-source "lock_api" "0.4.13"
                "0rd73p4299mjwl4hhlfj9qr88v3r0kc8s1nszkfmnq2ky43nb4wn"))

(define rust-log-0.4.27
  (crate-source "log" "0.4.27"
                "150x589dqil307rv0rwj0jsgz5bjbwvl83gyl61jf873a7rjvp0k"))

(define rust-mac-0.1.1
  (crate-source "mac" "0.1.1"
                "194vc7vrshqff72rl56f9xgb0cazyl4jda7qsv31m5l6xx7hq7n4"))

(define rust-maplit-1.0.2
  (crate-source "maplit" "1.0.2"
                "07b5kjnhrrmfhgqm9wprjw8adx6i225lqp49gasgqg74lahnabiy"))

(define rust-markup5ever-0.35.0
  (crate-source "markup5ever" "0.35.0"
                "1hy1xh07jkm13j7vdnsphynl3z7hfmh99csjjvqzhl26jfffc7ri"))

(define rust-match-token-0.35.0
  (crate-source "match_token" "0.35.0"
                "1ksqd8li4kdd463cb2qf10d7d4j1m416y62xbzf47k0g6qzzv15c"))

(define rust-matchbox-protocol-0.12.0
  (crate-source "matchbox_protocol" "0.12.0"
                "09qwl3bvn4dd9r3bwxjkz1qqjhypp1cqz0s4l0gdm5mgsqyy9kzh"))

(define rust-matchbox-socket-0.12.0
  (crate-source "matchbox_socket" "0.12.0"
                "17iyb4j6m03qwf2hjskz6haw9h1h9jm0zd6x8cqj8mg6sffc6aww"))

(define rust-matchers-0.2.0
  (crate-source "matchers" "0.2.0"
                "1sasssspdj2vwcwmbq3ra18d3qniapkimfcbr47zmx6750m5llni"))

(define rust-md-5-0.10.6
  (crate-source "md-5" "0.10.6"
                "1kvq5rnpm4fzwmyv5nmnxygdhhb2369888a06gdc9pxyrzh7x7nq"))

(define rust-md5-0.7.0
  (crate-source "md5" "0.7.0"
                "0wcps37hrhz59fkhf8di1ppdnqld6l1w5sdy7jp7p51z0i4c8329"))

(define rust-memchr-2.7.5
  (crate-source "memchr" "2.7.5"
                "1h2bh2jajkizz04fh047lpid5wgw2cr9igpkdhl3ibzscpd858ij"))

(define rust-memoffset-0.7.1
  (crate-source "memoffset" "0.7.1"
                "1x2zv8hv9c9bvgmhsjvr9bymqwyxvgbca12cm8xkhpyy5k1r7s2x"))

(define rust-mime-0.3.17
  (crate-source "mime" "0.3.17"
                "16hkibgvb9klh0w0jk5crr5xv90l3wlf77ggymzjmvl1818vnxv8"))

(define rust-mime-guess-2.0.5
  (crate-source "mime_guess" "2.0.5"
                "03jmg3yx6j39mg0kayf7w4a886dl3j15y8zs119zw01ccy74zi7p"))

(define rust-minimal-lexical-0.2.1
  (crate-source "minimal-lexical" "0.2.1"
                "16ppc5g84aijpri4jzv14rvcnslvlpphbszc7zzp6vfkddf4qdb8"))

(define rust-miniz-oxide-0.8.9
  (crate-source "miniz_oxide" "0.8.9"
                "05k3pdg8bjjzayq3rf0qhpirq9k37pxnasfn4arbs17phqn6m9qz"))

(define rust-mio-1.0.4
  (crate-source "mio" "1.0.4"
                "073n3kam3nz8j8had35fd2nn7j6a33pi3y5w3kq608cari2d9gkq"))

(define rust-ndk-context-0.1.1
  (crate-source "ndk-context" "0.1.1"
                "12sai3dqsblsvfd1l1zab0z6xsnlha3xsfl7kagdnmj3an3jvc17"))

(define rust-new-debug-unreachable-1.0.6
  (crate-source "new_debug_unreachable" "1.0.6"
                "11phpf1mjxq6khk91yzcbd3ympm78m3ivl7xg6lg2c0lf66fy3k5"))

(define rust-nix-0.26.4
  (crate-source "nix" "0.26.4"
                "06xgl4ybb8pvjrbmc3xggbgk3kbs1j0c4c0nzdfrmpbgrkrym2sr"))

(define rust-nix-0.30.1
  (crate-source "nix" "0.30.1"
                "1dixahq9hk191g0c2ydc0h1ppxj0xw536y6rl63vlnp06lx3ylkl"))

(define rust-nom-7.1.3
  (crate-source "nom" "7.1.3"
                "0jha9901wxam390jcf5pfa0qqfrgh8li787jx2ip0yk5b8y9hwyj"))

(define rust-ntapi-0.4.1
  (crate-source "ntapi" "0.4.1"
                "1r38zhbwdvkis2mzs6671cm1p6djgsl49i7bwxzrvhwicdf8k8z8"))

(define rust-nu-ansi-term-0.50.1
  (crate-source "nu-ansi-term" "0.50.1"
                "16a3isvbxx8pa3lk71h3cq2fsx2d17zzq42j4mhpxy81gl2qx8nl"))

(define rust-num-bigint-0.4.6
  (crate-source "num-bigint" "0.4.6"
                "1f903zd33i6hkjpsgwhqwi2wffnvkxbn6rv4mkgcjcqi7xr4zr55"))

(define rust-num-conv-0.1.0
  (crate-source "num-conv" "0.1.0"
                "1ndiyg82q73783jq18isi71a7mjh56wxrk52rlvyx0mi5z9ibmai"))

(define rust-num-cpus-1.17.0
  (crate-source "num_cpus" "1.17.0"
                "0fxjazlng4z8cgbmsvbzv411wrg7x3hyxdq8nxixgzjswyylppwi"))

(define rust-num-integer-0.1.46
  (crate-source "num-integer" "0.1.46"
                "13w5g54a9184cqlbsq80rnxw4jj4s0d8wv75jsq5r2lms8gncsbr"))

(define rust-num-modular-0.6.1
  (crate-source "num-modular" "0.6.1"
                "0zv4miws3q1i93a0bd9wgc4njrr5j5786kr99hzxi9vgycdjdfqp"))

(define rust-num-order-1.2.0
  (crate-source "num-order" "1.2.0"
                "1dhvdncf91ljxh9sawnfxcbiqj1gnag08lyias0cy3y4jxmmjysk"))

(define rust-num-threads-0.1.7
  (crate-source "num_threads" "0.1.7"
                "1ngajbmhrgyhzrlc4d5ga9ych1vrfcvfsiqz6zv0h2dpr2wrhwsw"))

(define rust-num-traits-0.2.19
  (crate-source "num-traits" "0.2.19"
                "0h984rhdkkqd4ny9cif7y2azl3xdfb7768hb9irhpsch4q3gq787"))

(define rust-objc2-0.6.2
  (crate-source "objc2" "0.6.2"
                "1g3qa1vxp6nlh4wllll921z299d3s1is31m1ccasd8pklxxka7sn"))

(define rust-objc2-core-foundation-0.3.1
  (crate-source "objc2-core-foundation" "0.3.1"
                "0rn19d70mwxyv74kx7aqm5in6x320vavq9v0vrm81vbg9a4w440w"))

(define rust-objc2-encode-4.1.0
  (crate-source "objc2-encode" "4.1.0"
                "0cqckp4cpf68mxyc2zgnazj8klv0z395nsgbafa61cjgsyyan9gg"))

(define rust-objc2-foundation-0.3.1
  (crate-source "objc2-foundation" "0.3.1"
                "0g5hl47dxzabs7wndcg6kz3q137v9hwfay1jd2da1q9gglj3224h"))

(define rust-objc2-io-kit-0.3.1
  (crate-source "objc2-io-kit" "0.3.1"
                "02iwv7pppxvna72xwd7y5q67hrnbn5v73xikc3c1rr90c56wdhbi"))

(define rust-object-0.36.7
  (crate-source "object" "0.36.7"
                "11vv97djn9nc5n6w1gc6bd96d2qk2c8cg1kw5km9bsi3v4a8x532"))

(define rust-oid-registry-0.7.1
  (crate-source "oid-registry" "0.7.1"
                "1navxdy0gx7f92ymwr6n02x35fypp2izdfcf49wszkc9ji6h7n58"))

(define rust-once-cell-1.21.3
  (crate-source "once_cell" "1.21.3"
                "0b9x77lb9f1j6nqgf5aka4s2qj0nly176bpbrv6f9iakk5ff3xa2"))

(define rust-once-cell-polyfill-1.70.1
  (crate-source "once_cell_polyfill" "1.70.1"
                "1bg0w99srq8h4mkl68l1mza2n2f2hvrg0n8vfa3izjr5nism32d4"))

(define rust-opaque-debug-0.3.1
  (crate-source "opaque-debug" "0.3.1"
                "10b3w0kydz5jf1ydyli5nv10gdfp97xh79bgz327d273bs46b3f0"))

(define rust-option-ext-0.2.0
  (crate-source "option-ext" "0.2.0"
                "0zbf7cx8ib99frnlanpyikm1bx8qn8x602sw1n7bg6p9x94lyx04"))

(define rust-p256-0.13.2
  (crate-source "p256" "0.13.2"
                "0jyd3c3k239ybs59ixpnl7dqkmm072fr1js8kh7ldx58bzc3m1n9"))

(define rust-p384-0.13.1
  (crate-source "p384" "0.13.1"
                "1dnnp133mbpp72mfss3fhm8wx3yp3p3abdhlix27v92j19kz2hpy"))

(define rust-parking-2.2.1
  (crate-source "parking" "2.2.1"
                "1fnfgmzkfpjd69v4j9x737b1k8pnn054bvzcn5dm3pkgq595d3gk"))

(define rust-parking-lot-0.12.4
  (crate-source "parking_lot" "0.12.4"
                "04sab1c7304jg8k0d5b2pxbj1fvgzcf69l3n2mfpkdb96vs8pmbh"))

(define rust-parking-lot-core-0.9.11
  (crate-source "parking_lot_core" "0.9.11"
                "19g4d6m5k4ggacinqprnn8xvdaszc3y5smsmbz1adcdmaqm8v0xw"))

(define rust-passwords-3.1.16
  (crate-source "passwords" "3.1.16"
                "176mr017icfz736c7j6bbm6pmkzxlr6kkhqfdgn19gf2ly9p2h0i"))

(define rust-pbkdf2-0.12.2
  (crate-source "pbkdf2" "0.12.2"
                "1wms79jh4flpy1zi8xdp4h8ccxv4d85adc6zjagknvppc5vnmvgq"))

(define rust-pem-3.0.5
  (crate-source "pem" "3.0.5"
                "1wwfk8sbyi9l18fvvn6z9p2gy7v7q7wimbhvrvixxj8a8zl3ibrq"))

(define rust-pem-rfc7468-0.7.0
  (crate-source "pem-rfc7468" "0.7.0"
                "04l4852scl4zdva31c1z6jafbak0ni5pi0j38ml108zwzjdrrcw8"))

(define rust-percent-encoding-2.3.2
  (crate-source "percent-encoding" "2.3.2"
                "083jv1ai930azvawz2khv7w73xh8mnylk7i578cifndjn5y64kwv"))

(define rust-pest-2.8.1
  (crate-source "pest" "2.8.1"
                "08s342r6vv6ml5in4jk7pb97wgpf0frcnrvg0sqshn23sdb5zc0x"))

(define rust-pest-derive-2.8.1
  (crate-source "pest_derive" "2.8.1"
                "1g20ma4y29axbjhi3z64ihhpqzmiix71qjn7bs224yd7isg6s1dv"))

(define rust-pest-generator-2.8.1
  (crate-source "pest_generator" "2.8.1"
                "0rj9a20g4bjb4sl3zyzpxqg8mbn8c1kxp0nw08rfp0gp73k09r47"))

(define rust-pest-meta-2.8.1
  (crate-source "pest_meta" "2.8.1"
                "1mf01iln7shbnyxpdfnpf59gzn83nndqjkwiw3yh6n8g2wgi1lgd"))

(define rust-pharos-0.5.3
  (crate-source "pharos" "0.5.3"
                "055lg1dzrxnryfy34a9cyrg21b7cl6l2frfx2p7fdvkz864p6mp9"))

(define rust-phf-0.11.3
  (crate-source "phf" "0.11.3"
                "0y6hxp1d48rx2434wgi5g8j1pr8s5jja29ha2b65435fh057imhz"))

(define rust-phf-0.12.1
  (crate-source "phf" "0.12.1"
                "1dz85g1wshfca83mrq3va9rm9n8qcdjlpv1i3908y5zc9j4p6cli"))

(define rust-phf-codegen-0.11.3
  (crate-source "phf_codegen" "0.11.3"
                "0si1n6zr93kzjs3wah04ikw8z6npsr39jw4dam8yi9czg2609y5f"))

(define rust-phf-generator-0.11.3
  (crate-source "phf_generator" "0.11.3"
                "0gc4np7s91ynrgw73s2i7iakhb4lzdv1gcyx7yhlc0n214a2701w"))

(define rust-phf-generator-0.12.1
  (crate-source "phf_generator" "0.12.1"
                "0nvcdbc2j7pznjin4qhw360c8zgfn5isx3bld1ixsqgdmwk13frc"))

(define rust-phf-macros-0.11.3
  (crate-source "phf_macros" "0.11.3"
                "05kjfbyb439344rhmlzzw0f9bwk9fp95mmw56zs7yfn1552c0jpq"))

(define rust-phf-macros-0.12.1
  (crate-source "phf_macros" "0.12.1"
                "0s230ikydi4n2vi9s03d49k5fznkg6kpq12jmlg0jbx8jf1ja4yp"))

(define rust-phf-shared-0.11.3
  (crate-source "phf_shared" "0.11.3"
                "1rallyvh28jqd9i916gk5gk2igdmzlgvv5q0l3xbf3m6y8pbrsk7"))

(define rust-phf-shared-0.12.1
  (crate-source "phf_shared" "0.12.1"
                "10cr16wpmbjxd7w6k98sxw9yw3zxnzscybl9jzyq3digi045a006"))

(define rust-pin-project-1.1.10
  (crate-source "pin-project" "1.1.10"
                "12kadbnfm1f43cyadw9gsbyln1cy7vj764wz5c8wxaiza3filzv7"))

(define rust-pin-project-internal-1.1.10
  (crate-source "pin-project-internal" "1.1.10"
                "0qgqzfl0f4lzaz7yl5llhbg97g68r15kljzihaw9wm64z17qx4bf"))

(define rust-pin-project-lite-0.2.16
  (crate-source "pin-project-lite" "0.2.16"
                "16wzc7z7dfkf9bmjin22f5282783f6mdksnr0nv0j5ym5f9gyg1v"))

(define rust-pin-utils-0.1.0
  (crate-source "pin-utils" "0.1.0"
                "117ir7vslsl2z1a7qzhws4pd01cg2d3338c47swjyvqv2n60v1wb"))

(define rust-piper-0.2.4
  (crate-source "piper" "0.2.4"
                "0rn0mjjm0cwagdkay77wgmz3sqf8fqmv9d9czm79mvr2yj8c9j4n"))

(define rust-pkcs8-0.10.2
  (crate-source "pkcs8" "0.10.2"
                "1dx7w21gvn07azszgqd3ryjhyphsrjrmq5mmz1fbxkj5g0vv4l7r"))

(define rust-polling-3.10.0
  (crate-source "polling" "3.10.0"
                "0afqlnm45081k06sngc052k9vmh33j2rqrmjgi7q1zjhcca1kgdm"))

(define rust-poly1305-0.8.0
  (crate-source "poly1305" "0.8.0"
                "1grs77skh7d8vi61ji44i8gpzs3r9x7vay50i6cg8baxfa8bsnc1"))

(define rust-polyval-0.6.2
  (crate-source "polyval" "0.6.2"
                "09gs56vm36ls6pyxgh06gw2875z2x77r8b2km8q28fql0q6yc7wx"))

(define rust-portable-atomic-1.11.1
  (crate-source "portable-atomic" "1.11.1"
                "10s4cx9y3jvw0idip09ar52s2kymq8rq9a668f793shn1ar6fhpq"))

(define rust-portpicker-0.1.1
  (crate-source "portpicker" "0.1.1"
                "1acvi1m6g7d3j8xvdsbn0b7yqyfy7yr7fm1pw5kbdyhvmxpxg5xy"))

(define rust-potential-utf-0.1.3
  (crate-source "potential_utf" "0.1.3"
                "12mhwvhpvvim6xqp6ifgkh1sniv9j2cmid6axn10fnjvpsnikpw4"))

(define rust-powerfmt-0.2.0
  (crate-source "powerfmt" "0.2.0"
                "14ckj2xdpkhv3h6l5sdmb9f1d57z8hbfpdldjc2vl5givq2y77j3"))

(define rust-ppv-lite86-0.2.21
  (crate-source "ppv-lite86" "0.2.21"
                "1abxx6qz5qnd43br1dd9b2savpihzjza8gb4fbzdql1gxp2f7sl5"))

(define rust-precomputed-hash-0.1.1
  (crate-source "precomputed-hash" "0.1.1"
                "075k9bfy39jhs53cb2fpb9klfakx2glxnf28zdw08ws6lgpq6lwj"))

(define rust-primeorder-0.13.6
  (crate-source "primeorder" "0.13.6"
                "1rp16710mxksagcjnxqjjq9r9wf5vf72fs8wxffnvhb6i6hiqgim"))

(define rust-proc-macro-crate-1.3.1
  (crate-source "proc-macro-crate" "1.3.1"
                "069r1k56bvgk0f58dm5swlssfcp79im230affwk6d9ck20g04k3z"))

(define rust-proc-macro-error-attr2-2.0.0
  (crate-source "proc-macro-error-attr2" "2.0.0"
                "1ifzi763l7swl258d8ar4wbpxj4c9c2im7zy89avm6xv6vgl5pln"))

(define rust-proc-macro-error2-2.0.1
  (crate-source "proc-macro-error2" "2.0.1"
                "00lq21vgh7mvyx51nwxwf822w2fpww1x0z8z0q47p8705g2hbv0i"))

(define rust-proc-macro-hack-0.5.20+deprecated
  (crate-source "proc-macro-hack" "0.5.20+deprecated"
                "0s402hmcs3k9nd6rlp07zkr1lz7yimkmcwcbgnly2zr44wamwdyw"))

(define rust-proc-macro2-1.0.101
  (crate-source "proc-macro2" "1.0.101"
                "1pijhychkpl7rcyf1h7mfk6gjfii1ywf5n0snmnqs5g4hvyl7bl9"))

(define rust-quick-error-1.2.3
  (crate-source "quick-error" "1.2.3"
                "1q6za3v78hsspisc197bg3g7rpc989qycy8ypr8ap8igv10ikl51"))

(define rust-quote-1.0.40
  (crate-source "quote" "1.0.40"
                "1394cxjg6nwld82pzp2d4fp6pmaz32gai1zh9z5hvh0dawww118q"))

(define rust-r-efi-5.3.0
  (crate-source "r-efi" "5.3.0"
                "03sbfm3g7myvzyylff6qaxk4z6fy76yv860yy66jiswc2m6b7kb9"))

(define rust-rand-0.8.5
  (crate-source "rand" "0.8.5"
                "013l6931nn7gkc23jz5mm3qdhf93jjf0fg64nz2lp4i51qd8vbrl"))

(define rust-rand-0.9.2
  (crate-source "rand" "0.9.2"
                "1lah73ainvrgl7brcxx0pwhpnqa3sm3qaj672034jz8i0q7pgckd"))

(define rust-rand-chacha-0.3.1
  (crate-source "rand_chacha" "0.3.1"
                "123x2adin558xbhvqb8w4f6syjsdkmqff8cxwhmjacpsl1ihmhg6"))

(define rust-rand-chacha-0.9.0
  (crate-source "rand_chacha" "0.9.0"
                "1jr5ygix7r60pz0s1cv3ms1f6pd1i9pcdmnxzzhjc3zn3mgjn0nk"))

(define rust-rand-core-0.6.4
  (crate-source "rand_core" "0.6.4"
                "0b4j2v4cb5krak1pv6kakv4sz6xcwbrmy2zckc32hsigbrwy82zc"))

(define rust-rand-core-0.9.3
  (crate-source "rand_core" "0.9.3"
                "0f3xhf16yks5ic6kmgxcpv1ngdhp48mmfy4ag82i1wnwh8ws3ncr"))

(define rust-random-number-0.1.9
  (crate-source "random-number" "0.1.9"
                "0rv0g538b1xfaq7gbaf6kj7vfd4cpd1sggrxzhppfr76kgacvj3z"))

(define rust-random-number-macro-impl-0.1.8
  (crate-source "random-number-macro-impl" "0.1.8"
                "18jazjmx42vplfc27snlpx69pmf0zsziaily2f4l5la8rd1m24zm"))

(define rust-random-pick-1.2.16
  (crate-source "random-pick" "1.2.16"
                "1y50la3p7cwn06cmkxvfz71f4b81lr55yz8j8kz9ly6sfa84jyf1"))

(define rust-rcgen-0.13.2
  (crate-source "rcgen" "0.13.2"
                "18l0rz228pvnc44bjmvq8cchhh5d2rrkk98y9lqvan9243jnkrkm"))

(define rust-redb-3.0.1
  (crate-source "redb" "3.0.1"
                "045vzaf13gjxa06krw89s6x9lvrxhyal3pynqccrhdjazzjs7vrz"))

(define rust-redox-syscall-0.5.17
  (crate-source "redox_syscall" "0.5.17"
                "0xrvpchkaxph3r5ww2i04v9nwg3843fp3prf8kqlh1gv01b4c1sl"))

(define rust-redox-users-0.5.2
  (crate-source "redox_users" "0.5.2"
                "1b17q7gf7w8b1vvl53bxna24xl983yn7bd00gfbii74bcg30irm4"))

(define rust-regex-1.11.2
  (crate-source "regex" "1.11.2"
                "04k9rzxd11hcahpyihlswy6f1zqw7lspirv4imm4h0lcdl8gvmr3"))

(define rust-regex-automata-0.4.10
  (crate-source "regex-automata" "0.4.10"
                "1mllcfmgjcl6d52d5k09lwwq9wj5mwxccix4bhmw5spy1gx5i53b"))

(define rust-regex-lite-0.1.7
  (crate-source "regex-lite" "0.1.7"
                "0c5s0kyc4gch0l0rzhm54prgfs169l2zwfvnzn91rvv33hr42gwl"))

(define rust-regex-syntax-0.8.6
  (crate-source "regex-syntax" "0.8.6"
                "00chjpglclfskmc919fj5aq308ffbrmcn7kzbkz92k231xdsmx6a"))

(define rust-rfc6979-0.4.0
  (crate-source "rfc6979" "0.4.0"
                "1chw95jgcfrysyzsq6a10b1j5qb7bagkx8h0wda4lv25in02mpgq"))

(define rust-ring-0.17.14
  (crate-source "ring" "0.17.14"
                "1dw32gv19ccq4hsx3ribhpdzri1vnrlcfqb2vj41xn4l49n9ws54"))

(define rust-rouille-3.6.2.31f772c
  ;; TODO: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/tomaka/rouille.git")
                        (commit "31f772c40007503d25e60e9aba77836a4273ec26")))
    (file-name (git-file-name "rust-rouille" "3.6.2.31f772c"))
    (sha256 (base32 "138kclylm6g4dpmc2ds45maal7yhcdsrrhblwynx05gpph9swq82"))))

(define rust-rouille-multipart-0.18.0.31f772c
  ;; TODO: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/tomaka/rouille.git")
                        (commit "31f772c40007503d25e60e9aba77836a4273ec26")))
    (file-name (git-file-name "rust-rouille-multipart" "0.18.0.31f772c"))
    (sha256 (base32 "138kclylm6g4dpmc2ds45maal7yhcdsrrhblwynx05gpph9swq82"))))

(define rust-rtcp-0.13.0
  (crate-source "rtcp" "0.13.0"
                "0wmz7p62gny1ry5grzz4khpl2inmdm8hb3ckzl8v77ispwl9as79"))

(define rust-rtp-0.13.0
  (crate-source "rtp" "0.13.0"
                "0jqg41cx8qw0z53pham9vnlix1y6rsiaf1xakjpnrmv7392k6iy5"))

(define rust-rust-embed-8.7.2
  (crate-source "rust-embed" "8.7.2"
                "12hprnl569f1pg2sn960gfla913mk1mxdwpn2a6vl9iad2w0hn82"))

(define rust-rust-embed-impl-8.7.2
  (crate-source "rust-embed-impl" "8.7.2"
                "171lshvdh122ypbf23gmhvrqnhbk0q9g27gaq6g82w9b76jg2rb0"))

(define rust-rust-embed-utils-8.7.2
  (crate-source "rust-embed-utils" "8.7.2"
                "151m1966qk75y10msazdp0xj4fqw1khcry0z946bf84bcj0hrk7n"))

(define rust-rustc-demangle-0.1.26
  (crate-source "rustc-demangle" "0.1.26"
                "1kja3nb0yhlm4j2p1hl8d7sjmn2g9fa1s4pj0qma5kj2lcndkxsn"))

(define rust-rustc-hash-1.1.0
  (crate-source "rustc-hash" "1.1.0"
                "1qkc5khrmv5pqi5l5ca9p5nl5hs742cagrndhbrlk3dhlrx3zm08"))

(define rust-rustc-hash-2.1.1
  (crate-source "rustc-hash" "2.1.1"
                "03gz5lvd9ghcwsal022cgkq67dmimcgdjghfb5yb5d352ga06xrm"))

(define rust-rustc-version-0.4.1
  (crate-source "rustc_version" "0.4.1"
                "14lvdsmr5si5qbqzrajgb6vfn69k0sfygrvfvr2mps26xwi3mjyg"))

(define rust-rusticata-macros-4.1.0
  (crate-source "rusticata-macros" "4.1.0"
                "0ch67lljmgl5pfrlb90bl5kkp2x6yby1qaxnpnd0p5g9xjkc9w7s"))

(define rust-rustix-1.0.8
  (crate-source "rustix" "1.0.8"
                "1j6ajqi61agdnh1avr4bplrsgydjw1n4mycdxw3v8g94pyx1y60i"))

(define rust-rustls-0.21.12
  (crate-source "rustls" "0.21.12"
                "0gjdg2a9r81sdwkyw3n5yfbkrr6p9gyk3xr2kcsr3cs83x6s2miz"))

(define rust-rustls-0.23.31
  (crate-source "rustls" "0.23.31"
                "1k5ncablbb2h7hzllq3j3panqnks295v56xd488zrq1xy39cpsy0"))

(define rust-rustls-pemfile-1.0.4
  (crate-source "rustls-pemfile" "1.0.4"
                "1324n5bcns0rnw6vywr5agff3rwfvzphi7rmbyzwnv6glkhclx0w"))

(define rust-rustls-pemfile-2.2.0
  (crate-source "rustls-pemfile" "2.2.0"
                "0l3f3mrfkgdjrava7ibwzgwc4h3dljw3pdkbsi9rkwz3zvji9qyw"))

(define rust-rustls-pki-types-1.12.0
  (crate-source "rustls-pki-types" "1.12.0"
                "0yawbdpix8jif6s8zj1p2hbyb7y3bj66fhx0y7hyf4qh4964m6i2"))

(define rust-rustls-webpki-0.101.7
  (crate-source "rustls-webpki" "0.101.7"
                "0rapfhpkqp75552i8r0y7f4vq7csb4k7gjjans0df73sxv8paqlb"))

(define rust-rustls-webpki-0.103.4
  (crate-source "rustls-webpki" "0.103.4"
                "1z4jmmgasjgk9glb160a66bshvgifa64mgfjrkqp7dy1w158h5qa"))

(define rust-rustversion-1.0.22
  (crate-source "rustversion" "1.0.22"
                "0vfl70jhv72scd9rfqgr2n11m5i9l1acnk684m2w83w0zbqdx75k"))

(define rust-ryu-1.0.20
  (crate-source "ryu" "1.0.20"
                "07s855l8sb333h6bpn24pka5sp7hjk2w667xy6a0khkf6sqv5lr8"))

(define rust-safemem-0.3.3
  (crate-source "safemem" "0.3.3"
                "0wp0d2b2284lw11xhybhaszsczpbq1jbdklkxgifldcknmy3nw7g"))

(define rust-salsa20-0.10.2
  (crate-source "salsa20" "0.10.2"
                "04w211x17xzny53f83p8f7cj7k2hi8zck282q5aajwqzydd2z8lp"))

(define rust-same-file-1.0.6
  (crate-source "same-file" "1.0.6"
                "00h5j1w87dmhnvbv9l8bic3y7xxsnjmssvifw2ayvgx9mb1ivz4k"))

(define rust-scopeguard-1.2.0
  (crate-source "scopeguard" "1.2.0"
                "0jcz9sd47zlsgcnm1hdw0664krxwb5gczlif4qngj2aif8vky54l"))

(define rust-scrypt-0.11.0
  (crate-source "scrypt" "0.11.0"
                "07zxfaqpns9jn0mnxm7wj3ksqsinyfpirkav1f7kc2bchs2s65h5"))

(define rust-sct-0.7.1
  (crate-source "sct" "0.7.1"
                "056lmi2xkzdg1dbai6ha3n57s18cbip4pnmpdhyljli3m99n216s"))

(define rust-sdp-0.8.0
  (crate-source "sdp" "0.8.0"
                "0sbs12hzdvapyi1b8bx67xbi1s7n7d6vi90hp05lm95dbq0pgljc"))

(define rust-sec1-0.7.3
  (crate-source "sec1" "0.7.3"
                "1p273j8c87pid6a1iyyc7vxbvifrw55wbxgr0dh3l8vnbxb7msfk"))

(define rust-secrecy-0.10.3
  (crate-source "secrecy" "0.10.3"
                "0nmfsf9qm8921v2jliz08bj8zrryqar4gj3d6irqfc3kaj2az4g8"))

(define rust-self-cell-0.10.3
  (crate-source "self_cell" "0.10.3"
                "0pci3zh23b7dg6jmlxbn8k4plb7hcg5jprd1qiz0rp04p1ilskp1"))

(define rust-self-cell-1.2.0
  (crate-source "self_cell" "1.2.0"
                "0jg70srf4hzrw96x8iclgf6i8dfgm1x8ds2i7yzcgq0i8njraz8g"))

(define rust-semver-1.0.26
  (crate-source "semver" "1.0.26"
                "1l5q2vb8fjkby657kdyfpvv40x2i2xqq9bg57pxqakfj92fgmrjn"))

(define rust-send-wrapper-0.4.0
  (crate-source "send_wrapper" "0.4.0"
                "1l7s28vfnwdbjyrrk3lx81jy4f0dcrv4iwyah2wj6vndxhqxaf7n"))

(define rust-send-wrapper-0.6.0
  (crate-source "send_wrapper" "0.6.0"
                "0wrxzsh9fzgkkkms621ydnz8mj30ilyq299a8cf65jn1y72hw2yd"))

(define rust-serde-1.0.219
  (crate-source "serde" "1.0.219"
                "1dl6nyxnsi82a197sd752128a4avm6mxnscywas1jq30srp2q3jz"))

(define rust-serde-derive-1.0.219
  (crate-source "serde_derive" "1.0.219"
                "001azhjmj7ya52pmfiw4ppxm16nd44y15j2pf5gkcwrcgz7pc0jv"))

(define rust-serde-json-1.0.143
  (crate-source "serde_json" "1.0.143"
                "0njabwzldvj13ykrf1aaf4gh5cgl25kf9hzbpafbv3qh3ppsn0fl"))

(define rust-serde-spanned-1.0.0
  (crate-source "serde_spanned" "1.0.0"
                "10rv91337k8x8zmfir4h8aiwmwgkq07gdv7h0jxhcwwgk10lqws0"))

(define rust-serde-wasm-bindgen-0.6.5
  (crate-source "serde-wasm-bindgen" "0.6.5"
                "0sz1l4v8059hiizf5z7r2spm6ws6sqcrs4qgqwww3p7dy1ly20l3"))

(define rust-sha1-0.10.6
  (crate-source "sha1" "0.10.6"
                "1fnnxlfg08xhkmwf2ahv634as30l1i3xhlhkvxflmasi5nd85gz3"))

(define rust-sha1-smol-1.0.1
  (crate-source "sha1_smol" "1.0.1"
                "0pbh2xjfnzgblws3hims0ib5bphv7r5rfdpizyh51vnzvnribymv"))

(define rust-sha2-0.10.9
  (crate-source "sha2" "0.10.9"
                "10xjj843v31ghsksd9sl9y12qfc48157j1xpb8v1ml39jy0psl57"))

(define rust-sharded-slab-0.1.7
  (crate-source "sharded-slab" "0.1.7"
                "1xipjr4nqsgw34k7a2cgj9zaasl2ds6jwn89886kww93d32a637l"))

(define rust-shlex-1.3.0
  (crate-source "shlex" "1.3.0"
                "0r1y6bv26c1scpxvhg2cabimrmwgbp4p3wy6syj9n0c4s3q2znhg"))

(define rust-signal-hook-registry-1.4.6
  (crate-source "signal-hook-registry" "1.4.6"
                "12y2v1ms5z111fymaw1v8k93m5chnkp21h0jknrydkj8zydp395j"))

(define rust-signature-2.2.0
  (crate-source "signature" "2.2.0"
                "1pi9hd5vqfr3q3k49k37z06p7gs5si0in32qia4mmr1dancr6m3p"))

(define rust-siphasher-1.0.1
  (crate-source "siphasher" "1.0.1"
                "17f35782ma3fn6sh21c027kjmd227xyrx06ffi8gw4xzv9yry6an"))

(define rust-slab-0.4.11
  (crate-source "slab" "0.4.11"
                "12bm4s88rblq02jjbi1dw31984w61y2ldn13ifk5gsqgy97f8aks"))

(define rust-smallvec-1.15.1
  (crate-source "smallvec" "1.15.1"
                "00xxdxxpgyq5vjnpljvkmy99xij5rxgh913ii1v16kzynnivgcb7"))

(define rust-smol-str-0.2.2
  (crate-source "smol_str" "0.2.2"
                "1bfylqf2vnqaglw58930vpxm2rfzji5gjp15a2c0kh8aj6v8ylyx"))

(define rust-socket2-0.5.10
  (crate-source "socket2" "0.5.10"
                "0y067ki5q946w91xlz2sb175pnfazizva6fi3kfp639mxnmpc8z2"))

(define rust-socket2-0.6.0
  (crate-source "socket2" "0.6.0"
                "01qqdzfnr0bvdwq6wl56c9c4m2cvbxn43dfpcv8gjx208sph8d93"))

(define rust-spki-0.7.3
  (crate-source "spki" "0.7.3"
                "17fj8k5fmx4w9mp27l970clrh5qa7r5sjdvbsln987xhb34dc7nr"))

(define rust-stable-deref-trait-1.2.0
  (crate-source "stable_deref_trait" "1.2.0"
                "1lxjr8q2n534b2lhkxd6l6wcddzjvnksi58zv11f9y0jjmr15wd8"))

(define rust-string-cache-0.8.9
  (crate-source "string_cache" "0.8.9"
                "03z7km2kzlwiv2r2qifq5riv4g8phazwng9wnvs3py3lzainnxxz"))

(define rust-string-cache-codegen-0.5.4
  (crate-source "string_cache_codegen" "0.5.4"
                "181ir4d6y053s1kka2idpjx5g9d9jgll6fy517jhzzpi2n3r44f7"))

(define rust-strsim-0.11.1
  (crate-source "strsim" "0.11.1"
                "0kzvqlw8hxqb7y598w1s0hxlnmi84sg5vsipp3yg5na5d1rvba3x"))

(define rust-stun-0.8.0
  (crate-source "stun" "0.8.0"
                "12h8ibw8hmk647diqf9qx5wzp8dn7yh65hrx2hy0j92m6ymjpg3x"))

(define rust-substring-1.4.5
  (crate-source "substring" "1.4.5"
                "11jcadn4h1xwx3dq5gbgs5y3x57ml9jfz1zmf8p3n8ggxhrn9vj2"))

(define rust-subtle-2.6.1
  (crate-source "subtle" "2.6.1"
                "14ijxaymghbl1p0wql9cib5zlwiina7kall6w7g89csprkgbvhhk"))

(define rust-syn-1.0.109
  (crate-source "syn" "1.0.109"
                "0ds2if4600bd59wsv7jjgfkayfzy3hnazs394kz6zdkmna8l3dkj"))

(define rust-syn-2.0.106
  (crate-source "syn" "2.0.106"
                "19mddxp1ia00hfdzimygqmr1jqdvyl86k48427bkci4d08wc9rzd"))

(define rust-synstructure-0.13.2
  (crate-source "synstructure" "0.13.2"
                "1lh9lx3r3jb18f8sbj29am5hm9jymvbwh6jb1izsnnxgvgrp12kj"))

(define rust-sysinfo-0.37.0
  (crate-source "sysinfo" "0.37.0"
                "0lvpm1728gazair9zs4z7vy10ypsw9yv1kqhwshpqd9f5pfc9kh7"))

(define rust-tempfile-3.21.0
  (crate-source "tempfile" "3.21.0"
                "07kx58ibjk3ydq1gcb7q637fs5zkxaa550lxckhgg9p3427izdhm"))

(define rust-tendril-0.4.3
  (crate-source "tendril" "0.4.3"
                "1c3vip59sqwxn148i714nmkrvjzbk7105vj0h92s6r64bw614jnj"))

(define rust-test-log-0.2.18
  (crate-source "test-log" "0.2.18"
                "0yxywma018rfr4mb409b1yz4ppg8ir9rg87bd08vx81fb25bjcqy"))

(define rust-test-log-macros-0.2.18
  (crate-source "test-log-macros" "0.2.18"
                "0djzwzwqnalwf00r81lv0yv71s4sqwmx7y7fn40pc3ck552kf6s5"))

(define rust-text-io-0.1.13
  (crate-source "text_io" "0.1.13"
                "058ifqlmnf15jy7rr1mm20m2sw8hx6aqj7c40d70k4k2n2ikr3ad"))

(define rust-thiserror-1.0.69
  (crate-source "thiserror" "1.0.69"
                "0lizjay08agcr5hs9yfzzj6axs53a2rgx070a1dsi3jpkcrzbamn"))

(define rust-thiserror-2.0.16
  (crate-source "thiserror" "2.0.16"
                "1h30bqyjn5s9ypm668yd9849371rzwk185klwgjg503k2hadcrrl"))

(define rust-thiserror-impl-1.0.69
  (crate-source "thiserror-impl" "1.0.69"
                "1h84fmn2nai41cxbhk6pqf46bxqq1b344v8yz089w1chzi76rvjg"))

(define rust-thiserror-impl-2.0.16
  (crate-source "thiserror-impl" "2.0.16"
                "0q3r1ipr1rhff6cgrcvc0njffw17rpcqz9hdc7p754cbqkhinpkc"))

(define rust-thread-local-1.1.9
  (crate-source "thread_local" "1.1.9"
                "1191jvl8d63agnq06pcnarivf63qzgpws5xa33hgc92gjjj4c0pn"))

(define rust-threadpool-1.8.1
  (crate-source "threadpool" "1.8.1"
                "1amgfyzvynbm8pacniivzq9r0fh3chhs7kijic81j76l6c5ycl6h"))

(define rust-time-0.3.41
  (crate-source "time" "0.3.41"
                "0h0cpiyya8cjlrh00d2r72bmgg4lsdcncs76qpwy0rn2kghijxla"))

(define rust-time-core-0.1.4
  (crate-source "time-core" "0.1.4"
                "0z5h9fknvdvbs2k2s1chpi3ab3jvgkfhdnqwrvixjngm263s7sf9"))

(define rust-time-macros-0.2.22
  (crate-source "time-macros" "0.2.22"
                "0jcaxpw220han2bzbrdlpqhy1s5k9i8ri3lw6n5zv4zcja9p69im"))

(define rust-tinystr-0.8.1
  (crate-source "tinystr" "0.8.1"
                "12sc6h3hnn6x78iycm5v6wrs2xhxph0ydm43yyn7gdfw8l8nsksx"))

(define rust-tokio-1.47.1
  (crate-source "tokio" "1.47.1"
                "0f2hp5v3payg6x04ijj67si1wsdhksskhmjs2k9p5f7bmpyrmr49"))

(define rust-tokio-macros-2.5.0
  (crate-source "tokio-macros" "2.5.0"
                "1f6az2xbvqp7am417b78d1za8axbvjvxnmkakz9vr8s52czx81kf"))

(define rust-tokio-util-0.7.16
  (crate-source "tokio-util" "0.7.16"
                "1r9wdrg1k5hna3m0kc8kcb8jdb6n52g7vnw93kw2xxw4cyc7qc0l"))

(define rust-toml-0.5.11
  (crate-source "toml" "0.5.11"
                "0d2266nx8b3n22c7k24x4428z6di8n83a9n466jm7a2hipfz1xzl"))

(define rust-toml-0.9.5
  (crate-source "toml" "0.9.5"
                "1s7n4l40hvpf46jmgidfknnzpyblz4hip7gfkymgn2q0qlfrw4km"))

(define rust-toml-datetime-0.6.11
  (crate-source "toml_datetime" "0.6.11"
                "077ix2hb1dcya49hmi1avalwbixmrs75zgzb3b2i7g2gizwdmk92"))

(define rust-toml-datetime-0.7.0
  (crate-source "toml_datetime" "0.7.0"
                "1qwivxqkjxxwcqsvfhxnphpwphci0grdfk197wyxfn1gj0z1rpms"))

(define rust-toml-edit-0.19.15
  (crate-source "toml_edit" "0.19.15"
                "08bl7rp5g6jwmfpad9s8jpw8wjrciadpnbaswgywpr9hv9qbfnqv"))

(define rust-toml-parser-1.0.2
  (crate-source "toml_parser" "1.0.2"
                "042wp5ni22yqcbrfqq9c63g2vbbp4m59zamxw97hvacs8ipqhldm"))

(define rust-toml-writer-1.0.2
  (crate-source "toml_writer" "1.0.2"
                "0r7x3m050c66s9lssaq965vmrsxvxj131db4fq0m5vrd3w4l5j7w"))

(define rust-tracing-0.1.41
  (crate-source "tracing" "0.1.41"
                "1l5xrzyjfyayrwhvhldfnwdyligi1mpqm8mzbi2m1d6y6p2hlkkq"))

(define rust-tracing-core-0.1.34
  (crate-source "tracing-core" "0.1.34"
                "0y3nc4mpnr79rzkrcylv5f5bnjjp19lsxwis9l4kzs97ya0jbldr"))

(define rust-tracing-log-0.2.0
  (crate-source "tracing-log" "0.2.0"
                "1hs77z026k730ij1a9dhahzrl0s073gfa2hm5p0fbl0b80gmz1gf"))

(define rust-tracing-subscriber-0.3.20
  (crate-source "tracing-subscriber" "0.3.20"
                "1m9447bxq7236avgl6n5yb2aqwplrghm61dgipw03mh7ad7s2m10"))

(define rust-tungstenite-0.26.2
  (crate-source "tungstenite" "0.26.2"
                "04rwwcxx95m3avi46rmn0kmpb6nynqimnla3v2qwn3k8argcp4s7"))

(define rust-turn-0.10.0
  (crate-source "turn" "0.10.0"
                "19592ra4gapn7ywmwhxz9wb46frfqy3nnn253jkisvj52q8ylniz"))

(define rust-twoway-0.1.8
  (crate-source "twoway" "0.1.8"
                "1lbf64snscr3vz71jbl6i2c8zr2ndsiqbk6316z39fj1a8mipcar"))

(define rust-type-map-0.5.1
  (crate-source "type-map" "0.5.1"
                "143v32wwgpymxfy4y8s694vyq0wdi7li4s5dmms5w59nj2yxnc6b"))

(define rust-typed-arena-2.0.2
  (crate-source "typed-arena" "2.0.2"
                "0shj0jpmglhgw2f1i4b33ycdzwd1z205pbs1rd5wx7ks2qhaxxka"))

(define rust-typenum-1.18.0
  (crate-source "typenum" "1.18.0"
                "0gwgz8n91pv40gabrr1lzji0b0hsmg0817njpy397bq7rvizzk0x"))

(define rust-ucd-trie-0.1.7
  (crate-source "ucd-trie" "0.1.7"
                "0wc9p07sqwz320848i52nvyjvpsxkx3kv5bfbmm6s35809fdk5i8"))

(define rust-unic-langid-0.9.6
  (crate-source "unic-langid" "0.9.6"
                "01bx59sqsx2jz4z7ppxq9kldcjq9dzadkmb2dr7iyc85kcnab2x2"))

(define rust-unic-langid-impl-0.9.6
  (crate-source "unic-langid-impl" "0.9.6"
                "0n66kdan4cz99n8ra18i27f7w136hmppi4wc0aa7ljsd0h4bzqfw"))

(define rust-unicase-2.8.1
  (crate-source "unicase" "2.8.1"
                "0fd5ddbhpva7wrln2iah054ar2pc1drqjcll0f493vj3fv8l9f3m"))

(define rust-unicode-ident-1.0.18
  (crate-source "unicode-ident" "1.0.18"
                "04k5r6sijkafzljykdq26mhjpmhdx4jwzvn1lh90g9ax9903jpss"))

(define rust-unicode-segmentation-1.12.0
  (crate-source "unicode-segmentation" "1.12.0"
                "14qla2jfx74yyb9ds3d2mpwpa4l4lzb9z57c6d2ba511458z5k7n"))

(define rust-unicode-xid-0.2.6
  (crate-source "unicode-xid" "0.2.6"
                "0lzqaky89fq0bcrh6jj6bhlz37scfd8c7dsj5dq7y32if56c1hgb"))

(define rust-universal-hash-0.5.1
  (crate-source "universal-hash" "0.5.1"
                "1sh79x677zkncasa95wz05b36134822w6qxmi1ck05fwi33f47gw"))

(define rust-untrusted-0.9.0
  (crate-source "untrusted" "0.9.0"
                "1ha7ib98vkc538x0z60gfn0fc5whqdd85mb87dvisdcaifi6vjwf"))

(define rust-ureq-3.1.0
  (crate-source "ureq" "3.1.0"
                "0w6xp9p7fwmpdgw0yigi26dnn8pq08xynnm68y75vnvi754jyhq0"))

(define rust-ureq-proto-0.5.0
  (crate-source "ureq-proto" "0.5.0"
                "1k14j8bcp1asqd8hi8n8v2kww1k2jlpva1mbi58w9i7cpfzcmdn5"))

(define rust-url-2.5.7
  (crate-source "url" "2.5.7"
                "0nzghdv0kcksyvri0npxbjzyx2ihprks5k590y77bld355m17g08"))

(define rust-urlencoding-2.1.3
  (crate-source "urlencoding" "2.1.3"
                "1nj99jp37k47n0hvaz5fvz7z6jd0sb4ppvfy3nphr1zbnyixpy6s"))

(define rust-utf-8-0.7.6
  (crate-source "utf-8" "0.7.6"
                "1a9ns3fvgird0snjkd3wbdhwd3zdpc2h5gpyybrfr6ra5pkqxk09"))

(define rust-utf8-iter-1.0.4
  (crate-source "utf8_iter" "1.0.4"
                "1gmna9flnj8dbyd8ba17zigrp9c4c3zclngf5lnb5yvz1ri41hdn"))

(define rust-utf8parse-0.2.2
  (crate-source "utf8parse" "0.2.2"
                "088807qwjq46azicqwbhlmzwrbkz7l4hpw43sdkdyyk524vdxaq6"))

(define rust-uuid-1.18.0
  (crate-source "uuid" "1.18.0"
                "1gn1vlggiwrdpizqcpc5hyxsqz9s5215bbay1b182mqn7rj9ccgk"))

(define rust-uuid-macro-internal-1.18.0
  (crate-source "uuid-macro-internal" "1.18.0"
                "0zmrvbnbvvxx3ksy8xnvc26ip2d7v9wblva3x9gxnxl20q0avdr2"))

(define rust-valuable-0.1.1
  (crate-source "valuable" "0.1.1"
                "0r9srp55v7g27s5bg7a2m095fzckrcdca5maih6dy9bay6fflwxs"))

(define rust-value-bag-1.11.1
  (crate-source "value-bag" "1.11.1"
                "1i89r6z2dxydybh43mxpc2qx3y943f35sm42c06v2gkliadf4g4l"))

(define rust-version-check-0.9.5
  (crate-source "version_check" "0.9.5"
                "0nhhi4i5x89gm911azqbn7avs9mdacw2i3vcz3cnmz3mv4rqz4hb"))

(define rust-waitgroup-0.1.2
  (crate-source "waitgroup" "0.1.2"
                "14ljyy4b921y41qbghgg75745g7l883d3y8009n7wil3lw001xfi"))

(define rust-walkdir-2.5.0
  (crate-source "walkdir" "2.5.0"
                "0jsy7a710qv8gld5957ybrnc07gavppp963gs32xk4ag8130jy99"))

(define rust-wasi-0.11.1+wasi-snapshot-preview1
  (crate-source "wasi" "0.11.1+wasi-snapshot-preview1"
                "0jx49r7nbkbhyfrfyhz0bm4817yrnxgd3jiwwwfv0zl439jyrwyc"))

(define rust-wasi-0.14.3+wasi-0.2.4
  (crate-source "wasi" "0.14.3+wasi-0.2.4"
                "158c0cy561zlncw71hjh1pficw60p1nj7ki8kqm2gpbv0f1swlba"))

(define rust-wasm-bindgen-0.2.100
  (crate-source "wasm-bindgen" "0.2.100"
                "1x8ymcm6yi3i1rwj78myl1agqv2m86i648myy3lc97s9swlqkp0y"))

(define rust-wasm-bindgen-backend-0.2.100
  (crate-source "wasm-bindgen-backend" "0.2.100"
                "1ihbf1hq3y81c4md9lyh6lcwbx6a5j0fw4fygd423g62lm8hc2ig"))

(define rust-wasm-bindgen-futures-0.4.50
  (crate-source "wasm-bindgen-futures" "0.4.50"
                "0q8ymi6i9r3vxly551dhxcyai7nc491mspj0j1wbafxwq074fpam"))

(define rust-wasm-bindgen-macro-0.2.100
  (crate-source "wasm-bindgen-macro" "0.2.100"
                "01xls2dvzh38yj17jgrbiib1d3nyad7k2yw9s0mpklwys333zrkz"))

(define rust-wasm-bindgen-macro-support-0.2.100
  (crate-source "wasm-bindgen-macro-support" "0.2.100"
                "1plm8dh20jg2id0320pbmrlsv6cazfv6b6907z19ys4z1jj7xs4a"))

(define rust-wasm-bindgen-shared-0.2.100
  (crate-source "wasm-bindgen-shared" "0.2.100"
                "0gffxvqgbh9r9xl36gprkfnh3w9gl8wgia6xrin7v11sjcxxf18s"))

(define rust-web-atoms-0.1.3
  (crate-source "web_atoms" "0.1.3"
                "056lg00xm67d2yiyi1fh3x15jpi3idk0acifk7wvsh0jq0fxxzsp"))

(define rust-web-sys-0.3.77
  ;; TODO: Check bundled sources.
  (crate-source "web-sys" "0.3.77"
                "1lnmc1ffbq34qw91nndklqqm75rasaffj2g4f8h1yvqqz4pdvdik"))

(define rust-web-time-1.1.0
  (crate-source "web-time" "1.1.0"
                "1fx05yqx83dhx628wb70fyy10yjfq1jpl20qfqhdkymi13rq0ras"))

(define rust-webbrowser-1.0.5
  (crate-source "webbrowser" "1.0.5"
                "166yrfz20a5qzxq881acw973537v0dq1bi6cwns853l3pb0g7x5a"))

(define rust-webpki-0.22.4
  (crate-source "webpki" "0.22.4"
                "0lwv7jdlcqjjqqhxcrapnyk5bz4lvr12q444b50gzl3krsjswqzd"))

(define rust-webpki-roots-0.22.6
  (crate-source "webpki-roots" "0.22.6"
                "11rd1aj73qzcvdj3x78crm1758sc4wrbc7rh0r8lmhyjsx01xixn"))

(define rust-webpki-roots-1.0.2
  (crate-source "webpki-roots" "1.0.2"
                "1ck1wa1prinrvz3q34c3xp4cpa2f3i4x5npwgj0gpmikmg1q72by"))

(define rust-webrtc-0.13.0
  (crate-source "webrtc" "0.13.0"
                "0yin1s4kh803mi8chqzlla85nrdsaaj90bvpcb40bmlqb4cvgfi4"))

(define rust-webrtc-data-0.11.0
  (crate-source "webrtc-data" "0.11.0"
                "07cxjifh2yz5cgs9c7j8qqpj58i5aj0crw7gcykk79jdhlrbk5sf"))

(define rust-webrtc-dtls-0.12.0
  (crate-source "webrtc-dtls" "0.12.0"
                "073ia5rhrvlfv8xmvc2zq4b7x1sw770lcdjwd59ap44k0kcy9jsw"))

(define rust-webrtc-ice-0.13.0
  (crate-source "webrtc-ice" "0.13.0"
                "15w9cx6vf6zjncy68cd2gmb1vdbgnpqh8kgybfhhkwchszhbslgb"))

(define rust-webrtc-mdns-0.9.0
  (crate-source "webrtc-mdns" "0.9.0"
                "1a3fx9jqgw73rd88h8jxa3x4c9aysc89sl0311i7nfy5b59ci74p"))

(define rust-webrtc-media-0.10.0
  (crate-source "webrtc-media" "0.10.0"
                "041qs68kda209c2ikdc3j5yd7h8bn7b97ahr7s57bnncvq8i4140"))

(define rust-webrtc-sctp-0.12.0
  (crate-source "webrtc-sctp" "0.12.0"
                "1bp5mbrgmdg8995fk7z1vj3vbpszh4psyylh20pivm958h9rqhq7"))

(define rust-webrtc-srtp-0.15.0
  (crate-source "webrtc-srtp" "0.15.0"
                "1fxm4cvlqrrhsq2qvks0l89b00rlnkkkzc56ppzmgc09kgvp7rq1"))

(define rust-webrtc-util-0.11.0
  (crate-source "webrtc-util" "0.11.0"
                "0n6frp0320la010d354m9dv1zymsabr7rq4s2s02yxkdpq6v3gv4"))

(define rust-winapi-0.3.9
  (crate-source "winapi" "0.3.9"
                "06gl025x418lchw1wxj64ycr7gha83m44cjr5sarhynd9xkrm0sw"))

(define rust-winapi-i686-pc-windows-gnu-0.4.0
  (crate-source "winapi-i686-pc-windows-gnu" "0.4.0"
                "1dmpa6mvcvzz16zg6d5vrfy4bxgg541wxrcip7cnshi06v38ffxc"))

(define rust-winapi-util-0.1.10
  (crate-source "winapi-util" "0.1.10"
                "08hb8rj3aq9lcrfmliqs4l7v9zh6srbcn0376yn0pndkf5qvyy09"))

(define rust-winapi-x86-64-pc-windows-gnu-0.4.0
  (crate-source "winapi-x86_64-pc-windows-gnu" "0.4.0"
                "0gqq64czqb64kskjryj8isp62m2sgvx25yyj3kpc2myh85w24bki"))

(define rust-windows-0.61.3
  (crate-source "windows" "0.61.3"
                "14v8dln7i4ccskd8danzri22bkjkbmgzh284j3vaxhd4cykx7awv"))

(define rust-windows-aarch64-gnullvm-0.42.2
  (crate-source "windows_aarch64_gnullvm" "0.42.2"
                "1y4q0qmvl0lvp7syxvfykafvmwal5hrjb4fmv04bqs0bawc52yjr"))

(define rust-windows-aarch64-gnullvm-0.52.6
  (crate-source "windows_aarch64_gnullvm" "0.52.6"
                "1lrcq38cr2arvmz19v32qaggvj8bh1640mdm9c2fr877h0hn591j"))

(define rust-windows-aarch64-gnullvm-0.53.0
  (crate-source "windows_aarch64_gnullvm" "0.53.0"
                "0r77pbpbcf8bq4yfwpz2hpq3vns8m0yacpvs2i5cn6fx1pwxbf46"))

(define rust-windows-aarch64-msvc-0.42.2
  (crate-source "windows_aarch64_msvc" "0.42.2"
                "0hsdikjl5sa1fva5qskpwlxzpc5q9l909fpl1w6yy1hglrj8i3p0"))

(define rust-windows-aarch64-msvc-0.52.6
  (crate-source "windows_aarch64_msvc" "0.52.6"
                "0sfl0nysnz32yyfh773hpi49b1q700ah6y7sacmjbqjjn5xjmv09"))

(define rust-windows-aarch64-msvc-0.53.0
  (crate-source "windows_aarch64_msvc" "0.53.0"
                "0v766yqw51pzxxwp203yqy39ijgjamp54hhdbsyqq6x1c8gilrf7"))

(define rust-windows-collections-0.2.0
  (crate-source "windows-collections" "0.2.0"
                "1s65anr609qvsjga7w971p6iq964h87670dkfqfypnfgwnswxviv"))

(define rust-windows-core-0.61.2
  (crate-source "windows-core" "0.61.2"
                "1qsa3iw14wk4ngfl7ipcvdf9xyq456ms7cx2i9iwf406p7fx7zf0"))

(define rust-windows-future-0.2.1
  (crate-source "windows-future" "0.2.1"
                "13mdzcdn51ckpzp3frb8glnmkyjr1c30ym9wnzj9zc97hkll2spw"))

(define rust-windows-i686-gnu-0.42.2
  (crate-source "windows_i686_gnu" "0.42.2"
                "0kx866dfrby88lqs9v1vgmrkk1z6af9lhaghh5maj7d4imyr47f6"))

(define rust-windows-i686-gnu-0.52.6
  (crate-source "windows_i686_gnu" "0.52.6"
                "02zspglbykh1jh9pi7gn8g1f97jh1rrccni9ivmrfbl0mgamm6wf"))

(define rust-windows-i686-gnu-0.53.0
  (crate-source "windows_i686_gnu" "0.53.0"
                "1hvjc8nv95sx5vdd79fivn8bpm7i517dqyf4yvsqgwrmkmjngp61"))

(define rust-windows-i686-gnullvm-0.52.6
  (crate-source "windows_i686_gnullvm" "0.52.6"
                "0rpdx1537mw6slcpqa0rm3qixmsb79nbhqy5fsm3q2q9ik9m5vhf"))

(define rust-windows-i686-gnullvm-0.53.0
  (crate-source "windows_i686_gnullvm" "0.53.0"
                "04df1in2k91qyf1wzizvh560bvyzq20yf68k8xa66vdzxnywrrlw"))

(define rust-windows-i686-msvc-0.42.2
  (crate-source "windows_i686_msvc" "0.42.2"
                "0q0h9m2aq1pygc199pa5jgc952qhcnf0zn688454i7v4xjv41n24"))

(define rust-windows-i686-msvc-0.52.6
  (crate-source "windows_i686_msvc" "0.52.6"
                "0rkcqmp4zzmfvrrrx01260q3xkpzi6fzi2x2pgdcdry50ny4h294"))

(define rust-windows-i686-msvc-0.53.0
  (crate-source "windows_i686_msvc" "0.53.0"
                "0pcvb25fkvqnp91z25qr5x61wyya12lx8p7nsa137cbb82ayw7sq"))

(define rust-windows-implement-0.60.0
  (crate-source "windows-implement" "0.60.0"
                "0dm88k3hlaax85xkls4gf597ar4z8m5vzjjagzk910ph7b8xszx4"))

(define rust-windows-interface-0.59.1
  (crate-source "windows-interface" "0.59.1"
                "1a4zr8740gyzzhq02xgl6vx8l669jwfby57xgf0zmkcdkyv134mx"))

(define rust-windows-link-0.1.3
  (crate-source "windows-link" "0.1.3"
                "12kr1p46dbhpijr4zbwr2spfgq8i8c5x55mvvfmyl96m01cx4sjy"))

(define rust-windows-numerics-0.2.0
  (crate-source "windows-numerics" "0.2.0"
                "1cf2j8nbqf0hqqa7chnyid91wxsl2m131kn0vl3mqk3c0rlayl4i"))

(define rust-windows-result-0.3.4
  (crate-source "windows-result" "0.3.4"
                "1il60l6idrc6hqsij0cal0mgva6n3w6gq4ziban8wv6c6b9jpx2n"))

(define rust-windows-strings-0.4.2
  (crate-source "windows-strings" "0.4.2"
                "0mrv3plibkla4v5kaakc2rfksdd0b14plcmidhbkcfqc78zwkrjn"))

(define rust-windows-sys-0.45.0
  ;; TODO: Check bundled sources.
  (crate-source "windows-sys" "0.45.0"
                "1l36bcqm4g89pknfp8r9rl1w4bn017q6a8qlx8viv0xjxzjkna3m"))

(define rust-windows-sys-0.52.0
  ;; TODO: Check bundled sources.
  (crate-source "windows-sys" "0.52.0"
                "0gd3v4ji88490zgb6b5mq5zgbvwv7zx1ibn8v3x83rwcdbryaar8"))

(define rust-windows-sys-0.59.0
  ;; TODO: Check bundled sources.
  (crate-source "windows-sys" "0.59.0"
                "0fw5672ziw8b3zpmnbp9pdv1famk74f1l9fcbc3zsrzdg56vqf0y"))

(define rust-windows-sys-0.60.2
  ;; TODO: Check bundled sources.
  (crate-source "windows-sys" "0.60.2"
                "1jrbc615ihqnhjhxplr2kw7rasrskv9wj3lr80hgfd42sbj01xgj"))

(define rust-windows-targets-0.42.2
  (crate-source "windows-targets" "0.42.2"
                "0wfhnib2fisxlx8c507dbmh97kgij4r6kcxdi0f9nk6l1k080lcf"))

(define rust-windows-targets-0.52.6
  (crate-source "windows-targets" "0.52.6"
                "0wwrx625nwlfp7k93r2rra568gad1mwd888h1jwnl0vfg5r4ywlv"))

(define rust-windows-targets-0.53.3
  (crate-source "windows-targets" "0.53.3"
                "14fwwm136dhs3i1impqrrip7nvkra3bdxa4nqkblj604qhqn1znm"))

(define rust-windows-threading-0.1.0
  (crate-source "windows-threading" "0.1.0"
                "19jpn37zpjj2q7pn07dpq0ay300w65qx7wdp13wbp8qf5snn6r5n"))

(define rust-windows-x86-64-gnu-0.42.2
  (crate-source "windows_x86_64_gnu" "0.42.2"
                "0dnbf2xnp3xrvy8v9mgs3var4zq9v9yh9kv79035rdgyp2w15scd"))

(define rust-windows-x86-64-gnu-0.52.6
  (crate-source "windows_x86_64_gnu" "0.52.6"
                "0y0sifqcb56a56mvn7xjgs8g43p33mfqkd8wj1yhrgxzma05qyhl"))

(define rust-windows-x86-64-gnu-0.53.0
  (crate-source "windows_x86_64_gnu" "0.53.0"
                "1flh84xkssn1n6m1riddipydcksp2pdl45vdf70jygx3ksnbam9f"))

(define rust-windows-x86-64-gnullvm-0.42.2
  (crate-source "windows_x86_64_gnullvm" "0.42.2"
                "18wl9r8qbsl475j39zvawlidp1bsbinliwfymr43fibdld31pm16"))

(define rust-windows-x86-64-gnullvm-0.52.6
  (crate-source "windows_x86_64_gnullvm" "0.52.6"
                "03gda7zjx1qh8k9nnlgb7m3w3s1xkysg55hkd1wjch8pqhyv5m94"))

(define rust-windows-x86-64-gnullvm-0.53.0
  (crate-source "windows_x86_64_gnullvm" "0.53.0"
                "0mvc8119xpbi3q2m6mrjcdzl6afx4wffacp13v76g4jrs1fh6vha"))

(define rust-windows-x86-64-msvc-0.42.2
  (crate-source "windows_x86_64_msvc" "0.42.2"
                "1w5r0q0yzx827d10dpjza2ww0j8iajqhmb54s735hhaj66imvv4s"))

(define rust-windows-x86-64-msvc-0.52.6
  (crate-source "windows_x86_64_msvc" "0.52.6"
                "1v7rb5cibyzx8vak29pdrk8nx9hycsjs4w0jgms08qk49jl6v7sq"))

(define rust-windows-x86-64-msvc-0.53.0
  (crate-source "windows_x86_64_msvc" "0.53.0"
                "11h4i28hq0zlnjcaqi2xdxr7ibnpa8djfggch9rki1zzb8qi8517"))

(define rust-winnow-0.5.40
  (crate-source "winnow" "0.5.40"
                "0xk8maai7gyxda673mmw3pj1hdizy5fpi7287vaywykkk19sk4zm"))

(define rust-winnow-0.7.13
  (crate-source "winnow" "0.7.13"
                "1krrjc1wj2vx0r57m9nwnlc1zrhga3fq41d8w9hysvvqb5mj7811"))

(define rust-wit-bindgen-0.45.0
  (crate-source "wit-bindgen" "0.45.0"
                "053q28k1hn9qgm0l05gr9751d8q34zcz6lbzviwxiqxs3n1q68h5"))

(define rust-writeable-0.6.1
  (crate-source "writeable" "0.6.1"
                "1fx29zncvbrqzgz7li88vzdm8zvgwgwy2r9bnjqxya09pfwi0bza"))

(define rust-ws-stream-wasm-0.7.5
  (crate-source "ws_stream_wasm" "0.7.5"
                "1g3a1kkkz06kv72kbrslwxpq8f2v25hf6gj02qzyh8mdmha305vc"))

(define rust-wtf8-0.1.0
  (crate-source "wtf8" "0.1.0"
                "01ia1dkq2zcb8mnj5h1r3vgz7g5njh4pv8fkxxv27x9q5i4yh6n0"))

(define rust-x25519-dalek-2.0.1
  (crate-source "x25519-dalek" "2.0.1"
                "0xyjgqpsa0q6pprakdp58q1hy45rf8wnqqscgzx0gyw13hr6ir67"))

(define rust-x509-parser-0.16.0
  (crate-source "x509-parser" "0.16.0"
                "0s8zyl6fafkzpylcpcn08bmcmrzzcb6gfjx2h8zny3bh60pidg7w"))

(define rust-yansi-1.0.1
  (crate-source "yansi" "1.0.1"
                "0jdh55jyv0dpd38ij4qh60zglbw9aa8wafqai6m0wa7xaxk3mrfg"))

(define rust-yasna-0.5.2
  (crate-source "yasna" "0.5.2"
                "1ka4ixrplnrfqyl1kymdj8cwpdp2k0kdr73b57hilcn1kiab6yz1"))

(define rust-yoke-0.8.0
  (crate-source "yoke" "0.8.0"
                "1k4mfr48vgi7wh066y11b7v1ilakghlnlhw9snzz8vi2p00vnhaz"))

(define rust-yoke-derive-0.8.0
  (crate-source "yoke-derive" "0.8.0"
                "1dha5jrjz9jaq8kmxq1aag86b98zbnm9lyjrihy5sv716sbkrniq"))

(define rust-zerocopy-0.8.26
  (crate-source "zerocopy" "0.8.26"
                "0bvsj0qzq26zc6nlrm3z10ihvjspyngs7n0jw1fz031i7h6xsf8h"))

(define rust-zerocopy-derive-0.8.26
  (crate-source "zerocopy-derive" "0.8.26"
                "10aiywi5qkha0mpsnb1zjwi44wl2rhdncaf3ykbp4i9nqm65pkwy"))

(define rust-zerofrom-0.1.6
  (crate-source "zerofrom" "0.1.6"
                "19dyky67zkjichsb7ykhv0aqws3q0jfvzww76l66c19y6gh45k2h"))

(define rust-zerofrom-derive-0.1.6
  (crate-source "zerofrom-derive" "0.1.6"
                "00l5niw7c1b0lf1vhvajpjmcnbdp2vn96jg4nmkhq2db0rp5s7np"))

(define rust-zeroize-1.8.1
  (crate-source "zeroize" "1.8.1"
                "1pjdrmjwmszpxfd7r860jx54cyk94qk59x13sc307cvr5256glyf"))

(define rust-zeroize-derive-1.4.2
  (crate-source "zeroize_derive" "1.4.2"
                "0sczjlqjdmrp3wn62g7mw6p438c9j4jgp2f9zamd56991mdycdnf"))

(define rust-zerotrie-0.2.2
  (crate-source "zerotrie" "0.2.2"
                "15gmka7vw5k0d24s0vxgymr2j6zn2iwl12wpmpnpjgsqg3abpw1n"))

(define rust-zerovec-0.11.4
  (crate-source "zerovec" "0.11.4"
                "0fz7j1ns8d86m2fqg2a4bzi5gnh5892bxv4kcr9apwc6a3ajpap7"))

(define rust-zerovec-derive-0.11.1
  (crate-source "zerovec-derive" "0.11.1"
                "13zms8hj7vzpfswypwggyfr4ckmyc7v3di49pmj8r1qcz9z275jv"))

(define ssss-separator 'end-of-crates)


;;;
;;; Cargo inputs.
;;;

(define-cargo-inputs lookup-cargo-inputs
                     (ctoolbox =>
                                   (list rust-addr2line-0.24.2
                                    rust-adler2-2.0.1
                                    rust-adler32-1.2.0
                                    rust-aead-0.5.2
                                    rust-aes-0.8.4
                                    rust-aes-gcm-0.10.3
                                    rust-age-0.11.1
                                    rust-age-core-0.11.0
                                    rust-aho-corasick-1.1.3
                                    rust-alloc-no-stdlib-2.0.4
                                    rust-alloc-stdlib-0.2.2
                                    rust-ammonia-4.1.1
                                    rust-android-tzdata-0.1.1
                                    rust-android-system-properties-0.1.5
                                    rust-anstream-0.6.20
                                    rust-anstyle-1.0.11
                                    rust-anstyle-parse-0.2.7
                                    rust-anstyle-query-1.1.4
                                    rust-anstyle-wincon-3.0.10
                                    rust-anyhow-1.0.99
                                    rust-arc-swap-1.7.1
                                    rust-ascii-1.1.0
                                    rust-asn1-rs-0.6.2
                                    rust-asn1-rs-derive-0.5.1
                                    rust-asn1-rs-impl-0.2.0
                                    rust-async-channel-1.9.0
                                    rust-async-channel-2.5.0
                                    rust-async-compat-0.2.5
                                    rust-async-executor-1.13.3
                                    rust-async-global-executor-2.4.1
                                    rust-async-io-2.5.0
                                    rust-async-lock-3.4.1
                                    rust-async-std-1.13.2
                                    rust-async-task-4.7.1
                                    rust-async-tls-0.13.0
                                    rust-async-trait-0.1.89
                                    rust-async-tungstenite-0.29.1
                                    rust-async-io-stream-0.3.3
                                    rust-atomic-waker-1.1.2
                                    rust-autocfg-1.5.0
                                    rust-backtrace-0.3.75
                                    rust-base16ct-0.2.0
                                    rust-base64-0.21.7
                                    rust-base64-0.22.1
                                    rust-base64ct-1.8.0
                                    rust-basic-toml-0.1.10
                                    rust-bcrypt-0.15.1
                                    rust-bech32-0.9.1
                                    rust-bincode-1.3.3
                                    rust-bitflags-1.3.2
                                    rust-bitflags-2.9.3
                                    rust-block-buffer-0.10.4
                                    rust-block-padding-0.3.3
                                    rust-blocking-1.6.2
                                    rust-blowfish-0.9.1
                                    rust-brotli-3.5.0
                                    rust-brotli-decompressor-2.5.1
                                    rust-buffer-redux-1.0.2
                                    rust-bumpalo-3.19.0
                                    rust-byteorder-1.5.0
                                    rust-bytes-1.10.1
                                    rust-cbc-0.1.2
                                    rust-cc-1.2.34
                                    rust-ccm-0.5.0
                                    rust-cesu8-1.1.0
                                    rust-cfg-if-1.0.3
                                    rust-cfg-aliases-0.2.1
                                    rust-chacha20-0.9.1
                                    rust-chacha20poly1305-0.10.1
                                    rust-chrono-0.4.41
                                    rust-chunked-transfer-1.5.0
                                    rust-cipher-0.4.4
                                    rust-colorchoice-1.0.4
                                    rust-combine-4.6.7
                                    rust-concurrent-queue-2.5.0
                                    rust-configparser-3.1.0
                                    rust-const-default-1.0.0
                                    rust-const-default-derive-0.2.0
                                    rust-const-oid-0.9.6
                                    rust-cookie-factory-0.3.3
                                    rust-core-foundation-0.10.1
                                    rust-core-foundation-sys-0.8.7
                                    rust-cpufeatures-0.2.17
                                    rust-crc-3.3.0
                                    rust-crc-catalog-2.4.0
                                    rust-crc32fast-1.5.0
                                    rust-crossbeam-utils-0.8.21
                                    rust-crypto-bigint-0.5.5
                                    rust-crypto-common-0.1.6
                                    rust-cssparser-0.35.0
                                    rust-cssparser-macros-0.6.1
                                    rust-csv-1.3.1
                                    rust-csv-core-0.1.12
                                    rust-ctr-0.9.2
                                    rust-ctrlc-3.4.7
                                    rust-curve25519-dalek-4.1.3
                                    rust-curve25519-dalek-derive-0.1.1
                                    rust-darling-0.20.11
                                    rust-darling-core-0.20.11
                                    rust-darling-macro-0.20.11
                                    rust-data-encoding-2.9.0
                                    rust-deflate-1.0.0
                                    rust-der-0.7.10
                                    rust-der-parser-9.0.0
                                    rust-deranged-0.4.0
                                    rust-derive-builder-0.20.2
                                    rust-derive-builder-core-0.20.2
                                    rust-derive-builder-macro-0.20.2
                                    rust-derive-more-2.0.1
                                    rust-derive-more-impl-2.0.1
                                    rust-digest-0.10.7
                                    rust-directories-6.0.0
                                    rust-dirs-sys-0.5.0
                                    rust-displaydoc-0.2.5
                                    rust-dtoa-1.0.10
                                    rust-dtoa-short-0.3.5
                                    rust-dyn-clone-1.0.20
                                    rust-ecdsa-0.16.9
                                    rust-elliptic-curve-0.13.8
                                    rust-encre-css-0.17.1
                                    rust-env-filter-0.1.3
                                    rust-env-logger-0.11.8
                                    rust-equivalent-1.0.2
                                    rust-errno-0.3.13
                                    rust-event-listener-2.5.3
                                    rust-event-listener-5.4.1
                                    rust-event-listener-strategy-0.5.4
                                    rust-fastrand-2.3.0
                                    rust-fern-0.7.1
                                    rust-ff-0.13.1
                                    rust-fiat-crypto-0.2.9
                                    rust-filetime-0.2.26
                                    rust-find-crate-0.6.3
                                    rust-flate2-1.1.2
                                    rust-fluent-0.16.1
                                    rust-fluent-bundle-0.15.3
                                    rust-fluent-langneg-0.13.0
                                    rust-fluent-syntax-0.11.1
                                    rust-fnv-1.0.7
                                    rust-foldhash-0.1.5
                                    rust-form-urlencoded-1.2.2
                                    rust-fs2-0.4.3
                                    rust-futf-0.1.5
                                    rust-futures-0.3.31
                                    rust-futures-channel-0.3.31
                                    rust-futures-core-0.3.31
                                    rust-futures-executor-0.3.31
                                    rust-futures-io-0.3.31
                                    rust-futures-lite-2.6.1
                                    rust-futures-macro-0.3.31
                                    rust-futures-sink-0.3.31
                                    rust-futures-task-0.3.31
                                    rust-futures-timer-3.0.3
                                    rust-futures-util-0.3.31
                                    rust-generic-array-0.14.7
                                    rust-getrandom-0.2.16
                                    rust-getrandom-0.3.3
                                    rust-ghash-0.5.1
                                    rust-gimli-0.31.1
                                    rust-glob-0.3.3
                                    rust-gloo-timers-0.2.6
                                    rust-gloo-timers-0.3.0
                                    rust-group-0.13.0
                                    rust-gzip-header-1.0.0
                                    rust-handlebars-6.3.2
                                    rust-hashbrown-0.15.5
                                    rust-hermit-abi-0.5.2
                                    rust-hex-0.4.3
                                    rust-hifijson-0.2.3
                                    rust-hkdf-0.12.4
                                    rust-hmac-0.12.1
                                    rust-html5ever-0.35.0
                                    rust-http-1.3.1
                                    rust-httparse-1.10.1
                                    rust-httpdate-1.0.3
                                    rust-humantime-2.2.0
                                    rust-i18n-config-0.4.8
                                    rust-i18n-embed-0.15.4
                                    rust-i18n-embed-fl-0.9.4
                                    rust-i18n-embed-impl-0.8.4
                                    rust-iana-time-zone-0.1.63
                                    rust-iana-time-zone-haiku-0.1.2
                                    rust-icu-collections-2.0.0
                                    rust-icu-locale-core-2.0.0
                                    rust-icu-normalizer-2.0.0
                                    rust-icu-normalizer-data-2.0.0
                                    rust-icu-properties-2.0.1
                                    rust-icu-properties-data-2.0.1
                                    rust-icu-provider-2.0.0
                                    rust-ident-case-1.0.1
                                    rust-idna-1.1.0
                                    rust-idna-adapter-1.2.1
                                    rust-include-dir-0.7.4
                                    rust-include-dir-macros-0.7.4
                                    rust-indexmap-2.11.0
                                    rust-inout-0.1.4
                                    rust-interceptor-0.14.0
                                    rust-intl-memoizer-0.5.3
                                    rust-intl-pluralrules-7.0.2
                                    rust-io-uring-0.7.10
                                    rust-io-tee-0.1.1
                                    rust-ipnet-2.11.0
                                    rust-is-terminal-polyfill-1.70.1
                                    rust-itoa-1.0.15
                                    rust-jaq-core-2.2.1
                                    rust-jaq-json-1.1.3
                                    rust-jaq-std-2.1.2
                                    rust-jni-0.21.1
                                    rust-jni-sys-0.3.0
                                    rust-js-sys-0.3.77
                                    rust-kv-log-macro-1.0.7
                                    rust-lazy-static-1.5.0
                                    rust-libc-0.2.175
                                    rust-libm-0.2.15
                                    rust-libredox-0.1.9
                                    rust-linux-raw-sys-0.9.4
                                    rust-litemap-0.8.0
                                    rust-lock-api-0.4.13
                                    rust-log-0.4.27
                                    rust-mac-0.1.1
                                    rust-maplit-1.0.2
                                    rust-markup5ever-0.35.0
                                    rust-match-token-0.35.0
                                    rust-matchbox-protocol-0.12.0
                                    rust-matchbox-socket-0.12.0
                                    rust-matchers-0.2.0
                                    rust-md-5-0.10.6
                                    rust-md5-0.7.0
                                    rust-memchr-2.7.5
                                    rust-memoffset-0.7.1
                                    rust-mime-0.3.17
                                    rust-mime-guess-2.0.5
                                    rust-minimal-lexical-0.2.1
                                    rust-miniz-oxide-0.8.9
                                    rust-mio-1.0.4
                                    rust-ndk-context-0.1.1
                                    rust-new-debug-unreachable-1.0.6
                                    rust-nix-0.26.4
                                    rust-nix-0.30.1
                                    rust-nom-7.1.3
                                    rust-ntapi-0.4.1
                                    rust-nu-ansi-term-0.50.1
                                    rust-num-bigint-0.4.6
                                    rust-num-conv-0.1.0
                                    rust-num-integer-0.1.46
                                    rust-num-modular-0.6.1
                                    rust-num-order-1.2.0
                                    rust-num-traits-0.2.19
                                    rust-num-cpus-1.17.0
                                    rust-num-threads-0.1.7
                                    rust-objc2-0.6.2
                                    rust-objc2-core-foundation-0.3.1
                                    rust-objc2-encode-4.1.0
                                    rust-objc2-foundation-0.3.1
                                    rust-objc2-io-kit-0.3.1
                                    rust-object-0.36.7
                                    rust-oid-registry-0.7.1
                                    rust-once-cell-1.21.3
                                    rust-once-cell-polyfill-1.70.1
                                    rust-opaque-debug-0.3.1
                                    rust-option-ext-0.2.0
                                    rust-p256-0.13.2
                                    rust-p384-0.13.1
                                    rust-parking-2.2.1
                                    rust-parking-lot-0.12.4
                                    rust-parking-lot-core-0.9.11
                                    rust-passwords-3.1.16
                                    rust-pbkdf2-0.12.2
                                    rust-pem-3.0.5
                                    rust-pem-rfc7468-0.7.0
                                    rust-percent-encoding-2.3.2
                                    rust-pest-2.8.1
                                    rust-pest-derive-2.8.1
                                    rust-pest-generator-2.8.1
                                    rust-pest-meta-2.8.1
                                    rust-pharos-0.5.3
                                    rust-phf-0.11.3
                                    rust-phf-0.12.1
                                    rust-phf-codegen-0.11.3
                                    rust-phf-generator-0.11.3
                                    rust-phf-generator-0.12.1
                                    rust-phf-macros-0.11.3
                                    rust-phf-macros-0.12.1
                                    rust-phf-shared-0.11.3
                                    rust-phf-shared-0.12.1
                                    rust-pin-project-1.1.10
                                    rust-pin-project-internal-1.1.10
                                    rust-pin-project-lite-0.2.16
                                    rust-pin-utils-0.1.0
                                    rust-piper-0.2.4
                                    rust-pkcs8-0.10.2
                                    rust-polling-3.10.0
                                    rust-poly1305-0.8.0
                                    rust-polyval-0.6.2
                                    rust-portable-atomic-1.11.1
                                    rust-portpicker-0.1.1
                                    rust-potential-utf-0.1.3
                                    rust-powerfmt-0.2.0
                                    rust-ppv-lite86-0.2.21
                                    rust-precomputed-hash-0.1.1
                                    rust-primeorder-0.13.6
                                    rust-proc-macro-crate-1.3.1
                                    rust-proc-macro-error-attr2-2.0.0
                                    rust-proc-macro-error2-2.0.1
                                    rust-proc-macro-hack-0.5.20+deprecated
                                    rust-proc-macro2-1.0.101
                                    rust-quick-error-1.2.3
                                    rust-quote-1.0.40
                                    rust-r-efi-5.3.0
                                    rust-rand-0.8.5
                                    rust-rand-0.9.2
                                    rust-rand-chacha-0.3.1
                                    rust-rand-chacha-0.9.0
                                    rust-rand-core-0.6.4
                                    rust-rand-core-0.9.3
                                    rust-random-number-0.1.9
                                    rust-random-number-macro-impl-0.1.8
                                    rust-random-pick-1.2.16
                                    rust-rcgen-0.13.2
                                    rust-redb-3.0.1
                                    rust-redox-syscall-0.5.17
                                    rust-redox-users-0.5.2
                                    rust-regex-1.11.2
                                    rust-regex-automata-0.4.10
                                    rust-regex-lite-0.1.7
                                    rust-regex-syntax-0.8.6
                                    rust-rfc6979-0.4.0
                                    rust-ring-0.17.14
                                    rust-rouille-3.6.2.31f772c
                                    rust-rouille-multipart-0.18.0.31f772c
                                    rust-rtcp-0.13.0
                                    rust-rtp-0.13.0
                                    rust-rust-embed-8.7.2
                                    rust-rust-embed-impl-8.7.2
                                    rust-rust-embed-utils-8.7.2
                                    rust-rustc-demangle-0.1.26
                                    rust-rustc-hash-1.1.0
                                    rust-rustc-hash-2.1.1
                                    rust-rustc-version-0.4.1
                                    rust-rusticata-macros-4.1.0
                                    rust-rustix-1.0.8
                                    rust-rustls-0.21.12
                                    rust-rustls-0.23.31
                                    rust-rustls-pemfile-1.0.4
                                    rust-rustls-pemfile-2.2.0
                                    rust-rustls-pki-types-1.12.0
                                    rust-rustls-webpki-0.101.7
                                    rust-rustls-webpki-0.103.4
                                    rust-rustversion-1.0.22
                                    rust-ryu-1.0.20
                                    rust-safemem-0.3.3
                                    rust-salsa20-0.10.2
                                    rust-same-file-1.0.6
                                    rust-scopeguard-1.2.0
                                    rust-scrypt-0.11.0
                                    rust-sct-0.7.1
                                    rust-sdp-0.8.0
                                    rust-sec1-0.7.3
                                    rust-secrecy-0.10.3
                                    rust-self-cell-0.10.3
                                    rust-self-cell-1.2.0
                                    rust-semver-1.0.26
                                    rust-send-wrapper-0.4.0
                                    rust-send-wrapper-0.6.0
                                    rust-serde-1.0.219
                                    rust-serde-wasm-bindgen-0.6.5
                                    rust-serde-derive-1.0.219
                                    rust-serde-json-1.0.143
                                    rust-serde-spanned-1.0.0
                                    rust-sha1-0.10.6
                                    rust-sha1-smol-1.0.1
                                    rust-sha2-0.10.9
                                    rust-sharded-slab-0.1.7
                                    rust-shlex-1.3.0
                                    rust-signal-hook-registry-1.4.6
                                    rust-signature-2.2.0
                                    rust-siphasher-1.0.1
                                    rust-slab-0.4.11
                                    rust-smallvec-1.15.1
                                    rust-smol-str-0.2.2
                                    rust-socket2-0.5.10
                                    rust-socket2-0.6.0
                                    rust-spki-0.7.3
                                    rust-stable-deref-trait-1.2.0
                                    rust-string-cache-0.8.9
                                    rust-string-cache-codegen-0.5.4
                                    rust-strsim-0.11.1
                                    rust-stun-0.8.0
                                    rust-substring-1.4.5
                                    rust-subtle-2.6.1
                                    rust-syn-1.0.109
                                    rust-syn-2.0.106
                                    rust-synstructure-0.13.2
                                    rust-sysinfo-0.37.0
                                    rust-tempfile-3.21.0
                                    rust-tendril-0.4.3
                                    rust-test-log-0.2.18
                                    rust-test-log-macros-0.2.18
                                    rust-text-io-0.1.13
                                    rust-thiserror-1.0.69
                                    rust-thiserror-2.0.16
                                    rust-thiserror-impl-1.0.69
                                    rust-thiserror-impl-2.0.16
                                    rust-thread-local-1.1.9
                                    rust-threadpool-1.8.1
                                    rust-time-0.3.41
                                    rust-time-core-0.1.4
                                    rust-time-macros-0.2.22
                                    rust-tinystr-0.8.1
                                    rust-tokio-1.47.1
                                    rust-tokio-macros-2.5.0
                                    rust-tokio-util-0.7.16
                                    rust-toml-0.5.11
                                    rust-toml-0.9.5
                                    rust-toml-datetime-0.6.11
                                    rust-toml-datetime-0.7.0
                                    rust-toml-edit-0.19.15
                                    rust-toml-parser-1.0.2
                                    rust-toml-writer-1.0.2
                                    rust-tracing-0.1.41
                                    rust-tracing-core-0.1.34
                                    rust-tracing-log-0.2.0
                                    rust-tracing-subscriber-0.3.20
                                    rust-tungstenite-0.26.2
                                    rust-turn-0.10.0
                                    rust-twoway-0.1.8
                                    rust-type-map-0.5.1
                                    rust-typed-arena-2.0.2
                                    rust-typenum-1.18.0
                                    rust-ucd-trie-0.1.7
                                    rust-unic-langid-0.9.6
                                    rust-unic-langid-impl-0.9.6
                                    rust-unicase-2.8.1
                                    rust-unicode-ident-1.0.18
                                    rust-unicode-segmentation-1.12.0
                                    rust-unicode-xid-0.2.6
                                    rust-universal-hash-0.5.1
                                    rust-untrusted-0.9.0
                                    rust-ureq-3.1.0
                                    rust-ureq-proto-0.5.0
                                    rust-url-2.5.7
                                    rust-urlencoding-2.1.3
                                    rust-utf-8-0.7.6
                                    rust-utf8-iter-1.0.4
                                    rust-utf8parse-0.2.2
                                    rust-uuid-1.18.0
                                    rust-uuid-macro-internal-1.18.0
                                    rust-valuable-0.1.1
                                    rust-value-bag-1.11.1
                                    rust-version-check-0.9.5
                                    rust-waitgroup-0.1.2
                                    rust-walkdir-2.5.0
                                    rust-wasi-0.11.1+wasi-snapshot-preview1
                                    rust-wasi-0.14.3+wasi-0.2.4
                                    rust-wasm-bindgen-0.2.100
                                    rust-wasm-bindgen-backend-0.2.100
                                    rust-wasm-bindgen-futures-0.4.50
                                    rust-wasm-bindgen-macro-0.2.100
                                    rust-wasm-bindgen-macro-support-0.2.100
                                    rust-wasm-bindgen-shared-0.2.100
                                    rust-web-sys-0.3.77
                                    rust-web-time-1.1.0
                                    rust-web-atoms-0.1.3
                                    rust-webbrowser-1.0.5
                                    rust-webpki-0.22.4
                                    rust-webpki-roots-0.22.6
                                    rust-webpki-roots-1.0.2
                                    rust-webrtc-0.13.0
                                    rust-webrtc-data-0.11.0
                                    rust-webrtc-dtls-0.12.0
                                    rust-webrtc-ice-0.13.0
                                    rust-webrtc-mdns-0.9.0
                                    rust-webrtc-media-0.10.0
                                    rust-webrtc-sctp-0.12.0
                                    rust-webrtc-srtp-0.15.0
                                    rust-webrtc-util-0.11.0
                                    rust-winapi-0.3.9
                                    rust-winapi-i686-pc-windows-gnu-0.4.0
                                    rust-winapi-util-0.1.10
                                    rust-winapi-x86-64-pc-windows-gnu-0.4.0
                                    rust-windows-0.61.3
                                    rust-windows-collections-0.2.0
                                    rust-windows-core-0.61.2
                                    rust-windows-future-0.2.1
                                    rust-windows-implement-0.60.0
                                    rust-windows-interface-0.59.1
                                    rust-windows-link-0.1.3
                                    rust-windows-numerics-0.2.0
                                    rust-windows-result-0.3.4
                                    rust-windows-strings-0.4.2
                                    rust-windows-sys-0.45.0
                                    rust-windows-sys-0.52.0
                                    rust-windows-sys-0.59.0
                                    rust-windows-sys-0.60.2
                                    rust-windows-targets-0.42.2
                                    rust-windows-targets-0.52.6
                                    rust-windows-targets-0.53.3
                                    rust-windows-threading-0.1.0
                                    rust-windows-aarch64-gnullvm-0.42.2
                                    rust-windows-aarch64-gnullvm-0.52.6
                                    rust-windows-aarch64-gnullvm-0.53.0
                                    rust-windows-aarch64-msvc-0.42.2
                                    rust-windows-aarch64-msvc-0.52.6
                                    rust-windows-aarch64-msvc-0.53.0
                                    rust-windows-i686-gnu-0.42.2
                                    rust-windows-i686-gnu-0.52.6
                                    rust-windows-i686-gnu-0.53.0
                                    rust-windows-i686-gnullvm-0.52.6
                                    rust-windows-i686-gnullvm-0.53.0
                                    rust-windows-i686-msvc-0.42.2
                                    rust-windows-i686-msvc-0.52.6
                                    rust-windows-i686-msvc-0.53.0
                                    rust-windows-x86-64-gnu-0.42.2
                                    rust-windows-x86-64-gnu-0.52.6
                                    rust-windows-x86-64-gnu-0.53.0
                                    rust-windows-x86-64-gnullvm-0.42.2
                                    rust-windows-x86-64-gnullvm-0.52.6
                                    rust-windows-x86-64-gnullvm-0.53.0
                                    rust-windows-x86-64-msvc-0.42.2
                                    rust-windows-x86-64-msvc-0.52.6
                                    rust-windows-x86-64-msvc-0.53.0
                                    rust-winnow-0.5.40
                                    rust-winnow-0.7.13
                                    rust-wit-bindgen-0.45.0
                                    rust-writeable-0.6.1
                                    rust-ws-stream-wasm-0.7.5
                                    rust-wtf8-0.1.0
                                    rust-x25519-dalek-2.0.1
                                    rust-x509-parser-0.16.0
                                    rust-yansi-1.0.1
                                    rust-yasna-0.5.2
                                    rust-yoke-0.8.0
                                    rust-yoke-derive-0.8.0
                                    rust-zerocopy-0.8.26
                                    rust-zerocopy-derive-0.8.26
                                    rust-zerofrom-0.1.6
                                    rust-zerofrom-derive-0.1.6
                                    rust-zeroize-1.8.1
                                    rust-zeroize-derive-1.4.2
                                    rust-zerotrie-0.2.2
                                    rust-zerovec-0.11.4
                                    rust-zerovec-derive-0.11.1)))
;;; Copyright 2025
;;; /packaging/guix/ctoolbox.scm is free software; you can redistribute it and/or modify it
;;; under the terms of the GNU General Public License as published by
;;; the Free Software Foundation; either version 3 of the License, or (at
;;; your option) any later version.
;;;
;;; /packaging/guix/ctoolbox.scm is distributed in the hope that it will be useful, but
;;; WITHOUT ANY WARRANTY; without even the implied warranty of
;;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;;; GNU General Public License for more details.
;;;
;;; You should have received a copy of the GNU General Public License
;;; along with /packaging/guix/ctoolbox.scm.  If not, see <http://www.gnu.org/licenses/>.

(use-modules (system base compile) (srfi srfi-1))

(define-module (ctoolbox)
    #:use-module (guix licenses)
    #:use-module (guix build-system cargo)
    #:use-module (guix build-system copy)
    #:use-module (guix packages)
    #:use-module (guix download)
    #:use-module (guix git-download)
    #:use-module (guix gexp)
    #:use-module (gnu packages)
    #:use-module (ctoolbox-rust-crates)
)

(define-public ctoolbox
    (package
        (name "ctoolbox")
        (version "0.1.0")
        (source
            (origin
                ;; FIXME: Kind of a hack. `git archive` puts all files in the
                ;; root directory, so zipbomb wraps it up in a subdirectory.
                (method url-fetch/zipbomb)
                (uri (string-append "file://" (getcwd) "/built/src/src.zip"))
                (sha256
                (base32 "060vbqxk0lnlagdq9wgznvmvplgcbrxgqf2bavpk4hwc17ansx0v"))
                (modules '((guix build utils)))
                (snippet '(begin
                    (for-each delete-file-recursively '(
                        ".github"
                        ".vscode"
                        "built"
                        "packaging/guix/generated"
                        "docs/eite/eite.js"
                        "docs/eite/implementation/platform-support/web/soccer.css"
                        "docs/eite/implementation/platform-support/web/soccer.otf"
                        "docs/eite/implementation/platform-support/web/soccer.svg"
                        "docs/mockups/Home screen, open, search/index.html"
                        "docs/mockups/Home screen, open, search/assets/apps"
                        "docs/mockups/Home screen, open, search/assets/fonts"
                        "docs/mockups/Home screen, open, search/assets/html5shim"
                        "docs/mockups/Home screen, open, search/assets/jquery-ui-1.9.2.custom"
                        "docs/mockups/Home screen, open, search/assets/json2"
                        "docs/mockups/Home screen, open, search/assets/react"
                        "docs/mockups/Home screen, open, search/assets/scripts.js"
                        "ctoolbox/Cargo.lock"
                    ))
                    (substitute* "ctoolbox/Cargo.toml"
                        (("^rouille .*") "rouille = { features = [ \"rustls\" ], version = \"*\" }\n")
                    )
                    #t
                ))
            )
        )
        (build-system cargo-build-system)
        (native-inputs (list
            (specification->package "unzip")
        ))
        (inputs (cons*
            (specification->package "bash")
            rust-rouille-3.6.2.31f772c
            (cargo-inputs 'ctoolbox #:module '(ctoolbox-rust-crates))
        ))
        (arguments
            (list
                #:phases
                #~(modify-phases %standard-phases
                    (add-before 'build 'pre-build
                        (lambda* (#:key inputs #:allow-other-keys)
                            (display (string-append "file://" (getcwd)))
                            (copy-recursively "guix-vendor/rust-rouille-multipart-0.18.0.31f772c-checkout" "guix-vendor/rouille-multipart-0.18.0")
                            (invoke "bash" "-c" "echo 'rouille-multipart = { path = \"../guix-vendor/rouille-multipart-0.18.0/rouille-multipart\" }'$'\\n''rouille = { path = \"../guix-vendor/rouille-3.6.2\" }' >> ctoolbox/Cargo.toml")
                            (invoke "packaging/pre-build")
                            (copy-file "src.zip" "built/src/src.zip")
                            ;; Guix needs the Rust package to be in the source
                            ;; directory, and the lib.rs expects the source
                            ;; files to be one level up from where the
                            ;; Cargo.toml is located.
                            ;; (invoke "bash" "-c" "mv ./* ../; mv ../ctoolbox/* ./")
                            (chdir "ctoolbox")
                        )
                    )
                )
            )
        )
        (synopsis "Collective Toolbox: A graph‑based workspace for linking documents and data")
        (description
        "Collective Toolbox: A graph‑based workspace for linking documents and data")
        (home-page "https://www.example.org/")
        (license (list agpl3+ expat silofl1.1))
    )
)

(define-public ctoolbox-vendored
    (hidden-package
        (package
            (name "ctoolbox-vendored")
            (version "0.1.0")
            (source
                (origin
                    (method url-fetch)
                    (uri (string-append "file://" (getcwd) "/built/src/vendor.tar"))
                    (sha256
                    (base32 "12x92q2w4xg9whbsb9wsxnk8n104x4zfiqcd9rss62d936a6zv6b"))
                    (modules '((guix build utils)))
                )
            )
            (build-system copy-build-system)
            (synopsis "Collective Toolbox: A graph‑based workspace for linking documents and data (vendored dependency tarball for rouille)")
            (description
            "Collective Toolbox: A graph‑based workspace for linking documents and data. (vendored dependency tarball for rouille)")
            (home-page "https://www.example.org/")
            (license (list expat asl2.0))
        )
    )
)

(define-public rust-rouille-3.6.2.31f772c
    (
    let (
        (commit "31f772c40007503d25e60e9aba77836a4273ec26")
        (revision "0")
    )
    (hidden-package
        (package
            (name "rust-rouille")
            ;; FIXME: rouille-multipart should be split into a separate package
            ;; see https://codeberg.org/guix/guix/src/commit/6dc9cee1dd51a69f8940215f4db294b0cde02c92/guix/build/cargo-build-system.scm#L79
            (version (git-version "3.6.2" revision commit))
            ;; tiny_http is vendored, so it's not available in rust_crates, even
            ;; though it's depended on by rouille
            (source
                (origin
                    (method git-fetch)
                    (uri (git-reference
                        (url "https://github.com/tomaka/rouille.git")
                        (commit commit)))
                    (file-name (git-file-name name version))
                    ;; Updating hash:
                    ;; git clone https://github.com/tomaka/rouille.git
                    ;; cd rouille
                    ;; git checkout 31f772c40007503d25e60e9aba77836a4273ec26
                    ;; guix hash -x --serializer=nar .
                    (sha256
                    (base32 "138kclylm6g4dpmc2ds45maal7yhcdsrrhblwynx05gpph9swq82"))
                    (modules '((guix build utils)))
                    (snippet '(begin
                        ;; Turn off openssl from rouille tiny_http dependency
                        ;; ctoolbox uses rustls instead
                        (substitute* "Cargo.toml"
                            (("^ssl =.*") "\n"))
                        ;; Remove optional dependencies that cause build to fail
                        ;; due to dependency not being available
                        (substitute* "Cargo.toml"
                            (("^postgres .*") "\n"))
                        (delete-file-recursively "examples/database.rs")
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^clippy .*") "\n"))
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^hyper .*") "\n"))
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^iron .*") "\n"))
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^nickel .*") "\n"))
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^rocket .*") "\n"))
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^env_logger .*") "\n"))
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^tiny_http .*") "tiny_http = { version = \"0.12\", features = [ \"zeroize\" ], optional = true }\n"))
                        (substitute* "rouille-multipart/Cargo.toml"
                            (("^default.*") "default = [\"client\", \"mock\", \"server\", \"tiny_http\"]\n"))
                        ;; Patch rouille to use vendored tiny http
                        (substitute* "Cargo.toml"
                            (("^tiny_http .*") "tiny_http = { version = \"0.12\", features = [ \"zeroize\" ] }\n"))
                        ;; Append patch for vendored tiny_http after members
                        ;; section
                        (substitute* "Cargo.toml"
                            (("^members .*") "members = [\"rouille-multipart\"]\n\n[patch.crates-io]\ntiny_http = { path = \"vendor/tiny_http\" }\n"))
                        #t
                    ))
                )
            )
            (build-system cargo-build-system)
            (inputs (cons*
                (specification->package "bash")
                ctoolbox-vendored
                (cargo-inputs 'ctoolbox #:module '(ctoolbox-rust-crates))
            ))
            (arguments
                (list
                    #:phases
                    #~(modify-phases %standard-phases
                        (add-before 'build 'pre-build
                            (lambda* (#:key inputs #:allow-other-keys)
                                (display (string-append "file://" (getcwd)))
                                (mkdir-p "vendor")
                                (copy-recursively (string-append (assoc-ref inputs "ctoolbox-vendored") "/tiny_http") "vendor/tiny_http")
                                (copy-recursively "rouille-multipart" "guix-vendor/rouille-multipart")
                                (copy-recursively "vendor/tiny_http" "guix-vendor/tiny_http")
                                (invoke "bash" "-c" "echo '{\"files\":{}}' > guix-vendor/rouille-multipart/.cargo-checksum.json")
                                (invoke "bash" "-c" "echo '{\"files\":{}}' > guix-vendor/tiny_http/.cargo-checksum.json")
                            )
                        )
                    )
                )
            )
            (home-page "https://github.com/tomaka/rouille/")
            (synopsis "Rust web framework Rouille")
            (description "This package provides Rust web framework Rouille.")
            (license (list expat asl2.0))
        )
    )
    )
)

ctoolbox

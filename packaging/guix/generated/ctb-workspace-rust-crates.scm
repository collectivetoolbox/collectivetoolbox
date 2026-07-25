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


(define-module (ctb-workspace-rust-crates)
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

(define rust-ab-glyph-0.2.32
  (crate-source "ab_glyph" "0.2.32"
                "1hkc7y8yjd261d5cm9771dawnwc26rgdlniv3jysb3n3f9s4bh01"))

(define rust-ab-glyph-rasterizer-0.1.10
  (crate-source "ab_glyph_rasterizer" "0.1.10"
                "065n6bj7kqk6f12336lm87fqmvf4lxg7rkg2j56nix228jmgnvrn"))

(define rust-accesskit-0.21.1
  (crate-source "accesskit" "0.21.1"
                "16balg6n7gyg05z7wm4a4iczf66z53vbw7rxhfc9zwnq7ffky86g"))

(define rust-accesskit-atspi-common-0.14.2
  (crate-source "accesskit_atspi_common" "0.14.2"
                "0rw25av7v66c1wdckix6kp9zh8blhky4vhssmkq89iqzylf283c9"))

(define rust-accesskit-consumer-0.30.1
  (crate-source "accesskit_consumer" "0.30.1"
                "1k81rbh5wgjpl90ib31gq8x5kwq9cy9bzpnlzw7ja6cqx9gnzl5x"))

(define rust-accesskit-consumer-0.31.0
  (crate-source "accesskit_consumer" "0.31.0"
                "0hyx2z5xbaql81hx54vif4xv8y850yccxrkjj1zp1n4md05030fv"))

(define rust-accesskit-macos-0.22.2
  (crate-source "accesskit_macos" "0.22.2"
                "0xdmjnj48k9ryzqgjnni536594wdi5rlfdza2cg2ijn019f9w250"))

(define rust-accesskit-unix-0.17.2
  (crate-source "accesskit_unix" "0.17.2"
                "0a65zsa5yn14lkpyil00cxj5398449bmzkj3i72dj5gwkjrma7ih"))

(define rust-accesskit-windows-0.29.2
  (crate-source "accesskit_windows" "0.29.2"
                "0wd9mi7v1h3dc5ph6564179p9c7cjsl1jm1zv1iw6j8y0kakvmnj"))

(define rust-accesskit-winit-0.29.2
  (crate-source "accesskit_winit" "0.29.2"
                "1820l6hkd73g0rdy3rzyjq23n7nkilcp07zv5d0sgb0fkpjspky8"))

(define rust-addr2line-0.25.1
  (crate-source "addr2line" "0.25.1"
                "0jwb96gv17vdr29hbzi0ha5q6jkpgjyn7rjlg5nis65k41rk0p8v"))

(define rust-adler2-2.0.1
  (crate-source "adler2" "2.0.1"
                "1ymy18s9hs7ya1pjc9864l30wk8p2qfqdi7mhhcc5nfakxbij09j"))

(define rust-aead-0.5.2
  (crate-source "aead" "0.5.2"
                "1c32aviraqag7926xcb9sybdm36v5vh9gnxpn4pxdwjc50zl28ni"))

(define rust-aead-0.6.1
  (crate-source "aead" "0.6.1"
                "16acx2vq8lfwr6v8yhg1q7cggrr8ih41ykp7a3srrbrd3aycywqr"))

(define rust-aegis-0.9.12
  (crate-source "aegis" "0.9.12"
                "0q5rnzww8g0nvqp7591zvl0l34x2z6hqyyqdnwslp4l4ag8kjzg0"))

(define rust-aes-0.8.4
  (crate-source "aes" "0.8.4"
                "1853796anlwp4kqim0s6wm1srl4ib621nm0cl2h3c8klsjkgfsdi"))

(define rust-aes-0.9.1
  (crate-source "aes" "0.9.1"
                "0f71l2rwx3jdghbrgvjppz2x1jcfvpzx8rn40r2idjf4xbm7dz7i"))

(define rust-aes-gcm-0.10.3
  (crate-source "aes-gcm" "0.10.3"
                "1lgaqgg1gh9crg435509lqdhajg1m2vgma6f7fdj1qa2yyh10443"))

(define rust-aes-gcm-0.11.0
  (crate-source "aes-gcm" "0.11.0"
                "0a50jfn93g2hvkj2kx5g6znpzd2lapdlkmwkambhvki15vdi3w7x"))

(define rust-ahash-0.7.8
  (crate-source "ahash" "0.7.8"
                "1y9014qsy6gs9xld4ch7a6xi9bpki8vaciawxq4p75d8qvh7f549"))

(define rust-ahash-0.8.12
  (crate-source "ahash" "0.8.12"
                "0xbsp9rlm5ki017c0w6ay8kjwinwm8knjncci95mii30rmwz25as"))

(define rust-aho-corasick-1.1.4
  (crate-source "aho-corasick" "1.1.4"
                "00a32wb2h07im3skkikc495jvncf62jl6s96vwc7bhi70h9imlyx"))

(define rust-aligned-0.4.3
  (crate-source "aligned" "0.4.3"
                "1186lhb3gb4x6spzw7ff0zcraa8cr9zqk4ldpm5g1vb2ijc0higf"))

(define rust-aligned-vec-0.6.4
  (crate-source "aligned-vec" "0.6.4"
                "16vnf78hvfix5cwzd5xs5a2g6afmgb4h7n6yfsc36bv0r22072fw"))

(define rust-alloc-no-stdlib-2.0.4
  (crate-source "alloc-no-stdlib" "2.0.4"
                "1cy6r2sfv5y5cigv86vms7n5nlwhx1rbyxwcraqnmm1rxiib2yyc"))

(define rust-alloc-stdlib-0.2.2
  (crate-source "alloc-stdlib" "0.2.2"
                "1kkfbld20ab4165p29v172h8g0wvq8i06z8vnng14whw0isq5ywl"))

(define rust-allocator-api2-0.2.21
  (crate-source "allocator-api2" "0.2.21"
                "08zrzs022xwndihvzdn78yqarv2b9696y67i6h78nla3ww87jgb8"))

(define rust-ammonia-4.1.3
  (crate-source "ammonia" "4.1.3"
                "18l53fai17g59yaks3jppa1w0hl9n8ccrz8hg95jz8c00lvx7fb8"))

(define rust-android-activity-0.6.1
  (crate-source "android-activity" "0.6.1"
                "1k8v4mw8kijvmjmqwr05cjvk2arklx2968bjjpa5szc5aaq1nahg"))

(define rust-android-properties-0.2.2
  (crate-source "android-properties" "0.2.2"
                "016slvg269c0y120p9qd8vdfqa2jbw4j0g18gfw6p3ain44v4zpw"))

(define rust-android-system-properties-0.1.5
  (crate-source "android_system_properties" "0.1.5"
                "04b3wrz12837j7mdczqd95b732gw5q7q66cv4yn4646lvccp57l1"))

(define rust-anstream-0.6.21
  (crate-source "anstream" "0.6.21"
                "0jjgixms4qjj58dzr846h2s29p8w7ynwr9b9x6246m1pwy0v5ma3"))

(define rust-anstream-1.0.0
  (crate-source "anstream" "1.0.0"
                "13d2bj0xfg012s4rmq44zc8zgy1q8k9yp7yhvfnarscnmwpj2jl2"))

(define rust-anstyle-1.0.14
  (crate-source "anstyle" "1.0.14"
                "0030szmgj51fxkic1hpakxxgappxzwm6m154a3gfml83lq63l2wl"))

(define rust-anstyle-parse-0.2.7
  (crate-source "anstyle-parse" "0.2.7"
                "1hhmkkfr95d462b3zf6yl2vfzdqfy5726ya572wwg8ha9y148xjf"))

(define rust-anstyle-parse-1.0.0
  (crate-source "anstyle-parse" "1.0.0"
                "03hkv2690s0crssbnmfkr76kw1k7ah2i6s5amdy9yca2n8w7zkjj"))

(define rust-anstyle-query-1.1.5
  (crate-source "anstyle-query" "1.1.5"
                "1p6shfpnbghs6jsa0vnqd8bb8gd7pjd0jr7w0j8jikakzmr8zi20"))

(define rust-anstyle-wincon-3.0.11
  (crate-source "anstyle-wincon" "3.0.11"
                "0zblannm70sk3xny337mz7c6d8q8i24vhbqi42ld8v7q1wjnl7i9"))

(define rust-antithesis-sdk-0.2.9
  (crate-source "antithesis_sdk" "0.2.9"
                "10lvb5v2r2jxsnjk2whizlq2n7mc292nrk86q1va8s9nr750yh88"))

(define rust-anyhow-1.0.103
  (crate-source "anyhow" "1.0.103"
                "1wsav2g6vxcvf2c0fv3jhxfr55l0p2g8nygy7rmmvcsfwgi8ahra"))

(define rust-ar-archive-writer-0.5.2
  (crate-source "ar_archive_writer" "0.5.2"
                "0j6kpl9pjybd5dpbvhax1kab5nvqlkcs2mxf1ccjfd0a9dmni1s0"))

(define rust-arbitrary-1.4.2
  (crate-source "arbitrary" "1.4.2"
                "1wcbi4x7i3lzcrkjda4810nqv03lpmvfhb0a85xrq1mbqjikdl63"))

(define rust-arboard-3.6.1
  (crate-source "arboard" "3.6.1"
                "1byx6q5iipxkb0pyjp80k7c4akp4n5m7nsmqdbz4n7s9ak0a2j03"))

(define rust-arc-swap-1.9.1
  (crate-source "arc-swap" "1.9.1"
                "01xjlahcya8igdalxmda375lnlhjqwjz0cdqhy0bc1jkyzb1yfka"))

(define rust-arg-enum-proc-macro-0.3.4
  (crate-source "arg_enum_proc_macro" "0.3.4"
                "1sjdfd5a8j6r99cf0bpqrd6b160x9vz97y5rysycsjda358jms8a"))

(define rust-argh-0.1.19
  (crate-source "argh" "0.1.19"
                "0cyhdi44qc61ciy1a1yqvyk6cww3134aar3s2rpwmafd43l1h611"))

(define rust-argh-derive-0.1.19
  (crate-source "argh_derive" "0.1.19"
                "0za2xi4nfnr6102gs3bd5nbnh3883m37vlh5fi3dwpggik8sjhn4"))

(define rust-argh-shared-0.1.19
  (crate-source "argh_shared" "0.1.19"
                "1w8ilkajhz080741djldhqjp0hn62262q4x02y2jgny4p89f1bg5"))

(define rust-argon2-0.5.3
  (crate-source "argon2" "0.5.3"
                "0wn0kk97k49wxidfigmz1pdqmygqzi4h6w72ib7cpq765s4i0diw"))

(define rust-array-init-2.1.0
  (crate-source "array-init" "2.1.0"
                "1z0bh6grrkxlbknq3xyipp42rasngi806y92fiddyb2n99lvfqix"))

(define rust-arrayref-0.3.9
  (crate-source "arrayref" "0.3.9"
                "1jzyp0nvp10dmahaq9a2rnxqdd5wxgbvp8xaibps3zai8c9fi8kn"))

(define rust-arrayvec-0.7.6
  (crate-source "arrayvec" "0.7.6"
                "0l1fz4ccgv6pm609rif37sl5nv5k6lbzi7kkppgzqzh1vwix20kw"))

(define rust-as-raw-xcb-connection-1.0.1
  (crate-source "as-raw-xcb-connection" "1.0.1"
                "0sqgpz2ymv5yx76r5j2npjq2x5qvvqnw0vrs35cyv30p3pfp2m8p"))

(define rust-as-slice-0.2.1
  (crate-source "as-slice" "0.2.1"
                "05j52y1ws8kir5zjxnl48ann0if79sb56p9nm76hvma01r7nnssi"))

(define rust-ascii-1.1.0
  (crate-source "ascii" "1.1.0"
                "05nyyp39x4wzc1959kv7ckwqpkdzjd9dw4slzyjh73qbhjcfqayr"))

(define rust-ascii-canvas-3.0.0
  (crate-source "ascii-canvas" "3.0.0"
                "1in38ziqn4kh9sw89ys4naaqzvvjscfs0m4djqbfq7455v5fq948"))

(define rust-asn1-rs-0.6.2
  (crate-source "asn1-rs" "0.6.2"
                "0j5h437ycgih5hnrma6kmaxi4zb8csynnd66h9rzvxxcvfzc74sl"))

(define rust-asn1-rs-derive-0.5.1
  (crate-source "asn1-rs-derive" "0.5.1"
                "140ldl0vp1d0090bpm0w9j8g80dwc03wp928w5kv5diwwlrjsp4n"))

(define rust-asn1-rs-impl-0.2.0
  (crate-source "asn1-rs-impl" "0.2.0"
                "1xv56m0wrwix4av3w86sih1nsa5g1dgfz135lz1qdznn5h60a63v"))

(define rust-assoc-0.1.3
  (crate-source "assoc" "0.1.3"
                "1a2rk2fcazrhv8bszxiibf8pdj4hbzqk7dm4gwldgfdd7lcp1p5z"))

(define rust-ast-node-5.0.0
  (crate-source "ast_node" "5.0.0"
                "155iy0h9f83l175rkqzc0ih6nlh8s34bjw08yif95nm603pjbc1f"))

(define rust-async-broadcast-0.7.2
  (crate-source "async-broadcast" "0.7.2"
                "0ckmqcwyqwbl2cijk1y4r0vy60i89gqc86ijrxzz5f2m4yjqfnj3"))

(define rust-async-channel-2.5.0
  (crate-source "async-channel" "2.5.0"
                "1ljq24ig8lgs2555myrrjighycpx2mbjgrm3q7lpa6rdsmnxjklj"))

(define rust-async-compression-0.4.42
  (crate-source "async-compression" "0.4.42"
                "1b59jb3y26pmxdshyjb7slxrp184ydlzq80ryfc2ik6cg653z6z7"))

(define rust-async-executor-1.14.0
  (crate-source "async-executor" "1.14.0"
                "0al1rmxjy7p7r6h50z698q5lwssqs5a2vzmqbazm1z2sv1rgjsy9"))

(define rust-async-io-2.6.0
  (crate-source "async-io" "2.6.0"
                "1z16s18bm4jxlmp6rif38mvn55442yd3wjvdfhvx4hkgxf7qlss5"))

(define rust-async-lock-3.4.2
  (crate-source "async-lock" "3.4.2"
                "04c3xrrdrfrvh9v0ajxrangpy38qi76qq268zslphnxxjqjpy3r9"))

(define rust-async-process-2.5.0
  (crate-source "async-process" "2.5.0"
                "0xfswxmng6835hjlfhv7k0jrfp7czqxpfj6y2s5dsp05q0g94l7w"))

(define rust-async-recursion-1.1.1
  (crate-source "async-recursion" "1.1.1"
                "04ac4zh8qz2xjc79lmfi4jlqj5f92xjvfaqvbzwkizyqd4pl4hrv"))

(define rust-async-signal-0.2.14
  (crate-source "async-signal" "0.2.14"
                "11dlpb15la279r5cazppy18gbk2xzzl60ahzl19m1kr0l2psmdaj"))

(define rust-async-task-4.7.1
  (crate-source "async-task" "4.7.1"
                "1pp3avr4ri2nbh7s6y9ws0397nkx1zymmcr14sq761ljarh3axcb"))

(define rust-async-trait-0.1.89
  (crate-source "async-trait" "0.1.89"
                "1fsxxmz3rzx1prn1h3rs7kyjhkap60i7xvi0ldapkvbb14nssdch"))

(define rust-atomic-polyfill-1.0.3
  (crate-source "atomic-polyfill" "1.0.3"
                "1x00ndablb89zvbr8m03cgjzgajg86fqn8pgz85yy2gy1pivrwlc"))

(define rust-atomic-waker-1.1.2
  (crate-source "atomic-waker" "1.1.2"
                "1h5av1lw56m0jf0fd3bchxq8a30xv0b4wv8s4zkp4s0i7mfvs18m"))

(define rust-atspi-0.25.0
  (crate-source "atspi" "0.25.0"
                "0p412rz8cnsqh1l3wx5zq0ahxvhyg406qcazmy68623m5rc4fcn8"))

(define rust-atspi-common-0.9.0
  (crate-source "atspi-common" "0.9.0"
                "1yzxdkkzzs43aslyysaar7vr93vqyljby0vq3659i46zgigc1prk"))

(define rust-atspi-connection-0.9.0
  (crate-source "atspi-connection" "0.9.0"
                "0f29g39w06dk15hmap2scfv4csr52i3h1q3a0l226cyq0c9xb4s1"))

(define rust-atspi-proxies-0.9.0
  (crate-source "atspi-proxies" "0.9.0"
                "073msx1xrf0xjy56kifvpqrny7ndw6ah4vzxpk82cvz7wywvrvnj"))

(define rust-autocfg-1.5.1
  (crate-source "autocfg" "1.5.1"
                "0lqasy5i30flcgih1b50kvsk6z32g09r1q4ql7q81pj6228jy0zj"))

(define rust-av-scenechange-0.14.1
  (crate-source "av-scenechange" "0.14.1"
                "1543y7riwcy4mmsgcalxcm3bnb41hvwiqiz774nbj68fq9vischg"))

(define rust-av1-grain-0.2.5
  (crate-source "av1-grain" "0.2.5"
                "1y3p43i5xncbny0pfh8kw09am3l3mgyg82ln65r3f434443xpzcc"))

(define rust-avif-serialize-0.8.9
  (crate-source "avif-serialize" "0.8.9"
                "0f3z55fma6xmdj0a0x15vz91cqisiardrfgbjlwb2q6lyzjqy5z7"))

(define rust-aws-lc-rs-1.17.0
  (crate-source "aws-lc-rs" "1.17.0"
                "003d69lq9qf12bj4j6csy3nrvilwa30yd9x9blx7h1f27vyg3hjy"))

(define rust-aws-lc-sys-0.41.0
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "aws-lc-sys" "0.41.0"
                "1x735y1qny5v2gzpl928z1ppddb906nl1n8d2yv3mfc5rrwrfbqs"))

(define rust-axum-0.8.9
  (crate-source "axum" "0.8.9"
                "146df5x8dhczm1sp939gr3839220wl6rxc1k65bzc450z72ridii"))

(define rust-axum-core-0.5.6
  (crate-source "axum-core" "0.5.6"
                "1lcjhxysnbc64rh21ag9m9fpiryd1iwcdh9mwxz1yadiswqqziq8"))

(define rust-axum-extra-0.12.6
  (crate-source "axum-extra" "0.12.6"
                "0w3r7w87726ycs1l5r15gblmpif3r4ah08x54cnspffc84xnhi5y"))

(define rust-axum-macros-0.5.1
  (crate-source "axum-macros" "0.5.1"
                "1jhawa9d6pgkcqflbqz7vylv4ksh9wm31kdrcd1jrggv7g16i8ks"))

(define rust-axum-server-0.8.0
  (crate-source "axum-server" "0.8.0"
                "1z1bsb3dhk3xq2wnx4ilrlwkcig6a5qi4f1b96ws10nrhcb37pxi"))

(define rust-axum-typed-multipart-0.16.6
  (crate-source "axum_typed_multipart" "0.16.6"
                "0r4pqdh886x0h4i5nha1zifchnxn40fm4zx3axbx07sir0hsl6di"))

(define rust-axum-typed-multipart-macros-0.16.6
  (crate-source "axum_typed_multipart_macros" "0.16.6"
                "18slcpqwd7q7gw7l7wy01slrh5qnv9m869w0q9b2ws4q83jghlnp"))

(define rust-backtrace-0.3.76
  (crate-source "backtrace" "0.3.76"
                "1mibx75x4jf6wz7qjifynld3hpw3vq6sy3d3c9y5s88sg59ihlxv"))

(define rust-base16ct-0.2.0
  (crate-source "base16ct" "0.2.0"
                "1kylrjhdzk7qpknrvlphw8ywdnvvg39dizw9622w3wk5xba04zsc"))

(define rust-base64-0.21.7
  (crate-source "base64" "0.21.7"
                "0rw52yvsk75kar9wgqfwgb414kvil1gn7mqkrhn9zf1537mpsacx"))

(define rust-base64-0.22.1
  (crate-source "base64" "0.22.1"
                "1imqzgh7bxcikp5vx3shqvw9j09g9ly0xr0jma0q66i52r7jbcvj"))

(define rust-base64-simd-0.8.0
  (crate-source "base64-simd" "0.8.0"
                "15cihnjqpxy0h7llpk816czyp5z613yrvsivw9i8f5vkivkvp6ik"))

(define rust-base64ct-1.8.3
  (crate-source "base64ct" "1.8.3"
                "01nyyyx84bhwrcc168hn47d8gvz2pzpv3y3lmck7mq4hw5vh3x9a"))

(define rust-bcrypt-0.15.1
  (crate-source "bcrypt" "0.15.1"
                "1iv2fvy5yywkx4kijqyy59bq92gldv3nqd4bry97vx4f0pnkhng6"))

(define rust-beef-0.5.2
  (crate-source "beef" "0.5.2"
                "1c95lbnhld96iwwbyh5kzykbpysq0fnjfhwxa1mhap5qxgrl30is"))

(define rust-better-scoped-tls-1.0.1
  (crate-source "better_scoped_tls" "1.0.1"
                "029nc2l4xbh3la5q8sz54rdr96y7k9hlggvms7p35c8mac92ilkw"))

(define rust-bigdecimal-0.4.10
  (crate-source "bigdecimal" "0.4.10"
                "159nc0bs6bbzxrpfxbnn83ccyzq8bc2ia40zd22ssfjvavqnfs2d"))

(define rust-bincode-1.3.3
  (crate-source "bincode" "1.3.3"
                "1bfw3mnwzx5g1465kiqllp5n4r10qrqy88kdlp3jfwnq2ya5xx5i"))

(define rust-bindgen-0.69.5
  (crate-source "bindgen" "0.69.5"
                "1240snlcfj663k04bjsg629g4wx6f83flgbjh5rzpgyagk3864r7"))

(define rust-binrw-0.15.1
  (crate-source "binrw" "0.15.1"
                "0hinmm55qzkax5d2qxsakv9rz4njkl280zn83i6vk2p8hpwracfm"))

(define rust-binrw-derive-0.15.1
  (crate-source "binrw_derive" "0.15.1"
                "1b80xqm4n2adiymbzl1zma09a8wrzdhmmzy86a87hssmxq2xl42r"))

(define rust-bit-field-0.10.3
  (crate-source "bit_field" "0.10.3"
                "1ikhbph4ap4w692c33r8bbv6yd2qxm1q3f64845grp1s6b3l0jqy"))

(define rust-bit-set-0.5.3
  (crate-source "bit-set" "0.5.3"
                "1wcm9vxi00ma4rcxkl3pzzjli6ihrpn9cfdi0c5b4cvga2mxs007"))

(define rust-bit-set-0.8.0
  (crate-source "bit-set" "0.8.0"
                "18riaa10s6n59n39vix0cr7l2dgwdhcpbcm97x1xbyfp1q47x008"))

(define rust-bit-vec-0.6.3
  (crate-source "bit-vec" "0.6.3"
                "1ywqjnv60cdh1slhz67psnp422md6jdliji6alq0gmly2xm9p7rl"))

(define rust-bit-vec-0.8.0
  (crate-source "bit-vec" "0.8.0"
                "1xxa1s2cj291r7k1whbxq840jxvmdsq9xgh7bvrxl46m80fllxjy"))

(define rust-bitflags-1.3.2
  (crate-source "bitflags" "1.3.2"
                "12ki6w8gn1ldq7yz9y680llwk5gmrhrzszaa17g1sbrw2r2qvwxy"))

(define rust-bitflags-2.13.0
  (crate-source "bitflags" "2.13.0"
                "1y239gpvl061rfvav7jds8mjs42kmwi39is7yx5d1qw3hvp8nf5l"))

(define rust-bitpacking-0.9.3
  (crate-source "bitpacking" "0.9.3"
                "06dh7qyax30q7xbg8cif2xv9bp7kkhw0m4kgrpwfp71xpnd179wn"))

(define rust-bitstream-io-4.10.0
  (crate-source "bitstream-io" "4.10.0"
                "07zxcy47l51k6vsxphzhgcnqyzl21pprs7212687c64s56z01zvy"))

(define rust-bitvec-1.0.1
  (crate-source "bitvec" "1.0.1"
                "173ydyj2q5vwj88k6xgjnfsshs4x9wbvjjv7sm0h36r34hn87hhv"))

(define rust-blake2-0.10.6
  (crate-source "blake2" "0.10.6"
                "1zlf7w7gql12v61d9jcbbswa3dw8qxsjglylsiljp9f9b3a2ll26"))

(define rust-blake3-1.8.5
  (crate-source "blake3" "1.8.5"
                "1khz6wq61fnr0gl1kmy4bxadc7gbcv4gbq05z4jdjhr8wqs3ra0a"))

(define rust-block-buffer-0.10.4
  (crate-source "block-buffer" "0.10.4"
                "0w9sa2ypmrsqqvc20nhwr75wbb5cjr4kkyhpjm1z1lv2kdicfy1h"))

(define rust-block-buffer-0.12.1
  (crate-source "block-buffer" "0.12.1"
                "1ak0cvmxz3yifqmzv6aba9606brsz7d5g3piv5xdcvjsx7dwgxnj"))

(define rust-block2-0.5.1
  (crate-source "block2" "0.5.1"
                "0pyiha5his2grzqr3mynmq244laql2j20992i59asp0gy7mjw4rc"))

(define rust-block2-0.6.2
  (crate-source "block2" "0.6.2"
                "1xcfllzx6c3jc554nmb5qy6xmlkl6l6j5ib4wd11800n0n3rvsyd"))

(define rust-blocking-1.6.2
  (crate-source "blocking" "1.6.2"
                "08bz3f9agqlp3102snkvsll6wc9ag7x5m1xy45ak2rv9pq18sgz8"))

(define rust-blowfish-0.9.1
  (crate-source "blowfish" "0.9.1"
                "1mw7bvj3bg5w8vh9xw9xawqh7ixk2xwsxkj34ph96b9b1z6y44p4"))

(define rust-boa-ast-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-ast" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-boa-engine-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-engine" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-boa-gc-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-gc" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-boa-interner-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-interner" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-boa-macros-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-macros" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-boa-parser-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-parser" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-boa-runtime-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-runtime" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-boa-string-1.0.0-dev.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-boa-string" "1.0.0-dev.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-bon-3.9.3
  (crate-source "bon" "3.9.3"
                "0qgm2bnnxidskacil9vqi36fmqj6gb6davxg29nyqj01gcycf0m6"))

(define rust-bon-macros-3.9.3
  (crate-source "bon-macros" "3.9.3"
                "07vwrphl9j3kdiy9mqrgs6h7qkgf5lv20pdzhgl2v5kavfq9ivkd"))

(define rust-borsh-1.6.1
  (crate-source "borsh" "1.6.1"
                "0nhqivq6rp7318hcns1rf25gpsdd7wvwhbxpzblpspasjpwf7lfg"))

(define rust-borsh-derive-1.6.1
  (crate-source "borsh-derive" "1.6.1"
                "0nfa63arbgl7f5ga1ycl1w35abszjwjrkr35g5d1s44r6q4drkxz"))

(define rust-branches-0.4.4
  (crate-source "branches" "0.4.4"
                "036zdj15m1l6i86bpivc6agz2s1f61xk2mf91s9k604hq5ffn9p4"))

(define rust-brotli-8.0.3
  (crate-source "brotli" "8.0.3"
                "0446ihwc6yk4dsjr4fp9jm9inbzibcwrsjj7pj5p1x9nci8y86c1"))

(define rust-brotli-decompressor-5.0.1
  (crate-source "brotli-decompressor" "5.0.1"
                "0929p5smsq5v0jy509gn746y3v1yjjwnk49xg5g1pklj3cz54qjr"))

(define rust-bstr-1.12.1
  (crate-source "bstr" "1.12.1"
                "1arc1v7h5l86vd6z76z3xykjzldqd5icldn7j9d3p7z6x0d4w133"))

(define rust-built-0.8.1
  (crate-source "built" "0.8.1"
                "1saq332pd6g3svvc9ah8myjpfvgqlzl2ksb1ypp3976kjcfm63jw"))

(define rust-bumpalo-3.20.3
  (crate-source "bumpalo" "3.20.3"
                "0jc6va3nwcqikm7chnpdv1s87my3gs2j7g1sc7g3k91brg3arxbj"))

(define rust-bytecheck-0.6.12
  (crate-source "bytecheck" "0.6.12"
                "1hmipv4yyxgbamcbw5r65wagv9khs033v9483s9kri9sw9ycbk93"))

(define rust-bytecheck-derive-0.6.12
  (crate-source "bytecheck_derive" "0.6.12"
                "0ng6230brd0hvqpbgcx83inn74mdv3abwn95x515bndwkz90dd1x"))

(define rust-bytemuck-1.25.0
  (crate-source "bytemuck" "1.25.0"
                "1v1z32igg9zq49phb3fra0ax5r2inf3aw473vldnm886sx5vdvy8"))

(define rust-bytemuck-derive-1.10.2
  (crate-source "bytemuck_derive" "1.10.2"
                "1zvmjmw1sdmx9znzm4dpbb2yvz9vyim8w6gp4z256l46qqdvvazr"))

(define rust-byteorder-1.5.0
  (crate-source "byteorder" "1.5.0"
                "0jzncxyf404mwqdbspihyzpkndfgda450l0893pz5xj685cg5l0z"))

(define rust-byteorder-lite-0.1.0
  (crate-source "byteorder-lite" "0.1.0"
                "15alafmz4b9az56z6x7glcbcb6a8bfgyd109qc3bvx07zx4fj7wg"))

(define rust-bytes-1.11.1
  (crate-source "bytes" "1.11.1"
                "0czwlhbq8z29wq0ia87yass2mzy1y0jcasjb8ghriiybnwrqfx0y"))

(define rust-bytes-str-0.2.8
  (crate-source "bytes-str" "0.2.8"
                "1hi7ybmrydjkp0mp0gjvqw7hl49m6p4mmbvjlgam918gcpsjnzap"))

(define rust-calendrical-calculations-0.2.4
  (crate-source "calendrical_calculations" "0.2.4"
                "09lwfy6j9ggmzwkg64fxwdl04vpairs6dp3y6n6h91b8vbpddfss"))

(define rust-calloop-0.13.0
  (crate-source "calloop" "0.13.0"
                "1v5zgidnhsyml403rzr7vm99f8q6r5bxq5gxyiqkr8lcapwa57dr"))

(define rust-calloop-0.14.4
  (crate-source "calloop" "0.14.4"
                "1xsd8xk53v9zbvhjy7ynf4gya9s4rvvh8jqx9psi1b2v6rw9kgsd"))

(define rust-calloop-wayland-source-0.3.0
  (crate-source "calloop-wayland-source" "0.3.0"
                "086x5mq16prrcwd9k6bw9an0sp8bj9l5daz4ziz5z4snf2c6m9lm"))

(define rust-calloop-wayland-source-0.4.1
  (crate-source "calloop-wayland-source" "0.4.1"
                "1yi1c23naqhd8m94q3v366s4cak8l50zy7ldrkqfn0hajkqgr3hk"))

(define rust-camino-1.2.2
  (crate-source "camino" "1.2.2"
                "0j0ayqfbbl8bxg0795ssk1hzkjix3dvl2kk63hdgzf9cd5nscag6"))

(define rust-capacity-builder-0.5.0
  (crate-source "capacity_builder" "0.5.0"
                "1ij1cqz77li23p4xlpywflsgyj9s1ws3apdn44m41kghvjk28bcg"))

(define rust-capacity-builder-macros-0.3.0
  (crate-source "capacity_builder_macros" "0.3.0"
                "1rgnpd4akpmy9p7hjig7hdf1cz295j6k7bwgpdncq17wksp6qjiv"))

(define rust-cargo-metadata-0.21.0
  (crate-source "cargo_metadata" "0.21.0"
                "0s3864q6qa1qw1jn2s5rs98aj55389is0n5giyl5p0wrlsma5z2w"))

(define rust-cargo-metadata-0.23.1
  (crate-source "cargo_metadata" "0.23.1"
                "1sddycfscjy47av3ykzykqgz8zjds0i00gcxs76vw4x1n0bpv67g"))

(define rust-cargo-platform-0.2.0
  (crate-source "cargo-platform" "0.2.0"
                "1m7bk5ry59lz52kwm0xir0skflb5z7gdxrjf79d66hz319n2r644"))

(define rust-cargo-platform-0.3.3
  (crate-source "cargo-platform" "0.3.3"
                "1fm418dzcc5rm8qm8a5vlrql7vamflwic3m05vhzl5crfgd6206x"))

(define rust-cargo-util-schemas-0.8.2
  (crate-source "cargo-util-schemas" "0.8.2"
                "0c3qhd0si6dxdmz3n1mwyjf7sfdyx2s38nmffibzh6k5npvsdhbx"))

(define rust-castaway-0.2.4
  (crate-source "castaway" "0.2.4"
                "0nn5his5f8q20nkyg1nwb40xc19a08yaj4y76a8q2y3mdsmm3ify"))

(define rust-cc-1.2.63
  (crate-source "cc" "1.2.63"
                "0zy2bqc4nvj6bv2cipx4h4bn65wf1zqf1fw1hsh64mmvg1hh2vjm"))

(define rust-census-0.4.2
  (crate-source "census" "0.4.2"
                "1q1bk548jy82drj509bxjgmfk5c9xbhhig8as61bx710d9y70k2g"))

(define rust-cexpr-0.6.0
  (crate-source "cexpr" "0.6.0"
                "0rl77bwhs5p979ih4r0202cn5jrfsrbgrksp40lkfz5vk1x3ib3g"))

(define rust-cfg-aliases-0.1.1
  (crate-source "cfg_aliases" "0.1.1"
                "17p821nc6jm830vzl2lmwz60g3a30hcm33nk6l257i1rjdqw85px"))

(define rust-cfg-aliases-0.2.1
  (crate-source "cfg_aliases" "0.2.1"
                "092pxdc1dbgjb6qvh83gk56rkic2n2ybm4yvy76cgynmzi3zwfk1"))

(define rust-cfg-block-0.1.1
  (crate-source "cfg_block" "0.1.1"
                "11z47bfb5qylcp9ryqbbni6139kdzksqd0vw9wkc6r11jxa80x8q"))

(define rust-cfg-if-1.0.4
  (crate-source "cfg-if" "1.0.4"
                "008q28ajc546z5p2hcwdnckmg0hia7rnx52fni04bwqkzyrghc4k"))

(define rust-chacha20-0.10.0
  (crate-source "chacha20" "0.10.0"
                "00bn2rn8l68qvlq93mhq7b4ns4zy9qbjsyjbb9kljgl4hqr9i3bg"))

(define rust-chrono-0.4.45
  (crate-source "chrono" "0.4.45"
                "09rkcgk6is2sdhqs9142zv8xqnj8ryx8m9hknllqwyv9wxi9x9qs"))

(define rust-ciborium-0.2.2
  (crate-source "ciborium" "0.2.2"
                "03hgfw4674im1pdqblcp77m7rc8x2v828si5570ga5q9dzyrzrj2"))

(define rust-ciborium-io-0.2.2
  (crate-source "ciborium-io" "0.2.2"
                "0my7s5g24hvp1rs1zd1cxapz94inrvqpdf1rslrvxj8618gfmbq5"))

(define rust-ciborium-ll-0.2.2
  (crate-source "ciborium-ll" "0.2.2"
                "1n8g4j5rwkfs3rzfi6g1p7ngmz6m5yxsksryzf5k72ll7mjknrjp"))

(define rust-cipher-0.4.4
  (crate-source "cipher" "0.4.4"
                "1b9x9agg67xq5nq879z66ni4l08m6m3hqcshk37d4is4ysd3ngvp"))

(define rust-cipher-0.5.2
  (crate-source "cipher" "0.5.2"
                "0v7sic43nmz4rgql62wmxq0z63s80gnmd0w5q1vlhw6djcn2mkz8"))

(define rust-clang-sys-1.8.1
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "clang-sys" "1.8.1"
                "1x1r9yqss76z8xwpdanw313ss6fniwc1r7dzb5ycjn0ph53kj0hb"))

(define rust-clap-4.5.60
  (crate-source "clap" "4.5.60"
                "02h3nzznssjgp815nnbzk0r62y2iw03kdli75c233kirld6z75r7"))

(define rust-clap-builder-4.5.60
  (crate-source "clap_builder" "4.5.60"
                "0xk8mdizvmmn6w5ij5cwhy5pbgyac4w9pfvl6nqmjl7a5hql38i4"))

(define rust-clap-derive-4.5.55
  (crate-source "clap_derive" "4.5.55"
                "1r949xis3jmhzh387smd70vc8a3b9734ck3g5ahg59a63bd969x9"))

(define rust-clap-lex-1.1.0
  (crate-source "clap_lex" "1.1.0"
                "1ycqkpygnlqnndghhcxjb44lzl0nmgsia64x9581030yifxs7m68"))

(define rust-clap-markdown-0.1.5
  (crate-source "clap-markdown" "0.1.5"
                "0f93ij22sxl1ik0sz25h23n9zc7b0x9pnschnj2lhvd0arwn38nj"))

(define rust-clipboard-win-5.4.1
  (crate-source "clipboard-win" "5.4.1"
                "1m44gqy11rq1ww7jls86ppif98v6kv2wkwk8p17is86zsdq3gq5x"))

(define rust-clru-0.6.3
  (crate-source "clru" "0.6.3"
                "1mb7vx7s8b3xzx7p2frly9w10b7k2yl3lvrpnvcxba0kn6fdjzqr"))

(define rust-clubcard-0.3.3
  (crate-source "clubcard" "0.3.3"
                "1j19rq7knd7v9f3l5v6ralxfckmpnpv29qhkyp4kiy7hnjhp9x1y"))

(define rust-clubcard-crlite-0.3.2
  (crate-source "clubcard-crlite" "0.3.2"
                "1yna1nc5v8vxx1l9286g92vxm5mfccd5i6if637h3fawbrjdvvsg"))

(define rust-cmake-0.1.58
  (crate-source "cmake" "0.1.58"
                "0y06zxw5sv1p5vvpp5rz1qwbrq7ccawrl09nqy5ahx1a5418mxy0"))

(define rust-cmov-0.5.4
  (crate-source "cmov" "0.5.4"
                "0yh22sqdvcdrfbhvnja4kaq5dyklpb4s70w5r6rplfdw4jna17hc"))

(define rust-cobs-0.3.0
  (crate-source "cobs" "0.3.0"
                "18f0kxxa1fqb8pz2dxwssnhsrvhrs5j4p8xllgin5d7h36sn3a8g"))

(define rust-color-quant-1.1.0
  (crate-source "color_quant" "1.1.0"
                "12q1n427h2bbmmm1mnglr57jaz2dj9apk0plcxw7nwqiai7qjyrx"))

(define rust-colorchoice-1.0.5
  (crate-source "colorchoice" "1.0.5"
                "0w75k89hw39p0mnnhlrwr23q50rza1yjki44qvh2mgrnj065a1qx"))

(define rust-colored-2.2.0
  (crate-source "colored" "2.2.0"
                "0g6s7j2qayjd7i3sivmwiawfdg8c8ldy0g2kl4vwk1yk16hjaxqi"))

(define rust-combine-4.6.7
  (crate-source "combine" "4.6.7"
                "1z8rh8wp59gf8k23ar010phgs0wgf5i8cx4fg01gwcnzfn5k0nms"))

(define rust-compact-str-0.7.1
  (crate-source "compact_str" "0.7.1"
                "0gvvfc2c6pg1rwr2w36ra4674w3lzwg97vq2v6k791w30169qszq"))

(define rust-compact-str-0.9.1
  (crate-source "compact_str" "0.9.1"
                "1aq0vx3xnaxf9k8p1pwch5v5av0xj2ddq2av25aa76jd4z1d3zcx"))

(define rust-compression-codecs-0.4.38
  (crate-source "compression-codecs" "0.4.38"
                "1kqq2b8hpv7y3jnakkp66cdlrzl6my02dapn3g12j6cw3qwlh9ff"))

(define rust-compression-core-0.4.32
  (crate-source "compression-core" "0.4.32"
                "12bp209x76flr67jm5fql4hq8d14nkjzkk24g9gi0yh2rxjza56c"))

(define rust-concurrent-queue-2.5.0
  (crate-source "concurrent-queue" "2.5.0"
                "0wrr3mzq2ijdkxwndhf79k952cp4zkz35ray8hvsxl96xrx1k82c"))

(define rust-console-0.15.11
  (crate-source "console" "0.15.11"
                "1n5gmsjk6isbnw6qss043377kln20lfwlmdk3vswpwpr21dwnk05"))

(define rust-const-default-1.0.0
  (crate-source "const-default" "1.0.0"
                "1apcnxfrz5xsfxaxbv1n9c5sdfqlmrk81v0q29z5amflfqgnsf8b"))

(define rust-const-default-derive-0.2.0
  (crate-source "const-default-derive" "0.2.0"
                "1nh3iwba073s9vsyhr5ci0pgbnc6zavmfs7za4vj64mqrgc4v08g"))

(define rust-const-oid-0.9.6
  (crate-source "const-oid" "0.9.6"
                "1y0jnqaq7p2wvspnx7qj76m7hjcqpz73qzvr9l2p9n2s51vr6if2"))

(define rust-constant-time-eq-0.4.2
  (crate-source "constant_time_eq" "0.4.2"
                "16zamq60dq80k3rqlzh9j9cpjhishmh924lnwbplgrnmkkvfylix"))

(define rust-constcat-0.6.1
  (crate-source "constcat" "0.6.1"
                "0b43b3w7yn0xsh8pvwfv9cjw7ca45lg6ia6afi6ylb2sj413wv8k"))

(define rust-convert-case-0.4.0
  (crate-source "convert_case" "0.4.0"
                "03jaf1wrsyqzcaah9jf8l1iznvdw5mlsca2qghhzr9w27sddaib2"))

(define rust-convert-case-0.6.0
  (crate-source "convert_case" "0.6.0"
                "1jn1pq6fp3rri88zyw6jlhwwgf6qiyc08d6gjv0qypgkl862n67c"))

(define rust-cookie-0.18.1
  (crate-source "cookie" "0.18.1"
                "0iy749flficrlvgr3hjmf3igr738lk81n5akzf4ym4cs6cxg7pjd"))

(define rust-core-foundation-0.10.1
  (crate-source "core-foundation" "0.10.1"
                "1xjns6dqf36rni2x9f47b65grxwdm20kwdg9lhmzdrrkwadcv9mj"))

(define rust-core-foundation-0.9.4
  (crate-source "core-foundation" "0.9.4"
                "13zvbbj07yk3b61b8fhwfzhy35535a583irf23vlcg59j7h9bqci"))

(define rust-core-foundation-sys-0.8.7
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "core-foundation-sys" "0.8.7"
                "12w8j73lazxmr1z0h98hf3z623kl8ms7g07jch7n4p8f9nwlhdkp"))

(define rust-core-graphics-0.23.2
  (crate-source "core-graphics" "0.23.2"
                "10dhv3gk4kmbzl14xxkrhhky4fdp8h6nzff6h0019qgr6nz84xy0"))

(define rust-core-graphics-types-0.1.3
  (crate-source "core-graphics-types" "0.1.3"
                "1bxg8nxc8fk4kxnqyanhf36wq0zrjr552c58qy6733zn2ihhwfa5"))

(define rust-core-maths-0.1.1
  (crate-source "core_maths" "0.1.1"
                "0c0dv11ixxpc9bsx5xasvl98mb1dlprzcm6qq6ls3nsygw0mwx3p"))

(define rust-cow-utils-0.1.3
  (crate-source "cow-utils" "0.1.3"
                "0y9cxf0hm2hy4bn050wl2785md5q4i5gy9asjq006ip1mwjfyys1"))

(define rust-cpubits-0.1.1
  (crate-source "cpubits" "0.1.1"
                "1bh6rvanxm00myf1rmnh44hq2jdxn69971c92s4klz0k76f5zf0m"))

(define rust-cpufeatures-0.2.17
  (crate-source "cpufeatures" "0.2.17"
                "10023dnnaghhdl70xcds12fsx2b966sxbxjq5sxs49mvxqw5ivar"))

(define rust-cpufeatures-0.3.0
  (crate-source "cpufeatures" "0.3.0"
                "00fjhygsqmh4kbxxlb99mcsbspxcai6hjydv4c46pwb67wwl2alb"))

(define rust-crc32c-0.6.8
  (crate-source "crc32c" "0.6.8"
                "0iwyr3jivcnhylczqgk1rkpp9b46r25vi5dj1y7il29dc8hsyirs"))

(define rust-crc32fast-1.5.0
  (crate-source "crc32fast" "1.5.0"
                "04d51liy8rbssra92p0qnwjw8i9rm9c4m3bwy19wjamz1k4w30cl"))

(define rust-critical-section-1.2.0
  (crate-source "critical-section" "1.2.0"
                "02ylhcykxjc40xrfhk1lwc21jqgz4dbwv3jr49ymw733c51yl3kr"))

(define rust-crossbeam-channel-0.5.15
  (crate-source "crossbeam-channel" "0.5.15"
                "1cicd9ins0fkpfgvz9vhz3m9rpkh6n8d3437c3wnfsdkd3wgif42"))

(define rust-crossbeam-deque-0.8.6
  (crate-source "crossbeam-deque" "0.8.6"
                "0l9f1saqp1gn5qy0rxvkmz4m6n7fc0b3dbm6q1r5pmgpnyvi3lcx"))

(define rust-crossbeam-epoch-0.9.18
  (crate-source "crossbeam-epoch" "0.9.18"
                "03j2np8llwf376m3fxqx859mgp9f83hj1w34153c7a9c7i5ar0jv"))

(define rust-crossbeam-skiplist-0.1.3
  (crate-source "crossbeam-skiplist" "0.1.3"
                "06qmzagqmrv4zwmrvppv6lja6lbm6hi3vv47wp32rjjq1i2dwafz"))

(define rust-crossbeam-utils-0.8.21
  (crate-source "crossbeam-utils" "0.8.21"
                "0a3aa2bmc8q35fb67432w16wvi54sfmb69rk9h5bhd18vw0c99fh"))

(define rust-crossterm-0.28.1
  (crate-source "crossterm" "0.28.1"
                "1im9vs6fvkql0sr378dfr4wdm1rrkrvr22v4i8byz05k1dd9b7c2"))

(define rust-crossterm-winapi-0.9.1
  (crate-source "crossterm_winapi" "0.9.1"
                "0axbfb2ykbwbpf1hmxwpawwfs8wvmkcka5m561l7yp36ldi7rpdc"))

(define rust-crunchy-0.2.4
  (crate-source "crunchy" "0.2.4"
                "1mbp5navim2qr3x48lyvadqblcxc1dm0lqr0swrkkwy2qblvw3s6"))

(define rust-crypto-bigint-0.5.5
  (crate-source "crypto-bigint" "0.5.5"
                "0xmbdff3g6ii5sbxjxc31xfkv9lrmyril4arh3dzckd4gjsjzj8d"))

(define rust-crypto-common-0.1.7
  (crate-source "crypto-common" "0.1.7"
                "02nn2rhfy7kvdkdjl457q2z0mklcvj9h662xrq6dzhfialh2kj3q"))

(define rust-crypto-common-0.2.2
  (crate-source "crypto-common" "0.2.2"
                "0lql5wjlrjkd3r0w32rwbgqfmgg84ms3h65ldnlckmkc3nb4qvnf"))

(define rust-cssparser-0.37.0
  (crate-source "cssparser" "0.37.0"
                "165s1d8n9i181ni50fzss99gyiigfmzmwyadn217ivfm06pdm74c"))

(define rust-csv-1.4.0
  (crate-source "csv" "1.4.0"
                "0f7r2ip0rbi7k377c3xmsh9xd69sillffhpfmbgnvz3yrxl9vkaj"))

(define rust-csv-core-0.1.13
  (crate-source "csv-core" "0.1.13"
                "10lppd3fdb1i5npgx9xqjs5mjmy2qbdi8n16i48lg03ak4k3qjkh"))

(define rust-ctr-0.10.1
  (crate-source "ctr" "0.10.1"
                "088z8sa9aw7ij1sy4hlpxz20jhffnsfiwmsdysb2a29pnb2a3b5s"))

(define rust-ctr-0.9.2
  (crate-source "ctr" "0.9.2"
                "0d88b73waamgpfjdml78icxz45d95q7vi2aqa604b0visqdfws83"))

(define rust-ctrlc-3.5.2
  (crate-source "ctrlc" "3.5.2"
                "0qh1lvlr6k58dliqllx1n7mjfwp1mzr607bks3r9m0a5msrgmcg0"))

(define rust-ctutils-0.4.2
  (crate-source "ctutils" "0.4.2"
                "17m2s9jv7i780k26cq2fcyslg0pakv9plwdrmygdwha1hfiiambx"))

(define rust-cursive-0.21.1
  (crate-source "cursive" "0.21.1"
                "0bjkmbyy5ivvvgjblmqq5lkb1mlvdi78mjsflglmdf0b08v5lv9q"))

(define rust-cursive-core-0.4.7
  (crate-source "cursive_core" "0.4.7"
                "0kqw2xg9az14zscbfiw18dxknasfczqv72f0s2cd2c2wzyy5836l"))

(define rust-cursive-macros-0.1.0
  (crate-source "cursive-macros" "0.1.0"
                "0gm7l3xzqsgwh4sd0py2g45p6np0ahiz5mglxggxzqzd1kmw0ymc"))

(define rust-cursor-icon-1.2.0
  (crate-source "cursor-icon" "1.2.0"
                "0bvkw7ak1mqwcpkgd9lh7n00hcvlh87jfl7188f231nz6zfy2ypj"))

(define rust-curve25519-dalek-4.1.3
  (crate-source "curve25519-dalek" "4.1.3"
                "1gmjb9dsknrr8lypmhkyjd67p1arb8mbfamlwxm7vph38my8pywp"))

(define rust-curve25519-dalek-derive-0.1.1
  (crate-source "curve25519-dalek-derive" "0.1.1"
                "1cry71xxrr0mcy5my3fb502cwfxy6822k4pm19cwrilrg7hq4s7l"))

(define rust-dark-light-2.0.0.0f18d2f
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url
                         "https://github.com/rust-dark-light/dark-light.git")
                        (commit "0f18d2fbcaa5d1c175db8aae7d53428988d7e961")))
    (file-name (git-file-name "rust-dark-light" "2.0.0.0f18d2f"))
    (sha256 (base32 "13552r0nww5cswnvfyj0g5ahrsxv4g7yiwwkqh6c6ciw54sxz4z0"))))

(define rust-darling-0.20.11
  (crate-source "darling" "0.20.11"
                "1vmlphlrlw4f50z16p4bc9p5qwdni1ba95qmxfrrmzs6dh8lczzw"))

(define rust-darling-0.21.3
  (crate-source "darling" "0.21.3"
                "1h281ah78pz05450r71h3gwm2n24hy8yngbz58g426l4j1q37pww"))

(define rust-darling-0.23.0
  (crate-source "darling" "0.23.0"
                "179fj6p6ajw4dnkrik51wjhifxwy02x5zhligyymcb905zd17bi5"))

(define rust-darling-core-0.20.11
  (crate-source "darling_core" "0.20.11"
                "0bj1af6xl4ablnqbgn827m43b8fiicgv180749f5cphqdmcvj00d"))

(define rust-darling-core-0.21.3
  (crate-source "darling_core" "0.21.3"
                "193ya45qgac0a4siwghk0bl8im8h89p3cald7kw8ag3yrmg1jiqj"))

(define rust-darling-core-0.23.0
  (crate-source "darling_core" "0.23.0"
                "1c033vrks38vpw8kwgd5w088dsr511kfz55n9db56prkgh7sarcq"))

(define rust-darling-macro-0.20.11
  (crate-source "darling_macro" "0.20.11"
                "1bbfbc2px6sj1pqqq97bgqn6c8xdnb2fmz66f7f40nrqrcybjd7w"))

(define rust-darling-macro-0.21.3
  (crate-source "darling_macro" "0.21.3"
                "10ac85n4lnx3rmf5rw8lijl2c0sbl6ghcpgfmzh0s26ihbghi0yk"))

(define rust-darling-macro-0.23.0
  (crate-source "darling_macro" "0.23.0"
                "13fvzji9xyp304mgq720z5l0xgm54qj68jibwscagkynggn88fdc"))

(define rust-dashmap-6.2.1
  (crate-source "dashmap" "6.2.1"
                "1705w9fx4g30287dx2b0xlmy86l29hnvipba2y5cfq920rf1sdp6"))

(define rust-data-encoding-2.11.0
  (crate-source "data-encoding" "2.11.0"
                "1j00wfmk4dzn4bnib07qlhylmd6a3kizwjz8mp00iix3vlamzbm4"))

(define rust-data-url-0.3.2
  (crate-source "data-url" "0.3.2"
                "0xl30jidc8s3kh2z3nvnn1nyzhbq5b2wpiqwzj9gjdrndk50n7my"))

(define rust-datasketches-0.2.0
  (crate-source "datasketches" "0.2.0"
                "0icp1n694mxxbskqyf51a1pmc341hc7lwxadqapr09gah57dx1n2"))

(define rust-dateparser-0.2.1
  (crate-source "dateparser" "0.2.1"
                "0f22d7c6is9w5pi496zsp1k95vmdv65p6bm0v3nfb6p0xqglbvy2"))

(define rust-debugid-0.8.0
  (crate-source "debugid" "0.8.0"
                "13f15dfvn07fa7087pmacixqqv0lmj4hv93biw4ldr48ypk55xdy"))

(define rust-deno-ast-0.53.2
  (crate-source "deno_ast" "0.53.2"
                "1xrgm8mwaq68bngbmkdihkfgs0b5x24k51dhzrzpcf3rnyr2yy85"))

(define rust-deno-error-0.7.3
  (crate-source "deno_error" "0.7.3"
                "1aq720b6fspz0ciddskgrywf4b8cmhx8h5df4hrm1sljxbqx61rh"))

(define rust-deno-error-macro-0.7.3
  (crate-source "deno_error_macro" "0.7.3"
                "1bwfv9ka8s12bs50aznsly9ccyk4z2snb1l85hqxyp38m5h5wmlv"))

(define rust-deno-lint-0.84.1
  (crate-source "deno_lint" "0.84.1"
                "1pyi3imll01jbkdz6gg0cs1d2nkvm8a87vg6ln3by591ayy7y1rg"))

(define rust-deno-media-type-0.4.0
  (crate-source "deno_media_type" "0.4.0"
                "0qq7g6gm02z7ijwqaaqbhgarf2pjvw18mc9gli5dckwzrm7b5fny"))

(define rust-deno-semver-0.10.1
  (crate-source "deno_semver" "0.10.1"
                "1k2nh3zxp9vzg35fhj2yx97qr6701nd012j18yybyfawyndpwj7z"))

(define rust-deno-terminal-0.2.3
  (crate-source "deno_terminal" "0.2.3"
                "1gp4xm5n2ivnccblb2w6w2xxrfhj87grkhv4db5b66bkmr0q1fpk"))

(define rust-der-0.7.10
  (crate-source "der" "0.7.10"
                "1jyxacyxdx6mxbkfw99jz59dzvcd9k17rq01a7xvn1dr6wl87hg7"))

(define rust-der-parser-9.0.0
  (crate-source "der-parser" "9.0.0"
                "0lxmykajggvaq5mvpm2avgzwib4n9nyxii0kqaz2d5k88g3abl2w"))

(define rust-deranged-0.5.8
  (crate-source "deranged" "0.5.8"
                "0711df3w16vx80k55ivkwzwswziinj4dz05xci3rvmn15g615n3w"))

(define rust-derive-builder-0.20.2
  (crate-source "derive_builder" "0.20.2"
                "0is9z7v3kznziqsxa5jqji3ja6ay9wzravppzhcaczwbx84znzah"))

(define rust-derive-builder-core-0.20.2
  (crate-source "derive_builder_core" "0.20.2"
                "1s640r6q46c2iiz25sgvxw3lk6b6v5y8hwylng7kas2d09xwynrd"))

(define rust-derive-builder-macro-0.20.2
  (crate-source "derive_builder_macro" "0.20.2"
                "0g1zznpqrmvjlp2w7p0jzsjvpmw5rvdag0rfyypjhnadpzib0qxb"))

(define rust-derive-more-0.99.20
  (crate-source "derive_more" "0.99.20"
                "0zvz94kbc5d4r817wni1l7xk8f289nhf73vqk677p5rxlij4pnvf"))

(define rust-derive-where-1.6.1
  (crate-source "derive-where" "1.6.1"
                "0d0m0hiw4nhwj338c28w3p0nw09mhjp2qss7rncr21qdrh5km2yh"))

(define rust-deunicode-1.6.2
  (crate-source "deunicode" "1.6.2"
                "013biy7hhy59jcbry4dqn2pf4qhaw083ksn8xxiw373wjc37imdb"))

(define rust-dify-0.7.4
  (crate-source "dify" "0.7.4"
                "1xl7jzl99fdibkcda5m9ni0b9jvwg6wiwrc4ml4vi8xgkr37s88i"))

(define rust-digest-0.10.7
  (crate-source "digest" "0.10.7"
                "14p2n6ih29x81akj097lvz7wi9b6b9hvls0lwrv7b6xwyy0s5ncy"))

(define rust-digest-0.11.3
  (crate-source "digest" "0.11.3"
                "1hnmhd4rkybr11292w42pz9ppzx1h49glrhqg107k4s1b2xnvpgi"))

(define rust-directories-6.0.0
  (crate-source "directories" "6.0.0"
                "0zgy2w088v8w865c11dmc3dih899fgrhvrfp7g83h6v6ai60kx8n"))

(define rust-dirs-6.0.0
  (crate-source "dirs" "6.0.0"
                "0knfikii29761g22pwfrb8d0nqpbgw77sni9h2224haisyaams63"))

(define rust-dirs-next-2.0.0
  (crate-source "dirs-next" "2.0.0"
                "1q9kr151h9681wwp6is18750ssghz6j9j7qm7qi1ngcwy7mzi35r"))

(define rust-dirs-sys-0.5.0
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "dirs-sys" "0.5.0"
                "1aqzpgq6ampza6v012gm2dppx9k35cdycbj54808ksbys9k366p0"))

(define rust-dirs-sys-next-0.1.2
  (crate-source "dirs-sys-next" "0.1.2"
                "0kavhavdxv4phzj4l0psvh55hszwnr0rcz8sxbvx20pyqi2a3gaf"))

(define rust-dispatch-0.2.0
  (crate-source "dispatch" "0.2.0"
                "0fwjr9b7582ic5689zxj8lf7zl94iklhlns3yivrnv8c9fxr635x"))

(define rust-dispatch2-0.3.1
  (crate-source "dispatch2" "0.3.1"
                "0f5xmnbzpaz1g80m27kd804p75nswh0ikb6wvqh4ba3x9rz3c3hy"))

(define rust-displaydoc-0.2.6
  (crate-source "displaydoc" "0.2.6"
                "0kyxwfbdmagd8afzb2pzja7wj8dhah7smxdsgw00iq8pa2jhmiqs"))

(define rust-dlib-0.5.3
  (crate-source "dlib" "0.5.3"
                "0jpr4smrwrv8xj70mz4ixnbc6ljm82f12z2mz1hv89056y3wv3mb"))

(define rust-doctest-file-1.1.1
  (crate-source "doctest-file" "1.1.1"
                "0nfkscv8gf3ixhradrmwm3f2p6sc0ab0psah7c8976ha9zkh9ny2"))

(define rust-downcast-rs-1.2.1
  (crate-source "downcast-rs" "1.2.1"
                "1lmrq383d1yszp7mg5i7i56b17x2lnn3kb91jwsq0zykvg2jbcvm"))

(define rust-downcast-rs-2.0.2
  (crate-source "downcast-rs" "2.0.2"
                "1g0crs9qgz0sd9cwdgmm0zvjin2v549v46xfnc859rk903v40whi"))

(define rust-dpi-0.1.2
  (crate-source "dpi" "0.1.2"
                "0xhsvzgjvdch2fwmfc9vkb708b0q59b6imypyjlgbiigyb74rcfq"))

(define rust-dprint-swc-ext-0.26.0
  (crate-source "dprint-swc-ext" "0.26.0"
                "0k1zhg287xzdm5f3v7a5748kn2vi9b8np5mjra4qahbdgbdms5rk"))

(define rust-drm-0.14.1
  (crate-source "drm" "0.14.1"
                "0vvmj9n0wslrbw3rinpzlfyhwwgr02gqspy1al5gfh99dif8rg40"))

(define rust-drm-ffi-0.9.1
  (crate-source "drm-ffi" "0.9.1"
                "147n13dnkr4kzdj4662dqgbjfvnnw14yhmf2vq2q2kmc6adiraai"))

(define rust-drm-fourcc-2.2.0
  (crate-source "drm-fourcc" "2.2.0"
                "1x76v9a0pkgym4n6cah4barnai9gsssm7gjzxskw2agwibdvrbqa"))

(define rust-drm-sys-0.8.1
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "drm-sys" "0.8.1"
                "1y59h9x5yn9p36f9bqjvw76kx75yqfin1w6gzigiznb620vf3j7c"))

(define rust-dtoa-1.0.11
  (crate-source "dtoa" "1.0.11"
                "1405jvczpxf1zd3nsvw02r50hr2k6argq6jkgdf04prd9s1g8g2c"))

(define rust-dtoa-short-0.3.5
  (crate-source "dtoa-short" "0.3.5"
                "11rwnkgql5jilsmwxpx6hjzkgyrbdmx1d71s0jyrjqm5nski25fd"))

(define rust-dunce-1.0.5
  (crate-source "dunce" "1.0.5"
                "04y8wwv3vvcqaqmqzssi6k0ii9gs6fpz96j5w9nky2ccsl23axwj"))

(define rust-dyn-clone-1.0.20
  (crate-source "dyn-clone" "1.0.20"
                "0m956cxcg8v2n8kmz6xs5zl13k2fak3zkapzfzzp7pxih6hix26h"))

(define rust-dynify-0.1.2
  (crate-source "dynify" "0.1.2"
                "0irpf9rsxafzqydgrddwgmvql2rnc9z5xpkkpxc27qm351bb3b41"))

(define rust-dynify-macros-0.1.2
  (crate-source "dynify-macros" "0.1.2"
                "0zdadka855zabr7caf9xvl7dcifnqmcm4lsnjc1dac44f36k3i0y"))

(define rust-ecolor-0.33.3
  (crate-source "ecolor" "0.33.3"
                "13hsf5b0blff250b3z4fhfd4bp868j00w482pfhxpla3fsnbipbi"))

(define rust-ecow-0.2.6
  (crate-source "ecow" "0.2.6"
                "00mxjzbmz215z8s2x8z7xgh0y0bz9k5m4bg2w9napfkg56dzgr3q"))

(define rust-ed25519-2.2.3
  (crate-source "ed25519" "2.2.3"
                "0lydzdf26zbn82g7xfczcac9d7mzm3qgx934ijjrd5hjpjx32m8i"))

(define rust-ed25519-dalek-2.2.0
  (crate-source "ed25519-dalek" "2.2.0"
                "1agcwij1z687hg26ngzwhnmpz29b2w56m8z1ap3pvrnfh709drvh"))

(define rust-egui-0.33.3
  (crate-source "egui" "0.33.3"
                "1qw4qazx1g4inrr6j7z7kw2ixhx7n3gzxlqy2ajyjx366mymd6va"))

(define rust-egui-extras-0.33.3
  (crate-source "egui_extras" "0.33.3"
                "1vd49ikj9aqpk2dlj32pcryl2w1f15hnjwpdzpin477h8pl387fh"))

(define rust-egui-kittest-0.33.3
  (crate-source "egui_kittest" "0.33.3"
                "185nbj3zd75rn566yclsyx52q6p1768an2g6yp4fdafzd3wvbbs3"))

(define rust-either-1.16.0
  (crate-source "either" "1.16.0"
                "17k7jfbdz7k440h6lws9baz8p9zlxgb41sig3w81h80nwzsjyqli"))

(define rust-elliptic-curve-0.13.8
  (crate-source "elliptic-curve" "0.13.8"
                "0ixx4brgnzi61z29r3g1606nh2za88hzyz8c5r3p6ydzhqq09rmm"))

(define rust-emath-0.33.3
  (crate-source "emath" "0.33.3"
                "0ckwl0s3xi3zma8hni522g085yl8hp2g2k0dssddspgjidrdy6s9"))

(define rust-embedded-io-0.4.0
  (crate-source "embedded-io" "0.4.0"
                "1v9wrc5nsgaaady7i3ya394sik5251j0iq5rls7mrx7fv696h6pg"))

(define rust-embedded-io-0.6.1
  (crate-source "embedded-io" "0.6.1"
                "0v901xykajh3zffn6x4cnn4fhgfw3c8qpjwbsk6gai3gaccg3l7d"))

(define rust-ena-0.14.4
  (crate-source "ena" "0.14.4"
                "1wfmb3pbgs2h2z4w8mpaf9z32i044qqyqz7gqnavzlabwapgvgza"))

(define rust-encode-unicode-1.0.0
  (crate-source "encode_unicode" "1.0.0"
                "1h5j7j7byi289by63s3w4a8b3g6l5ccdrws7a67nn07vdxj77ail"))

(define rust-encoding-rs-0.8.35
  (crate-source "encoding_rs" "0.8.35"
                "1wv64xdrr9v37rqqdjsyb8l8wzlcbab80ryxhrszvnj59wy0y0vm"))

(define rust-encre-css-0.20.1
  (crate-source "encre-css" "0.20.1"
                "14y04ds7w9csvcxv2h5gd1km74krngmj3dbdkx65ylyacpmrhk61"))

(define rust-endi-1.1.1
  (crate-source "endi" "1.1.1"
                "16a0076dx41vgrzzimm9clcym77h732czqjiajanmzvd1i1y5dv6"))

(define rust-enum-map-2.7.3
  (crate-source "enum-map" "2.7.3"
                "1sgjgl4mmz93jdkfdsmapc3dmaq8gddagw9s0fd501w2vyzz6rk8"))

(define rust-enum-map-derive-0.17.0
  (crate-source "enum-map-derive" "0.17.0"
                "1sv4mb343rsz4lc3rh7cyn0pdhf7fk18k1dgq8kfn5i5x7gwz0pj"))

(define rust-enumflags2-0.7.12
  (crate-source "enumflags2" "0.7.12"
                "1vzcskg4dca2jiflsfx1p9yw1fvgzcakcs7cpip0agl51ilgf9qh"))

(define rust-enumflags2-derive-0.7.12
  (crate-source "enumflags2_derive" "0.7.12"
                "09rqffacafl1b83ir55hrah9gza0x7pzjn6lr6jm76fzix6qmiv7"))

(define rust-enumset-1.1.13
  (crate-source "enumset" "1.1.13"
                "0zfyzvcl260157aysl6qs6w978ln372v443163vwhx8ynis43743"))

(define rust-enumset-derive-0.15.0
  (crate-source "enumset_derive" "0.15.0"
                "0fnnffq4w1vpkvqj61i6zwzcsizzznd1kyxq2yr85ijqgdakdmab"))

(define rust-env-filter-1.0.1
  (crate-source "env_filter" "1.0.1"
                "1vvf9xhaxm0m78bp23b8j3cbv1vm5vffn3gaas27mc64rhm0rs9j"))

(define rust-env-logger-0.11.10
  (crate-source "env_logger" "0.11.10"
                "0smmk1hqzk7z91rg7fdq98d03gh9kidkd0ymim43zb4n457w0886"))

(define rust-epaint-0.33.3
  (crate-source "epaint" "0.33.3"
                "0qmb1dwzmwdg8cdyjpjlxj6pjy5wxi8r92fvmfh26f0nqb9hv780"))

(define rust-equator-0.4.2
  (crate-source "equator" "0.4.2"
                "1z760z5r0haxjyakbqxvswrz9mq7c29arrivgq8y1zldhc9v44a7"))

(define rust-equator-macro-0.4.2
  (crate-source "equator-macro" "0.4.2"
                "1cqzx3cqn9rxln3a607xr54wippzff56zs5chqdf3z2bnks3rwj4"))

(define rust-equivalent-1.0.2
  (crate-source "equivalent" "1.0.2"
                "03swzqznragy8n0x31lqc78g2af054jwivp7lkrbrc0khz74lyl7"))

(define rust-erased-serde-0.4.10
  (crate-source "erased-serde" "0.4.10"
                "1v1dy16ff8mck2rfqdmwdxl14phlvr8rq0i7yqzxka6ngnhdibfj"))

(define rust-errno-0.3.14
  (crate-source "errno" "0.3.14"
                "1szgccmh8vgryqyadg8xd58mnwwicf39zmin3bsn63df2wbbgjir"))

(define rust-error-code-3.3.2
  (crate-source "error-code" "3.3.2"
                "0nacxm9xr3s1rwd6fabk3qm89fyglahmbi4m512y0hr8ym6dz8ny"))

(define rust-event-listener-5.4.1
  (crate-source "event-listener" "5.4.1"
                "1asnp3agbr8shcl001yd935m167ammyi8hnvl0q1ycajryn6cfz1"))

(define rust-event-listener-strategy-0.5.4
  (crate-source "event-listener-strategy" "0.5.4"
                "14rv18av8s7n8yixg38bxp5vg2qs394rl1w052by5npzmbgz7scb"))

(define rust-exr-1.74.0
  (crate-source "exr" "1.74.0"
                "1gk3cc2qkfm0jqw4v1d7g4c356k9iz583bq17iiwp8kalm1y0023"))

(define rust-fallible-iterator-0.3.0
  (crate-source "fallible-iterator" "0.3.0"
                "0ja6l56yka5vn4y4pk6hn88z0bpny7a8k1919aqjzp0j1yhy9k1a"))

(define rust-fancy-regex-0.17.0
  (crate-source "fancy-regex" "0.17.0"
                "1f314z64ilbbnn17ic1hghpq9dm2sqyn8gspvjvjp1jwhqgldkvj"))

(define rust-fast-float2-0.2.3
  (crate-source "fast-float2" "0.2.3"
                "0mbadcgq221clfpihsfiahizfsgfwk8n3dbgi1fd48vlbi65dszq"))

(define rust-fastbloom-0.14.1
  (crate-source "fastbloom" "0.14.1"
                "1i639bpknnr5vfs8lvc1g3ra30gzmy68mxgax07wcsdy5m238zsf"))

(define rust-fastcdc-3.2.1
  (crate-source "fastcdc" "3.2.1"
                "1g765hzrcq9dg4iq22w243i61zaxmy160vswvpjbzbwn7sscwldz"))

(define rust-fastdivide-0.4.2
  (crate-source "fastdivide" "0.4.2"
                "0w84d21qk2l8vcyvkh521drk4bqw839p63fiagfhccd7spa2pz4s"))

(define rust-faster-hex-0.10.0
  (crate-source "faster-hex" "0.10.0"
                "0wzvv4a1czxfxmh99cza2y0jps97hm3k1j6r6cs816qp5wnsw8vj"))

(define rust-fastrand-2.4.1
  (crate-source "fastrand" "2.4.1"
                "1mnqxxnxvd69ma9mczabpbbsgwlhd6l78yv3vd681453a9s247wz"))

(define rust-fax-0.2.7
  (crate-source "fax" "0.2.7"
                "0nmc65jjdym0f7lr4qm2q7awz1p5arm8i19wv1cmsg92cfahgwfa"))

(define rust-fdeflate-0.3.7
  (crate-source "fdeflate" "0.3.7"
                "130ga18vyxbb5idbgi07njymdaavvk6j08yh1dfarm294ssm6s0y"))

(define rust-ff-0.13.1
  (crate-source "ff" "0.13.1"
                "14v3bc6q24gbcjnxjfbq2dddgf4as2z2gd4mj35gjlrncpxhpdf0"))

(define rust-fiat-crypto-0.2.9
  (crate-source "fiat-crypto" "0.2.9"
                "07c1vknddv3ak7w89n85ik0g34nzzpms6yb845vrjnv9m4csbpi8"))

(define rust-filetime-0.2.29
  (crate-source "filetime" "0.2.29"
                "0napyyfccb26r7fyh9hg7ixrh4vph9h7y7k4iv1j19phqwrpla2w"))

(define rust-find-msvc-tools-0.1.9
  (crate-source "find-msvc-tools" "0.1.9"
                "10nmi0qdskq6l7zwxw5g56xny7hb624iki1c39d907qmfh3vrbjv"))

(define rust-fixedbitset-0.4.2
  (crate-source "fixedbitset" "0.4.2"
                "101v41amgv5n9h4hcghvrbfk5vrncx1jwm35rn5szv4rk55i7rqc"))

(define rust-fixedbitset-0.5.7
  (crate-source "fixedbitset" "0.5.7"
                "16fd3v9d2cms2vddf9xhlm56sz4j0zgrk3d2h6v1l7hx760lwrqx"))

(define rust-flate2-1.1.9
  (crate-source "flate2" "1.1.9"
                "0g2pb7cxnzcbzrj8bw4v6gpqqp21aycmf6d84rzb6j748qkvlgw4"))

(define rust-float16-0.1.5
  (crate-source "float16" "0.1.5"
                "10w4zwbrdw4zclzps5pldhk02xkmrzlrlxy2qy8h2llx0yyszzvv"))

(define rust-fluent-0.17.0
  (crate-source "fluent" "0.17.0"
                "0xq4cxw4mkdh1k9i5w850sky0m41la8sm6nbpw76n3f5lbascdw1"))

(define rust-fluent-bundle-0.15.3
  (crate-source "fluent-bundle" "0.15.3"
                "14zl0cjn361is69pb1zry4k2zzh5nzsfv0iz05wccl00x0ga5q3z"))

(define rust-fluent-bundle-0.16.0
  (crate-source "fluent-bundle" "0.16.0"
                "1x1v8bmym6x9pl87f82lbzwlc84kdn0lgcwi73ki2mwgj6w3q801"))

(define rust-fluent-langneg-0.13.1
  (crate-source "fluent-langneg" "0.13.1"
                "1c78jl8lpwg5hdg589qbn3m9ls6mzqxnyrvi5llfibhb8mcvxsvy"))

(define rust-fluent-syntax-0.11.1
  (crate-source "fluent-syntax" "0.11.1"
                "0gd3cdvsx9ymbb8hijcsc9wyf8h1pbcbpsafg4ldba56ji30qlra"))

(define rust-fluent-syntax-0.12.0
  (crate-source "fluent-syntax" "0.12.0"
                "1661sp6kl268n445x7jjhnbkgiaa1xcpyryq0i6iiz9zqn3x5w2l"))

(define rust-fluent-template-macros-0.13.3
  (crate-source "fluent-template-macros" "0.13.3"
                "0v3cd26jnb3mjbmznnixwxiwk28k0ag8xzzdd9b7pnbgzfrm103l"))

(define rust-fluent-templates-0.13.3
  (crate-source "fluent-templates" "0.13.3"
                "0xllpndnwkd5s7v2nj2j1y5gcdnplka5zk7rmrll8h0zl13489jn"))

(define rust-flume-0.11.1
  (crate-source "flume" "0.11.1"
                "15ch0slxa8sqsi6c73a0ky6vdnh48q8cxjf7rksa3243m394s3ns"))

(define rust-fnv-1.0.7
  (crate-source "fnv" "1.0.7"
                "1hc2mcqha06aibcaza94vbi81j6pr9a1bbxrxjfhc91zin8yr7iz"))

(define rust-foldhash-0.1.5
  (crate-source "foldhash" "0.1.5"
                "1wisr1xlc2bj7hk4rgkcjkz3j2x4dhd1h9lwk7mj8p71qpdgbi6r"))

(define rust-foldhash-0.2.0
  (crate-source "foldhash" "0.2.0"
                "1nvgylb099s11xpfm1kn2wcsql080nqmnhj1l25bp3r2b35j9kkp"))

(define rust-foreign-types-0.5.0
  (crate-source "foreign-types" "0.5.0"
                "0rfr2zfxnx9rz3292z5nyk8qs2iirznn5ff3rd4vgdwza6mdjdyp"))

(define rust-foreign-types-macros-0.2.3
  (crate-source "foreign-types-macros" "0.2.3"
                "0hjpii8ny6l7h7jpns2cp9589016l8mlrpaigcnayjn9bdc6qp0s"))

(define rust-foreign-types-shared-0.3.1
  (crate-source "foreign-types-shared" "0.3.1"
                "0nykdvv41a3d4py61bylmlwjhhvdm0b3bcj9vxhqgxaxnp5ik6ma"))

(define rust-fork-0.6.0
  (crate-source "fork" "0.6.0"
                "0alznwj8plk9hgdij892by12cbiz01ccqf3y21pmvlslias4ww99"))

(define rust-form-urlencoded-1.2.2
  (crate-source "form_urlencoded" "1.2.2"
                "1kqzb2qn608rxl3dws04zahcklpplkd5r1vpabwga5l50d2v4k6b"))

(define rust-from-variant-3.0.0
  (crate-source "from_variant" "3.0.0"
                "1aay6hgrcyyhkglhcq8a8rirygcv4s8dch031894kydfj6ikbzz5"))

(define rust-fs-err-3.3.0
  (crate-source "fs-err" "3.3.0"
                "1h08mdjhdv3c48j3m32kj487rk6lwv3f5j6jrw1h14pwvd9f1zbk"))

(define rust-fs-extra-1.3.0
  (crate-source "fs_extra" "1.3.0"
                "075i25z70j2mz9r7i9p9r521y8xdj81q7skslyb7zhqnnw33fw22"))

(define rust-fs2-0.4.3
  (crate-source "fs2" "0.4.3"
                "04v2hwk7035c088f19mfl5b1lz84gnvv2hv6m935n0hmirszqr4m"))

(define rust-fs4-0.13.1
  (crate-source "fs4" "0.13.1"
                "1m0y2kmwzifkrivw7gjav0km5s9agaiv324yrq424rgpi15y6h46"))

(define rust-funty-2.0.0
  (crate-source "funty" "2.0.0"
                "177w048bm0046qlzvp33ag3ghqkqw4ncpzcm5lq36gxf2lla7mg6"))

(define rust-futures-0.3.32
  (crate-source "futures" "0.3.32"
                "0b9q86r5ar18v5xjiyqn7sb8sa32xv98qqnfz779gl7ns7lpw54b"))

(define rust-futures-channel-0.3.32
  (crate-source "futures-channel" "0.3.32"
                "07fcyzrmbmh7fh4ainilf1s7gnwvnk07phdq77jkb9fpa2ffifq7"))

(define rust-futures-concurrency-7.7.1
  (crate-source "futures-concurrency" "7.7.1"
                "19lfx85mc4p15pj86bsyzv2f7690iw47bylgy63mpm71m76dhp0p"))

(define rust-futures-core-0.3.32
  (crate-source "futures-core" "0.3.32"
                "07bbvwjbm5g2i330nyr1kcvjapkmdqzl4r6mqv75ivvjaa0m0d3y"))

(define rust-futures-executor-0.3.32
  (crate-source "futures-executor" "0.3.32"
                "17aplz3ns74qn7a04qg7qlgsdx5iwwwkd4jvdfra6hl3h4w9rwms"))

(define rust-futures-io-0.3.32
  (crate-source "futures-io" "0.3.32"
                "063pf5m6vfmyxj74447x8kx9q8zj6m9daamj4hvf49yrg9fs7jyf"))

(define rust-futures-lite-2.6.1
  (crate-source "futures-lite" "2.6.1"
                "1ba4dg26sc168vf60b1a23dv1d8rcf3v3ykz2psb7q70kxh113pp"))

(define rust-futures-macro-0.3.32
  (crate-source "futures-macro" "0.3.32"
                "0ys4b1lk7s0bsj29pv42bxsaavalch35rprp64s964p40c1bfdg8"))

(define rust-futures-sink-0.3.32
  (crate-source "futures-sink" "0.3.32"
                "14q8ml7hn5a6gyy9ri236j28kh0svqmrk4gcg0wh26rkazhm95y3"))

(define rust-futures-task-0.3.32
  (crate-source "futures-task" "0.3.32"
                "14s3vqf8llz3kjza33vn4ixg6kwxp61xrysn716h0cwwsnri2xq3"))

(define rust-futures-timer-3.0.4
  (crate-source "futures-timer" "3.0.4"
                "0s39in8ivw7g4d37pf31q02y44zd1hpfkd1pgra2slcqibdzlhxg"))

(define rust-futures-util-0.3.32
  (crate-source "futures-util" "0.3.32"
                "1mn60lw5kh32hz9isinjlpw34zx708fk5q1x0m40n6g6jq9a971q"))

(define rust-genawaiter-0.99.1
  (crate-source "genawaiter" "0.99.1"
                "1861a6vy9lc9a8lbw496m9j9jcjcn9nf7rkm6jqkkpnb3cvd0sy8"))

(define rust-genawaiter-macro-0.99.1
  (crate-source "genawaiter-macro" "0.99.1"
                "1g6zmr88fk48f1ksz9ik1i2mwjsiam9s4p9aybhvs2zwzphxychb"))

(define rust-generator-0.8.9
  (crate-source "generator" "0.8.9"
                "1bhk2m8alf9nfmmq2y2whyriigppgjnzrchq7yix3sl4wnq59f5k"))

(define rust-generic-array-0.14.7
  (crate-source "generic-array" "0.14.7"
                "16lyyrzrljfq424c3n8kfwkqihlimmsg5nhshbbp48np3yjrqr45"))

(define rust-gethostname-1.1.0
  (crate-source "gethostname" "1.1.0"
                "1n6bj9gh503ggjblfjcai96gmxynxsrykaynljlrfdra34q95m0v"))

(define rust-getopts-0.2.24
  (crate-source "getopts" "0.2.24"
                "1pylvsmq7fillnxmd6g58r7igdrlby412q37ws41z39va2ngpr6g"))

(define rust-getrandom-0.2.17
  (crate-source "getrandom" "0.2.17"
                "1l2ac6jfj9xhpjjgmcx6s1x89bbnw9x6j9258yy6xjkzpq0bqapz"))

(define rust-getrandom-0.3.4
  (crate-source "getrandom" "0.3.4"
                "1zbpvpicry9lrbjmkd4msgj3ihff1q92i334chk7pzf46xffz7c9"))

(define rust-getrandom-0.4.2
  (crate-source "getrandom" "0.4.2"
                "0mb5833hf9pvn9dhvxjgfg5dx0m77g8wavvjdpvpnkp9fil1xr8d"))

(define rust-getset-0.1.6
  (crate-source "getset" "0.1.6"
                "04pr6qj9xf5krk1sqkwpn84zhk4z46y7fj8mjxrx8qbmwh8zrw4w"))

(define rust-ghash-0.5.1
  (crate-source "ghash" "0.5.1"
                "1wbg4vdgzwhkpkclz1g6bs4r5x984w5gnlsj4q5wnafb5hva9n7h"))

(define rust-ghash-0.6.0
  (crate-source "ghash" "0.6.0"
                "1mg8nf20qz3pmf9k2xzb4c2x7c8614hs01vpp4rbfrlvvkaz5v1f"))

(define rust-gif-0.14.2
  (crate-source "gif" "0.14.2"
                "0n81js7vlb9bwrjb765sicza3k0vrihjddrgm2mvpbfr272gr37f"))

(define rust-gimli-0.32.3
  (crate-source "gimli" "0.32.3"
                "1iqk5xznimn5bfa8jy4h7pa1dv3c624hzgd2dkz8mpgkiswvjag6"))

(define rust-gix-0.77.0
  (crate-source "gix" "0.77.0"
                "1k3q5cydhxkaxizgvx8qbaph9k0mi099l8gpzf3hjp1gdbc890ix"))

(define rust-gix-actor-0.37.1
  (crate-source "gix-actor" "0.37.1"
                "1i2mm9yq55xydcn2kq4l6sap62b8lkhmypsh1z953asy826m4if3"))

(define rust-gix-attributes-0.29.0
  (crate-source "gix-attributes" "0.29.0"
                "0rjr27v9dg7dnh1yyyw9mj3jzwn4qx0895sxlp1mh58glpwanzgl"))

(define rust-gix-bitmap-0.2.16
  (crate-source "gix-bitmap" "0.2.16"
                "1qj1pxxqb97ja6jdms17b86wcx5f3laadlnha6c6d3k0y1zgr0nr"))

(define rust-gix-chunk-0.4.12
  (crate-source "gix-chunk" "0.4.12"
                "1swf50dk3i9gbq8bsg25hkhj2658261vnlcmazzvcz374lw6ndaw"))

(define rust-gix-command-0.6.5
  (crate-source "gix-command" "0.6.5"
                "0r2wil9m2h954z89ckldid7q18cvqv1shv8y6lslhr8afcjw9ya6"))

(define rust-gix-commitgraph-0.31.0
  (crate-source "gix-commitgraph" "0.31.0"
                "0fp6mf271lpvlwi5w91hs61n2gkwb6fr9bsx48asynq4920bmp7g"))

(define rust-gix-config-0.50.0
  (crate-source "gix-config" "0.50.0"
                "013lqvx6knlvlb2mqz02fkrsyp27l0ng0q72qpr72szrxvw2z3mm"))

(define rust-gix-config-value-0.16.0
  (crate-source "gix-config-value" "0.16.0"
                "0h4qwzymmb0cx9sf8ppw0b9nbfm9kpgshssvgn207cz89zxcy294"))

(define rust-gix-date-0.12.1
  (crate-source "gix-date" "0.12.1"
                "1ryqz14al79806pfcrrzb6gg3iiyzpjx4w7sjhq277hmp2x32jpy"))

(define rust-gix-diff-0.57.1
  (crate-source "gix-diff" "0.57.1"
                "142fkcjsjwf21g63h8hwykwjxfa3irnd13pjnmacs56fcdp961im"))

(define rust-gix-dir-0.19.0
  (crate-source "gix-dir" "0.19.0"
                "01hya3ifj8vc6m0hqzl7snv03s39clj78245540qpsyj6anrz7bh"))

(define rust-gix-discover-0.45.0
  (crate-source "gix-discover" "0.45.0"
                "1pjrngnkj0hdkvkf5ps1hc1z4n4803adavwxl013hlrjq5nhkkj2"))

(define rust-gix-features-0.45.2
  (crate-source "gix-features" "0.45.2"
                "1lb0fn89xbzk7anila7izhyjbijackgk6l3h6ja485p0g8ssssnm"))

(define rust-gix-filter-0.24.1
  (crate-source "gix-filter" "0.24.1"
                "1p15k9ica0idcrs80gl09x8r2hwcc0d4bbcl3w65g0i4jrj29h0h"))

(define rust-gix-fs-0.18.2
  (crate-source "gix-fs" "0.18.2"
                "1v2rsd8cw6gdasl2krwy6xaqd8gwnchqq50wp3bpig26kr4rqnvq"))

(define rust-gix-glob-0.23.0
  (crate-source "gix-glob" "0.23.0"
                "1qmp2942iaa6wl6vda3199jrp9i424r3wan2c9c5rip4mq066m78"))

(define rust-gix-hash-0.21.2
  (crate-source "gix-hash" "0.21.2"
                "0kbiwz55c3lbs5sc9l3ck6lgcya8ab6jf43b62ivinnc887r6lz1"))

(define rust-gix-hashtable-0.11.0
  (crate-source "gix-hashtable" "0.11.0"
                "1w6j5dh5mfvgw4m8b69z36rmzgs8x8rxhzm8fbrbw830ccl78br2"))

(define rust-gix-ignore-0.18.0
  (crate-source "gix-ignore" "0.18.0"
                "088vp0jkxb1wbwavydp3x1rk0b8p2xf1mfrzz99zpnagypyjg9yz"))

(define rust-gix-index-0.45.1
  (crate-source "gix-index" "0.45.1"
                "0l90yibnjkh9v55glhzzfll6vk4l4iwa1pj1yi4vliqnw7lx79ly"))

(define rust-gix-lock-20.0.1
  (crate-source "gix-lock" "20.0.1"
                "1i9s3al7yiimbnj8qca5ir2wmb05xvn0w9kpzk3pnfrvbsp6hlhi"))

(define rust-gix-object-0.54.1
  (crate-source "gix-object" "0.54.1"
                "10dws3l802w6iqvalz40gq5zsr53r7cagzz0h00qkr2jkj3nlg9n"))

(define rust-gix-odb-0.74.0
  (crate-source "gix-odb" "0.74.0"
                "1if0pj83vya9hvnv9b6g13dsm5z5gapqpbqg6ga2x8b9ydyr0nhn"))

(define rust-gix-pack-0.64.1
  (crate-source "gix-pack" "0.64.1"
                "0rj3rn4jqfpf9lwl1qccmijy6djfybk0lb2ywnm0zsh7mgap6jmh"))

(define rust-gix-packetline-0.20.0
  (crate-source "gix-packetline" "0.20.0"
                "0cfyqywjrpahn0v14hx5597p5b6b97ld6rd1hy08i2d2hawzzl7s"))

(define rust-gix-path-0.10.22
  (crate-source "gix-path" "0.10.22"
                "0rpksdgf0wv6w6x6irx09qgm32p28lqsjpwizlj6xvcf9wz6rc3w"))

(define rust-gix-pathspec-0.14.0
  (crate-source "gix-pathspec" "0.14.0"
                "1180p6g85ng1axvc7d20s8x7njlwfz2xd22jyiz7mhrk3640r7pd"))

(define rust-gix-protocol-0.55.0
  (crate-source "gix-protocol" "0.55.0"
                "01sd7wv297bl2c5by4csq3r6a5w55ps2ww4yf32l553qd38dzi82"))

(define rust-gix-quote-0.6.2
  (crate-source "gix-quote" "0.6.2"
                "0mv7qgy955378bf163c1qagn20pb00gsnbph0wlckh4cxkr2zz4n"))

(define rust-gix-ref-0.57.0
  (crate-source "gix-ref" "0.57.0"
                "0qwarw34p0rmvxn41c6ldz8jxc36k9b37qpxhfg7xqq6f2lkmcyc"))

(define rust-gix-refspec-0.35.0
  (crate-source "gix-refspec" "0.35.0"
                "00nsaxgnhg8xa8jlns5vx2qpvkmajm0jmmm2fcgh5x49afpadfyw"))

(define rust-gix-revision-0.39.0
  (crate-source "gix-revision" "0.39.0"
                "1j64xcsqjxk7yn0rby1g08w2z9blz8f1fp9myyb5cqwcn61qr2ci"))

(define rust-gix-revwalk-0.25.0
  (crate-source "gix-revwalk" "0.25.0"
                "1clyjjbgs1mz2ybkg2cspbld57v0nw6vplnhcdl031c44yckc1hd"))

(define rust-gix-sec-0.12.2
  (crate-source "gix-sec" "0.12.2"
                "1gm6zwymkb03i64a41xmbci3qa215xskiq7g03qzf54idpnn56ga"))

(define rust-gix-shallow-0.7.0
  (crate-source "gix-shallow" "0.7.0"
                "0mi3hdaikiy4rn9sf8l29fwbq5755m4aabiwc4rivv7pp5zlc74w"))

(define rust-gix-status-0.24.0
  (crate-source "gix-status" "0.24.0"
                "1llnjsxbqn6q0357gwywa0zj730f6lpw4m2lr9wwccd8hp3983gd"))

(define rust-gix-submodule-0.24.0
  (crate-source "gix-submodule" "0.24.0"
                "0694xhdp4c01fs0ygbbnswkphls4fd8ala00w46xh4w435hjmvpg"))

(define rust-gix-tempfile-20.0.1
  (crate-source "gix-tempfile" "20.0.1"
                "0h6xn1fcqzl7pf8xy0imag47810z573pff7dck9l43w5fj7232dd"))

(define rust-gix-trace-0.1.20
  (crate-source "gix-trace" "0.1.20"
                "0y6n39aay8pd7ad9k9mn8m6wyk92dqp1a3ry2wafph45wzm4bp24"))

(define rust-gix-transport-0.52.1
  (crate-source "gix-transport" "0.52.1"
                "13cinjmw2lbjy36c1ng16pnmi2xrl36nx28ic6i73rzbl81fvm54"))

(define rust-gix-traverse-0.51.1
  (crate-source "gix-traverse" "0.51.1"
                "102jkzilcwg5kjz6dr10cy7hldszz41aqj34mjavwi0p3lyvhlnh"))

(define rust-gix-url-0.34.0
  (crate-source "gix-url" "0.34.0"
                "1nbkzszvih031brjpidcbqssxhb985klq8l9kmlv6c4lzdnrkwfg"))

(define rust-gix-utils-0.3.3
  (crate-source "gix-utils" "0.3.3"
                "0wdpn67c6rlmw67ypz67y6bqb1qs0cl4x9pzh3swl8s131k0kib6"))

(define rust-gix-validate-0.10.1
  (crate-source "gix-validate" "0.10.1"
                "1r7xvdhvvf0dl83x8ysf6j5cpzd8f52ysw7qjjjp1s8nnnjn67jv"))

(define rust-gix-worktree-0.46.0
  (crate-source "gix-worktree" "0.46.0"
                "1jka6b0lgdnf138sy2hy6ch8sijijcravl9mscbn3q5zrpl7ryqw"))

(define rust-glob-0.3.3
  (crate-source "glob" "0.3.3"
                "106jpd3syfzjfj2k70mwm0v436qbx96wig98m4q8x071yrq35hhc"))

(define rust-globset-0.4.18
  (crate-source "globset" "0.4.18"
                "1qsp3wg0mgxzmshcgymdlpivqlc1bihm6133pl6dx2x4af8w3psj"))

(define rust-group-0.13.0
  (crate-source "group" "0.13.0"
                "0qqs2p5vqnv3zvq9mfjkmw3qlvgqb0c3cm6p33srkh7pc9sfzygh"))

(define rust-h2-0.4.14
  (crate-source "h2" "0.4.14"
                "0cw7jk7kn2vn6f8w8ssh6gis1mljnfjxd606gvi4sjpyjayfy7qp"))

(define rust-half-2.7.1
  (crate-source "half" "2.7.1"
                "0jyq42xfa6sghc397mx84av7fayd4xfxr4jahsqv90lmjr5xi8kf"))

(define rust-handlebars-6.4.1
  (crate-source "handlebars" "6.4.1"
                "000g9df39apgqxp5npwwzmma9j9788jr17k3mylb06m82pzcsg6l"))

(define rust-hash32-0.2.1
  (crate-source "hash32" "0.2.1"
                "0rrbv5pc5b1vax6j6hk7zvlrpw0h6aybshxy9vbpgsrgfrc5zhxh"))

(define rust-hash32-0.3.1
  (crate-source "hash32" "0.3.1"
                "01h68z8qi5gl9lnr17nz10lay8wjiidyjdyd60kqx8ibj090pmj7"))

(define rust-hashbrown-0.12.3
  (crate-source "hashbrown" "0.12.3"
                "1268ka4750pyg2pbgsr43f0289l5zah4arir2k4igx5a8c6fg7la"))

(define rust-hashbrown-0.14.5
  (crate-source "hashbrown" "0.14.5"
                "1wa1vy1xs3mp11bn3z9dv0jricgr6a2j0zkf1g19yz3vw4il89z5"))

(define rust-hashbrown-0.15.5
  (crate-source "hashbrown" "0.15.5"
                "189qaczmjxnikm9db748xyhiw04kpmhm9xj9k9hg0sgx7pjwyacj"))

(define rust-hashbrown-0.16.1
  (crate-source "hashbrown" "0.16.1"
                "004i3njw38ji3bzdp9z178ba9x3k0c1pgy8x69pj7yfppv4iq7c4"))

(define rust-hashbrown-0.17.1
  (crate-source "hashbrown" "0.17.1"
                "0jmqz7i4yl6cm7rbn0i2ffkfrmwi6xkmzkaldr2v8bcsx2v0jngd"))

(define rust-heapless-0.7.17
  (crate-source "heapless" "0.7.17"
                "0kwn2wzk9fnsqnwp6rqjqhvh6hfq4rh225xwqjm72b5n1ry4bind"))

(define rust-heapless-0.8.0
  (crate-source "heapless" "0.8.0"
                "1b9zpdjv4qkl2511s2c80fz16fx9in4m9qkhbaa8j73032v9xyqb"))

(define rust-heck-0.4.1
  (crate-source "heck" "0.4.1"
                "1a7mqsnycv5z4z5vnv1k34548jzmc0ajic7c1j8jsaspnhw5ql4m"))

(define rust-heck-0.5.0
  (crate-source "heck" "0.5.0"
                "1sjmpsdl8czyh9ywl3qcsfsq9a307dg4ni2vnlwgnzzqhc4y0113"))

(define rust-hermit-abi-0.5.2
  (crate-source "hermit-abi" "0.5.2"
                "1744vaqkczpwncfy960j2hxrbjl1q01csm84jpd9dajbdr2yy3zw"))

(define rust-hex-0.4.3
  (crate-source "hex" "0.4.3"
                "0w1a4davm1lgzpamwnba907aysmlrnygbqmfis2mqjx5m552a93z"))

(define rust-hifijson-0.2.3
  (crate-source "hifijson" "0.2.3"
                "038g1xbdrrsc1dcd2mb41mj6qrz7jyqrmgwqwrclz8m8ifwn6xqa"))

(define rust-hipstr-0.6.0
  (crate-source "hipstr" "0.6.0"
                "1mn6qij1nwvwn92v1z95nvmr8lizlj9fj2165vhqvjflhpy1z5wp"))

(define rust-home-0.5.12
  (crate-source "home" "0.5.12"
                "13bjyzgx6q9srnfvl43dvmhn93qc8mh5w7cylk2g13sj3i3pyqnc"))

(define rust-hstr-3.0.6
  (crate-source "hstr" "3.0.6"
                "1qnvq9q0fkcrwl32np0m3fsm453lwwp05iywyq939mq0ngj8gfw3"))

(define rust-html-escape-0.2.13
  (crate-source "html-escape" "0.2.13"
                "0xml3hswv0205fbm5iq7dqiwjkr6d245xkfppwi7wqjdfr4x86kd"))

(define rust-html2text-0.16.7
  (crate-source "html2text" "0.16.7"
                "11z7wszhgss41hhxpd9g37vb09zzavdb5yj8mlvnpgjdx9b33lhj"))

(define rust-html5ever-0.38.0
  (crate-source "html5ever" "0.38.0"
                "1hnbs7d7v26gdgf6mm8rschsjrxazc139lik3q3f051gmqml6m0h"))

(define rust-html5ever-0.39.0
  (crate-source "html5ever" "0.39.0"
                "1f5pphabfbywvvf6xy86cc31803182zlp546kshwkk7s0wc7d8a6"))

(define rust-htmlescape-0.3.1
  (crate-source "htmlescape" "0.3.1"
                "0qria8paf19qy5sgzzk3iiii9fp2j7spbhqf0zjxwrg7v9c500p9"))

(define rust-http-1.4.1
  (crate-source "http" "1.4.1"
                "1l7k2ia57z3q7q3ka497krzps795kd3fymm2k12lr623y4nldrwb"))

(define rust-http-body-1.0.1
  (crate-source "http-body" "1.0.1"
                "111ir5k2b9ihz5nr9cz7cwm7fnydca7dx4hc7vr16scfzghxrzhy"))

(define rust-http-body-util-0.1.3
  (crate-source "http-body-util" "0.1.3"
                "0jm6jv4gxsnlsi1kzdyffjrj8cfr3zninnxpw73mvkxy4qzdj8dh"))

(define rust-httparse-1.10.1
  (crate-source "httparse" "1.10.1"
                "11ycd554bw2dkgw0q61xsa7a4jn1wb1xbfacmf3dbwsikvkkvgvd"))

(define rust-httpdate-1.0.3
  (crate-source "httpdate" "1.0.3"
                "1aa9rd2sac0zhjqh24c9xvir96g188zldkx0hr6dnnlx5904cfyz"))

(define rust-humantime-2.3.0
  (crate-source "humantime" "2.3.0"
                "092lpipp32ayz4kyyn4k3vz59j9blng36wprm5by0g2ykqr14nqk"))

(define rust-hybrid-array-0.4.12
  (crate-source "hybrid-array" "0.4.12"
                "1njpm3mmsb6lgr9nn97ld5aavwjzrvijjb4nav0anhnimf1aamci"))

(define rust-hyper-1.10.1
  (crate-source "hyper" "1.10.1"
                "1624nwrh1ci34psqcl3q8q266kha8kd6fmqjj14qck49l59iqa2m"))

(define rust-hyper-rustls-0.27.9
  (crate-source "hyper-rustls" "0.27.9"
                "03vfnsm873wsp1dk0q85nxvk7w6syp8c2m5bcdjcyfgg4786ijik"))

(define rust-hyper-util-0.1.20
  (crate-source "hyper-util" "0.1.20"
                "186zdc58hmm663csmjvrzgkr6jdh93sfmi3q2pxi57gcaqjpqm4n"))

(define rust-iana-time-zone-0.1.65
  (crate-source "iana-time-zone" "0.1.65"
                "0w64khw5p8s4nzwcf36bwnsmqzf61vpwk9ca1920x82bk6nwj6z3"))

(define rust-iana-time-zone-haiku-0.1.2
  (crate-source "iana-time-zone-haiku" "0.1.2"
                "17r6jmj31chn7xs9698r122mapq85mfnv98bb4pg6spm0si2f67k"))

(define rust-icu-calendar-2.2.1
  (crate-source "icu_calendar" "2.2.1"
                "0i9y9ydaw66m4fff6vswgpj6jy75z0zvb186ylflyj9z4v3arcm2"))

(define rust-icu-calendar-data-2.2.0
  (crate-source "icu_calendar_data" "2.2.0"
                "0kdgxy6b044d9pxnh7wvdgjlxa2dh58ykmm7q1m7rym0yfy7g18i"))

(define rust-icu-collections-2.2.0
  (crate-source "icu_collections" "2.2.0"
                "070r7xd0pynm0hnc1v2jzlbxka6wf50f81wybf9xg0y82v6x3119"))

(define rust-icu-locale-2.2.0
  (crate-source "icu_locale" "2.2.0"
                "09ifkafdqk4rci4x3kqkfr5826gy7lyn4dbfr0fi423j7hs9d8ym"))

(define rust-icu-locale-core-2.2.0
  (crate-source "icu_locale_core" "2.2.0"
                "0a9cmin5w1x3bg941dlmgszn33qgq428k7qiqn5did72ndi9n8cj"))

(define rust-icu-locale-data-2.2.0
  (crate-source "icu_locale_data" "2.2.0"
                "14srd4pisigvfwxcwvxi0chg6shx33rmxrpnbkzp8vbwqydcrzfm"))

(define rust-icu-normalizer-2.2.0
  (crate-source "icu_normalizer" "2.2.0"
                "1d7krxr0xpc4x9635k1100a24nh0nrc59n65j6yk6gbfkplmwvn5"))

(define rust-icu-normalizer-data-2.2.0
  (crate-source "icu_normalizer_data" "2.2.0"
                "0f5d5d5fhhr9937m2z6z38fzh6agf14z24kwlr6lyczafypf0fys"))

(define rust-icu-properties-2.2.0
  (crate-source "icu_properties" "2.2.0"
                "1pkh3s837808cbwxvfagwc28cvwrz2d9h5rl02jwrhm51ryvdqxy"))

(define rust-icu-properties-data-2.2.0
  (crate-source "icu_properties_data" "2.2.0"
                "052awny0qwkbcbpd5jg2cd7vl5ry26pq4hz1nfsgf10c3qhbnawf"))

(define rust-icu-provider-2.2.0
  (crate-source "icu_provider" "2.2.0"
                "08dl8pxbwr8zsz4c5vphqb7xw0hykkznwi4rw7bk6pwb3krlr70k"))

(define rust-id-arena-2.3.0
  (crate-source "id-arena" "2.3.0"
                "0m6rs0jcaj4mg33gkv98d71w3hridghp5c4yr928hplpkgbnfc1x"))

(define rust-ident-case-1.0.1
  (crate-source "ident_case" "1.0.1"
                "0fac21q6pwns8gh1hz3nbq15j8fi441ncl6w4vlnd1cmc55kiq5r"))

(define rust-idna-1.1.0
  (crate-source "idna" "1.1.0"
                "1pp4n7hppm480zcx411dsv9wfibai00wbpgnjj4qj0xa7kr7a21v"))

(define rust-idna-adapter-1.2.1
  (crate-source "idna_adapter" "1.2.1"
                "0i0339pxig6mv786nkqcxnwqa87v4m94b2653f6k3aj0jmhfkjis"))

(define rust-if-chain-1.0.3
  (crate-source "if_chain" "1.0.3"
                "1jzm319jbb3lbm5vdsxjjyih3g3a1a405pmiipmyxa3fx2sycqnd"))

(define rust-ignore-0.4.26
  (crate-source "ignore" "0.4.26"
                "0zg65dcwq8qnni4jg3iqj8vpnln6pivj8nr6a18g1cqxs0fnc5dr"))

(define rust-image-0.25.10
  (crate-source "image" "0.25.10"
                "0131b9fsd5grxf3lchfs2ci0rg8ga2mh1ygai7k2zh1k8cwq1aw5"))

(define rust-image-webp-0.2.4
  (crate-source "image-webp" "0.2.4"
                "1hz814csyi9283vinzlkix6qpnd6hs3fkw7xl6z2zgm4w7rrypjj"))

(define rust-imara-diff-0.1.8
  (crate-source "imara-diff" "0.1.8"
                "1lmk5dpha2fhahrnsrgavxn1qz6ydp1w8jz8fpvlb28p89ylplqp"))

(define rust-imgref-1.12.2
  (crate-source "imgref" "1.12.2"
                "1msc8g8x8a9dy3l85ila4sijvnhr1rxrxsbjhqk1bawkm64lc6c9"))

(define rust-include-dir-0.7.4
  (crate-source "include_dir" "0.7.4"
                "1pfh3g45z88kwq93skng0n6g3r7zkhq9ldqs9y8rvr7i11s12gcj"))

(define rust-include-dir-macros-0.7.4
  (crate-source "include_dir_macros" "0.7.4"
                "0x8smnf6knd86g69p19z5lpfsaqp8w0nx14kdpkz1m8bxnkqbavw"))

(define rust-indexmap-1.9.3
  (crate-source "indexmap" "1.9.3"
                "16dxmy7yvk51wvnih3a3im6fp5lmx0wx76i03n06wyak6cwhw1xx"))

(define rust-indexmap-2.14.0
  (crate-source "indexmap" "2.14.0"
                "1na9z6f0d5pkjr1lgsni470v98gv2r7c41j8w48skr089x2yjrnl"))

(define rust-indicatif-0.17.11
  (crate-source "indicatif" "0.17.11"
                "0db2b2r79r9x8x4lysq1ci9xm13c0xg0sqn3z960yh2bk2430fqq"))

(define rust-inout-0.1.4
  (crate-source "inout" "0.1.4"
                "008xfl1jn9rxsq19phnhbimccf4p64880jmnpg59wqi07kk117w7"))

(define rust-inout-0.2.2
  (crate-source "inout" "0.2.2"
                "1iq39s01d3y56j2r6hf75yqhpa7s2ifwr316yzyi0879a9jcwl22"))

(define rust-interpolate-name-0.2.4
  (crate-source "interpolate_name" "0.2.4"
                "0q7s5mrfkx4p56dl8q9zq71y1ysdj4shh6f28qf9gly35l21jj63"))

(define rust-interprocess-2.4.2
  (crate-source "interprocess" "2.4.2"
                "0nsr54v0i2ac0cfnccf5ks8vcdhxj7nw3rcgdaq7mjq06is274q6"))

(define rust-intl-memoizer-0.5.3
  (crate-source "intl-memoizer" "0.5.3"
                "0gqn5wwhzacvj0z25r5r3l2pajg9c8i1ivh7g8g8dszm8pis439i"))

(define rust-intl-pluralrules-7.0.2
  (crate-source "intl_pluralrules" "7.0.2"
                "0wprd3h6h8nfj62d8xk71h178q7zfn3srxm787w4sawsqavsg3h7"))

(define rust-intrusive-collections-0.10.2
  (crate-source "intrusive-collections" "0.10.2"
                "0qh03cnj4m3gg03sk9919r5fxmwfwa3nglm6888aryhw4icrqwab"))

(define rust-intrusive-collections-0.9.7
  (crate-source "intrusive-collections" "0.9.7"
                "11hy9ny6rr6qsh289ljrdq20ayw8ik0h4dfzzrgcgs6bwjbhi78q"))

(define rust-inventory-0.3.24
  (crate-source "inventory" "0.3.24"
                "16y3vbab2ld8ykjap1xxwk001jliyqsj8np57zpcrx7jfq6c7w54"))

(define rust-ipnet-2.12.0
  (crate-source "ipnet" "2.12.0"
                "1qpq2y0asyv0jppw7zww9y96fpnpinwap8a0phhqqgyy3znnz3yr"))

(define rust-is-macro-0.3.7
  (crate-source "is-macro" "0.3.7"
                "1r5hvxy697qrrp284qg1f9pyrq7i3mzn1r1qfxj24k728zja6mqx"))

(define rust-is-terminal-polyfill-1.70.2
  (crate-source "is_terminal_polyfill" "1.70.2"
                "15anlc47sbz0jfs9q8fhwf0h3vs2w4imc030shdnq54sny5i7jx6"))

(define rust-itertools-0.11.0
  (crate-source "itertools" "0.11.0"
                "0mzyqcc59azx9g5cg6fs8k529gvh4463smmka6jvzs3cd2jp7hdi"))

(define rust-itertools-0.14.0
  (crate-source "itertools" "0.14.0"
                "118j6l1vs2mx65dqhwyssbrxpawa90886m3mzafdvyip41w2q69b"))

(define rust-itoa-1.0.18
  (crate-source "itoa" "1.0.18"
                "10jnd1vpfkb8kj38rlkn2a6k02afvj3qmw054dfpzagrpl6achlg"))

(define rust-ixdtf-0.6.5
  (crate-source "ixdtf" "0.6.5"
                "1q9y5y6s1fbaqswk541366xyr1hcxv2bg85nijnvwrc4qk3g9sic"))

(define rust-jaq-core-2.2.1
  (crate-source "jaq-core" "2.2.1"
                "1670g3ldack5w5pma00fnhfcpgwvajk6f5qlzlljqhbrxdr6llkp"))

(define rust-jaq-json-1.1.3
  (crate-source "jaq-json" "1.1.3"
                "1g3j27lf205zfyzl9g1vjp6jwgnr8ivwws5cmc1q8vh7gg8dpnq1"))

(define rust-jaq-std-2.1.2
  (crate-source "jaq-std" "2.1.2"
                "0zfnpm3y31g8gzjm1s9alpnvk0h30bz1n7y7frcp10f9jzily9ic"))

(define rust-jiff-0.2.28
  (crate-source "jiff" "0.2.28"
                "00lixngcc7amh2fcsxfr0z38j06lllhapz192biv1qj97q1x60s6"))

(define rust-jiff-static-0.2.28
  (crate-source "jiff-static" "0.2.28"
                "0irbhfh2f4i9w5l53jcmh6ssnhdd92wfy76978chgwnxilvk4bbq"))

(define rust-jiff-tzdb-0.1.6
  (crate-source "jiff-tzdb" "0.1.6"
                "0xihzlnnyk0xnrzpq4xcyjdcmy8xc3ychzb9ayjkh4vgha2fy069"))

(define rust-jiff-tzdb-platform-0.1.3
  (crate-source "jiff-tzdb-platform" "0.1.3"
                "1s1ja692wyhbv7f60mc0x90h7kn1pv65xkqi2y4imarbmilmlnl7"))

(define rust-jni-0.22.4
  (crate-source "jni" "0.22.4"
                "161lza8gz071h22pgyqyx4n91ixd691z2dbb1pq2g97k5i49mzay"))

(define rust-jni-macros-0.22.4
  (crate-source "jni-macros" "0.22.4"
                "18v02mcn5c7mb2yw6r930xg6ynsn7hwkxv8z2kdhn3qprjn0j0d0"))

(define rust-jni-sys-0.3.1
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "jni-sys" "0.3.1"
                "0n1j8fbz081w1igfrpc79n6vgm7h3ik34nziy5fjgq5nz7hm59j1"))

(define rust-jni-sys-0.4.1
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "jni-sys" "0.4.1"
                "1wlahx6f2zhczdjqyn8mk7kshb8x5vsd927sn3lvw41rrf47ldy6"))

(define rust-jni-sys-macros-0.4.1
  (crate-source "jni-sys-macros" "0.4.1"
                "0r32gbabrak15a7p487765b5wc0jcna2yv88mk6m1zjqyi1bkh1q"))

(define rust-jobserver-0.1.34
  (crate-source "jobserver" "0.1.34"
                "0cwx0fllqzdycqn4d6nb277qx5qwnmjdxdl0lxkkwssx77j3vyws"))

(define rust-js-sys-0.3.99
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "js-sys" "0.3.99"
                "04azrzsz91gr5s3z0ij36lz0kj9ry4lw3jz0mmbiwb251rsc8aql"))

(define rust-keccak-0.1.6
  (crate-source "keccak" "0.1.6"
                "0lynp77kk3xw5kbdnmpc4wzx3qqn9cyfvg5prfb3sfnfik4ww9nb"))

(define rust-kittest-0.3.0
  (crate-source "kittest" "0.3.0"
                "1j23y1ssk2pjvgyzw0nd736m0fjf6fwsqf0h21ha6lg2rk96vz81"))

(define rust-kstring-2.0.2
  (crate-source "kstring" "2.0.2"
                "1lfvqlqkg2x23nglznb7ah6fk3vv3y5i759h5l2151ami98gk2sm"))

(define rust-lalrpop-0.20.2
  (crate-source "lalrpop" "0.20.2"
                "1jn1qg7gs9kka6sy2sbxx8wp6z8lm892ksr414b9yaansrx0gjsm"))

(define rust-lalrpop-util-0.20.2
  (crate-source "lalrpop-util" "0.20.2"
                "0lr5r12bh9gjjlmnjrbblf4bfcwnad4gz1hqjvp34yzb22ln0x2h"))

(define rust-lazy-static-1.5.0
  (crate-source "lazy_static" "1.5.0"
                "1zk6dqqni0193xg6iijh7i3i44sryglwgvx20spdvwk3r6sbrlmv"))

(define rust-lazycell-1.3.0
  (crate-source "lazycell" "1.3.0"
                "0m8gw7dn30i0zjjpjdyf6pc16c34nl71lpv461mix50x3p70h3c3"))

(define rust-leb128fmt-0.1.0
  (crate-source "leb128fmt" "0.1.0"
                "1chxm1484a0bly6anh6bd7a99sn355ymlagnwj3yajafnpldkv89"))

(define rust-lebe-0.5.3
  (crate-source "lebe" "0.5.3"
                "1f459clndzzm35nyd15vj5dlasyagfasp7hcgl6lh2b658rs6ybs"))

(define rust-levenshtein-automata-0.2.1
  (crate-source "levenshtein_automata" "0.2.1"
                "09dv3rahqgslyv347s5ymwv0krw44d6xpfymz9mz7sa5dsvdwb0c"))

(define rust-libc-0.2.186
  (crate-source "libc" "0.2.186"
                "0rnyhzjyqq9x56skkllbjzzzwym3r61lq3l4hqj64v71gw0r3av8"))

(define rust-libfuzzer-sys-0.4.13
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "libfuzzer-sys" "0.4.13"
                "1li9z5q55wi81zzyifm7a4rw1xvcclsnqsqbkbvrk86bl50jzzd9"))

(define rust-libloading-0.8.9
  (crate-source "libloading" "0.8.9"
                "0mfwxwjwi2cf0plxcd685yxzavlslz7xirss3b9cbrzyk4hv1i6p"))

(define rust-libm-0.2.16
  (crate-source "libm" "0.2.16"
                "10brh0a3qjmbzkr5mf5xqi887nhs5y9layvnki89ykz9xb1wxlmn"))

(define rust-libmimalloc-sys-0.1.49
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "libmimalloc-sys" "0.1.49"
                "1sdqq31sbf8dbdng8fsyzl2c5xxphn6dvr6ggik6zhg18cpsaiba"))

(define rust-libredox-0.1.17
  (crate-source "libredox" "0.1.17"
                "1ly9hnhiy0f6ccnlg3h8lca9smvv268gj5iwia4gnm10rsxbcaph"))

(define rust-linkme-0.3.36
  (crate-source "linkme" "0.3.36"
                "1ks1mrhf4nc7vy9scfncsiqkv3vw7sn7jib8rbn8vyvkcga74cp8"))

(define rust-linkme-impl-0.3.36
  (crate-source "linkme-impl" "0.3.36"
                "0js4vz4223vdd1wm6imiwvqjxyy7wpgnwdxlcbz0hz9w80h9xm9j"))

(define rust-linux-raw-sys-0.12.1
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "linux-raw-sys" "0.12.1"
                "0lwasljrqxjjfk9l2j8lyib1babh2qjlnhylqzl01nihw14nk9ij"))

(define rust-linux-raw-sys-0.4.15
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "linux-raw-sys" "0.4.15"
                "1aq7r2g7786hyxhv40spzf2nhag5xbw2axxc1k8z5k1dsgdm4v6j"))

(define rust-linux-raw-sys-0.9.4
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "linux-raw-sys" "0.9.4"
                "04kyjdrq79lz9ibrf7czk6cv9d3jl597pb9738vzbsbzy1j5i56d"))

(define rust-litemap-0.8.2
  (crate-source "litemap" "0.8.2"
                "1w7628bc7wwcxc4n4s5kw0610xk06710nh2hn5kwwk2wa91z9nlj"))

(define rust-lnk-0.6.4
  (crate-source "lnk" "0.6.4"
                "021rndfhnfc5knlrkpwxj76hf0nmg33r9gnla2q6rny04ag0gw29"))

(define rust-lock-api-0.4.14
  (crate-source "lock_api" "0.4.14"
                "0rg9mhx7vdpajfxvdjmgmlyrn20ligzqvn8ifmaz7dc79gkrjhr2"))

(define rust-log-0.4.32
  (crate-source "log" "0.4.32"
                "0fmdg0cxig7i4fwf1sw7fmg4d1gdbfzniawcfpwydy1q7320fgwm"))

(define rust-logos-0.14.4
  (crate-source "logos" "0.14.4"
                "0n349vin9mx326fkz68bsa4vc5sdn9n8qnfz7n1yqynbz1p3albj"))

(define rust-logos-codegen-0.14.4
  (crate-source "logos-codegen" "0.14.4"
                "0gwnx7lk4y7xc4yk6pr0knrddard5z22rxaz9xrnc38cc1lh1y2r"))

(define rust-logos-derive-0.14.4
  (crate-source "logos-derive" "0.14.4"
                "07bk3q4jry9f8blrnsiy872ivilzy62xaglnn2ni5p590qmp5yr4"))

(define rust-loom-0.7.2
  (crate-source "loom" "0.7.2"
                "1jpszf9qxv8ydpsm2h9vcyvxvyxcfkhmmfbylzd4gfbc0k40v7j1"))

(define rust-loop9-0.1.5
  (crate-source "loop9" "0.1.5"
                "0qphc1c0cbbx43pwm6isnwzwbg6nsxjh7jah04n1sg5h4p0qgbhg"))

(define rust-lru-0.16.4
  (crate-source "lru" "0.16.4"
                "0fgg35wrpfdrkv9hcabkg92g3sv4867g1rir7ay9lq1zs3ayhrkz"))

(define rust-lru-slab-0.1.2
  (crate-source "lru-slab" "0.1.2"
                "0m2139k466qj3bnpk66bwivgcx3z88qkxvlzk70vd65jq373jaqi"))

(define rust-lz4-flex-0.13.1
  (crate-source "lz4_flex" "0.13.1"
                "0zmrvmrcwnwgypldakvca19z5a2a11wch3dhds1giy39hvnx9w3y"))

(define rust-malachite-0.9.1
  (crate-source "malachite" "0.9.1"
                "11sf2xx0ppb3b8dq0rr0a449p1b7diy8wzlrahzgmk0b1mg1ks4d"))

(define rust-malachite-base-0.9.1
  (crate-source "malachite-base" "0.9.1"
                "1kaw9y2qr6zwsl3vnl8svfnb1z4x4dsigsbbjiarksxivdpzidm8"))

(define rust-malachite-float-0.9.1
  (crate-source "malachite-float" "0.9.1"
                "0a7xavblclxg56d4yywhxzy1vp7s2w443rqcn4h2hmf1fcbh5ma7"))

(define rust-malachite-nz-0.9.1
  (crate-source "malachite-nz" "0.9.1"
                "1xh9mab3hsfb1bsdqakaw8rr56m7dif9i0p2g28xa6gfrzss55q1"))

(define rust-malachite-q-0.9.1
  (crate-source "malachite-q" "0.9.1"
                "0sqiri3gh4c1b12am01irr3qhckxx8dya3lgqj8f1v9a2saxsamy"))

(define rust-maplit-1.0.2
  (crate-source "maplit" "1.0.2"
                "07b5kjnhrrmfhgqm9wprjw8adx6i225lqp49gasgqg74lahnabiy"))

(define rust-markdown-1.0.0
  (crate-source "markdown" "1.0.0"
                "1sqxbclkxw615kcwglcisda1dcw8cfaa30z7sa16lhfwrbrbijm5"))

(define rust-markup5ever-0.38.0
  (crate-source "markup5ever" "0.38.0"
                "0qhqx70ak6pi9pbyddb61fhd37kypkbbvfnnnamfmzhm547x70w9"))

(define rust-markup5ever-0.39.0
  (crate-source "markup5ever" "0.39.0"
                "1pjpjwzlsv03ljpprq3swamfj8ipv6kl2nvfdzjlww2zxj3xj8ki"))

(define rust-matchers-0.2.0
  (crate-source "matchers" "0.2.0"
                "1sasssspdj2vwcwmbq3ra18d3qniapkimfcbr47zmx6750m5llni"))

(define rust-matchit-0.8.4
  (crate-source "matchit" "0.8.4"
                "1hzl48fwq1cn5dvshfly6vzkzqhfihya65zpj7nz7lfx82mgzqa7"))

(define rust-maybe-async-0.2.11
  (crate-source "maybe-async" "0.2.11"
                "036anp4dzz7sjgdq3zfwzf52ggblpbx1sivlvg2ssq5dhjip6s3l"))

(define rust-maybe-rayon-0.1.1
  (crate-source "maybe-rayon" "0.1.1"
                "06cmvhj4n36459g327ng5dnj8d58qs472pv5ahlhm7ynxl6g78cf"))

(define rust-md-5-0.10.6
  (crate-source "md-5" "0.10.6"
                "1kvq5rnpm4fzwmyv5nmnxygdhhb2369888a06gdc9pxyrzh7x7nq"))

(define rust-md5-0.7.0
  (crate-source "md5" "0.7.0"
                "0wcps37hrhz59fkhf8di1ppdnqld6l1w5sdy7jp7p51z0i4c8329"))

(define rust-measure-time-0.9.0
  (crate-source "measure_time" "0.9.0"
                "03pw6ni975lmm16fkr4ajwvxp161yhbgmicn8dqaphrgwxhmviai"))

(define rust-memchr-2.8.1
  (crate-source "memchr" "2.8.1"
                "1n448jx01h5z2xknj6x2dhxgr8s8fb717cf6vfqj5lmhkpj7m53b"))

(define rust-memmap2-0.9.11
  (crate-source "memmap2" "0.9.11"
                "1h4qnzgarnn488ljjpg9ns5y4bw0sq0xv0fj0iqywagjnz8rw8fi"))

(define rust-memoffset-0.9.1
  (crate-source "memoffset" "0.9.1"
                "12i17wh9a9plx869g7j4whf62xw68k5zd4k0k5nh6ys5mszid028"))

(define rust-miette-7.6.0
  (crate-source "miette" "7.6.0"
                "1dwjnnpcff4jzpf5ns1m19di2p0n5j31zmjv5dskrih7i3nfz62z"))

(define rust-miette-derive-7.6.0
  (crate-source "miette-derive" "7.6.0"
                "12w13a67n2cc37nzidvv0v0vrvf4rsflzxz6slhbn3cm9rqjjnyv"))

(define rust-mimalloc-0.1.52
  (crate-source "mimalloc" "0.1.52"
                "0qkqr4yga7fkyqwnn89d2xp346q54n4fpm91rzxd2jni52xkjh9d"))

(define rust-mime-0.3.17
  (crate-source "mime" "0.3.17"
                "16hkibgvb9klh0w0jk5crr5xv90l3wlf77ggymzjmvl1818vnxv8"))

(define rust-mime-guess-2.0.5
  (crate-source "mime_guess" "2.0.5"
                "03jmg3yx6j39mg0kayf7w4a886dl3j15y8zs119zw01ccy74zi7p"))

(define rust-mime-guess2-2.3.1
  (crate-source "mime_guess2" "2.3.1"
                "1jphmmvrl93bj05wdmjvx20hp2fmlgchjwd0lz0dwh71l8adq1hp"))

(define rust-minimal-lexical-0.2.1
  (crate-source "minimal-lexical" "0.2.1"
                "16ppc5g84aijpri4jzv14rvcnslvlpphbszc7zzp6vfkddf4qdb8"))

(define rust-miniz-oxide-0.8.9
  (crate-source "miniz_oxide" "0.8.9"
                "05k3pdg8bjjzayq3rf0qhpirq9k37pxnasfn4arbs17phqn6m9qz"))

(define rust-miniz-oxide-0.9.1
  (crate-source "miniz_oxide" "0.9.1"
                "0k2bgjzk2sbsynpsv4wizwxbqp6vs7g08y5anbkrh3l6a15bqgxn"))

(define rust-mint-0.5.9
  (crate-source "mint" "0.5.9"
                "1zw5glv8z2d99c82jy2za97hh9p6377xmf4rbwz7jynsdfxfngg5"))

(define rust-mio-1.2.1
  (crate-source "mio" "1.2.1"
                "1nkggmrlnjs93w8rja4lvjj4aml1xqahgimv1h0p7d373kvhmg82"))

(define rust-monch-0.6.0
  (crate-source "monch" "0.6.0"
                "0500hckbfyrj24v9mab0qji4dlmqj5r7r7yj68jnyx9zp79nmk3b"))

(define rust-moxcms-0.8.1
  (crate-source "moxcms" "0.8.1"
                "0jz4fd5f7pdn1rngqc96lxriqjkym1lswdhdbjr037s8p9ac31dv"))

(define rust-multer-3.1.0
  (crate-source "multer" "3.1.0"
                "0jr2snfay5fjz50yvdja4vbnddlj1iriiqjym88pbj3daiv7gs43"))

(define rust-murmurhash32-0.3.1
  (crate-source "murmurhash32" "0.3.1"
                "16yqpn81xx4g7wr0wihcc0wzxlzfcdv2mmi97d48394nm5mbz591"))

(define rust-ndk-0.9.0
  (crate-source "ndk" "0.9.0"
                "1m32zpmi5w1pf3j47k6k5fw395dc7aj8d0mdpsv53lqkprxjxx63"))

(define rust-ndk-context-0.1.1
  (crate-source "ndk-context" "0.1.1"
                "12sai3dqsblsvfd1l1zab0z6xsnlha3xsfl7kagdnmj3an3jvc17"))

(define rust-ndk-sys-0.6.0+11769913
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "ndk-sys" "0.6.0+11769913"
                "0wx8r6pji20if4xs04g73gxl98nmjrfc73z0v6w1ypv6a4qdlv7f"))

(define rust-new-debug-unreachable-1.0.6
  (crate-source "new_debug_unreachable" "1.0.6"
                "11phpf1mjxq6khk91yzcbd3ympm78m3ivl7xg6lg2c0lf66fy3k5"))

(define rust-nix-0.28.0
  (crate-source "nix" "0.28.0"
                "1r0rylax4ycx3iqakwjvaa178jrrwiiwghcw95ndzy72zk25c8db"))

(define rust-nix-0.30.1
  (crate-source "nix" "0.30.1"
                "1dixahq9hk191g0c2ydc0h1ppxj0xw536y6rl63vlnp06lx3ylkl"))

(define rust-nix-0.31.3
  (crate-source "nix" "0.31.3"
                "0gbwnjfny9rq9hl5bz4ry520n9rnfknna4bg88n66f7zx3yx486g"))

(define rust-no-std-io2-0.9.4
  (crate-source "no_std_io2" "0.9.4"
                "00w0ggkaaacbwiv4qw188ih5llmhf53qgp20wk5gdyrldldvv2j1"))

(define rust-nohash-hasher-0.2.0
  (crate-source "nohash-hasher" "0.2.0"
                "0lf4p6k01w4wm7zn4grnihzj8s7zd5qczjmzng7wviwxawih5x9b"))

(define rust-nom-7.1.3
  (crate-source "nom" "7.1.3"
                "0jha9901wxam390jcf5pfa0qqfrgh8li787jx2ip0yk5b8y9hwyj"))

(define rust-nom-8.0.0
  (crate-source "nom" "8.0.0"
                "01cl5xng9d0gxf26h39m0l8lprgpa00fcc75ps1yzgbib1vn35yz"))

(define rust-noop-proc-macro-0.3.0
  (crate-source "noop_proc_macro" "0.3.0"
                "1j2v1c6ric4w9v12h34jghzmngcwmn0hll1ywly4h6lcm4rbnxh6"))

(define rust-ntapi-0.4.3
  (crate-source "ntapi" "0.4.3"
                "1bl0d73avwla7laa4pkqvzvifjbs0avg65w01zxjydgx3likbcy3"))

(define rust-nu-ansi-term-0.50.3
  (crate-source "nu-ansi-term" "0.50.3"
                "1ra088d885lbd21q1bxgpqdlk1zlndblmarn948jz2a40xsbjmvr"))

(define rust-num-0.4.3
  (crate-source "num" "0.4.3"
                "08yb2fc1psig7pkzaplm495yp7c30m4pykpkwmi5bxrgid705g9m"))

(define rust-num-bigint-0.4.6
  (crate-source "num-bigint" "0.4.6"
                "1f903zd33i6hkjpsgwhqwi2wffnvkxbn6rv4mkgcjcqi7xr4zr55"))

(define rust-num-complex-0.4.6
  (crate-source "num-complex" "0.4.6"
                "15cla16mnw12xzf5g041nxbjjm9m85hdgadd5dl5d0b30w9qmy3k"))

(define rust-num-conv-0.2.2
  (crate-source "num-conv" "0.2.2"
                "0hg4f9bwmy7cwpxdkm165dmkfc8jhkkayci234jsmi5ssb33j5sj"))

(define rust-num-cpus-1.17.0
  (crate-source "num_cpus" "1.17.0"
                "0fxjazlng4z8cgbmsvbzv411wrg7x3hyxdq8nxixgzjswyylppwi"))

(define rust-num-derive-0.3.3
  (crate-source "num-derive" "0.3.3"
                "0gbl94ckzqjdzy4j8b1p55mz01g6n1l9bckllqvaj0wfz7zm6sl7"))

(define rust-num-derive-0.4.2
  (crate-source "num-derive" "0.4.2"
                "00p2am9ma8jgd2v6xpsz621wc7wbn1yqi71g15gc3h67m7qmafgd"))

(define rust-num-enum-0.7.6
  (crate-source "num_enum" "0.7.6"
                "09kg0c2y08npdv0c9dbm4m9a9wz8w2qaiqqxl4gj3v22hj1wl2sx"))

(define rust-num-enum-derive-0.7.6
  (crate-source "num_enum_derive" "0.7.6"
                "1y0x9z49s27vdas6mglqbv02sgkdmbr8ns2kwspzrp2ra81rh2b8"))

(define rust-num-integer-0.1.46
  (crate-source "num-integer" "0.1.46"
                "13w5g54a9184cqlbsq80rnxw4jj4s0d8wv75jsq5r2lms8gncsbr"))

(define rust-num-iter-0.1.45
  (crate-source "num-iter" "0.1.45"
                "1gzm7vc5g9qsjjl3bqk9rz1h6raxhygbrcpbfl04swlh0i506a8l"))

(define rust-num-modular-0.6.1
  (crate-source "num-modular" "0.6.1"
                "0zv4miws3q1i93a0bd9wgc4njrr5j5786kr99hzxi9vgycdjdfqp"))

(define rust-num-order-1.2.0
  (crate-source "num-order" "1.2.0"
                "1dhvdncf91ljxh9sawnfxcbiqj1gnag08lyias0cy3y4jxmmjysk"))

(define rust-num-rational-0.4.2
  (crate-source "num-rational" "0.4.2"
                "093qndy02817vpgcqjnj139im3jl7vkq4h68kykdqqh577d18ggq"))

(define rust-num-threads-0.1.7
  (crate-source "num_threads" "0.1.7"
                "1ngajbmhrgyhzrlc4d5ga9ych1vrfcvfsiqz6zv0h2dpr2wrhwsw"))

(define rust-num-traits-0.2.19
  (crate-source "num-traits" "0.2.19"
                "0h984rhdkkqd4ny9cif7y2azl3xdfb7768hb9irhpsch4q3gq787"))

(define rust-number-prefix-0.4.0
  (crate-source "number_prefix" "0.4.0"
                "1wvh13wvlajqxkb1filsfzbrnq0vrmrw298v2j3sy82z1rm282w3"))

(define rust-number-to-words-0.1.1
  (crate-source "number_to_words" "0.1.1"
                "18iai3p1mgdygmbdijfyschmdcgd4j6zh35jcb7rs81xf6zx2mdh"))

(define rust-objc-sys-0.3.5
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "objc-sys" "0.3.5"
                "0423gry7s3rmz8s3pzzm1zy5mdjif75g6dbzc2lf2z0c77fipffd"))

(define rust-objc2-0.5.2
  (crate-source "objc2" "0.5.2"
                "015qa2d3vh7c1j2736h5wjrznri7x5ic35vl916c22gzxva8b9s6"))

(define rust-objc2-0.6.4
  (crate-source "objc2" "0.6.4"
                "17x8qpl512frscfqbmgjr20kg3y4r0xdqxphja17dz5f0znsh4is"))

(define rust-objc2-app-kit-0.2.2
  (crate-source "objc2-app-kit" "0.2.2"
                "1zqyi5l1bm26j1bgmac9783ah36m5kcrxlqp5carglnpwgcrms74"))

(define rust-objc2-app-kit-0.3.2
  (crate-source "objc2-app-kit" "0.3.2"
                "132ijwni8lsi8phq7wnmialkxp46zx998fns3zq5np0ya1mr77nl"))

(define rust-objc2-cloud-kit-0.2.2
  (crate-source "objc2-cloud-kit" "0.2.2"
                "02dhjvmcq8c2bwj31jx423jygif1scs9f0lmlab0ayhw75b3ppbl"))

(define rust-objc2-contacts-0.2.2
  (crate-source "objc2-contacts" "0.2.2"
                "12a8m927xrrxa54xhqhqnkkl1a6l07pyrpnqfk9jz09kkh755zx5"))

(define rust-objc2-core-data-0.2.2
  (crate-source "objc2-core-data" "0.2.2"
                "1vvk8zjylfjjj04dzawydmqqz5ajvdkhf22cnb07ihbiw14vyzv1"))

(define rust-objc2-core-foundation-0.3.2
  (crate-source "objc2-core-foundation" "0.3.2"
                "0dnmg7606n4zifyjw4ff554xvjmi256cs8fpgpdmr91gckc0s61a"))

(define rust-objc2-core-graphics-0.3.2
  (crate-source "objc2-core-graphics" "0.3.2"
                "01x8413pxq0m5rwidlaczni8v5cz9dc3xqzq8l9zlpl9cv8cj8p0"))

(define rust-objc2-core-image-0.2.2
  (crate-source "objc2-core-image" "0.2.2"
                "102csfb82zi2sbzliwsfd589ckz0gysf7y6434c9zj97lmihj9jm"))

(define rust-objc2-core-location-0.2.2
  (crate-source "objc2-core-location" "0.2.2"
                "10apgsrigqryvi4rcc0f6yfjflvrl83f4bi5hkr48ck89vizw300"))

(define rust-objc2-encode-4.1.0
  (crate-source "objc2-encode" "4.1.0"
                "0cqckp4cpf68mxyc2zgnazj8klv0z395nsgbafa61cjgsyyan9gg"))

(define rust-objc2-foundation-0.2.2
  (crate-source "objc2-foundation" "0.2.2"
                "1a6mi77jsig7950vmx9ydvsxaighzdiglk5d229k569pvajkirhf"))

(define rust-objc2-foundation-0.3.2
  (crate-source "objc2-foundation" "0.3.2"
                "0wijkxzzvw2xkzssds3fj8279cbykz2rz9agxf6qh7y2agpsvq73"))

(define rust-objc2-io-kit-0.3.2
  (crate-source "objc2-io-kit" "0.3.2"
                "05dvfcf97w39daaj5qsbfc399lw9hbx3s4h9nwgxrmlpjnizpyik"))

(define rust-objc2-io-surface-0.3.2
  (crate-source "objc2-io-surface" "0.3.2"
                "07fqx4fmwydf2arrc4xs4awv7zyzzxh60fyqdfmrpm9n148qh1qq"))

(define rust-objc2-link-presentation-0.2.2
  (crate-source "objc2-link-presentation" "0.2.2"
                "160k4qh00yrx57dabn3hzas4r98kmk9bc0qsy1jvwday3irax8d1"))

(define rust-objc2-metal-0.2.2
  (crate-source "objc2-metal" "0.2.2"
                "1mmdga66qpxrcfq3gxxhysfx3zg1hpx4z886liv3j0pnfq9bl36x"))

(define rust-objc2-quartz-core-0.2.2
  (crate-source "objc2-quartz-core" "0.2.2"
                "0ynw8819c36l11rim8n0yzk0fskbzrgaqayscyqi8swhzxxywaz4"))

(define rust-objc2-quartz-core-0.3.2
  (crate-source "objc2-quartz-core" "0.3.2"
                "07vzaf6y1lk7zygkgvpp23mm19ipdm9yq8af22gvywdkaa23bhcn"))

(define rust-objc2-symbols-0.2.2
  (crate-source "objc2-symbols" "0.2.2"
                "1p04hjkxan18g2b7h9n2n8xxsvazapv2h6mfmmdk06zc7pz4ws0a"))

(define rust-objc2-ui-kit-0.2.2
  (crate-source "objc2-ui-kit" "0.2.2"
                "0vrb5r8z658l8c19bx78qks8c5hg956544yirf8npk90idwldfxq"))

(define rust-objc2-uniform-type-identifiers-0.2.2
  (crate-source "objc2-uniform-type-identifiers" "0.2.2"
                "1ziv4wkbxcaw015ypg0q49ycl7m14l3x56mpq2k1rznv92bmzyj4"))

(define rust-objc2-user-notifications-0.2.2
  (crate-source "objc2-user-notifications" "0.2.2"
                "1cscv2w3vxzaslz101ddv0z9ycrrs4ayikk4my4qd3im8bvcpkvn"))

(define rust-object-0.37.3
  (crate-source "object" "0.37.3"
                "1zikiy9xhk6lfx1dn2gn2pxbnfpmlkn0byd7ib1n720x0cgj0xpz"))

(define rust-oid-registry-0.7.1
  (crate-source "oid-registry" "0.7.1"
                "1navxdy0gx7f92ymwr6n02x35fypp2izdfcf49wszkc9ji6h7n58"))

(define rust-once-cell-1.21.4
  (crate-source "once_cell" "1.21.4"
                "0l1v676wf71kjg2khch4dphwh1jp3291ffiymr2mvy1kxd5kwz4z"))

(define rust-once-cell-polyfill-1.70.2
  (crate-source "once_cell_polyfill" "1.70.2"
                "1zmla628f0sk3fhjdjqzgxhalr2xrfna958s632z65bjsfv8ljrq"))

(define rust-oneshot-0.1.13
  (crate-source "oneshot" "0.1.13"
                "01x1rp6s5hxx87n2pc5101lxgdrj0gnxj45zss2qb8li4m6cm6r6"))

(define rust-oneshot-0.2.1
  (crate-source "oneshot" "0.2.1"
                "033z8p0w8xcnigwapi0w7l5hpizc62rgrgl0z6wkys9cl0b19qng"))

(define rust-opaque-debug-0.3.1
  (crate-source "opaque-debug" "0.3.1"
                "10b3w0kydz5jf1ydyli5nv10gdfp97xh79bgz327d273bs46b3f0"))

(define rust-openssl-probe-0.2.1
  (crate-source "openssl-probe" "0.2.1"
                "1gpwpb7smfhkscwvbri8xzbab39wcnby1jgz1s49vf1aqgsdx1vw"))

(define rust-option-ext-0.2.0
  (crate-source "option-ext" "0.2.0"
                "0zbf7cx8ib99frnlanpyikm1bx8qn8x602sw1n7bg6p9x94lyx04"))

(define rust-orbclient-0.3.55
  (crate-source "orbclient" "0.3.55"
                "0iqps9qnyyhzbmb321r90dr8ql7jqbpm13bnf7in16pa4vskkwsx"))

(define rust-ordered-float-2.10.1
  (crate-source "ordered-float" "2.10.1"
                "075i108hr95pr7hy4fgxivib5pky3b6b22rywya5qyd2wmkrvwb8"))

(define rust-ordered-float-5.3.0
  (crate-source "ordered-float" "5.3.0"
                "03mx5yg3ncp0g524y7zbyvhwcxpd8l9v30lgybm5bhqx2v551ndp"))

(define rust-ordered-stream-0.2.0
  (crate-source "ordered-stream" "0.2.0"
                "0l0xxp697q7wiix1gnfn66xsss7fdhfivl2k7bvpjs4i3lgb18ls"))

(define rust-outref-0.5.2
  (crate-source "outref" "0.5.2"
                "03pzw9aj4qskqhh0fkagy2mkgfwgj5a1m67ajlba5hw80h68100s"))

(define rust-owned-ttf-parser-0.25.1
  (crate-source "owned_ttf_parser" "0.25.1"
                "0fsqzcbc4sq8qhkmc3rgcfg1xg389nmhlxvmvi6h38dca680x0in"))

(define rust-ownedbytes-0.9.0
  (crate-source "ownedbytes" "0.9.0"
                "0zjamcdmwag60fh4vajapmsl8gs01xcghhywhhbycrqpcgvmdg9g"))

(define rust-owo-colors-3.5.0
  (crate-source "owo-colors" "3.5.0"
                "0vyvry6ba1xmpd45hpi6savd8mbx09jpmvnnwkf6z62pk6s4zc61"))

(define rust-owo-colors-4.3.0
  (crate-source "owo-colors" "4.3.0"
                "0kgrf4r9vcczhw5r30nkcl6abm99l0ay8dr2fxl0ymvbkcxq04fj"))

(define rust-pack1-1.1.0
  (crate-source "pack1" "1.1.0"
                "08sm42zs7pmr0ccwwdiw9ffixvbfxxwyx57f40gpni1frw7bpdz3"))

(define rust-par-core-2.0.0
  (crate-source "par-core" "2.0.0"
                "03y9yhzg90wm0dn7bqjrpldaj5xmg6kkivsibndb4zsv4lhvsv79"))

(define rust-parking-2.2.1
  (crate-source "parking" "2.2.1"
                "1fnfgmzkfpjd69v4j9x737b1k8pnn054bvzcn5dm3pkgq595d3gk"))

(define rust-parking-lot-0.12.5
  (crate-source "parking_lot" "0.12.5"
                "06jsqh9aqmc94j2rlm8gpccilqm6bskbd67zf6ypfc0f4m9p91ck"))

(define rust-parking-lot-core-0.9.12
  (crate-source "parking_lot_core" "0.9.12"
                "1hb4rggy70fwa1w9nb0svbyflzdc69h047482v2z3sx2hmcnh896"))

(define rust-password-hash-0.5.0
  (crate-source "password-hash" "0.5.0"
                "0ri1mim11zk0a9s40zdi288dfqvmdiryc7lw8vl46b59ifa08vrl"))

(define rust-passwords-3.1.16
  (crate-source "passwords" "3.1.16"
                "176mr017icfz736c7j6bbm6pmkzxlr6kkhqfdgn19gf2ly9p2h0i"))

(define rust-paste-1.0.15
  (crate-source "paste" "1.0.15"
                "02pxffpdqkapy292harq6asfjvadgp1s005fip9ljfsn9fvxgh2p"))

(define rust-pastey-0.1.1
  (crate-source "pastey" "0.1.1"
                "1v389jkifv757903flrrps67dvc6q6giwlyx3xi33hcfjmgjxyrm"))

(define rust-pastey-0.2.3
  (crate-source "pastey" "0.2.3"
                "1d1mk45ma9w54ppws8x096q96qhqirxmj9j3hchj7fmi1087zrif"))

(define rust-pathdiff-0.2.3
  (crate-source "pathdiff" "0.2.3"
                "1lrqp4ip05df8dzldq6gb2c1sq2gs54gly8lcnv3rhav1qhwx56z"))

(define rust-percent-encoding-2.3.2
  (crate-source "percent-encoding" "2.3.2"
                "083jv1ai930azvawz2khv7w73xh8mnylk7i578cifndjn5y64kwv"))

(define rust-pest-2.8.6
  (crate-source "pest" "2.8.6"
                "0qm6kpqsbn2p6vkd7v4j3g7wsjby2ip6di1h6kx7vlq921h8r170"))

(define rust-pest-derive-2.8.6
  (crate-source "pest_derive" "2.8.6"
                "0xzysvcyfs0pkn2801rg811y83jx2rvpqnjxs47c3ri1xbqqdx0i"))

(define rust-pest-generator-2.8.6
  (crate-source "pest_generator" "2.8.6"
                "0kzrcik2ww0qh84jlv8xqc0zmzgl3xy41vf1cfli1chkgdjc8h40"))

(define rust-pest-meta-2.8.6
  (crate-source "pest_meta" "2.8.6"
                "08126skq2lxysinp6v917niszhnnh6d6a9kg2i0a28b0sdlmr0c9"))

(define rust-petgraph-0.6.5
  (crate-source "petgraph" "0.6.5"
                "1ns7mbxidnn2pqahbbjccxkrqkrll2i5rbxx43ns6rh6fn3cridl"))

(define rust-phf-0.11.3
  (crate-source "phf" "0.11.3"
                "0y6hxp1d48rx2434wgi5g8j1pr8s5jja29ha2b65435fh057imhz"))

(define rust-phf-0.13.1
  (crate-source "phf" "0.13.1"
                "1pzswx5gdglgjgp4azyzwyr4gh031r0kcnpqq6jblga72z3jsmn1"))

(define rust-phf-codegen-0.11.3
  (crate-source "phf_codegen" "0.11.3"
                "0si1n6zr93kzjs3wah04ikw8z6npsr39jw4dam8yi9czg2609y5f"))

(define rust-phf-codegen-0.13.1
  (crate-source "phf_codegen" "0.13.1"
                "1qfnsl2hiny0yg4lwn888xla5iwccszgxnx8dhbwl6s2h2fpzaj9"))

(define rust-phf-generator-0.11.3
  (crate-source "phf_generator" "0.11.3"
                "0gc4np7s91ynrgw73s2i7iakhb4lzdv1gcyx7yhlc0n214a2701w"))

(define rust-phf-generator-0.13.1
  (crate-source "phf_generator" "0.13.1"
                "0dwpp11l41dy9mag4phkyyvhpf66lwbp79q3ik44wmhyfqxcwnhk"))

(define rust-phf-macros-0.11.3
  (crate-source "phf_macros" "0.11.3"
                "05kjfbyb439344rhmlzzw0f9bwk9fp95mmw56zs7yfn1552c0jpq"))

(define rust-phf-macros-0.13.1
  (crate-source "phf_macros" "0.13.1"
                "1vv9h8pr7xh18sigpvq1hxc8q9nmjmv6gdpqsp65krxiahmh6bw1"))

(define rust-phf-shared-0.11.3
  (crate-source "phf_shared" "0.11.3"
                "1rallyvh28jqd9i916gk5gk2igdmzlgvv5q0l3xbf3m6y8pbrsk7"))

(define rust-phf-shared-0.13.1
  (crate-source "phf_shared" "0.13.1"
                "0rpjchnswm0x5l4mz9xqfpw0j4w68sjvyqrdrv13h7lqqmmyyzz5"))

(define rust-pico-args-0.5.0
  (crate-source "pico-args" "0.5.0"
                "05d30pvxd6zlnkg2i3ilr5a70v3f3z2in18m67z25vinmykngqav"))

(define rust-pin-project-1.1.13
  (crate-source "pin-project" "1.1.13"
                "09091qp946lpmjz4yp0xil1r5v4hgc91fi19dg5csayhdqrv4ri4"))

(define rust-pin-project-internal-1.1.13
  (crate-source "pin-project-internal" "1.1.13"
                "12rzlh07i1sdgrvzj6wgkka5bjqyvbfsl8knq6qi7g16m7q9aqy9"))

(define rust-pin-project-lite-0.2.17
  (crate-source "pin-project-lite" "0.2.17"
                "1kfmwvs271si96zay4mm8887v5khw0c27jc9srw1a75ykvgj54x8"))

(define rust-piper-0.2.5
  (crate-source "piper" "0.2.5"
                "1hd3j94mw5dwc457gs9ssb2r5b9iipywndf5srqx7pj38jd4fdf8"))

(define rust-pkcs8-0.10.2
  (crate-source "pkcs8" "0.10.2"
                "1dx7w21gvn07azszgqd3ryjhyphsrjrmq5mmz1fbxkj5g0vv4l7r"))

(define rust-pkg-config-0.3.33
  (crate-source "pkg-config" "0.3.33"
                "17jnqmcbxsnwhg9gjf0nh6dj5k0x3hgwi3mb9krjnmfa9v435w8r"))

(define rust-plain-0.2.3
  (crate-source "plain" "0.2.3"
                "19n1xbxb4wa7w891268bzf6cbwq4qvdb86bik1z129qb0xnnnndl"))

(define rust-pluralizer-0.5.0
  (crate-source "pluralizer" "0.5.0"
                "1glcfznfyc730fsmj9wrkcp4xsbhm13ph51rdz0zd880591vlgjb"))

(define rust-png-0.18.1
  (crate-source "png" "0.18.1"
                "0qca282xp8a6d7mikxrwji3f52mjn4vnqxz2v9iz5adj665rnxk0"))

(define rust-polling-3.11.0
  (crate-source "polling" "3.11.0"
                "0622qfbxi3gb0ly2c99n3xawp878fkrd1sl83hjdhisx11cly3jx"))

(define rust-polyval-0.6.2
  (crate-source "polyval" "0.6.2"
                "09gs56vm36ls6pyxgh06gw2875z2x77r8b2km8q28fql0q6yc7wx"))

(define rust-polyval-0.7.1
  (crate-source "polyval" "0.7.1"
                "1ppf0gyjp9d7b9iwnlcxb4zr0v3aj4jhgfa9ax7s3zhn0hjn7z3x"))

(define rust-portable-atomic-1.13.1
  (crate-source "portable-atomic" "1.13.1"
                "0j8vlar3n5acyigq8q6f4wjx3k3s5yz0rlpqrv76j73gi5qr8fn3"))

(define rust-portable-atomic-util-0.2.7
  (crate-source "portable-atomic-util" "0.2.7"
                "0616j0fhy6y71hyxg3n86f6hng0fmsc269s3wp4gl8ww4p8hd8f2"))

(define rust-portpicker-0.1.1
  (crate-source "portpicker" "0.1.1"
                "1acvi1m6g7d3j8xvdsbn0b7yqyfy7yr7fm1pw5kbdyhvmxpxg5xy"))

(define rust-postcard-1.1.3
  (crate-source "postcard" "1.1.3"
                "094srff139n7m8g5ssq36ag6s29ikf7fgpz660x2hkj5vnsw6r37"))

(define rust-potential-utf-0.1.5
  (crate-source "potential_utf" "0.1.5"
                "0r0518fr32xbkgzqap509s3r60cr0iancsg9j1jgf37cyz7b20q1"))

(define rust-powerfmt-0.2.0
  (crate-source "powerfmt" "0.2.0"
                "14ckj2xdpkhv3h6l5sdmb9f1d57z8hbfpdldjc2vl5givq2y77j3"))

(define rust-ppv-lite86-0.2.21
  (crate-source "ppv-lite86" "0.2.21"
                "1abxx6qz5qnd43br1dd9b2savpihzjza8gb4fbzdql1gxp2f7sl5"))

(define rust-precomputed-hash-0.1.1
  (crate-source "precomputed-hash" "0.1.1"
                "075k9bfy39jhs53cb2fpb9klfakx2glxnf28zdw08ws6lgpq6lwj"))

(define rust-prettyplease-0.2.37
  (crate-source "prettyplease" "0.2.37"
                "0azn11i1kh0byabhsgab6kqs74zyrg69xkirzgqyhz6xmjnsi727"))

(define rust-proc-macro-crate-1.3.1
  (crate-source "proc-macro-crate" "1.3.1"
                "069r1k56bvgk0f58dm5swlssfcp79im230affwk6d9ck20g04k3z"))

(define rust-proc-macro-crate-3.5.0
  (crate-source "proc-macro-crate" "3.5.0"
                "0kv1g1d1zjwxlgcaba2qlshzyy32j03xic8rskqlcr5mnblsfyz6"))

(define rust-proc-macro-error-attr2-2.0.0
  (crate-source "proc-macro-error-attr2" "2.0.0"
                "1ifzi763l7swl258d8ar4wbpxj4c9c2im7zy89avm6xv6vgl5pln"))

(define rust-proc-macro-error2-2.0.1
  (crate-source "proc-macro-error2" "2.0.1"
                "00lq21vgh7mvyx51nwxwf822w2fpww1x0z8z0q47p8705g2hbv0i"))

(define rust-proc-macro-hack-0.5.20+deprecated
  (crate-source "proc-macro-hack" "0.5.20+deprecated"
                "0s402hmcs3k9nd6rlp07zkr1lz7yimkmcwcbgnly2zr44wamwdyw"))

(define rust-proc-macro2-1.0.106
  (crate-source "proc-macro2" "1.0.106"
                "0d09nczyaj67x4ihqr5p7gxbkz38gxhk4asc0k8q23g9n85hzl4g"))

(define rust-process-wrap-9.1.0
  (crate-source "process-wrap" "9.1.0"
                "0mfzgksv68wn6ixiv05dsr24pvpbwa16cg0r9m1mi48iv7x2x11f"))

(define rust-prodash-30.0.1
  (crate-source "prodash" "30.0.1"
                "0fdi0wxgy3s9643dgyfkwgmm12g4a360djy56zbxkls9d1bgqvjs"))

(define rust-profiling-1.0.18
  (crate-source "profiling" "1.0.18"
                "1xdwlvxlgy99nn1dra7arzinkc8lbqljvcwpq70m7g16lda5wn9x"))

(define rust-profiling-procmacros-1.0.18
  (crate-source "profiling-procmacros" "1.0.18"
                "1jxvqff6j1z7ph3qghw2xhv18z7pf6cs6cja6fwscjwsdfis9224"))

(define rust-prost-0.14.4
  (crate-source "prost" "0.14.4"
                "1qas5v5rap45f43v3ja0jngxrrafrkcwl0iw5a3ld1pz2rscd2jj"))

(define rust-prost-derive-0.14.4
  (crate-source "prost-derive" "0.14.4"
                "1pqa77d7da5pf6ba3kjj7510m5cynz6902ax01ckvr0pfrgv4w5m"))

(define rust-psm-0.1.31
  (crate-source "psm" "0.1.31"
                "1sk1wzb8j64b9f3z863lv45cgri6ikhys5pgwdfrnv9ldr4bwpb4"))

(define rust-ptr-meta-0.1.4
  (crate-source "ptr_meta" "0.1.4"
                "1wd4wy0wxrcays4f1gy8gwcmxg7mskmivcv40p0hidh6xbvwqf07"))

(define rust-ptr-meta-derive-0.1.4
  (crate-source "ptr_meta_derive" "0.1.4"
                "1b69cav9wn67cixshizii0q5mlbl0lihx706vcrzm259zkdlbf0n"))

(define rust-pxfm-0.1.29
  (crate-source "pxfm" "0.1.29"
                "0gvfd9r73i2mqf1cdc2y5yf0m0skhc16a5aglxiwsv2c57swrig0"))

(define rust-qoi-0.4.1
  (crate-source "qoi" "0.4.1"
                "00c0wkb112annn2wl72ixyd78mf56p4lxkhlmsggx65l3v3n8vbz"))

(define rust-quick-error-2.0.1
  (crate-source "quick-error" "2.0.1"
                "18z6r2rcjvvf8cn92xjhm2qc3jpd1ljvcbf12zv0k9p565gmb4x9"))

(define rust-quick-xml-0.41.0
  (crate-source "quick-xml" "0.41.0"
                "1h9y8zry34r3mxfd5vqfj50vvvzvri4kzbx5d657jkqjalg4aq76"))

(define rust-quinn-0.11.9
  (crate-source "quinn" "0.11.9"
                "086gzj666dr3slmlynkvxlndy28hahgl361d6bf93hk3i6ahmqmr"))

(define rust-quinn-proto-0.11.14
  (crate-source "quinn-proto" "0.11.14"
                "1660jkxhzi1pnywzs13ifczwrlv6ds9qds111vsnxjciqpz44js3"))

(define rust-quinn-udp-0.5.14
  (crate-source "quinn-udp" "0.5.14"
                "1gacawr17a2zkyri0r3m0lc9spzmxbq1by3ilyb8v2mdvjhcdpmd"))

(define rust-quote-1.0.45
  (crate-source "quote" "1.0.45"
                "095rb5rg7pbnwdp6v8w5jw93wndwyijgci1b5lw8j1h5cscn3wj1"))

(define rust-r-efi-5.3.0
  (crate-source "r-efi" "5.3.0"
                "03sbfm3g7myvzyylff6qaxk4z6fy76yv860yy66jiswc2m6b7kb9"))

(define rust-r-efi-6.0.0
  (crate-source "r-efi" "6.0.0"
                "1gyrl2k5fyzj9k7kchg2n296z5881lg7070msabid09asp3wkp7q"))

(define rust-radium-0.7.0
  (crate-source "radium" "0.7.0"
                "02cxfi3ky3c4yhyqx9axqwhyaca804ws46nn4gc1imbk94nzycyw"))

(define rust-rand-0.10.1
  (crate-source "rand" "0.10.1"
                "01r22vdpw6z69jzy6khnyr0ljq9im337h4j0mkyz26lnqyyfis6j"))

(define rust-rand-0.8.6
  (crate-source "rand" "0.8.6"
                "12kd4rljn86m00rcaz4c1rcya4mb4gk5ig6i8xq00a8wjgxfr82w"))

(define rust-rand-0.9.4
  (crate-source "rand" "0.9.4"
                "1sknbxgs6nfg0nxdd7689lwbyr2i4vaswchrv4b34z8vpc3azia4"))

(define rust-rand-chacha-0.3.1
  (crate-source "rand_chacha" "0.3.1"
                "123x2adin558xbhvqb8w4f6syjsdkmqff8cxwhmjacpsl1ihmhg6"))

(define rust-rand-chacha-0.9.0
  (crate-source "rand_chacha" "0.9.0"
                "1jr5ygix7r60pz0s1cv3ms1f6pd1i9pcdmnxzzhjc3zn3mgjn0nk"))

(define rust-rand-core-0.10.1
  (crate-source "rand_core" "0.10.1"
                "0s9wiacxrr100icl7i41308gcj85nlcclrc5jx1jd6p10dhigf33"))

(define rust-rand-core-0.6.4
  (crate-source "rand_core" "0.6.4"
                "0b4j2v4cb5krak1pv6kakv4sz6xcwbrmy2zckc32hsigbrwy82zc"))

(define rust-rand-core-0.9.5
  (crate-source "rand_core" "0.9.5"
                "0g6qc5r3f0hdmz9b11nripyp9qqrzb0xqk9piip8w8qlvqkcibvn"))

(define rust-rand-pcg-0.3.1
  (crate-source "rand_pcg" "0.3.1"
                "0gn79wzs5b19iivybwa09wv4lhi4kbcqciasiqqynggnr8cd1jjr"))

(define rust-rand-xoshiro-0.7.0
  (crate-source "rand_xoshiro" "0.7.0"
                "0h9dv9mn703zb2z5dys7vc4rzy3az8xg99fc5m8zbnh0axkg80zp"))

(define rust-random-pick-1.2.17
  (crate-source "random-pick" "1.2.17"
                "1pqy19c53b21yiylkwihialzl4fph4ijp1lp7hpqlkxvzqn18gar"))

(define rust-rapidhash-4.4.1
  (crate-source "rapidhash" "4.4.1"
                "0n8bp0ln1kcfk8cq4r7b5crq3vmm34qsndma6cpmw5cwjwq8kr5m"))

(define rust-rav1e-0.8.1
  (crate-source "rav1e" "0.8.1"
                "0axk3ji3jmlr81svmsy5zvj8shmhpp8lz5nyghkq752xx1bdvdj3"))

(define rust-ravif-0.13.0
  (crate-source "ravif" "0.13.0"
                "0ifcpczxf6kcsqlky08vbjrvw9yd1m9mfszywxdhy6wpglci08z5"))

(define rust-raw-window-handle-0.6.2
  (crate-source "raw-window-handle" "0.6.2"
                "0ff5c648hncwx7hm2a8fqgqlbvbl4xawb6v3xxv9wkpjyrr5arr0"))

(define rust-rayon-1.12.0
  (crate-source "rayon" "1.12.0"
                "0vcj63xgnk72c30vdrak7dhl53snnaqv9x2faf1d94hzg1kb2fgv"))

(define rust-rayon-core-1.13.0
  (crate-source "rayon-core" "1.13.0"
                "14dbr0sq83a6lf1rfjq5xdpk5r6zgzvmzs5j6110vlv2007qpq92"))

(define rust-recvmsg-1.0.0
  (crate-source "recvmsg" "1.0.0"
                "0xa173gbg1cx8q7wyzi6c4kmcsz5rka68r4jb6kg14icskax9vfk"))

(define rust-redb-2.6.3
  (crate-source "redb" "2.6.3"
                "00bczfznxw427a439c9fjkp0l04vba6y24q05l0fk9ymk2fixjlf"))

(define rust-redox-syscall-0.4.1
  (crate-source "redox_syscall" "0.4.1"
                "1aiifyz5dnybfvkk4cdab9p2kmphag1yad6iknc7aszlxxldf8j7"))

(define rust-redox-syscall-0.5.18
  (crate-source "redox_syscall" "0.5.18"
                "0b9n38zsxylql36vybw18if68yc9jczxmbyzdwyhb9sifmag4azd"))

(define rust-redox-syscall-0.8.1
  (crate-source "redox_syscall" "0.8.1"
                "1rrcn3nxva589cdhq1bhbvnxdbb6726f1lb5srbn9qx6yaabhi2v"))

(define rust-redox-users-0.4.6
  (crate-source "redox_users" "0.4.6"
                "0hya2cxx6hxmjfxzv9n8rjl5igpychav7zfi1f81pz6i4krry05s"))

(define rust-redox-users-0.5.2
  (crate-source "redox_users" "0.5.2"
                "1b17q7gf7w8b1vvl53bxna24xl983yn7bd00gfbii74bcg30irm4"))

(define rust-ref-cast-1.0.25
  (crate-source "ref-cast" "1.0.25"
                "0zdzc34qjva9xxgs889z5iz787g81hznk12zbk4g2xkgwq530m7k"))

(define rust-ref-cast-impl-1.0.25
  (crate-source "ref-cast-impl" "1.0.25"
                "1nkhn1fklmn342z5c4mzfzlxddv3x8yhxwwk02cj06djvh36065p"))

(define rust-regex-1.12.4
  (crate-source "regex" "1.12.4"
                "1fm6si2xpmhwqflabdqsakc0qkq718wx2ljl37nbj75fb5vjnagi"))

(define rust-regex-automata-0.4.14
  (crate-source "regex-automata" "0.4.14"
                "13xf7hhn4qmgfh784llcp2kzrvljd13lb2b1ca0mwnf15w9d87bf"))

(define rust-regex-filtered-0.2.1
  (crate-source "regex-filtered" "0.2.1"
                "04l63jnk33f4r4pkfpmsy33cgmgn2yxgkha3cv28qx7gzcqpnpxc"))

(define rust-regex-lite-0.1.9
  (crate-source "regex-lite" "0.1.9"
                "0wzr31ysmiy9sw48i36raqbm1iyk2xnq0lp4zbs6fzi47p3k9f6a"))

(define rust-regex-syntax-0.8.11
  (crate-source "regex-syntax" "0.8.11"
                "1m25h5q2wp976fb9gc3dsc9l99svcvd5cri8lncb51c46ydgzxnn"))

(define rust-regress-0.11.1
  (crate-source "regress" "0.11.1"
                "12lhp2cqf2ykbsz9sy2as0jxiy56l29kns0za3ika8jq6x27d2hm"))

(define rust-rend-0.4.2
  (crate-source "rend" "0.4.2"
                "0z4rrkycva0lcw0hxq479h4amxj9syn5vq4vb2qid5v2ylj3izki"))

(define rust-reqwest-0.13.4
  (crate-source "reqwest" "0.13.4"
                "1hy1plns9krbh3h1dy2sdjygsfkdcnxm6pbxdi0ya9b5vq8mi711"))

(define rust-rgb-0.8.53
  (crate-source "rgb" "0.8.53"
                "1i0c55whln68zs6f5qqrkbg1mzai0p3qk1mwkwzdgr9i3dw4pcs7"))

(define rust-ring-0.17.14
  (crate-source "ring" "0.17.14"
                "1dw32gv19ccq4hsx3ribhpdzri1vnrlcfqb2vj41xn4l49n9ws54"))

(define rust-rkyv-0.7.46
  (crate-source "rkyv" "0.7.46"
                "18fngrp1kzsmkkl7asl25661cm7hi05kf8cmpjbdrw53h6fbz5r2"))

(define rust-rkyv-derive-0.7.46
  (crate-source "rkyv_derive" "0.7.46"
                "1x9q626kkppbnbrbbw09nyz2r56b3frhxny87a6h81ld9cnv9mw4"))

(define rust-roaring-0.11.4
  (crate-source "roaring" "0.11.4"
                "1af4qdcpc7vb7z00457xygf44gr5p5djkwzmbvdkpjvfiijwbv8x"))

(define rust-ron-0.10.1
  (crate-source "ron" "0.10.1"
                "0zvv5mbzjd5hb4zgrw71154jn6wsdlsx2vggmrrkxiw1pzvvdkmy"))

(define rust-rust-decimal-1.42.0
  (crate-source "rust_decimal" "1.42.0"
                "15b9s1ll34n7ji45wcjbykwk8svv6yjjpw97mhdf40yrskihhl8c"))

(define rust-rust-stemmers-1.2.0
  (crate-source "rust-stemmers" "1.2.0"
                "0m6acgdflrrcm17dj7lp7x4sfqqhga24qynv660qinwz04v20sp4"))

(define rust-rustc-demangle-0.1.27
  (crate-source "rustc-demangle" "0.1.27"
                "17f0jl6lgsy8kwxdzxp3s2wmipvlpna03kkc4vkqr1gwv5lqh2xm"))

(define rust-rustc-hash-1.1.0
  (crate-source "rustc-hash" "1.1.0"
                "1qkc5khrmv5pqi5l5ca9p5nl5hs742cagrndhbrlk3dhlrx3zm08"))

(define rust-rustc-hash-2.1.2
  (crate-source "rustc-hash" "2.1.2"
                "1gjdc5bw9982cj176jvgz9rrqf9xvr1q1ddpzywf5qhs7yzhlc4l"))

(define rust-rustc-version-0.2.3
  (crate-source "rustc_version" "0.2.3"
                "02h3x57lcr8l2pm0a645s9whdh33pn5cnrwvn5cb57vcrc53x3hk"))

(define rust-rustc-version-0.4.1
  (crate-source "rustc_version" "0.4.1"
                "14lvdsmr5si5qbqzrajgb6vfn69k0sfygrvfvr2mps26xwi3mjyg"))

(define rust-rustc-version-runtime-0.3.0
  (crate-source "rustc_version_runtime" "0.3.0"
                "0787mz3zqkh7fmb88pxhag63y3qxlps58pmdnvq0m0p1pb98rl9d"))

(define rust-rusticata-macros-4.1.0
  (crate-source "rusticata-macros" "4.1.0"
                "0ch67lljmgl5pfrlb90bl5kkp2x6yby1qaxnpnd0p5g9xjkc9w7s"))

(define rust-rustix-0.38.44
  (crate-source "rustix" "0.38.44"
                "0m61v0h15lf5rrnbjhcb9306bgqrhskrqv7i1n0939dsw8dbrdgx"))

(define rust-rustix-1.1.4
  (crate-source "rustix" "1.1.4"
                "14511f9yjqh0ix07xjrjpllah3325774gfwi9zpq72sip5jlbzmn"))

(define rust-rustls-0.23.40
  (crate-source "rustls" "0.23.40"
                "12qnv3ag4wrw7aj8jng74kgrilpjm2b1rfcjaac8h691frccv1pg"))

(define rust-rustls-native-certs-0.8.4
  (crate-source "rustls-native-certs" "0.8.4"
                "0kgazl8zc1sv63qg179bz96ilzh56lzfa5k92ji7d265f4kibdfs"))

(define rust-rustls-pki-types-1.14.1
  (crate-source "rustls-pki-types" "1.14.1"
                "1a9pr54y0f3qr97bxpd3ahjldq0gqdld0h799xbnwdzbwxx1k9rh"))

(define rust-rustls-platform-verifier-0.7.0
  (crate-source "rustls-platform-verifier" "0.7.0"
                "181v4d0vl53vdh2wq56vghal1zyhdgqvy4xa8r45zwz4di9y5l96"))

(define rust-rustls-platform-verifier-android-0.1.1
  (crate-source "rustls-platform-verifier-android" "0.1.1"
                "13vq6sxsgz9547xm2zbdxiw8x7ad1g8n8ax6xvxsjqszk7q6awgq"))

(define rust-rustls-webpki-0.103.13
  (crate-source "rustls-webpki" "0.103.13"
                "0vkm7z9pnxz5qz66p2kmyy2pwx0g4jnsbqk5xzfhs4czcjl2ki31"))

(define rust-rustversion-1.0.22
  (crate-source "rustversion" "1.0.22"
                "0vfl70jhv72scd9rfqgr2n11m5i9l1acnk684m2w83w0zbqdx75k"))

(define rust-ryu-1.0.23
  (crate-source "ryu" "1.0.23"
                "0zs70sg00l2fb9jwrf6cbkdyscjs53anrvai2hf7npyyfi5blx4p"))

(define rust-ryu-js-1.0.2
  (crate-source "ryu-js" "1.0.2"
                "05gaq3mraijpinin02cxanpfjcic28z6f8wjnq1hkyyng0b66afx"))

(define rust-safe-arch-1.0.0
  (crate-source "safe_arch" "1.0.0"
                "1vgg8l61kqpg3wsrb29k3xfiyg0cf9576rylpicihmmxjk8alz0z"))

(define rust-same-file-1.0.6
  (crate-source "same-file" "1.0.6"
                "00h5j1w87dmhnvbv9l8bic3y7xxsnjmssvifw2ayvgx9mb1ivz4k"))

(define rust-schannel-0.1.29
  (crate-source "schannel" "0.1.29"
                "0ffrzz5vf2s3gnzvphgb5gg8fqifvryl07qcf7q3x1scj3jbghci"))

(define rust-schemars-0.9.0
  (crate-source "schemars" "0.9.0"
                "0pqncln5hqbzbl2r3yayyr4a82jjf93h2cfxrn0xamvx77wr3lac"))

(define rust-schemars-1.2.1
  (crate-source "schemars" "1.2.1"
                "1k16qzpdpy6p9hrh18q2l6cwawxzyqi25f8masa13l0wm8v2zd52"))

(define rust-scoped-tls-1.0.1
  (crate-source "scoped-tls" "1.0.1"
                "15524h04mafihcvfpgxd8f4bgc3k95aclz8grjkg9a0rxcvn9kz1"))

(define rust-scopeguard-1.2.0
  (crate-source "scopeguard" "1.2.0"
                "0jcz9sd47zlsgcnm1hdw0664krxwb5gczlif4qngj2aif8vky54l"))

(define rust-sea-query-1.0.1
  (crate-source "sea-query" "1.0.1"
                "1pxs17q392fbzm3wpkbrg5icfq3zqyhfbph4gn6qmsyc7gxhq6cd"))

(define rust-sea-query-derive-1.0.0
  (crate-source "sea-query-derive" "1.0.0"
                "0dvmxj0y2pb47895nfdbffl8kfix2z13lp4xp3s3rp8wj9kg9c50"))

(define rust-seahash-4.1.0
  (crate-source "seahash" "4.1.0"
                "0sxsb64np6bvnppjz5hg4rqpnkczhsl8w8kf2a5lr1c08xppn40w"))

(define rust-sec1-0.7.3
  (crate-source "sec1" "0.7.3"
                "1p273j8c87pid6a1iyyc7vxbvifrw55wbxgr0dh3l8vnbxb7msfk"))

(define rust-security-framework-3.7.0
  (crate-source "security-framework" "3.7.0"
                "07fd0j29j8yczb3hd430vwz784lx9knb5xwbvqna1nbkbivvrx5p"))

(define rust-security-framework-sys-2.17.0
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "security-framework-sys" "2.17.0"
                "1qr0w0y9iwvmv3hwg653q1igngnc5b74xcf0679cbv23z0fnkqkc"))

(define rust-self-cell-0.10.3
  (crate-source "self_cell" "0.10.3"
                "0pci3zh23b7dg6jmlxbn8k4plb7hcg5jprd1qiz0rp04p1ilskp1"))

(define rust-self-cell-1.2.2
  (crate-source "self_cell" "1.2.2"
                "12cdmh9p2h72rmw923kj841jji4k0vrykihvx19fn059az8pcbmi"))

(define rust-self-replace-1.5.0
  (crate-source "self-replace" "1.5.0"
                "1drganasvf5b0x6c9g60jkfhzjc9in3r6cznjfw0lhmbbrdq3v03"))

(define rust-semver-0.9.0
  (crate-source "semver" "0.9.0"
                "00q4lkcj0rrgbhviv9sd4p6qmdsipkwkbra7rh11jrhq5kpvjzhx"))

(define rust-semver-1.0.28
  (crate-source "semver" "1.0.28"
                "1kaimrpy876bcgi8bfj0qqfxk77zm9iz2zhn1hp9hj685z854y4a"))

(define rust-semver-parser-0.7.0
  (crate-source "semver-parser" "0.7.0"
                "18vhypw6zgccnrlm5ps1pwa0khz7ry927iznpr88b87cagr1v2iq"))

(define rust-seq-macro-0.3.6
  (crate-source "seq-macro" "0.3.6"
                "1k4sshn0x2i6a9g97sy5jl7ghlqgmmh3n76aj3rrjwxy1x0i3iqv"))

(define rust-serde-1.0.228
  (crate-source "serde" "1.0.228"
                "17mf4hhjxv5m90g42wmlbc61hdhlm6j9hwfkpcnd72rpgzm993ls"))

(define rust-serde-bytes-0.11.19
  (crate-source "serde_bytes" "0.11.19"
                "1a1y1v0r9akqyvprxnmpgc0i8wybqqpvgi01mi8qxn3rkrq41m55"))

(define rust-serde-core-1.0.228
  (crate-source "serde_core" "1.0.228"
                "1bb7id2xwx8izq50098s5j2sqrrvk31jbbrjqygyan6ask3qbls1"))

(define rust-serde-derive-1.0.228
  (crate-source "serde_derive" "1.0.228"
                "0y8xm7fvmr2kjcd029g9fijpndh8csv5m20g4bd76w8qschg4h6m"))

(define rust-serde-html-form-0.2.8
  (crate-source "serde_html_form" "0.2.8"
                "0kqmp0m7vj8lrs1n2hjcp1jhhpzw81f9ycmv30vk6h11ibzxgwmj"))

(define rust-serde-html-form-0.4.0
  (crate-source "serde_html_form" "0.4.0"
                "0vwy16xk91ps0kaj0ybkmnawa4hhj3myrfxf90qq4a3y9cmxaih9"))

(define rust-serde-json-1.0.150
  (crate-source "serde_json" "1.0.150"
                "1ffgfhy9kndjnrz8lmy95pr758p2zk8dxv6yi99x0vkkni24w0g8"))

(define rust-serde-path-to-error-0.1.20
  (crate-source "serde_path_to_error" "0.1.20"
                "0mxls44p2ycmnxh03zpnlxxygq42w61ws7ir7r0ba6rp5s1gza8h"))

(define rust-serde-repr-0.1.20
  (crate-source "serde_repr" "0.1.20"
                "1755gss3f6lwvv23pk7fhnjdkjw7609rcgjlr8vjg6791blf6php"))

(define rust-serde-spanned-0.6.9
  (crate-source "serde_spanned" "0.6.9"
                "18vmxq6qfrm110caszxrzibjhy2s54n1g5w1bshxq9kjmz7y0hdz"))

(define rust-serde-spanned-1.1.1
  (crate-source "serde_spanned" "1.1.1"
                "09jzk7i6wihn3d8i3wi4j4n98ghi93c3b8m8k64nxq0ijn3vaqk6"))

(define rust-serde-untagged-0.1.9
  (crate-source "serde-untagged" "0.1.9"
                "0n2hdjzas7w949klw1rpfzmpc9sm4sz9sa664jz969id9a5g9ypr"))

(define rust-serde-urlencoded-0.7.1
  (crate-source "serde_urlencoded" "0.7.1"
                "1zgklbdaysj3230xivihs30qi5vkhigg323a9m62k8jwf4a1qjfk"))

(define rust-serde-value-0.7.0
  (crate-source "serde-value" "0.7.0"
                "0b18ngk7n4f9zmwsfdkhgsp31192smzyl5z143qmx1qi28sa78gk"))

(define rust-serde-with-3.17.0
  (crate-source "serde_with" "3.17.0"
                "1ff3pzf4dyxl9pv2ffv35djk6rnks1czp5ijj1nlfsxwwwy2h6rq"))

(define rust-serde-with-macros-3.17.0
  (crate-source "serde_with_macros" "3.17.0"
                "1q17icvf0mcl752my58fx9is9jgf4f2cl7dbsrp31jy8fc2y7m56"))

(define rust-serde-yaml-0.9.34+deprecated
  (crate-source "serde_yaml" "0.9.34+deprecated"
                "0isba1fjyg3l6rxk156k600ilzr8fp7crv82rhal0rxz5qd1m2va"))

(define rust-serial-test-3.5.0
  (crate-source "serial_test" "3.5.0"
                "0v8w12g3isnabvs6wqn4dk1gxzyn9dd336lwy5zpx2jv26bl37v9"))

(define rust-serial-test-derive-3.5.0
  (crate-source "serial_test_derive" "3.5.0"
                "0g378b1pn2fcb7d46zjb3h35rcd0132jjv9xf1la1ip1fvy57qcl"))

(define rust-sha1-0.10.6
  (crate-source "sha1" "0.10.6"
                "1fnnxlfg08xhkmwf2ahv634as30l1i3xhlhkvxflmasi5nd85gz3"))

(define rust-sha1-checked-0.10.0
  (crate-source "sha1-checked" "0.10.0"
                "08s4h1drgwxzfn1mk11rn0r9i0rbjra1m0l2c0fbngij1jn9kxc9"))

(define rust-sha1-smol-1.0.1
  (crate-source "sha1_smol" "1.0.1"
                "0pbh2xjfnzgblws3hims0ib5bphv7r5rfdpizyh51vnzvnribymv"))

(define rust-sha2-0.10.9
  (crate-source "sha2" "0.10.9"
                "10xjj843v31ghsksd9sl9y12qfc48157j1xpb8v1ml39jy0psl57"))

(define rust-sha3-0.10.9
  (crate-source "sha3" "0.10.9"
                "0x1qv415b59x9vw4afr3fh98bcca9z6pg1yg6i05lhax6hl71zbp"))

(define rust-sharded-slab-0.1.7
  (crate-source "sharded-slab" "0.1.7"
                "1xipjr4nqsgw34k7a2cgj9zaasl2ds6jwn89886kww93d32a637l"))

(define rust-shell-words-1.1.1
  (crate-source "shell-words" "1.1.1"
                "0xzd5p53xl0ndnk63r0by52rhdrh6pd37szfxszkg73zb6ffcvyw"))

(define rust-shlex-1.3.0
  (crate-source "shlex" "1.3.0"
                "0r1y6bv26c1scpxvhg2cabimrmwgbp4p3wy6syj9n0c4s3q2znhg"))

(define rust-shlex-2.0.1
  (crate-source "shlex" "2.0.1"
                "1fjsll1cd7d2bcpdij9kd6w62rpbc7qqzvydvs021vsmr1cxvypq"))

(define rust-shuttle-0.8.1
  (crate-source "shuttle" "0.8.1"
                "0caf5cfdvhd5i6394j60qbz6fx6g18vgf33q8rzh8qwdlgdpxc9a"))

(define rust-signal-hook-0.3.18
  (crate-source "signal-hook" "0.3.18"
                "1qnnbq4g2vixfmlv28i1whkr0hikrf1bsc4xjy2aasj2yina30fq"))

(define rust-signal-hook-0.4.4
  (crate-source "signal-hook" "0.4.4"
                "0gdm8kmi1mcd30gkxcwagxiqiasq0fhdlvrfsnybv3chln6c585j"))

(define rust-signal-hook-mio-0.2.5
  (crate-source "signal-hook-mio" "0.2.5"
                "1k20rr76ngvmzr6kskkl7dv8iyb84cbydpjbjk3mpcj0lykijnmp"))

(define rust-signal-hook-registry-1.4.8
  (crate-source "signal-hook-registry" "1.4.8"
                "06vc7pmnki6lmxar3z31gkyg9cw7py5x9g7px70gy2hil75nkny4"))

(define rust-signature-2.2.0
  (crate-source "signature" "2.2.0"
                "1pi9hd5vqfr3q3k49k37z06p7gs5si0in32qia4mmr1dancr6m3p"))

(define rust-simd-adler32-0.3.9
  (crate-source "simd-adler32" "0.3.9"
                "0532ysdwcvzyp2bwpk8qz0hijplcdwpssr5gy5r7qwqqy5z5qgbh"))

(define rust-simd-cesu8-1.1.1
  (crate-source "simd_cesu8" "1.1.1"
                "0crcbgvyycmazji2vqj9vxn2czdyl3gxmicp4xqdzkc7pdbh3ycl"))

(define rust-simd-helpers-0.1.0
  (crate-source "simd_helpers" "0.1.0"
                "19idqicn9k4vhd04ifh2ff41wvna79zphdf2c81rlmpc7f3hz2cm"))

(define rust-simdutf8-0.1.5
  (crate-source "simdutf8" "0.1.5"
                "0vmpf7xaa0dnaikib5jlx6y4dxd3hxqz6l830qb079g7wcsgxag3"))

(define rust-simsimd-6.5.16
  (crate-source "simsimd" "6.5.16"
                "0w7saci0l4149fsq413986zgcrx9i97wb96askbsf1yfrp1kpyzl"))

(define rust-siphasher-0.3.11
  (crate-source "siphasher" "0.3.11"
                "03axamhmwsrmh0psdw3gf7c0zc4fyl5yjxfifz9qfka6yhkqid9q"))

(define rust-siphasher-1.0.3
  (crate-source "siphasher" "1.0.3"
                "0jg6l9xyzca5vy4h6gf8r6p4kk84g98fk95pzig1kq6cr4z8grcf"))

(define rust-sketches-ddsketch-0.4.0
  (crate-source "sketches-ddsketch" "0.4.0"
                "1sj6pfzv89qci5jfz45d0q4y7af9d6wk2d92lb0qv62dymn0pr05"))

(define rust-slab-0.4.12
  (crate-source "slab" "0.4.12"
                "1xcwik6s6zbd3lf51kkrcicdq2j4c1fw0yjdai2apy9467i0sy8c"))

(define rust-small-btree-0.1.0.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-small-btree" "0.1.0.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-smallvec-1.15.1
  (crate-source "smallvec" "1.15.1"
                "00xxdxxpgyq5vjnpljvkmy99xij5rxgh913ii1v16kzynnivgcb7"))

(define rust-smart-default-0.7.1
  (crate-source "smart-default" "0.7.1"
                "1hgzs1250559bpayxmn46gzas5ycqn39wkf4srjgqh4461k1ic0f"))

(define rust-smartstring-1.0.1
  (crate-source "smartstring" "1.0.1"
                "0agf4x0jz79r30aqibyfjm1h9hrjdh0harcqcvb2vapv7rijrdrz"))

(define rust-smithay-client-toolkit-0.19.2
  (crate-source "smithay-client-toolkit" "0.19.2"
                "05h05hg4dn3v6br5jbdbs5nalk076a64s7fn6i01nqzby2hxwmrl"))

(define rust-smithay-client-toolkit-0.20.0
  (crate-source "smithay-client-toolkit" "0.20.0"
                "1h2cacmsh9zpw6sgmap49zx7cqhksfwas91mm40i5cz2ylwdl4h5"))

(define rust-smol-str-0.2.2
  (crate-source "smol_str" "0.2.2"
                "1bfylqf2vnqaglw58930vpxm2rfzji5gjp15a2c0kh8aj6v8ylyx"))

(define rust-socket2-0.5.10
  (crate-source "socket2" "0.5.10"
                "0y067ki5q946w91xlz2sb175pnfazizva6fi3kfp639mxnmpc8z2"))

(define rust-socket2-0.6.4
  (crate-source "socket2" "0.6.4"
                "0ldyp5rhba15spwxj1n94xh7sjks1398c3vwpwkxkd1087nwzlaj"))

(define rust-softaes-0.1.5
  (crate-source "softaes" "0.1.5"
                "1spf9hd915mgsa22d768rzajf2ddmr95ghkpygfrgrndvsbl5qa5"))

(define rust-spin-0.9.8
  (crate-source "spin" "0.9.8"
                "0rvam5r0p3a6qhc18scqpvpgb3ckzyqxpgdfyjnghh8ja7byi039"))

(define rust-spki-0.7.3
  (crate-source "spki" "0.7.3"
                "17fj8k5fmx4w9mp27l970clrh5qa7r5sjdvbsln987xhb34dc7nr"))

(define rust-sptr-0.3.2
  (crate-source "sptr" "0.3.2"
                "0shddkys046nnrng929mrnjjrh31mlxl95ky7dgxd6i4kclkk6rv"))

(define rust-stability-0.2.1
  (crate-source "stability" "0.2.1"
                "1b7w6qknq0w5y7s358j62pzi9kbh6g73lal3jx9aydpikl0ff16r"))

(define rust-stable-deref-trait-1.2.1
  (crate-source "stable_deref_trait" "1.2.1"
                "15h5h73ppqyhdhx6ywxfj88azmrpml9gl6zp3pwy2malqa6vxqkc"))

(define rust-stacker-0.1.24
  (crate-source "stacker" "0.1.24"
                "141i1f49xgnsfymvk5kbddg6xfaspwxwl0qqrddjzcdnjbfqq334"))

(define rust-static-assertions-1.1.0
  (crate-source "static_assertions" "1.1.0"
                "0gsl6xmw10gvn3zs1rv99laj5ig7ylffnh71f9l34js4nr4r7sx2"))

(define rust-strength-reduce-0.2.4
  (crate-source "strength_reduce" "0.2.4"
                "10jdq9dijjdkb20wg1dmwg447rnj37jbq0mwvbadvqi2gys5x2gy"))

(define rust-string-cache-0.8.9
  (crate-source "string_cache" "0.8.9"
                "03z7km2kzlwiv2r2qifq5riv4g8phazwng9wnvs3py3lzainnxxz"))

(define rust-string-cache-0.9.0
  (crate-source "string_cache" "0.9.0"
                "008rwf8gd1xhwr523r5zzzgypgkfmrz6l3wwh7r2k9w5qzw9d1d1"))

(define rust-string-cache-codegen-0.6.1
  (crate-source "string_cache_codegen" "0.6.1"
                "0scvya8dsfard2r8m7pb2cjnar312jc9g165fsghacdjdpj3amjq"))

(define rust-string-enum-1.0.2
  (crate-source "string_enum" "1.0.2"
                "03h3wijj8gzvjgvd9rzimzv70jl2m621a90wk7yirgd73jas8dmf"))

(define rust-strsim-0.11.1
  (crate-source "strsim" "0.11.1"
                "0kzvqlw8hxqb7y598w1s0hxlnmi84sg5vsipp3yg5na5d1rvba3x"))

(define rust-strum-0.26.3
  (crate-source "strum" "0.26.3"
                "01lgl6jvrf4j28v5kmx9bp480ygf1nhvac8b4p7rcj9hxw50zv4g"))

(define rust-strum-0.28.0
  (crate-source "strum" "0.28.0"
                "1ggr0if083c1mz9w33hkdjsp0iqk2fz9n49bvb73knwihydxwa4n"))

(define rust-strum-macros-0.26.4
  (crate-source "strum_macros" "0.26.4"
                "1gl1wmq24b8md527cpyd5bw9rkbqldd7k1h38kf5ajd2ln2ywssc"))

(define rust-strum-macros-0.28.0
  (crate-source "strum_macros" "0.28.0"
                "0r7n6v5b3x85m52isyc8wq78irmr22g0hmj1xn3pbq8f4yhfx1db"))

(define rust-substring-1.4.5
  (crate-source "substring" "1.4.5"
                "11jcadn4h1xwx3dq5gbgs5y3x57ml9jfz1zmf8p3n8ggxhrn9vj2"))

(define rust-subtle-2.6.1
  (crate-source "subtle" "2.6.1"
                "14ijxaymghbl1p0wql9cib5zlwiina7kall6w7g89csprkgbvhhk"))

(define rust-swc-allocator-4.0.1
  (crate-source "swc_allocator" "4.0.1"
                "1fs8riq8bjnfalxqwqq7pqghg8dkwhm2nj2n633sha5jr39fyzlx"))

(define rust-swc-atoms-9.0.0
  (crate-source "swc_atoms" "9.0.0"
                "12mvg2h2636hhhqsag78msl75ndi0yhpiy00451xf2nir8pbxk6l"))

(define rust-swc-common-17.0.1
  (crate-source "swc_common" "17.0.1"
                "0k6gl9b2v81s5lid6qphx3k1ijaqi3caj0iqzr7d49iscdfng6r5"))

(define rust-swc-config-3.1.2
  (crate-source "swc_config" "3.1.2"
                "15rzad529gc30gilzxsczy3ss8hp20c24q84f63fskbkxr90psbj"))

(define rust-swc-config-macro-1.0.1
  (crate-source "swc_config_macro" "1.0.1"
                "1cxh1m4kdngfcphd4f7g7w7bqnxk29q0rqcnligdq5yyws66whbv"))

(define rust-swc-ecma-ast-18.0.0
  (crate-source "swc_ecma_ast" "18.0.0"
                "0z6sfq3xblfj8ar96c0caai3bbvqawniapq8v3acipjh533s0wx5"))

(define rust-swc-ecma-codegen-20.0.2
  (crate-source "swc_ecma_codegen" "20.0.2"
                "1v9l7lwpl9c3i6r4mcwvilkkz2sfgcaf8m5cvvgaipa9xkhnwapz"))

(define rust-swc-ecma-codegen-macros-2.0.2
  (crate-source "swc_ecma_codegen_macros" "2.0.2"
                "1f6v9p684nlkq2p7yvygbxagv4rar24pk0lp0db5lqm2q1idqxp2"))

(define rust-swc-ecma-lexer-26.0.0
  (crate-source "swc_ecma_lexer" "26.0.0"
                "1z28vd8zc0bnf7pyakz9wrpb8csfn7g4myhiw7v6yb05grsgg0jy"))

(define rust-swc-ecma-loader-17.0.0
  (crate-source "swc_ecma_loader" "17.0.0"
                "1xf8s8yfj8c06dcfazb8k7x5cfpbfz2v4m58l23nbm7h92xvmjpv"))

(define rust-swc-ecma-parser-27.0.7
  (crate-source "swc_ecma_parser" "27.0.7"
                "1msjdzfn3j9lgci5hpd402293h5vj7j97ckkq024kkcj3apm26kz"))

(define rust-swc-ecma-transforms-base-30.0.1
  (crate-source "swc_ecma_transforms_base" "30.0.1"
                "1zl331dzmdgj8zgr2rgayi5wk5wm3df5hmxx8zp4zjkqalb6y3r5"))

(define rust-swc-ecma-transforms-classes-30.0.0
  (crate-source "swc_ecma_transforms_classes" "30.0.0"
                "1xgxfbvlsmbvggzbzbcj49high2vqgj8ly88sw10x62azxgb6fix"))

(define rust-swc-ecma-transforms-macros-1.0.1
  (crate-source "swc_ecma_transforms_macros" "1.0.1"
                "1n2wrz86mcbh1j5a993409cvmazvwib5lch0a9p7ixlvg6474xxw"))

(define rust-swc-ecma-transforms-proposal-30.0.0
  (crate-source "swc_ecma_transforms_proposal" "30.0.0"
                "0mx2c13s183pgm5ifh0scvkvzkj399g06q2ji3hprj0j866p9my2"))

(define rust-swc-ecma-transforms-react-33.0.0
  (crate-source "swc_ecma_transforms_react" "33.0.0"
                "10l697r554gqh4w21wplzcb7p7bsscx7jvspmjb1rb27ivii5ph3"))

(define rust-swc-ecma-transforms-typescript-33.0.0
  (crate-source "swc_ecma_transforms_typescript" "33.0.0"
                "1c7pwkyrvwag57wh6qbvyn3b7bm0i4hn5nsr6szzlhdmvq7q0224"))

(define rust-swc-ecma-utils-24.0.0
  (crate-source "swc_ecma_utils" "24.0.0"
                "1c35d82qyapjp8ycm0v92xvvgg22p4m4b93r6x3vvjl8k4brxf8g"))

(define rust-swc-ecma-visit-18.0.1
  (crate-source "swc_ecma_visit" "18.0.1"
                "0qqxrjl6h8x5jmnhknq4650laliaszjr98s7hmh65380lir1lqd9"))

(define rust-swc-eq-ignore-macros-1.0.1
  (crate-source "swc_eq_ignore_macros" "1.0.1"
                "0cmnsh8pg2r708vi3vbg95jpgfky41mblrchw2anwcd64hsffv61"))

(define rust-swc-macros-common-1.0.1
  (crate-source "swc_macros_common" "1.0.1"
                "1bma5z6lsayznk2z721isvjdfxwbsz5idyx2s9ddqhs9lyxfzqda"))

(define rust-swc-sourcemap-9.3.4
  (crate-source "swc_sourcemap" "9.3.4"
                "1my4cimzlmpj25bgxblk43pyz7m5w70832pfb0ddvb0nz00fy26y"))

(define rust-swc-visit-2.0.1
  (crate-source "swc_visit" "2.0.1"
                "18ijw8nvp5544vs01mxa88kp5x77mc72y5yj6ig1hv289d473yv2"))

(define rust-symlink-0.1.0
  (crate-source "symlink" "0.1.0"
                "02h1i0b81mxb4vns4xrvrfibpcvs7jqqav8p3yilwik8cv73r5x7"))

(define rust-syn-1.0.109
  (crate-source "syn" "1.0.109"
                "0ds2if4600bd59wsv7jjgfkayfzy3hnazs394kz6zdkmna8l3dkj"))

(define rust-syn-2.0.117
  (crate-source "syn" "2.0.117"
                "16cv7c0wbn8amxc54n4w15kxlx5ypdmla8s0gxr2l7bv7s0bhrg6"))

(define rust-sync-wrapper-1.0.2
  (crate-source "sync_wrapper" "1.0.2"
                "0qvjyasd6w18mjg5xlaq5jgy84jsjfsvmnn12c13gypxbv75dwhb"))

(define rust-synstructure-0.13.2
  (crate-source "synstructure" "0.13.2"
                "1lh9lx3r3jb18f8sbj29am5hm9jymvbwh6jb1izsnnxgvgrp12kj"))

(define rust-sys-locale-0.3.2
  (crate-source "sys-locale" "0.3.2"
                "1i16hq9mkwpzqvixjfy1ph4i2q5klgagjg4hibz6k894l2crmawf"))

(define rust-sysinfo-0.37.2
  (crate-source "sysinfo" "0.37.2"
                "07xizvikp5j2f6jky0j4vlaxp21djznzja1m0z70f77xmxf7sq0n"))

(define rust-system-configuration-0.7.0
  (crate-source "system-configuration" "0.7.0"
                "12rwilylzc625qnxl30h5kf8wj5ka61zjrwpmb034cd0mc6ksgx1"))

(define rust-system-configuration-sys-0.6.0
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "system-configuration-sys" "0.6.0"
                "1i5sqrmgy58l4704hibjbl36hclddglh73fb3wx95jnmrq81n7cf"))

(define rust-tag-ptr-0.1.0.ffec924
  ;; TODO REVIEW: Define standalone package if this is a workspace.
  (origin
    (method git-fetch)
    (uri (git-reference (url "https://github.com/boa-dev/boa.git")
                        (commit "ffec9244d4267406d66aef8b3c8a1d89730df5b4")))
    (file-name (git-file-name "rust-tag-ptr" "0.1.0.ffec924"))
    (sha256 (base32 "1810sdy40xf99xpdml34j5r0pq1j95s44qxxvrlf8dy2nzxxw409"))))

(define rust-takecrate-1.1.1
  (crate-source "takecrate" "1.1.1"
                "163wizlqsm3x4h6f4g5jy3x090kshh39srhv726l5zqk8cv48fph"))

(define rust-tantivy-0.26.1
  (crate-source "tantivy" "0.26.1"
                "01xc9qy14zxhy8wjsz8v62nwpwzm1c1fzjd8w6j01zrzfh86mppd"))

(define rust-tantivy-bitpacker-0.10.0
  (crate-source "tantivy-bitpacker" "0.10.0"
                "191zylapdbww069ykk2ykkyqmvaz96jilv8abpgd5g198ikkvvag"))

(define rust-tantivy-columnar-0.7.0
  (crate-source "tantivy-columnar" "0.7.0"
                "1p65hdamky5aa72fap7s054f9p3q8vxmli5q18vqyizxpksncwf5"))

(define rust-tantivy-common-0.11.0
  (crate-source "tantivy-common" "0.11.0"
                "1r93p9mxwpbiqfj84j9dyfxfz2g8s99qidaq1lxkrnkmm8ahkwdv"))

(define rust-tantivy-fst-0.5.0
  (crate-source "tantivy-fst" "0.5.0"
                "067wcvc39h209j9srj6cdfndrg1bfbzcsw1cgf53v5fp1aw6j1yn"))

(define rust-tantivy-query-grammar-0.26.0
  (crate-source "tantivy-query-grammar" "0.26.0"
                "10hx1zimrp729cs46ccq2km65bka39qb14xjzq20gabddd9bibfz"))

(define rust-tantivy-sstable-0.7.0
  (crate-source "tantivy-sstable" "0.7.0"
                "01n6jrjk3m6qka2rfn5fc0m5hiwgb8agnpwn53fblk0nqlxgqb4a"))

(define rust-tantivy-stacker-0.7.0
  (crate-source "tantivy-stacker" "0.7.0"
                "0ldrzsgz2b9ps0ahwb2fn8w96c8fn6ll7zwgkv5577fs88bhbfvc"))

(define rust-tantivy-tokenizer-api-0.7.0
  (crate-source "tantivy-tokenizer-api" "0.7.0"
                "161z4qvwr3c38vyd73fyww74x36bmzpay4sqd3r761irqv15ihpa"))

(define rust-tap-1.0.1
  (crate-source "tap" "1.0.1"
                "0sc3gl4nldqpvyhqi3bbd0l9k7fngrcl4zs47n314nqqk4bpx4sm"))

(define rust-tar-0.4.46
  (crate-source "tar" "0.4.46"
                "0h68bc0y1nma3h2ypj28vxc84msjydlrj8rviqwphg00lvcj2qiz"))

(define rust-tempfile-3.27.0
  (crate-source "tempfile" "3.27.0"
                "1gblhnyfjsbg9wjg194n89wrzah7jy3yzgnyzhp56f3v9jd7wj9j"))

(define rust-temporal-rs-0.2.3
  (crate-source "temporal_rs" "0.2.3"
                "1pnl1gak7qy9in0r4bbfzqg60r15r7pmblr1dcc7al9f512jm44s"))

(define rust-tendril-0.5.0
  (crate-source "tendril" "0.5.0"
                "090dcvslanahwjnm4ihggjiv7fc82gir9c24nps319fmd71hyyf4"))

(define rust-term-0.7.0
  (crate-source "term" "0.7.0"
                "07xzxmg7dbhlirpyfq09v7cfb9gxn0077sqqvszgjvyrjnngi7f5"))

(define rust-termcolor-1.4.1
  (crate-source "termcolor" "1.4.1"
                "0mappjh3fj3p2nmrg4y7qv94rchwi9mzmgmfflr8p2awdj7lyy86"))

(define rust-termsize-0.1.9
  (crate-source "termsize" "0.1.9"
                "1zb80dcqngbvw1mgkdsagwx6hvcsxr1zpql5bf6n0wn14mfgy4bg"))

(define rust-text-io-0.1.13
  (crate-source "text_io" "0.1.13"
                "058ifqlmnf15jy7rr1mm20m2sw8hx6aqj7c40d70k4k2n2ikr3ad"))

(define rust-text-lines-0.6.0
  (crate-source "text_lines" "0.6.0"
                "1kwv0ln0gy7cczmpk6r7scrfwfvbx5m004yp3lp7ianywy6q5mbz"))

(define rust-thin-vec-0.2.18
  (crate-source "thin-vec" "0.2.18"
                "10ml7530igcr5xdnl21z6z07zihcnljgm0362k87s2lgnily5xxh"))

(define rust-thiserror-1.0.69
  (crate-source "thiserror" "1.0.69"
                "0lizjay08agcr5hs9yfzzj6axs53a2rgx070a1dsi3jpkcrzbamn"))

(define rust-thiserror-2.0.18
  (crate-source "thiserror" "2.0.18"
                "1i7vcmw9900bvsmay7mww04ahahab7wmr8s925xc083rpjybb222"))

(define rust-thiserror-impl-1.0.69
  (crate-source "thiserror-impl" "1.0.69"
                "1h84fmn2nai41cxbhk6pqf46bxqq1b344v8yz089w1chzi76rvjg"))

(define rust-thiserror-impl-2.0.18
  (crate-source "thiserror-impl" "2.0.18"
                "1mf1vrbbimj1g6dvhdgzjmn6q09yflz2b92zs1j9n3k7cxzyxi7b"))

(define rust-thread-local-1.1.9
  (crate-source "thread_local" "1.1.9"
                "1191jvl8d63agnq06pcnarivf63qzgpws5xa33hgc92gjjj4c0pn"))

(define rust-tiff-0.11.3
  (crate-source "tiff" "0.11.3"
                "0lmw68ic77sixk17r4rl2vsv00rqhja3yj2h9p5bcd9x6krylgxn"))

(define rust-time-0.3.47
  (crate-source "time" "0.3.47"
                "0b7g9ly2iabrlgizliz6v5x23yq5d6bpp0mqz6407z1s526d8fvl"))

(define rust-time-core-0.1.8
  (crate-source "time-core" "0.1.8"
                "1jidl426mw48i7hjj4hs9vxgd9lwqq4vyalm4q8d7y4iwz7y353n"))

(define rust-time-macros-0.2.27
  (crate-source "time-macros" "0.2.27"
                "058ja265waq275wxvnfwavbz9r1hd4dgwpfn7a1a9a70l32y8w1f"))

(define rust-timezone-provider-0.2.3
  (crate-source "timezone_provider" "0.2.3"
                "1hm8k7ik23hr8g1rxl5bbri4j7i8cmyfkpz4a4q82awac829p3y4"))

(define rust-tiny-keccak-2.0.2
  (crate-source "tiny-keccak" "2.0.2"
                "0dq2x0hjffmixgyf6xv9wgsbcxkd65ld0wrfqmagji8a829kg79c"))

(define rust-tinystr-0.8.3
  (crate-source "tinystr" "0.8.3"
                "0vfr8x285w6zsqhna0a9jyhylwiafb2kc8pj2qaqaahw48236cn8"))

(define rust-tinyvec-1.11.0
  (crate-source "tinyvec" "1.11.0"
                "1wvycrghzmaysnw34kzwnf0mfx6r75045s24r214wnnjadqfcq9y"))

(define rust-tinyvec-macros-0.1.1
  (crate-source "tinyvec_macros" "0.1.1"
                "081gag86208sc3y6sdkshgw3vysm5d34p431dzw0bshz66ncng0z"))

(define rust-tokio-1.52.3
  (crate-source "tokio" "1.52.3"
                "1zpzazypkg61sw91na1m85x5s4rsjym335fwwhwm1hcs70dz1iwg"))

(define rust-tokio-macros-2.7.0
  (crate-source "tokio-macros" "2.7.0"
                "15m4f37mdafs0gg36sh0rskm1i768lb7zmp8bw67kaxr3avnqniq"))

(define rust-tokio-rustls-0.26.4
  (crate-source "tokio-rustls" "0.26.4"
                "0qggwknz9w4bbsv1z158hlnpkm97j3w8v31586jipn99byaala8p"))

(define rust-tokio-stream-0.1.18
  (crate-source "tokio-stream" "0.1.18"
                "0w3cj33605ab58wqd382gnla5pnd9hnr00xgg333np5bka04knij"))

(define rust-tokio-util-0.7.18
  (crate-source "tokio-util" "0.7.18"
                "1600rd47pylwn7cap1k7s5nvdaa9j7w8kqigzp1qy7mh0p4cxscs"))

(define rust-toml-0.8.23
  (crate-source "toml" "0.8.23"
                "0qnkrq4lm2sdhp3l6cb6f26i8zbnhqb7mhbmksd550wxdfcyn6yw"))

(define rust-toml-0.9.12+spec-1.1.0
  (crate-source "toml" "0.9.12+spec-1.1.0"
                "0qwqbrymqn88mg2yqyq3rj52z6p20448z0jxdbpjsbpwg5g894ng"))

(define rust-toml-1.1.2+spec-1.1.0
  (crate-source "toml" "1.1.2+spec-1.1.0"
                "1vpggpamqhw4852kic7465zsidczsla06wz6friqkkfbhigd3ww1"))

(define rust-toml-datetime-0.6.11
  (crate-source "toml_datetime" "0.6.11"
                "077ix2hb1dcya49hmi1avalwbixmrs75zgzb3b2i7g2gizwdmk92"))

(define rust-toml-datetime-0.7.5+spec-1.1.0
  (crate-source "toml_datetime" "0.7.5+spec-1.1.0"
                "0iqkgvgsxmszpai53dbip7sf2igic39s4dby29dbqf1h9bnwzqcj"))

(define rust-toml-datetime-1.1.1+spec-1.1.0
  (crate-source "toml_datetime" "1.1.1+spec-1.1.0"
                "1mws2mkkf46l7inn77azhm0vdwxngv9vsbhbl0ah33p2c9gzcr9i"))

(define rust-toml-edit-0.19.15
  (crate-source "toml_edit" "0.19.15"
                "08bl7rp5g6jwmfpad9s8jpw8wjrciadpnbaswgywpr9hv9qbfnqv"))

(define rust-toml-edit-0.22.27
  (crate-source "toml_edit" "0.22.27"
                "16l15xm40404asih8vyjvnka9g0xs9i4hfb6ry3ph9g419k8rzj1"))

(define rust-toml-edit-0.25.12+spec-1.1.0
  (crate-source "toml_edit" "0.25.12+spec-1.1.0"
                "1mx5paq837rjw7w51zprrjynk1vaig9yzxfqz9ac79jmd7f3w5fj"))

(define rust-toml-parser-1.1.2+spec-1.1.0
  (crate-source "toml_parser" "1.1.2+spec-1.1.0"
                "09kmzc55a0j21whm290wlf5a8b18a0qc87a1s8sncrckc6wfkax2"))

(define rust-toml-write-0.1.2
  (crate-source "toml_write" "0.1.2"
                "008qlhqlqvljp1gpp9rn5cqs74gwvdgbvs92wnpq8y3jlz4zi6ax"))

(define rust-toml-writer-1.1.1+spec-1.1.0
  (crate-source "toml_writer" "1.1.1+spec-1.1.0"
                "1nwjhvvrxz8f4ck1qi4xcz2x9qhpci37nrknhxxf9sqk22dsyvbm"))

(define rust-tower-0.5.3
  (crate-source "tower" "0.5.3"
                "1m5i3a2z1sgs8nnz1hgfq2nr4clpdmizlp1d9qsg358ma5iyzrgb"))

(define rust-tower-http-0.6.11
  (crate-source "tower-http" "0.6.11"
                "0h08wjgs3hwnq11iwwzlmnabn1h4cl0fzd48svaccvqffkiggz2c"))

(define rust-tower-layer-0.3.3
  (crate-source "tower-layer" "0.3.3"
                "03kq92fdzxin51w8iqix06dcfgydyvx7yr6izjq0p626v9n2l70j"))

(define rust-tower-service-0.3.3
  (crate-source "tower-service" "0.3.3"
                "1hzfkvkci33ra94xjx64vv3pp0sq346w06fpkcdwjcid7zhvdycd"))

(define rust-tracing-0.1.44
  (crate-source "tracing" "0.1.44"
                "006ilqkg1lmfdh3xhg3z762izfwmxcvz0w7m4qx2qajbz9i1drv3"))

(define rust-tracing-appender-0.2.5
  (crate-source "tracing-appender" "0.2.5"
                "0g4a6q5s3wafid5lqw1ljzvh1nhk3a4zmb627fxv96dr7qcqc1h5"))

(define rust-tracing-attributes-0.1.31
  (crate-source "tracing-attributes" "0.1.31"
                "1np8d77shfvz0n7camx2bsf1qw0zg331lra0hxb4cdwnxjjwz43l"))

(define rust-tracing-core-0.1.36
  (crate-source "tracing-core" "0.1.36"
                "16mpbz6p8vd6j7sf925k9k8wzvm9vdfsjbynbmaxxyq6v7wwm5yv"))

(define rust-tracing-log-0.2.0
  (crate-source "tracing-log" "0.2.0"
                "1hs77z026k730ij1a9dhahzrl0s073gfa2hm5p0fbl0b80gmz1gf"))

(define rust-tracing-serde-0.2.0
  (crate-source "tracing-serde" "0.2.0"
                "1wbgzi364vzfswfkvy48a3p0z5xmv98sx342r57sil70ggmiljvh"))

(define rust-tracing-subscriber-0.3.23
  (crate-source "tracing-subscriber" "0.3.23"
                "06fkr0qhggvrs861d7f74pn3i3a10h5jsp4n70jj9ys5b675fzyb"))

(define rust-tracing-test-0.2.6
  (crate-source "tracing-test" "0.2.6"
                "0l80kq8x2hm11dbhrkr6qljgn6z75qzzgffxqlj4ykaivd4c990r"))

(define rust-tracing-test-macro-0.2.6
  (crate-source "tracing-test-macro" "0.2.6"
                "179jcllv4gq1kwlp2kzaihqmx28bqislnrinda3cfrgvg9xq81md"))

(define rust-triomphe-0.1.15
  (crate-source "triomphe" "0.1.15"
                "0fazg0zgq2zbjx50vkwg1zxr8nxc9skqj9rpsqcpak4jiymcasfx"))

(define rust-try-lock-0.2.5
  (crate-source "try-lock" "0.2.5"
                "0jqijrrvm1pyq34zn1jmy2vihd4jcrjlvsh4alkjahhssjnsn8g4"))

(define rust-ttf-parser-0.25.1
  (crate-source "ttf-parser" "0.25.1"
                "0cbgqglcwwjg3hirwq6xlza54w04mb5x02kf7zx4hrw50xmr1pyj"))

(define rust-turso-0.6.1
  (crate-source "turso" "0.6.1"
                "1m56xfmi421brwr7nac64ig8sck2pmm5qp188pnx7d6l6q2bfaw3"))

(define rust-turso-core-0.6.1
  (crate-source "turso_core" "0.6.1"
                "0jx86xrn3dfjnmqik2ain2p4wr4df8dxq84vhahj95rn619zk3ms"))

(define rust-turso-ext-0.6.1
  (crate-source "turso_ext" "0.6.1"
                "1000367qg5qpch1cbv8m6r9q1khd2mg2s2fz79zaxfb6swkpk97r"))

(define rust-turso-macros-0.6.1
  (crate-source "turso_macros" "0.6.1"
                "11rxgk2k1yc25qjl51b5l94din8r1r6mlncvm0hslqcw0ypcdkfh"))

(define rust-turso-parser-0.6.1
  (crate-source "turso_parser" "0.6.1"
                "0xqqnzb0wdds6xmv68r2n20j3dynq9nbv8fmvg6i2m30wl3lgfal"))

(define rust-turso-sdk-kit-macros-0.6.1
  (crate-source "turso_sdk_kit_macros" "0.6.1"
                "06l47p5whiqlhclnlvxrbqcvayxa867jrp9xlsklhnnwlc1yx9pw"))

(define rust-turso-sync-engine-0.6.1
  (crate-source "turso_sync_engine" "0.6.1"
                "1f64a25w44ggxld8yshgjm0b63x1xjn973i7pn8lysbvy5hgwn0w"))

(define rust-turso-sync-sdk-kit-0.6.1
  (crate-source "turso_sync_sdk_kit" "0.6.1"
                "14sba3ngr7sq4f1k9jc6cvmvpnaf4dhfx0543fz0kg5kifrcf4j5"))

(define rust-twox-hash-2.1.2
  (crate-source "twox-hash" "2.1.2"
                "1721278f1yc5zvkpdb8gsb1x6nlfjdmwm5fk9ff3fismcxmi78wy"))

(define rust-type-map-0.5.1
  (crate-source "type-map" "0.5.1"
                "143v32wwgpymxfy4y8s694vyq0wdi7li4s5dmms5w59nj2yxnc6b"))

(define rust-typed-arena-2.0.2
  (crate-source "typed-arena" "2.0.2"
                "0shj0jpmglhgw2f1i4b33ycdzwd1z205pbs1rd5wx7ks2qhaxxka"))

(define rust-typeid-1.0.3
  (crate-source "typeid" "1.0.3"
                "0727ypay2p6mlw72gz3yxkqayzdmjckw46sxqpaj08v0b0r64zdw"))

(define rust-typenum-1.20.1
  (crate-source "typenum" "1.20.1"
                "086s9ly0906kw5yw41249fba97w5zfxf03pyfwdkffvcprqfixdn"))

(define rust-typetag-0.2.22
  (crate-source "typetag" "0.2.22"
                "1319xnp85v2yn1yh4fxwjcgk1p2j0asqv2qk1fnm24bc5jqrga65"))

(define rust-typetag-impl-0.2.22
  (crate-source "typetag-impl" "0.2.22"
                "1if14v3k0lwqcd1115a84w0xbci1iy6yqxrj1yx16zpdqrbq706g"))

(define rust-ua-parser-0.2.2
  (crate-source "ua-parser" "0.2.2"
                "18ddw8m1vng5660mcj78md1p214jsxbwkihn69j4p1zq76ipn6zh"))

(define rust-ubyte-0.10.4
  (crate-source "ubyte" "0.10.4"
                "1spj3k9sx6xvfn7am9vm1b463hsr79nyvj8asi2grqhyrvvdw87p"))

(define rust-ucd-trie-0.1.7
  (crate-source "ucd-trie" "0.1.7"
                "0wc9p07sqwz320848i52nvyjvpsxkx3kv5bfbmm6s35809fdk5i8"))

(define rust-uds-windows-1.2.1
  (crate-source "uds_windows" "1.2.1"
                "0vidqwwfgn8wyzvbxiqil787b4wyqjia50zpdbbjqx7n8wlgpxpj"))

(define rust-uncased-0.9.10
  (crate-source "uncased" "0.9.10"
                "15q6r6g4fszr8c2lzg9z9k9g52h8g29h24awda3d72cyw37qzf71"))

(define rust-unic-langid-0.9.6
  (crate-source "unic-langid" "0.9.6"
                "01bx59sqsx2jz4z7ppxq9kldcjq9dzadkmb2dr7iyc85kcnab2x2"))

(define rust-unic-langid-impl-0.9.6
  (crate-source "unic-langid-impl" "0.9.6"
                "0n66kdan4cz99n8ra18i27f7w136hmppi4wc0aa7ljsd0h4bzqfw"))

(define rust-unic-langid-macros-0.9.6
  (crate-source "unic-langid-macros" "0.9.6"
                "09gwlpdzxnzhywvarfm43d7g1672lwak6ahq2kfplv9l5sw7x5fm"))

(define rust-unic-langid-macros-impl-0.9.6
  (crate-source "unic-langid-macros-impl" "0.9.6"
                "1dbmgybjxn4b3a7mb21grc5r98xwal9h1cgc46w39bg3imi9l951"))

(define rust-unicase-2.8.1
  (crate-source "unicase" "2.8.1"
                "0fd5ddbhpva7wrln2iah054ar2pc1drqjcll0f493vj3fv8l9f3m"))

(define rust-unicode-bom-2.0.3
  (crate-source "unicode-bom" "2.0.3"
                "05s2sqyjanqrbds3fxam35f92npp5ci2wz9zg7v690r0448mvv3y"))

(define rust-unicode-id-0.3.6
  (crate-source "unicode-id" "0.3.6"
                "1015prrd0dmy1p6zxymi7zjnnc5y6y6p2xp4rd1w09wrf272ifkh"))

(define rust-unicode-id-start-1.4.0
  (crate-source "unicode-id-start" "1.4.0"
                "01v0ig6a5dy75r9wwhnjfw1fzcj3nhcqj3q2c11dw6aykg99mdw1"))

(define rust-unicode-ident-1.0.24
  (crate-source "unicode-ident" "1.0.24"
                "0xfs8y1g7syl2iykji8zk5hgfi5jw819f5zsrbaxmlzwsly33r76"))

(define rust-unicode-normalization-0.1.25
  (crate-source "unicode-normalization" "0.1.25"
                "1s76dcrxw7vs32yhpi0p074apdc3s7lak7809f3qvclwij3zdm2z"))

(define rust-unicode-segmentation-1.13.3
  (crate-source "unicode-segmentation" "1.13.3"
                "1a47zaq83p386r3baq4m018xd5q4q0grdg56i1x042dzn71x7xf6"))

(define rust-unicode-width-0.1.14
  (crate-source "unicode-width" "0.1.14"
                "1bzn2zv0gp8xxbxbhifw778a7fc93pa6a1kj24jgg9msj07f7mkx"))

(define rust-unicode-width-0.2.2
  (crate-source "unicode-width" "0.2.2"
                "0m7jjzlcccw716dy9423xxh0clys8pfpllc5smvfxrzdf66h9b5l"))

(define rust-unicode-xid-0.2.6
  (crate-source "unicode-xid" "0.2.6"
                "0lzqaky89fq0bcrh6jj6bhlz37scfd8c7dsj5dq7y32if56c1hgb"))

(define rust-universal-hash-0.5.1
  (crate-source "universal-hash" "0.5.1"
                "1sh79x677zkncasa95wz05b36134822w6qxmi1ck05fwi33f47gw"))

(define rust-universal-hash-0.6.1
  (crate-source "universal-hash" "0.6.1"
                "15la0jq3jpzvabwx3kdrk34spylgfd85r9n4pvh84cvm2bf7p67l"))

(define rust-unsafe-libyaml-0.2.11
  (crate-source "unsafe-libyaml" "0.2.11"
                "0qdq69ffl3v5pzx9kzxbghzn0fzn266i1xn70y88maybz9csqfk7"))

(define rust-untrusted-0.9.0
  (crate-source "untrusted" "0.9.0"
                "1ha7ib98vkc538x0z60gfn0fc5whqdd85mb87dvisdcaifi6vjwf"))

(define rust-url-2.5.8
  (crate-source "url" "2.5.8"
                "1v8f7nx3hpr1qh76if0a04sj08k86amsq4h8cvpw6wvk76jahrzz"))

(define rust-urlencoding-2.1.3
  (crate-source "urlencoding" "2.1.3"
                "1nj99jp37k47n0hvaz5fvz7z6jd0sb4ppvfy3nphr1zbnyixpy6s"))

(define rust-utf-8-0.7.6
  (crate-source "utf-8" "0.7.6"
                "1a9ns3fvgird0snjkd3wbdhwd3zdpc2h5gpyybrfr6ra5pkqxk09"))

(define rust-utf16-iter-1.0.5
  (crate-source "utf16_iter" "1.0.5"
                "0ik2krdr73hfgsdzw0218fn35fa09dg2hvbi1xp3bmdfrp9js8y8"))

(define rust-utf8-iter-1.0.4
  (crate-source "utf8_iter" "1.0.4"
                "1gmna9flnj8dbyd8ba17zigrp9c4c3zclngf5lnb5yvz1ri41hdn"))

(define rust-utf8-ranges-1.0.5
  (crate-source "utf8-ranges" "1.0.5"
                "1fk46654sqis2dqamihlj9b1sv162kp3brgmmqpa0lqfz4kwikvz"))

(define rust-utf8-width-0.1.8
  (crate-source "utf8-width" "0.1.8"
                "14d08vrz878wqpmqw46yl5l1vwmdf00zx4i49z8iahdmf3cw14hj"))

(define rust-utf8parse-0.2.2
  (crate-source "utf8parse" "0.2.2"
                "088807qwjq46azicqwbhlmzwrbkz7l4hpw43sdkdyyk524vdxaq6"))

(define rust-uuid-1.23.2
  (crate-source "uuid" "1.23.2"
                "1xy942s4z0bi8p3441wvd4ry3hx6ry1c7s6fgrr38462xqybhn6j"))

(define rust-v-frame-0.3.9
  (crate-source "v_frame" "0.3.9"
                "1qkvb4ks33zck931vzqckjn36hkngj6l2cwmvfsnlpc7r0kpfsv6"))

(define rust-valuable-0.1.1
  (crate-source "valuable" "0.1.1"
                "0r9srp55v7g27s5bg7a2m095fzckrcdca5maih6dy9bay6fflwxs"))

(define rust-vergen-9.1.0
  (crate-source "vergen" "9.1.0"
                "0xdgrs146p81vbhg5y8svch3cghyi3yf07p8c7i8v7k3v3va2jdq"))

(define rust-vergen-gix-9.1.0
  (crate-source "vergen-gix" "9.1.0"
                "1w24nrcfzc11cvsab7gmcskflbc5mpxfs1qrykwcd13bpq93jhr4"))

(define rust-vergen-lib-9.1.0
  (crate-source "vergen-lib" "9.1.0"
                "0sd5b5d5ygwi86k1b4n9vipqmyxqn4pr7qcs48pycncwgsx2jjmk"))

(define rust-version-check-0.9.5
  (crate-source "version_check" "0.9.5"
                "0nhhi4i5x89gm911azqbn7avs9mdacw2i3vcz3cnmz3mv4rqz4hb"))

(define rust-voprf-0.5.0
  (crate-source "voprf" "0.5.0"
                "0w4lwdg2c93mi2kmi5migbxfybv6w9aa1rpcrmaflbvfqwq9rx98"))

(define rust-vsimd-0.8.0
  (crate-source "vsimd" "0.8.0"
                "0r4wn54jxb12r0x023r5yxcrqk785akmbddqkcafz9fm03584c2w"))

(define rust-walkdir-2.5.0
  (crate-source "walkdir" "2.5.0"
                "0jsy7a710qv8gld5957ybrnc07gavppp963gs32xk4ag8130jy99"))

(define rust-want-0.3.1
  (crate-source "want" "0.3.1"
                "03hbfrnvqqdchb5kgxyavb9jabwza0dmh2vw5kg0dq8rxl57d9xz"))

(define rust-wasi-0.11.1+wasi-snapshot-preview1
  (crate-source "wasi" "0.11.1+wasi-snapshot-preview1"
                "0jx49r7nbkbhyfrfyhz0bm4817yrnxgd3jiwwwfv0zl439jyrwyc"))

(define rust-wasip2-1.0.3+wasi-0.2.9
  (crate-source "wasip2" "1.0.3+wasi-0.2.9"
                "1mi3w855dz99xzjqc4aa8c9q5b6z1y5c963pkk4cvmr6vdr4c1i0"))

(define rust-wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06
  (crate-source "wasip3" "0.4.0+wasi-0.3.0-rc-2026-01-06"
                "19dc8p0y2mfrvgk3qw3c3240nfbylv22mvyxz84dqpgai2zzha2l"))

(define rust-wasm-bindgen-0.2.122
  (crate-source "wasm-bindgen" "0.2.122"
                "02flix96brsb2r1i3grnikii302iqpdm337kl3xv5lklz5v4bl1y"))

(define rust-wasm-bindgen-futures-0.4.72
  (crate-source "wasm-bindgen-futures" "0.4.72"
                "03qb24gfr072rk8hb69glfdc8yhqqqq2rhy3j5i0ps8sk79dnwwl"))

(define rust-wasm-bindgen-macro-0.2.122
  (crate-source "wasm-bindgen-macro" "0.2.122"
                "1inyl55bvdifx7l60q9wl0ivmw7236jg7jqmcqpxhsx3knq52qci"))

(define rust-wasm-bindgen-macro-support-0.2.122
  (crate-source "wasm-bindgen-macro-support" "0.2.122"
                "0pjw5kc2mbfz59agk5l21kh4hxzp94rygdvsnr4f3z6b5hv4g419"))

(define rust-wasm-bindgen-shared-0.2.122
  (crate-source "wasm-bindgen-shared" "0.2.122"
                "0ds4mmfqvxwc5fp33hn0jblf0f6b4lghrd9mpkls66zic4n9p4ls"))

(define rust-wasm-encoder-0.244.0
  (crate-source "wasm-encoder" "0.244.0"
                "06c35kv4h42vk3k51xjz1x6hn3mqwfswycmr6ziky033zvr6a04r"))

(define rust-wasm-metadata-0.244.0
  (crate-source "wasm-metadata" "0.244.0"
                "02f9dhlnryd2l7zf03whlxai5sv26x4spfibjdvc3g9gd8z3a3mv"))

(define rust-wasm-streams-0.5.0
  (crate-source "wasm-streams" "0.5.0"
                "1fqbcx33w8ys5i5dv3p28a82g4yiclmhn80fcfp137kwa7vc87lx"))

(define rust-wasmparser-0.244.0
  (crate-source "wasmparser" "0.244.0"
                "1zi821hrlsxfhn39nqpmgzc0wk7ax3dv6vrs5cw6kb0v5v3hgf27"))

(define rust-wayland-backend-0.3.15
  (crate-source "wayland-backend" "0.3.15"
                "0pbm8j3vv6baqz312biwqfi4qzadbi6nng9v4p3nx4afnlhdsmr8"))

(define rust-wayland-client-0.31.14
  (crate-source "wayland-client" "0.31.14"
                "0i014rcfjgccknnlyfk94fxn4w32l56cpjdmi4qhqsblpfb7qp34"))

(define rust-wayland-csd-frame-0.3.0
  (crate-source "wayland-csd-frame" "0.3.0"
                "0zjcmcqprfzx57hlm741n89ssp4sha5yh5cnmbk2agflvclm0p32"))

(define rust-wayland-cursor-0.31.14
  (crate-source "wayland-cursor" "0.31.14"
                "0kdk7xwj465idk54jf1f24024gdp63wyagca68a176xyh23x2lja"))

(define rust-wayland-protocols-0.32.12
  (crate-source "wayland-protocols" "0.32.12"
                "13rdk2akpdg90v42sjlz7c86541isxgq347772cl5qmd7i98afjn"))

(define rust-wayland-protocols-experimental-20250721.0.1
  (crate-source "wayland-protocols-experimental" "20250721.0.1"
                "1cfbimd2qbbcgv21i3l7kq3pm6lvrjbb7d6pj33sxjld29izi8a0"))

(define rust-wayland-protocols-misc-0.3.12
  (crate-source "wayland-protocols-misc" "0.3.12"
                "1j19dg8h98s153rj2fvbqkghjicdfgjjkr6nvaw0jgpjkrcng5bf"))

(define rust-wayland-protocols-plasma-0.3.12
  (crate-source "wayland-protocols-plasma" "0.3.12"
                "14adi3xgkldbih60705gshlq2lskds5chhsn3znk271cxgqqqv9b"))

(define rust-wayland-protocols-wlr-0.3.12
  (crate-source "wayland-protocols-wlr" "0.3.12"
                "0d424vn2hj27r4gjlshm6hy8fcqysr805jkqdjbwgmrng0pya17b"))

(define rust-wayland-sys-0.31.11
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "wayland-sys" "0.31.11"
                "1gp3hlkxx13i55lyyi794vnw9a780z3skx0xhj71zr69xwzv5snq"))

(define rust-web-atoms-0.2.4
  (crate-source "web_atoms" "0.2.4"
                "0f65zxzg1g8xra01kg7im614s11nyhpkl3i5zls1ipqmz3pgdkyp"))

(define rust-web-sys-0.3.99
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "web-sys" "0.3.99"
                "0dilfvl9jnyhi4skl6cry9wc300r693j0w82jjbq8yy3rx0i8qkd"))

(define rust-web-time-1.1.0
  (crate-source "web-time" "1.1.0"
                "1fx05yqx83dhx628wb70fyy10yjfq1jpl20qfqhdkymi13rq0ras"))

(define rust-webbrowser-1.2.1
  (crate-source "webbrowser" "1.2.1"
                "0wlz31z5zgwvjgg95w0wyzmp7ny5dx20ggm7ys7ydwbaj605bj8g"))

(define rust-webpki-root-certs-1.0.7
  (crate-source "webpki-root-certs" "1.0.7"
                "0b59x5mzsilk42w59nif3lfhc24pgzb0v35pi6p01qy37z7424gk"))

(define rust-weezl-0.1.12
  (crate-source "weezl" "0.1.12"
                "122a1dhha6cib5az4ihcqlh60ns2bi6rskdv875p94lbvj6wk2m2"))

(define rust-which-4.4.2
  (crate-source "which" "4.4.2"
                "1ixzmx3svsv5hbdvd8vdhd3qwvf6ns8jdpif1wmwsy10k90j9fl7"))

(define rust-wide-1.5.0
  (crate-source "wide" "1.5.0"
                "12dz1l33d44jrhkjrrwi04psqv59m12mi2dqd2rd3wkk56iydpyz"))

(define rust-widestring-1.2.1
  (crate-source "widestring" "1.2.1"
                "0wg4qdbs70xqnlbm8wb0bs4idm2mxk3b6kaqwllsncmb2cqrq1kj"))

(define rust-winapi-0.3.9
  (crate-source "winapi" "0.3.9"
                "06gl025x418lchw1wxj64ycr7gha83m44cjr5sarhynd9xkrm0sw"))

(define rust-winapi-i686-pc-windows-gnu-0.4.0
  (crate-source "winapi-i686-pc-windows-gnu" "0.4.0"
                "1dmpa6mvcvzz16zg6d5vrfy4bxgg541wxrcip7cnshi06v38ffxc"))

(define rust-winapi-util-0.1.11
  (crate-source "winapi-util" "0.1.11"
                "08hdl7mkll7pz8whg869h58c1r9y7in0w0pk8fm24qc77k0b39y2"))

(define rust-winapi-x86-64-pc-windows-gnu-0.4.0
  (crate-source "winapi-x86_64-pc-windows-gnu" "0.4.0"
                "0gqq64czqb64kskjryj8isp62m2sgvx25yyj3kpc2myh85w24bki"))

(define rust-windows-0.61.3
  (crate-source "windows" "0.61.3"
                "14v8dln7i4ccskd8danzri22bkjkbmgzh284j3vaxhd4cykx7awv"))

(define rust-windows-0.62.2
  (crate-source "windows" "0.62.2"
                "10457l9ihrbw8j79z2v4plyjxkf6xvb5npd0lqwmkh702gpaszsj"))

(define rust-windows-aarch64-gnullvm-0.52.6
  (crate-source "windows_aarch64_gnullvm" "0.52.6"
                "1lrcq38cr2arvmz19v32qaggvj8bh1640mdm9c2fr877h0hn591j"))

(define rust-windows-aarch64-msvc-0.52.6
  (crate-source "windows_aarch64_msvc" "0.52.6"
                "0sfl0nysnz32yyfh773hpi49b1q700ah6y7sacmjbqjjn5xjmv09"))

(define rust-windows-collections-0.2.0
  (crate-source "windows-collections" "0.2.0"
                "1s65anr609qvsjga7w971p6iq964h87670dkfqfypnfgwnswxviv"))

(define rust-windows-collections-0.3.2
  (crate-source "windows-collections" "0.3.2"
                "0436rjbkqn3j9m2v2lcmwwk0l3n2r57yvqb7fcy4m8d8y5ddkci3"))

(define rust-windows-core-0.61.2
  (crate-source "windows-core" "0.61.2"
                "1qsa3iw14wk4ngfl7ipcvdf9xyq456ms7cx2i9iwf406p7fx7zf0"))

(define rust-windows-core-0.62.2
  (crate-source "windows-core" "0.62.2"
                "1swxpv1a8qvn3bkxv8cn663238h2jccq35ff3nsj61jdsca3ms5q"))

(define rust-windows-future-0.2.1
  (crate-source "windows-future" "0.2.1"
                "13mdzcdn51ckpzp3frb8glnmkyjr1c30ym9wnzj9zc97hkll2spw"))

(define rust-windows-future-0.3.2
  (crate-source "windows-future" "0.3.2"
                "1jq5qs2dwzf6rl60f8gr49z2mifxsrdh4y4yfdws467ya41gkmp1"))

(define rust-windows-i686-gnu-0.52.6
  (crate-source "windows_i686_gnu" "0.52.6"
                "02zspglbykh1jh9pi7gn8g1f97jh1rrccni9ivmrfbl0mgamm6wf"))

(define rust-windows-i686-gnullvm-0.52.6
  (crate-source "windows_i686_gnullvm" "0.52.6"
                "0rpdx1537mw6slcpqa0rm3qixmsb79nbhqy5fsm3q2q9ik9m5vhf"))

(define rust-windows-i686-msvc-0.52.6
  (crate-source "windows_i686_msvc" "0.52.6"
                "0rkcqmp4zzmfvrrrx01260q3xkpzi6fzi2x2pgdcdry50ny4h294"))

(define rust-windows-implement-0.60.2
  (crate-source "windows-implement" "0.60.2"
                "1psxhmklzcf3wjs4b8qb42qb6znvc142cb5pa74rsyxm1822wgh5"))

(define rust-windows-interface-0.59.3
  (crate-source "windows-interface" "0.59.3"
                "0n73cwrn4247d0axrk7gjp08p34x1723483jxjxjdfkh4m56qc9z"))

(define rust-windows-link-0.1.3
  (crate-source "windows-link" "0.1.3"
                "12kr1p46dbhpijr4zbwr2spfgq8i8c5x55mvvfmyl96m01cx4sjy"))

(define rust-windows-link-0.2.1
  (crate-source "windows-link" "0.2.1"
                "1rag186yfr3xx7piv5rg8b6im2dwcf8zldiflvb22xbzwli5507h"))

(define rust-windows-numerics-0.2.0
  (crate-source "windows-numerics" "0.2.0"
                "1cf2j8nbqf0hqqa7chnyid91wxsl2m131kn0vl3mqk3c0rlayl4i"))

(define rust-windows-numerics-0.3.1
  (crate-source "windows-numerics" "0.3.1"
                "09hgbg8pf89r4090yyhh9q29ppi7yyxkgmga9ascshy19a240bkf"))

(define rust-windows-registry-0.5.3
  (crate-source "windows-registry" "0.5.3"
                "17j9cxlnksdypanazss6cnh36v3rwvs86j4mpixwkvv5hz99x2jv"))

(define rust-windows-result-0.3.4
  (crate-source "windows-result" "0.3.4"
                "1il60l6idrc6hqsij0cal0mgva6n3w6gq4ziban8wv6c6b9jpx2n"))

(define rust-windows-result-0.4.1
  (crate-source "windows-result" "0.4.1"
                "1d9yhmrmmfqh56zlj751s5wfm9a2aa7az9rd7nn5027nxa4zm0bp"))

(define rust-windows-strings-0.4.2
  (crate-source "windows-strings" "0.4.2"
                "0mrv3plibkla4v5kaakc2rfksdd0b14plcmidhbkcfqc78zwkrjn"))

(define rust-windows-strings-0.5.1
  (crate-source "windows-strings" "0.5.1"
                "14bhng9jqv4fyl7lqjz3az7vzh8pw0w4am49fsqgcz67d67x0dvq"))

(define rust-windows-sys-0.52.0
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "windows-sys" "0.52.0"
                "0gd3v4ji88490zgb6b5mq5zgbvwv7zx1ibn8v3x83rwcdbryaar8"))

(define rust-windows-sys-0.59.0
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "windows-sys" "0.59.0"
                "0fw5672ziw8b3zpmnbp9pdv1famk74f1l9fcbc3zsrzdg56vqf0y"))

(define rust-windows-sys-0.61.2
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "windows-sys" "0.61.2"
                "1z7k3y9b6b5h52kid57lvmvm05362zv1v8w0gc7xyv5xphlp44xf"))

(define rust-windows-targets-0.52.6
  (crate-source "windows-targets" "0.52.6"
                "0wwrx625nwlfp7k93r2rra568gad1mwd888h1jwnl0vfg5r4ywlv"))

(define rust-windows-threading-0.1.0
  (crate-source "windows-threading" "0.1.0"
                "19jpn37zpjj2q7pn07dpq0ay300w65qx7wdp13wbp8qf5snn6r5n"))

(define rust-windows-threading-0.2.1
  (crate-source "windows-threading" "0.2.1"
                "0dsvsy33vxs0153z4n39sqkzx382cjjkrd46rb3z3zfak5dvsj9r"))

(define rust-windows-x86-64-gnu-0.52.6
  (crate-source "windows_x86_64_gnu" "0.52.6"
                "0y0sifqcb56a56mvn7xjgs8g43p33mfqkd8wj1yhrgxzma05qyhl"))

(define rust-windows-x86-64-gnullvm-0.52.6
  (crate-source "windows_x86_64_gnullvm" "0.52.6"
                "03gda7zjx1qh8k9nnlgb7m3w3s1xkysg55hkd1wjch8pqhyv5m94"))

(define rust-windows-x86-64-msvc-0.52.6
  (crate-source "windows_x86_64_msvc" "0.52.6"
                "1v7rb5cibyzx8vak29pdrk8nx9hycsjs4w0jgms08qk49jl6v7sq"))

(define rust-winnow-0.5.40
  (crate-source "winnow" "0.5.40"
                "0xk8maai7gyxda673mmw3pj1hdizy5fpi7287vaywykkk19sk4zm"))

(define rust-winnow-0.7.15
  (crate-source "winnow" "0.7.15"
                "0i9rkl2rqpbnnxlgs20gmkj3nd0b2k8q55mjmpc2ybb84xwxjyfz"))

(define rust-winnow-1.0.3
  (crate-source "winnow" "1.0.3"
                "1wajycd3krn6h699vydjv7hm0ll5l31p899qzpk59y2is74y34h5"))

(define rust-winreg-0.55.0
  (crate-source "winreg" "0.55.0"
                "15xy060vylrsp91bc0ximx3xziwipzlrn1n2ab19w3n56x9pcnnb"))

(define rust-winresource-0.1.31
  (crate-source "winresource" "0.1.31"
                "11v0hr6kfyi8kl8am96fkn325bjinjgs77ixzvjd7dw6snqsi1h9"))

(define rust-winstructs-0.3.2
  (crate-source "winstructs" "0.3.2"
                "1s9dsiflxb6pwcw21mrr4ihmkg4mnb2mvzd4qff3s5rnv5n41ivd"))

(define rust-wit-bindgen-0.51.0
  (crate-source "wit-bindgen" "0.51.0"
                "19fazgch8sq5cvjv3ynhhfh5d5x08jq2pkw8jfb05vbcyqcr496p"))

(define rust-wit-bindgen-0.57.1
  (crate-source "wit-bindgen" "0.57.1"
                "0vjk2jb593ri9k1aq4iqs2si9mrw5q46wxnn78im7hm7hx799gqy"))

(define rust-wit-bindgen-core-0.51.0
  (crate-source "wit-bindgen-core" "0.51.0"
                "1p2jszqsqbx8k7y8nwvxg65wqzxjm048ba5phaq8r9iy9ildwqga"))

(define rust-wit-bindgen-rust-0.51.0
  (crate-source "wit-bindgen-rust" "0.51.0"
                "08bzn5fsvkb9x9wyvyx98qglknj2075xk1n7c5jxv15jykh6didp"))

(define rust-wit-bindgen-rust-macro-0.51.0
  (crate-source "wit-bindgen-rust-macro" "0.51.0"
                "0ymizapzv2id89igxsz2n587y2hlfypf6n8kyp68x976fzyrn3qc"))

(define rust-wit-component-0.244.0
  (crate-source "wit-component" "0.244.0"
                "1clwxgsgdns3zj2fqnrjcp8y5gazwfa1k0sy5cbk0fsmx4hflrlx"))

(define rust-wit-parser-0.244.0
  (crate-source "wit-parser" "0.244.0"
                "0dm7avvdxryxd5b02l0g5h6933z1cw5z0d4wynvq2cywq55srj7c"))

(define rust-workspace-filter-0.1.0
  (crate-source "workspace-filter" "0.1.0"
                "1fy5hgzl0vq44qlxwyisnhfhw62lx48fgasdjnx7ffzyf974aqvw"))

(define rust-workspace-filter-build-0.1.0
  (crate-source "workspace-filter-build" "0.1.0"
                "03nyqabzi9dv8s380pkx4ml69p1jxgwfk75aq4y1k08rmzf9c1gq"))

(define rust-write16-1.0.0
  (crate-source "write16" "1.0.0"
                "0dnryvrrbrnl7vvf5vb1zkmwldhjkf2n5znliviam7bm4900z2fi"))

(define rust-writeable-0.6.3
  (crate-source "writeable" "0.6.3"
                "1i54d13h9bpap2hf13xcry1s4lxh7ap3923g8f3c0grd7c9fbyhz"))

(define rust-wyz-0.5.1
  (crate-source "wyz" "0.5.1"
                "1vdrfy7i2bznnzjdl9vvrzljvs4s3qm8bnlgqwln6a941gy61wq5"))

(define rust-x11-dl-2.21.0
  (crate-source "x11-dl" "2.21.0"
                "0vsiq62xpcfm0kn9zjw5c9iycvccxl22jya8wnk18lyxzqj5jwrq"))

(define rust-x11rb-protocol-0.13.2
  (crate-source "x11rb-protocol" "0.13.2"
                "1g81cznbyn522b0fbis0i44wh3adad2vhsz5pzf99waf3sbc4vza"))

(define rust-x509-parser-0.16.0
  (crate-source "x509-parser" "0.16.0"
                "0s8zyl6fafkzpylcpcn08bmcmrzzcb6gfjx2h8zny3bh60pidg7w"))

(define rust-xattr-1.6.1
  (crate-source "xattr" "1.6.1"
                "0ml1mb43gqasawillql6b344m0zgq8mz0isi11wj8vbg43a5mr1j"))

(define rust-xcursor-0.3.10
  (crate-source "xcursor" "0.3.10"
                "0awgy98awg4ydcfmynqfcwvl4bnnfcm4i2vvnk2n926a02jy9jdy"))

(define rust-xi-unicode-0.3.0
  (crate-source "xi-unicode" "0.3.0"
                "12mvjgrhr7557cib69wm4q5s4srba27pg2df9l1zihrxgnbh0wx6"))

(define rust-xkbcommon-dl-0.4.2
  (crate-source "xkbcommon-dl" "0.4.2"
                "1iai0r3b5skd9vbr8z5b0qixiz8jblzfm778ddm8ba596a0dwffh"))

(define rust-xkbcommon-rs-codegen-0.1.1
  (crate-source "xkbcommon-rs-codegen" "0.1.1"
                "1j1z9sn3pxc8chjahb49nm752qcfxcp9nalg8vdvmh9b64268d1x"))

(define rust-xkeysym-0.2.1
  (crate-source "xkeysym" "0.2.1"
                "0mksx670cszyd7jln6s7dhkw11hdfv7blwwr3isq98k22ljh1k5r"))

(define rust-xml-1.3.0
  (crate-source "xml" "1.3.0"
                "128s58qhq8whrx90zbw8r5algr7lakgbf7mn05jfk234rbjqavv3"))

(define rust-xml-rs-1.0.0
  (crate-source "xml-rs" "1.0.0"
                "0lbbdghy162ag2mlrzzgxz7c93vq5wii1p1f6mvvxv6nl0r639f3"))

(define rust-xmltree-0.12.0
  (crate-source "xmltree" "0.12.0"
                "0w7zwk9680c6qpnx17jv83fbc1q8f8wyf90pmfcf895ir89l7h6b"))

(define rust-xsum-0.1.6
  (crate-source "xsum" "0.1.6"
                "1nrjqvcvh6v9xdzw18z8v16bk7wcpj3r1s5s2i9gm0kaasjx6dq6"))

(define rust-xxhash-rust-0.8.15
  (crate-source "xxhash-rust" "0.8.15"
                "1lrmffpn45d967afw7f1p300rsx7ill66irrskxpcm1p41a0rlpx"))

(define rust-y4m-0.8.0
  (crate-source "y4m" "0.8.0"
                "0j24y2zf60lpxwd7kyg737hqfyqx16y32s0fjyi6fax6w4hlnnks"))

(define rust-yansi-1.0.1
  (crate-source "yansi" "1.0.1"
                "0jdh55jyv0dpd38ij4qh60zglbw9aa8wafqai6m0wa7xaxk3mrfg"))

(define rust-yoke-0.8.3
  (crate-source "yoke" "0.8.3"
                "1xgyj6c2lxj2bp891ynmhws87c6z7yyv2li1v0ss9di40hxf57vh"))

(define rust-yoke-derive-0.8.2
  (crate-source "yoke-derive" "0.8.2"
                "13l5y5sz4lqm7rmyakjbh6vwgikxiql51xfff9hq2j485hk4r16y"))

(define rust-zbus-5.16.0
  (crate-source "zbus" "5.16.0"
                "11a28qwmwgn9k1pzdp9lik0gb2mjlx4dyarc7fgllzm70b985rpf"))

(define rust-zbus-lockstep-0.5.2
  (crate-source "zbus-lockstep" "0.5.2"
                "0qsqsk67c2vpg26rp0x0ya0cv92fs11r92kjg1sln23s442xx639"))

(define rust-zbus-lockstep-macros-0.5.2
  (crate-source "zbus-lockstep-macros" "0.5.2"
                "1853gk2fymvr2yaird9jpvz4mdp6ms8zmy6dr19payrsgwv0bnhh"))

(define rust-zbus-macros-5.16.0
  (crate-source "zbus_macros" "5.16.0"
                "1mnih15n4cf4irn4sbjkwh0wvs1659v58xvmn52kf40sm12vvwdd"))

(define rust-zbus-names-4.3.2
  (crate-source "zbus_names" "4.3.2"
                "0bg5c1bq4xdq9xqdkgvxwvl9pa6h61nh2hh1fn8sqkl91gjz6x3h"))

(define rust-zerocopy-0.8.50
  (crate-source "zerocopy" "0.8.50"
                "1laahnfxs4qyfb1fdf5nbb2qfshi72b1hbi0ffp2zy2m1r7ms1iv"))

(define rust-zerocopy-derive-0.8.50
  (crate-source "zerocopy-derive" "0.8.50"
                "0fdnr9qslx1hbn2i9rsvy9s95mychfy2vj90ajsjm2basccinqqb"))

(define rust-zerofrom-0.1.8
  (crate-source "zerofrom" "0.1.8"
                "0wjjdj7gdmd0iq91gzkxl7dlv0nhkk80l4bmdpzh3a1yh48mmh0f"))

(define rust-zerofrom-derive-0.1.7
  (crate-source "zerofrom-derive" "0.1.7"
                "18c4wsnznhdxx6m80piil1lbyszdiwsshgjrybqcm4b6qic22lqi"))

(define rust-zeroize-1.8.2
  (crate-source "zeroize" "1.8.2"
                "1l48zxgcv34d7kjskr610zqsm6j2b4fcr2vfh9jm9j1jgvk58wdr"))

(define rust-zeroize-derive-1.4.3
  (crate-source "zeroize_derive" "1.4.3"
                "0bl5vd1lz27p4z336nximg5wrlw5j7jc8fxh7iv6r1wrhhav99c5"))

(define rust-zerotrie-0.2.4
  (crate-source "zerotrie" "0.2.4"
                "1gr0pkcn3qsr6in6iixqyp0vbzwf2j1jzyvh7yl2yydh3p9m548g"))

(define rust-zerovec-0.11.6
  (crate-source "zerovec" "0.11.6"
                "0fdjsy6b31q9i0d73sl7xjd12xadbwi45lkpfgqnmasrqg5i3ych"))

(define rust-zerovec-derive-0.11.3
  (crate-source "zerovec-derive" "0.11.3"
                "0m85qj92mmfvhjra6ziqky5b1p4kcmp5069k7kfadp5hr8jw8pb2"))

(define rust-zlib-rs-0.5.5
  (crate-source "zlib-rs" "0.5.5"
                "1lxa1hf3bs8ip15jq8i8h9wdaaahcgxlzxvsj2vc5qmaa7fhx6a0"))

(define rust-zmij-1.0.21
  (crate-source "zmij" "1.0.21"
                "1amb5i6gz7yjb0dnmz5y669674pqmwbj44p4yfxfv2ncgvk8x15q"))

(define rust-zstd-0.13.3
  (crate-source "zstd" "0.13.3"
                "12n0h4w9l526li7jl972rxpyf012jw3nwmji2qbjghv9ll8y67p9"))

(define rust-zstd-safe-7.2.4
  (crate-source "zstd-safe" "7.2.4"
                "179vxmkzhpz6cq6mfzvgwc99bpgllkr6lwxq7ylh5dmby3aw8jcg"))

(define rust-zstd-sys-2.0.16+zstd.1.5.7
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "zstd-sys" "2.0.16+zstd.1.5.7"
                "0j1pd2iaqpvaxlgqmmijj68wma7xwdv9grrr63j873yw5ay9xqci"))

(define rust-zune-core-0.5.1
  (crate-source "zune-core" "0.5.1"
                "1ya0zdqxlr5v57791j7bvm408ri2cfx81a4v6z85f560yw3hi2nb"))

(define rust-zune-inflate-0.2.54
  (crate-source "zune-inflate" "0.2.54"
                "00kg24jh3zqa3i6rg6yksnb71bch9yi1casqydl00s7nw8pk7avk"))

(define rust-zune-jpeg-0.5.15
  (crate-source "zune-jpeg" "0.5.15"
                "15kjpn6pywxlwb8w5irfd68x31wi3mb4y1da8bqh7havh5drvg17"))

(define rust-zvariant-5.12.0
  (crate-source "zvariant" "5.12.0"
                "1q3vwaiv2h65wi335ig24ylp0q6fnka37j13fmxdfq1kwsys14m1"))

(define rust-zvariant-derive-5.12.0
  (crate-source "zvariant_derive" "5.12.0"
                "0dw7cgacz7cr5s0jwrpqwf4xml0rdk5pqzz99c3i3i81kkg6rg4h"))

(define rust-zvariant-utils-3.4.0
  (crate-source "zvariant_utils" "3.4.0"
                "1mn4fa0rvibzxzj32s07xspa82cr2gl6i34xamz58xgsbj8kb18y"))

(define ssss-separator 'end-of-crates)


;;;
;;; Cargo inputs.
;;;

(define-cargo-inputs lookup-cargo-inputs
                     (ctb-workspace =>
                                    (list rust-ab-glyph-0.2.32
                                     rust-ab-glyph-rasterizer-0.1.10
                                     rust-accesskit-0.21.1
                                     rust-accesskit-atspi-common-0.14.2
                                     rust-accesskit-consumer-0.30.1
                                     rust-accesskit-consumer-0.31.0
                                     rust-accesskit-macos-0.22.2
                                     rust-accesskit-unix-0.17.2
                                     rust-accesskit-windows-0.29.2
                                     rust-accesskit-winit-0.29.2
                                     rust-addr2line-0.25.1
                                     rust-adler2-2.0.1
                                     rust-aead-0.5.2
                                     rust-aead-0.6.1
                                     rust-aegis-0.9.12
                                     rust-aes-0.8.4
                                     rust-aes-0.9.1
                                     rust-aes-gcm-0.10.3
                                     rust-aes-gcm-0.11.0
                                     rust-ahash-0.7.8
                                     rust-ahash-0.8.12
                                     rust-aho-corasick-1.1.4
                                     rust-aligned-0.4.3
                                     rust-aligned-vec-0.6.4
                                     rust-alloc-no-stdlib-2.0.4
                                     rust-alloc-stdlib-0.2.2
                                     rust-allocator-api2-0.2.21
                                     rust-ammonia-4.1.3
                                     rust-android-activity-0.6.1
                                     rust-android-properties-0.2.2
                                     rust-android-system-properties-0.1.5
                                     rust-anstream-0.6.21
                                     rust-anstream-1.0.0
                                     rust-anstyle-1.0.14
                                     rust-anstyle-parse-0.2.7
                                     rust-anstyle-parse-1.0.0
                                     rust-anstyle-query-1.1.5
                                     rust-anstyle-wincon-3.0.11
                                     rust-antithesis-sdk-0.2.9
                                     rust-anyhow-1.0.103
                                     rust-ar-archive-writer-0.5.2
                                     rust-arbitrary-1.4.2
                                     rust-arboard-3.6.1
                                     rust-arc-swap-1.9.1
                                     rust-arg-enum-proc-macro-0.3.4
                                     rust-argh-0.1.19
                                     rust-argh-derive-0.1.19
                                     rust-argh-shared-0.1.19
                                     rust-argon2-0.5.3
                                     rust-array-init-2.1.0
                                     rust-arrayref-0.3.9
                                     rust-arrayvec-0.7.6
                                     rust-as-raw-xcb-connection-1.0.1
                                     rust-as-slice-0.2.1
                                     rust-ascii-1.1.0
                                     rust-ascii-canvas-3.0.0
                                     rust-asn1-rs-0.6.2
                                     rust-asn1-rs-derive-0.5.1
                                     rust-asn1-rs-impl-0.2.0
                                     rust-assoc-0.1.3
                                     rust-ast-node-5.0.0
                                     rust-async-broadcast-0.7.2
                                     rust-async-channel-2.5.0
                                     rust-async-compression-0.4.42
                                     rust-async-executor-1.14.0
                                     rust-async-io-2.6.0
                                     rust-async-lock-3.4.2
                                     rust-async-process-2.5.0
                                     rust-async-recursion-1.1.1
                                     rust-async-signal-0.2.14
                                     rust-async-task-4.7.1
                                     rust-async-trait-0.1.89
                                     rust-atomic-polyfill-1.0.3
                                     rust-atomic-waker-1.1.2
                                     rust-atspi-0.25.0
                                     rust-atspi-common-0.9.0
                                     rust-atspi-connection-0.9.0
                                     rust-atspi-proxies-0.9.0
                                     rust-autocfg-1.5.1
                                     rust-av-scenechange-0.14.1
                                     rust-av1-grain-0.2.5
                                     rust-avif-serialize-0.8.9
                                     rust-aws-lc-rs-1.17.0
                                     rust-aws-lc-sys-0.41.0
                                     rust-axum-0.8.9
                                     rust-axum-core-0.5.6
                                     rust-axum-extra-0.12.6
                                     rust-axum-macros-0.5.1
                                     rust-axum-server-0.8.0
                                     rust-axum-typed-multipart-0.16.6
                                     rust-axum-typed-multipart-macros-0.16.6
                                     rust-backtrace-0.3.76
                                     rust-base16ct-0.2.0
                                     rust-base64-0.21.7
                                     rust-base64-0.22.1
                                     rust-base64-simd-0.8.0
                                     rust-base64ct-1.8.3
                                     rust-bcrypt-0.15.1
                                     rust-beef-0.5.2
                                     rust-better-scoped-tls-1.0.1
                                     rust-bigdecimal-0.4.10
                                     rust-bincode-1.3.3
                                     rust-bindgen-0.69.5
                                     rust-binrw-0.15.1
                                     rust-binrw-derive-0.15.1
                                     rust-bit-set-0.5.3
                                     rust-bit-set-0.8.0
                                     rust-bit-vec-0.6.3
                                     rust-bit-vec-0.8.0
                                     rust-bit-field-0.10.3
                                     rust-bitflags-1.3.2
                                     rust-bitflags-2.13.0
                                     rust-bitpacking-0.9.3
                                     rust-bitstream-io-4.10.0
                                     rust-bitvec-1.0.1
                                     rust-blake2-0.10.6
                                     rust-blake3-1.8.5
                                     rust-block-buffer-0.10.4
                                     rust-block-buffer-0.12.1
                                     rust-block2-0.5.1
                                     rust-block2-0.6.2
                                     rust-blocking-1.6.2
                                     rust-blowfish-0.9.1
                                     rust-boa-ast-1.0.0-dev.ffec924
                                     rust-boa-engine-1.0.0-dev.ffec924
                                     rust-boa-gc-1.0.0-dev.ffec924
                                     rust-boa-interner-1.0.0-dev.ffec924
                                     rust-boa-macros-1.0.0-dev.ffec924
                                     rust-boa-parser-1.0.0-dev.ffec924
                                     rust-boa-runtime-1.0.0-dev.ffec924
                                     rust-boa-string-1.0.0-dev.ffec924
                                     rust-bon-3.9.3
                                     rust-bon-macros-3.9.3
                                     rust-borsh-1.6.1
                                     rust-borsh-derive-1.6.1
                                     rust-branches-0.4.4
                                     rust-brotli-8.0.3
                                     rust-brotli-decompressor-5.0.1
                                     rust-bstr-1.12.1
                                     rust-built-0.8.1
                                     rust-bumpalo-3.20.3
                                     rust-bytecheck-0.6.12
                                     rust-bytecheck-derive-0.6.12
                                     rust-bytemuck-1.25.0
                                     rust-bytemuck-derive-1.10.2
                                     rust-byteorder-1.5.0
                                     rust-byteorder-lite-0.1.0
                                     rust-bytes-1.11.1
                                     rust-bytes-str-0.2.8
                                     rust-calendrical-calculations-0.2.4
                                     rust-calloop-0.13.0
                                     rust-calloop-0.14.4
                                     rust-calloop-wayland-source-0.3.0
                                     rust-calloop-wayland-source-0.4.1
                                     rust-camino-1.2.2
                                     rust-capacity-builder-0.5.0
                                     rust-capacity-builder-macros-0.3.0
                                     rust-cargo-platform-0.2.0
                                     rust-cargo-platform-0.3.3
                                     rust-cargo-util-schemas-0.8.2
                                     rust-cargo-metadata-0.21.0
                                     rust-cargo-metadata-0.23.1
                                     rust-castaway-0.2.4
                                     rust-cc-1.2.63
                                     rust-census-0.4.2
                                     rust-cexpr-0.6.0
                                     rust-cfg-if-1.0.4
                                     rust-cfg-aliases-0.1.1
                                     rust-cfg-aliases-0.2.1
                                     rust-cfg-block-0.1.1
                                     rust-chacha20-0.10.0
                                     rust-chrono-0.4.45
                                     rust-ciborium-0.2.2
                                     rust-ciborium-io-0.2.2
                                     rust-ciborium-ll-0.2.2
                                     rust-cipher-0.4.4
                                     rust-cipher-0.5.2
                                     rust-clang-sys-1.8.1
                                     rust-clap-4.5.60
                                     rust-clap-markdown-0.1.5
                                     rust-clap-builder-4.5.60
                                     rust-clap-derive-4.5.55
                                     rust-clap-lex-1.1.0
                                     rust-clipboard-win-5.4.1
                                     rust-clru-0.6.3
                                     rust-clubcard-0.3.3
                                     rust-clubcard-crlite-0.3.2
                                     rust-cmake-0.1.58
                                     rust-cmov-0.5.4
                                     rust-cobs-0.3.0
                                     rust-color-quant-1.1.0
                                     rust-colorchoice-1.0.5
                                     rust-colored-2.2.0
                                     rust-combine-4.6.7
                                     rust-compact-str-0.7.1
                                     rust-compact-str-0.9.1
                                     rust-compression-codecs-0.4.38
                                     rust-compression-core-0.4.32
                                     rust-concurrent-queue-2.5.0
                                     rust-console-0.15.11
                                     rust-const-default-1.0.0
                                     rust-const-default-derive-0.2.0
                                     rust-const-oid-0.9.6
                                     rust-constant-time-eq-0.4.2
                                     rust-constcat-0.6.1
                                     rust-convert-case-0.4.0
                                     rust-convert-case-0.6.0
                                     rust-cookie-0.18.1
                                     rust-core-foundation-0.9.4
                                     rust-core-foundation-0.10.1
                                     rust-core-foundation-sys-0.8.7
                                     rust-core-graphics-0.23.2
                                     rust-core-graphics-types-0.1.3
                                     rust-core-maths-0.1.1
                                     rust-cow-utils-0.1.3
                                     rust-cpubits-0.1.1
                                     rust-cpufeatures-0.2.17
                                     rust-cpufeatures-0.3.0
                                     rust-crc32c-0.6.8
                                     rust-crc32fast-1.5.0
                                     rust-critical-section-1.2.0
                                     rust-crossbeam-channel-0.5.15
                                     rust-crossbeam-deque-0.8.6
                                     rust-crossbeam-epoch-0.9.18
                                     rust-crossbeam-skiplist-0.1.3
                                     rust-crossbeam-utils-0.8.21
                                     rust-crossterm-0.28.1
                                     rust-crossterm-winapi-0.9.1
                                     rust-crunchy-0.2.4
                                     rust-crypto-bigint-0.5.5
                                     rust-crypto-common-0.1.7
                                     rust-crypto-common-0.2.2
                                     rust-cssparser-0.37.0
                                     rust-csv-1.4.0
                                     rust-csv-core-0.1.13
                                     rust-ctr-0.9.2
                                     rust-ctr-0.10.1
                                     rust-ctrlc-3.5.2
                                     rust-ctutils-0.4.2
                                     rust-cursive-0.21.1
                                     rust-cursive-macros-0.1.0
                                     rust-cursive-core-0.4.7
                                     rust-cursor-icon-1.2.0
                                     rust-curve25519-dalek-4.1.3
                                     rust-curve25519-dalek-derive-0.1.1
                                     rust-dark-light-2.0.0.0f18d2f
                                     rust-darling-0.20.11
                                     rust-darling-0.21.3
                                     rust-darling-0.23.0
                                     rust-darling-core-0.20.11
                                     rust-darling-core-0.21.3
                                     rust-darling-core-0.23.0
                                     rust-darling-macro-0.20.11
                                     rust-darling-macro-0.21.3
                                     rust-darling-macro-0.23.0
                                     rust-dashmap-6.2.1
                                     rust-data-encoding-2.11.0
                                     rust-data-url-0.3.2
                                     rust-datasketches-0.2.0
                                     rust-dateparser-0.2.1
                                     rust-debugid-0.8.0
                                     rust-deno-ast-0.53.2
                                     rust-deno-error-0.7.3
                                     rust-deno-error-macro-0.7.3
                                     rust-deno-lint-0.84.1
                                     rust-deno-media-type-0.4.0
                                     rust-deno-semver-0.10.1
                                     rust-deno-terminal-0.2.3
                                     rust-der-0.7.10
                                     rust-der-parser-9.0.0
                                     rust-deranged-0.5.8
                                     rust-derive-where-1.6.1
                                     rust-derive-builder-0.20.2
                                     rust-derive-builder-core-0.20.2
                                     rust-derive-builder-macro-0.20.2
                                     rust-derive-more-0.99.20
                                     rust-deunicode-1.6.2
                                     rust-dify-0.7.4
                                     rust-digest-0.10.7
                                     rust-digest-0.11.3
                                     rust-directories-6.0.0
                                     rust-dirs-6.0.0
                                     rust-dirs-next-2.0.0
                                     rust-dirs-sys-0.5.0
                                     rust-dirs-sys-next-0.1.2
                                     rust-dispatch-0.2.0
                                     rust-dispatch2-0.3.1
                                     rust-displaydoc-0.2.6
                                     rust-dlib-0.5.3
                                     rust-doctest-file-1.1.1
                                     rust-downcast-rs-1.2.1
                                     rust-downcast-rs-2.0.2
                                     rust-dpi-0.1.2
                                     rust-dprint-swc-ext-0.26.0
                                     rust-drm-0.14.1
                                     rust-drm-ffi-0.9.1
                                     rust-drm-fourcc-2.2.0
                                     rust-drm-sys-0.8.1
                                     rust-dtoa-1.0.11
                                     rust-dtoa-short-0.3.5
                                     rust-dunce-1.0.5
                                     rust-dyn-clone-1.0.20
                                     rust-dynify-0.1.2
                                     rust-dynify-macros-0.1.2
                                     rust-ecolor-0.33.3
                                     rust-ecow-0.2.6
                                     rust-ed25519-2.2.3
                                     rust-ed25519-dalek-2.2.0
                                     rust-egui-0.33.3
                                     rust-egui-extras-0.33.3
                                     rust-egui-kittest-0.33.3
                                     rust-either-1.16.0
                                     rust-elliptic-curve-0.13.8
                                     rust-emath-0.33.3
                                     rust-embedded-io-0.4.0
                                     rust-embedded-io-0.6.1
                                     rust-ena-0.14.4
                                     rust-encode-unicode-1.0.0
                                     rust-encoding-rs-0.8.35
                                     rust-encre-css-0.20.1
                                     rust-endi-1.1.1
                                     rust-enum-map-2.7.3
                                     rust-enum-map-derive-0.17.0
                                     rust-enumflags2-0.7.12
                                     rust-enumflags2-derive-0.7.12
                                     rust-enumset-1.1.13
                                     rust-enumset-derive-0.15.0
                                     rust-env-filter-1.0.1
                                     rust-env-logger-0.11.10
                                     rust-epaint-0.33.3
                                     rust-equator-0.4.2
                                     rust-equator-macro-0.4.2
                                     rust-equivalent-1.0.2
                                     rust-erased-serde-0.4.10
                                     rust-errno-0.3.14
                                     rust-error-code-3.3.2
                                     rust-event-listener-5.4.1
                                     rust-event-listener-strategy-0.5.4
                                     rust-exr-1.74.0
                                     rust-fallible-iterator-0.3.0
                                     rust-fancy-regex-0.17.0
                                     rust-fast-float2-0.2.3
                                     rust-fastbloom-0.14.1
                                     rust-fastcdc-3.2.1
                                     rust-fastdivide-0.4.2
                                     rust-faster-hex-0.10.0
                                     rust-fastrand-2.4.1
                                     rust-fax-0.2.7
                                     rust-fdeflate-0.3.7
                                     rust-ff-0.13.1
                                     rust-fiat-crypto-0.2.9
                                     rust-filetime-0.2.29
                                     rust-find-msvc-tools-0.1.9
                                     rust-fixedbitset-0.4.2
                                     rust-fixedbitset-0.5.7
                                     rust-flate2-1.1.9
                                     rust-float16-0.1.5
                                     rust-fluent-0.17.0
                                     rust-fluent-bundle-0.15.3
                                     rust-fluent-bundle-0.16.0
                                     rust-fluent-langneg-0.13.1
                                     rust-fluent-syntax-0.11.1
                                     rust-fluent-syntax-0.12.0
                                     rust-fluent-template-macros-0.13.3
                                     rust-fluent-templates-0.13.3
                                     rust-flume-0.11.1
                                     rust-fnv-1.0.7
                                     rust-foldhash-0.1.5
                                     rust-foldhash-0.2.0
                                     rust-foreign-types-0.5.0
                                     rust-foreign-types-macros-0.2.3
                                     rust-foreign-types-shared-0.3.1
                                     rust-fork-0.6.0
                                     rust-form-urlencoded-1.2.2
                                     rust-from-variant-3.0.0
                                     rust-fs-err-3.3.0
                                     rust-fs2-0.4.3
                                     rust-fs4-0.13.1
                                     rust-fs-extra-1.3.0
                                     rust-funty-2.0.0
                                     rust-futures-0.3.32
                                     rust-futures-channel-0.3.32
                                     rust-futures-concurrency-7.7.1
                                     rust-futures-core-0.3.32
                                     rust-futures-executor-0.3.32
                                     rust-futures-io-0.3.32
                                     rust-futures-lite-2.6.1
                                     rust-futures-macro-0.3.32
                                     rust-futures-sink-0.3.32
                                     rust-futures-task-0.3.32
                                     rust-futures-timer-3.0.4
                                     rust-futures-util-0.3.32
                                     rust-genawaiter-0.99.1
                                     rust-genawaiter-macro-0.99.1
                                     rust-generator-0.8.9
                                     rust-generic-array-0.14.7
                                     rust-gethostname-1.1.0
                                     rust-getopts-0.2.24
                                     rust-getrandom-0.2.17
                                     rust-getrandom-0.3.4
                                     rust-getrandom-0.4.2
                                     rust-getset-0.1.6
                                     rust-ghash-0.5.1
                                     rust-ghash-0.6.0
                                     rust-gif-0.14.2
                                     rust-gimli-0.32.3
                                     rust-gix-0.77.0
                                     rust-gix-actor-0.37.1
                                     rust-gix-attributes-0.29.0
                                     rust-gix-bitmap-0.2.16
                                     rust-gix-chunk-0.4.12
                                     rust-gix-command-0.6.5
                                     rust-gix-commitgraph-0.31.0
                                     rust-gix-config-0.50.0
                                     rust-gix-config-value-0.16.0
                                     rust-gix-date-0.12.1
                                     rust-gix-diff-0.57.1
                                     rust-gix-dir-0.19.0
                                     rust-gix-discover-0.45.0
                                     rust-gix-features-0.45.2
                                     rust-gix-filter-0.24.1
                                     rust-gix-fs-0.18.2
                                     rust-gix-glob-0.23.0
                                     rust-gix-hash-0.21.2
                                     rust-gix-hashtable-0.11.0
                                     rust-gix-ignore-0.18.0
                                     rust-gix-index-0.45.1
                                     rust-gix-lock-20.0.1
                                     rust-gix-object-0.54.1
                                     rust-gix-odb-0.74.0
                                     rust-gix-pack-0.64.1
                                     rust-gix-packetline-0.20.0
                                     rust-gix-path-0.10.22
                                     rust-gix-pathspec-0.14.0
                                     rust-gix-protocol-0.55.0
                                     rust-gix-quote-0.6.2
                                     rust-gix-ref-0.57.0
                                     rust-gix-refspec-0.35.0
                                     rust-gix-revision-0.39.0
                                     rust-gix-revwalk-0.25.0
                                     rust-gix-sec-0.12.2
                                     rust-gix-shallow-0.7.0
                                     rust-gix-status-0.24.0
                                     rust-gix-submodule-0.24.0
                                     rust-gix-tempfile-20.0.1
                                     rust-gix-trace-0.1.20
                                     rust-gix-transport-0.52.1
                                     rust-gix-traverse-0.51.1
                                     rust-gix-url-0.34.0
                                     rust-gix-utils-0.3.3
                                     rust-gix-validate-0.10.1
                                     rust-gix-worktree-0.46.0
                                     rust-glob-0.3.3
                                     rust-globset-0.4.18
                                     rust-group-0.13.0
                                     rust-h2-0.4.14
                                     rust-half-2.7.1
                                     rust-handlebars-6.4.1
                                     rust-hash32-0.2.1
                                     rust-hash32-0.3.1
                                     rust-hashbrown-0.12.3
                                     rust-hashbrown-0.14.5
                                     rust-hashbrown-0.15.5
                                     rust-hashbrown-0.16.1
                                     rust-hashbrown-0.17.1
                                     rust-heapless-0.7.17
                                     rust-heapless-0.8.0
                                     rust-heck-0.4.1
                                     rust-heck-0.5.0
                                     rust-hermit-abi-0.5.2
                                     rust-hex-0.4.3
                                     rust-hifijson-0.2.3
                                     rust-hipstr-0.6.0
                                     rust-home-0.5.12
                                     rust-hstr-3.0.6
                                     rust-html-escape-0.2.13
                                     rust-html2text-0.16.7
                                     rust-html5ever-0.38.0
                                     rust-html5ever-0.39.0
                                     rust-htmlescape-0.3.1
                                     rust-http-1.4.1
                                     rust-http-body-1.0.1
                                     rust-http-body-util-0.1.3
                                     rust-httparse-1.10.1
                                     rust-httpdate-1.0.3
                                     rust-humantime-2.3.0
                                     rust-hybrid-array-0.4.12
                                     rust-hyper-1.10.1
                                     rust-hyper-rustls-0.27.9
                                     rust-hyper-util-0.1.20
                                     rust-iana-time-zone-0.1.65
                                     rust-iana-time-zone-haiku-0.1.2
                                     rust-icu-calendar-2.2.1
                                     rust-icu-calendar-data-2.2.0
                                     rust-icu-collections-2.2.0
                                     rust-icu-locale-2.2.0
                                     rust-icu-locale-core-2.2.0
                                     rust-icu-locale-data-2.2.0
                                     rust-icu-normalizer-2.2.0
                                     rust-icu-normalizer-data-2.2.0
                                     rust-icu-properties-2.2.0
                                     rust-icu-properties-data-2.2.0
                                     rust-icu-provider-2.2.0
                                     rust-id-arena-2.3.0
                                     rust-ident-case-1.0.1
                                     rust-idna-1.1.0
                                     rust-idna-adapter-1.2.1
                                     rust-if-chain-1.0.3
                                     rust-ignore-0.4.26
                                     rust-image-0.25.10
                                     rust-image-webp-0.2.4
                                     rust-imara-diff-0.1.8
                                     rust-imgref-1.12.2
                                     rust-include-dir-0.7.4
                                     rust-include-dir-macros-0.7.4
                                     rust-indexmap-1.9.3
                                     rust-indexmap-2.14.0
                                     rust-indicatif-0.17.11
                                     rust-inout-0.1.4
                                     rust-inout-0.2.2
                                     rust-interpolate-name-0.2.4
                                     rust-interprocess-2.4.2
                                     rust-intl-memoizer-0.5.3
                                     rust-intl-pluralrules-7.0.2
                                     rust-intrusive-collections-0.9.7
                                     rust-intrusive-collections-0.10.2
                                     rust-inventory-0.3.24
                                     rust-ipnet-2.12.0
                                     rust-is-macro-0.3.7
                                     rust-is-terminal-polyfill-1.70.2
                                     rust-itertools-0.11.0
                                     rust-itertools-0.14.0
                                     rust-itoa-1.0.18
                                     rust-ixdtf-0.6.5
                                     rust-jaq-core-2.2.1
                                     rust-jaq-json-1.1.3
                                     rust-jaq-std-2.1.2
                                     rust-jiff-0.2.28
                                     rust-jiff-static-0.2.28
                                     rust-jiff-tzdb-0.1.6
                                     rust-jiff-tzdb-platform-0.1.3
                                     rust-jni-0.22.4
                                     rust-jni-macros-0.22.4
                                     rust-jni-sys-0.3.1
                                     rust-jni-sys-0.4.1
                                     rust-jni-sys-macros-0.4.1
                                     rust-jobserver-0.1.34
                                     rust-js-sys-0.3.99
                                     rust-keccak-0.1.6
                                     rust-kittest-0.3.0
                                     rust-kstring-2.0.2
                                     rust-lalrpop-0.20.2
                                     rust-lalrpop-util-0.20.2
                                     rust-lazy-static-1.5.0
                                     rust-lazycell-1.3.0
                                     rust-leb128fmt-0.1.0
                                     rust-lebe-0.5.3
                                     rust-levenshtein-automata-0.2.1
                                     rust-libc-0.2.186
                                     rust-libfuzzer-sys-0.4.13
                                     rust-libloading-0.8.9
                                     rust-libm-0.2.16
                                     rust-libmimalloc-sys-0.1.49
                                     rust-libredox-0.1.17
                                     rust-linkme-0.3.36
                                     rust-linkme-impl-0.3.36
                                     rust-linux-raw-sys-0.4.15
                                     rust-linux-raw-sys-0.9.4
                                     rust-linux-raw-sys-0.12.1
                                     rust-litemap-0.8.2
                                     rust-lnk-0.6.4
                                     rust-lock-api-0.4.14
                                     rust-log-0.4.32
                                     rust-logos-0.14.4
                                     rust-logos-codegen-0.14.4
                                     rust-logos-derive-0.14.4
                                     rust-loom-0.7.2
                                     rust-loop9-0.1.5
                                     rust-lru-0.16.4
                                     rust-lru-slab-0.1.2
                                     rust-lz4-flex-0.13.1
                                     rust-malachite-0.9.1
                                     rust-malachite-base-0.9.1
                                     rust-malachite-float-0.9.1
                                     rust-malachite-nz-0.9.1
                                     rust-malachite-q-0.9.1
                                     rust-maplit-1.0.2
                                     rust-markdown-1.0.0
                                     rust-markup5ever-0.38.0
                                     rust-markup5ever-0.39.0
                                     rust-matchers-0.2.0
                                     rust-matchit-0.8.4
                                     rust-maybe-async-0.2.11
                                     rust-maybe-rayon-0.1.1
                                     rust-md-5-0.10.6
                                     rust-md5-0.7.0
                                     rust-measure-time-0.9.0
                                     rust-memchr-2.8.1
                                     rust-memmap2-0.9.11
                                     rust-memoffset-0.9.1
                                     rust-miette-7.6.0
                                     rust-miette-derive-7.6.0
                                     rust-mimalloc-0.1.52
                                     rust-mime-0.3.17
                                     rust-mime-guess-2.0.5
                                     rust-mime-guess2-2.3.1
                                     rust-minimal-lexical-0.2.1
                                     rust-miniz-oxide-0.8.9
                                     rust-miniz-oxide-0.9.1
                                     rust-mint-0.5.9
                                     rust-mio-1.2.1
                                     rust-monch-0.6.0
                                     rust-moxcms-0.8.1
                                     rust-multer-3.1.0
                                     rust-murmurhash32-0.3.1
                                     rust-ndk-0.9.0
                                     rust-ndk-context-0.1.1
                                     rust-ndk-sys-0.6.0+11769913
                                     rust-new-debug-unreachable-1.0.6
                                     rust-nix-0.28.0
                                     rust-nix-0.30.1
                                     rust-nix-0.31.3
                                     rust-no-std-io2-0.9.4
                                     rust-nohash-hasher-0.2.0
                                     rust-nom-7.1.3
                                     rust-nom-8.0.0
                                     rust-noop-proc-macro-0.3.0
                                     rust-ntapi-0.4.3
                                     rust-nu-ansi-term-0.50.3
                                     rust-num-0.4.3
                                     rust-num-bigint-0.4.6
                                     rust-num-complex-0.4.6
                                     rust-num-conv-0.2.2
                                     rust-num-derive-0.3.3
                                     rust-num-derive-0.4.2
                                     rust-num-integer-0.1.46
                                     rust-num-iter-0.1.45
                                     rust-num-modular-0.6.1
                                     rust-num-order-1.2.0
                                     rust-num-rational-0.4.2
                                     rust-num-traits-0.2.19
                                     rust-num-cpus-1.17.0
                                     rust-num-enum-0.7.6
                                     rust-num-enum-derive-0.7.6
                                     rust-num-threads-0.1.7
                                     rust-number-prefix-0.4.0
                                     rust-number-to-words-0.1.1
                                     rust-objc-sys-0.3.5
                                     rust-objc2-0.5.2
                                     rust-objc2-0.6.4
                                     rust-objc2-app-kit-0.2.2
                                     rust-objc2-app-kit-0.3.2
                                     rust-objc2-cloud-kit-0.2.2
                                     rust-objc2-contacts-0.2.2
                                     rust-objc2-core-data-0.2.2
                                     rust-objc2-core-foundation-0.3.2
                                     rust-objc2-core-graphics-0.3.2
                                     rust-objc2-core-image-0.2.2
                                     rust-objc2-core-location-0.2.2
                                     rust-objc2-encode-4.1.0
                                     rust-objc2-foundation-0.2.2
                                     rust-objc2-foundation-0.3.2
                                     rust-objc2-io-kit-0.3.2
                                     rust-objc2-io-surface-0.3.2
                                     rust-objc2-link-presentation-0.2.2
                                     rust-objc2-metal-0.2.2
                                     rust-objc2-quartz-core-0.2.2
                                     rust-objc2-quartz-core-0.3.2
                                     rust-objc2-symbols-0.2.2
                                     rust-objc2-ui-kit-0.2.2
                                     rust-objc2-uniform-type-identifiers-0.2.2
                                     rust-objc2-user-notifications-0.2.2
                                     rust-object-0.37.3
                                     rust-oid-registry-0.7.1
                                     rust-once-cell-1.21.4
                                     rust-once-cell-polyfill-1.70.2
                                     rust-oneshot-0.1.13
                                     rust-oneshot-0.2.1
                                     rust-opaque-debug-0.3.1
                                     rust-openssl-probe-0.2.1
                                     rust-option-ext-0.2.0
                                     rust-orbclient-0.3.55
                                     rust-ordered-float-2.10.1
                                     rust-ordered-float-5.3.0
                                     rust-ordered-stream-0.2.0
                                     rust-outref-0.5.2
                                     rust-owned-ttf-parser-0.25.1
                                     rust-ownedbytes-0.9.0
                                     rust-owo-colors-3.5.0
                                     rust-owo-colors-4.3.0
                                     rust-pack1-1.1.0
                                     rust-par-core-2.0.0
                                     rust-parking-2.2.1
                                     rust-parking-lot-0.12.5
                                     rust-parking-lot-core-0.9.12
                                     rust-password-hash-0.5.0
                                     rust-passwords-3.1.16
                                     rust-paste-1.0.15
                                     rust-pastey-0.1.1
                                     rust-pastey-0.2.3
                                     rust-pathdiff-0.2.3
                                     rust-percent-encoding-2.3.2
                                     rust-pest-2.8.6
                                     rust-pest-derive-2.8.6
                                     rust-pest-generator-2.8.6
                                     rust-pest-meta-2.8.6
                                     rust-petgraph-0.6.5
                                     rust-phf-0.11.3
                                     rust-phf-0.13.1
                                     rust-phf-codegen-0.11.3
                                     rust-phf-codegen-0.13.1
                                     rust-phf-generator-0.11.3
                                     rust-phf-generator-0.13.1
                                     rust-phf-macros-0.11.3
                                     rust-phf-macros-0.13.1
                                     rust-phf-shared-0.11.3
                                     rust-phf-shared-0.13.1
                                     rust-pico-args-0.5.0
                                     rust-pin-project-1.1.13
                                     rust-pin-project-internal-1.1.13
                                     rust-pin-project-lite-0.2.17
                                     rust-piper-0.2.5
                                     rust-pkcs8-0.10.2
                                     rust-pkg-config-0.3.33
                                     rust-plain-0.2.3
                                     rust-pluralizer-0.5.0
                                     rust-png-0.18.1
                                     rust-polling-3.11.0
                                     rust-polyval-0.6.2
                                     rust-polyval-0.7.1
                                     rust-portable-atomic-1.13.1
                                     rust-portable-atomic-util-0.2.7
                                     rust-portpicker-0.1.1
                                     rust-postcard-1.1.3
                                     rust-potential-utf-0.1.5
                                     rust-powerfmt-0.2.0
                                     rust-ppv-lite86-0.2.21
                                     rust-precomputed-hash-0.1.1
                                     rust-prettyplease-0.2.37
                                     rust-proc-macro-crate-1.3.1
                                     rust-proc-macro-crate-3.5.0
                                     rust-proc-macro-error-attr2-2.0.0
                                     rust-proc-macro-error2-2.0.1
                                     rust-proc-macro-hack-0.5.20+deprecated
                                     rust-proc-macro2-1.0.106
                                     rust-process-wrap-9.1.0
                                     rust-prodash-30.0.1
                                     rust-profiling-1.0.18
                                     rust-profiling-procmacros-1.0.18
                                     rust-prost-0.14.4
                                     rust-prost-derive-0.14.4
                                     rust-psm-0.1.31
                                     rust-ptr-meta-0.1.4
                                     rust-ptr-meta-derive-0.1.4
                                     rust-pxfm-0.1.29
                                     rust-qoi-0.4.1
                                     rust-quick-error-2.0.1
                                     rust-quick-xml-0.41.0
                                     rust-quinn-0.11.9
                                     rust-quinn-proto-0.11.14
                                     rust-quinn-udp-0.5.14
                                     rust-quote-1.0.45
                                     rust-r-efi-5.3.0
                                     rust-r-efi-6.0.0
                                     rust-radium-0.7.0
                                     rust-rand-0.8.6
                                     rust-rand-0.9.4
                                     rust-rand-0.10.1
                                     rust-rand-chacha-0.3.1
                                     rust-rand-chacha-0.9.0
                                     rust-rand-core-0.6.4
                                     rust-rand-core-0.9.5
                                     rust-rand-core-0.10.1
                                     rust-rand-pcg-0.3.1
                                     rust-rand-xoshiro-0.7.0
                                     rust-random-pick-1.2.17
                                     rust-rapidhash-4.4.1
                                     rust-rav1e-0.8.1
                                     rust-ravif-0.13.0
                                     rust-raw-window-handle-0.6.2
                                     rust-rayon-1.12.0
                                     rust-rayon-core-1.13.0
                                     rust-recvmsg-1.0.0
                                     rust-redb-2.6.3
                                     rust-redox-syscall-0.4.1
                                     rust-redox-syscall-0.5.18
                                     rust-redox-syscall-0.8.1
                                     rust-redox-users-0.4.6
                                     rust-redox-users-0.5.2
                                     rust-ref-cast-1.0.25
                                     rust-ref-cast-impl-1.0.25
                                     rust-regex-1.12.4
                                     rust-regex-automata-0.4.14
                                     rust-regex-filtered-0.2.1
                                     rust-regex-lite-0.1.9
                                     rust-regex-syntax-0.8.11
                                     rust-regress-0.11.1
                                     rust-rend-0.4.2
                                     rust-reqwest-0.13.4
                                     rust-rgb-0.8.53
                                     rust-ring-0.17.14
                                     rust-rkyv-0.7.46
                                     rust-rkyv-derive-0.7.46
                                     rust-roaring-0.11.4
                                     rust-ron-0.10.1
                                     rust-rust-stemmers-1.2.0
                                     rust-rust-decimal-1.42.0
                                     rust-rustc-demangle-0.1.27
                                     rust-rustc-hash-1.1.0
                                     rust-rustc-hash-2.1.2
                                     rust-rustc-version-0.2.3
                                     rust-rustc-version-0.4.1
                                     rust-rustc-version-runtime-0.3.0
                                     rust-rusticata-macros-4.1.0
                                     rust-rustix-0.38.44
                                     rust-rustix-1.1.4
                                     rust-rustls-0.23.40
                                     rust-rustls-native-certs-0.8.4
                                     rust-rustls-pki-types-1.14.1
                                     rust-rustls-platform-verifier-0.7.0
                                     rust-rustls-platform-verifier-android-0.1.1
                                     rust-rustls-webpki-0.103.13
                                     rust-rustversion-1.0.22
                                     rust-ryu-1.0.23
                                     rust-ryu-js-1.0.2
                                     rust-safe-arch-1.0.0
                                     rust-same-file-1.0.6
                                     rust-schannel-0.1.29
                                     rust-schemars-0.9.0
                                     rust-schemars-1.2.1
                                     rust-scoped-tls-1.0.1
                                     rust-scopeguard-1.2.0
                                     rust-sea-query-1.0.1
                                     rust-sea-query-derive-1.0.0
                                     rust-seahash-4.1.0
                                     rust-sec1-0.7.3
                                     rust-security-framework-3.7.0
                                     rust-security-framework-sys-2.17.0
                                     rust-self-replace-1.5.0
                                     rust-self-cell-0.10.3
                                     rust-self-cell-1.2.2
                                     rust-semver-0.9.0
                                     rust-semver-1.0.28
                                     rust-semver-parser-0.7.0
                                     rust-seq-macro-0.3.6
                                     rust-serde-1.0.228
                                     rust-serde-untagged-0.1.9
                                     rust-serde-value-0.7.0
                                     rust-serde-bytes-0.11.19
                                     rust-serde-core-1.0.228
                                     rust-serde-derive-1.0.228
                                     rust-serde-html-form-0.2.8
                                     rust-serde-html-form-0.4.0
                                     rust-serde-json-1.0.150
                                     rust-serde-path-to-error-0.1.20
                                     rust-serde-repr-0.1.20
                                     rust-serde-spanned-0.6.9
                                     rust-serde-spanned-1.1.1
                                     rust-serde-urlencoded-0.7.1
                                     rust-serde-with-3.17.0
                                     rust-serde-with-macros-3.17.0
                                     rust-serde-yaml-0.9.34+deprecated
                                     rust-serial-test-3.5.0
                                     rust-serial-test-derive-3.5.0
                                     rust-sha1-0.10.6
                                     rust-sha1-checked-0.10.0
                                     rust-sha1-smol-1.0.1
                                     rust-sha2-0.10.9
                                     rust-sha3-0.10.9
                                     rust-sharded-slab-0.1.7
                                     rust-shell-words-1.1.1
                                     rust-shlex-1.3.0
                                     rust-shlex-2.0.1
                                     rust-shuttle-0.8.1
                                     rust-signal-hook-0.3.18
                                     rust-signal-hook-0.4.4
                                     rust-signal-hook-mio-0.2.5
                                     rust-signal-hook-registry-1.4.8
                                     rust-signature-2.2.0
                                     rust-simd-adler32-0.3.9
                                     rust-simd-cesu8-1.1.1
                                     rust-simd-helpers-0.1.0
                                     rust-simdutf8-0.1.5
                                     rust-simsimd-6.5.16
                                     rust-siphasher-0.3.11
                                     rust-siphasher-1.0.3
                                     rust-sketches-ddsketch-0.4.0
                                     rust-slab-0.4.12
                                     rust-small-btree-0.1.0.ffec924
                                     rust-smallvec-1.15.1
                                     rust-smart-default-0.7.1
                                     rust-smartstring-1.0.1
                                     rust-smithay-client-toolkit-0.19.2
                                     rust-smithay-client-toolkit-0.20.0
                                     rust-smol-str-0.2.2
                                     rust-socket2-0.5.10
                                     rust-socket2-0.6.4
                                     rust-softaes-0.1.5
                                     rust-spin-0.9.8
                                     rust-spki-0.7.3
                                     rust-sptr-0.3.2
                                     rust-stability-0.2.1
                                     rust-stable-deref-trait-1.2.1
                                     rust-stacker-0.1.24
                                     rust-static-assertions-1.1.0
                                     rust-strength-reduce-0.2.4
                                     rust-string-cache-0.8.9
                                     rust-string-cache-0.9.0
                                     rust-string-cache-codegen-0.6.1
                                     rust-string-enum-1.0.2
                                     rust-strsim-0.11.1
                                     rust-strum-0.26.3
                                     rust-strum-0.28.0
                                     rust-strum-macros-0.26.4
                                     rust-strum-macros-0.28.0
                                     rust-substring-1.4.5
                                     rust-subtle-2.6.1
                                     rust-swc-allocator-4.0.1
                                     rust-swc-atoms-9.0.0
                                     rust-swc-common-17.0.1
                                     rust-swc-config-3.1.2
                                     rust-swc-config-macro-1.0.1
                                     rust-swc-ecma-ast-18.0.0
                                     rust-swc-ecma-codegen-20.0.2
                                     rust-swc-ecma-codegen-macros-2.0.2
                                     rust-swc-ecma-lexer-26.0.0
                                     rust-swc-ecma-loader-17.0.0
                                     rust-swc-ecma-parser-27.0.7
                                     rust-swc-ecma-transforms-base-30.0.1
                                     rust-swc-ecma-transforms-classes-30.0.0
                                     rust-swc-ecma-transforms-macros-1.0.1
                                     rust-swc-ecma-transforms-proposal-30.0.0
                                     rust-swc-ecma-transforms-react-33.0.0
                                     rust-swc-ecma-transforms-typescript-33.0.0
                                     rust-swc-ecma-utils-24.0.0
                                     rust-swc-ecma-visit-18.0.1
                                     rust-swc-eq-ignore-macros-1.0.1
                                     rust-swc-macros-common-1.0.1
                                     rust-swc-sourcemap-9.3.4
                                     rust-swc-visit-2.0.1
                                     rust-symlink-0.1.0
                                     rust-syn-1.0.109
                                     rust-syn-2.0.117
                                     rust-sync-wrapper-1.0.2
                                     rust-synstructure-0.13.2
                                     rust-sys-locale-0.3.2
                                     rust-sysinfo-0.37.2
                                     rust-system-configuration-0.7.0
                                     rust-system-configuration-sys-0.6.0
                                     rust-tag-ptr-0.1.0.ffec924
                                     rust-takecrate-1.1.1
                                     rust-tantivy-0.26.1
                                     rust-tantivy-bitpacker-0.10.0
                                     rust-tantivy-columnar-0.7.0
                                     rust-tantivy-common-0.11.0
                                     rust-tantivy-fst-0.5.0
                                     rust-tantivy-query-grammar-0.26.0
                                     rust-tantivy-sstable-0.7.0
                                     rust-tantivy-stacker-0.7.0
                                     rust-tantivy-tokenizer-api-0.7.0
                                     rust-tap-1.0.1
                                     rust-tar-0.4.46
                                     rust-tempfile-3.27.0
                                     rust-temporal-rs-0.2.3
                                     rust-tendril-0.5.0
                                     rust-term-0.7.0
                                     rust-termcolor-1.4.1
                                     rust-termsize-0.1.9
                                     rust-text-io-0.1.13
                                     rust-text-lines-0.6.0
                                     rust-thin-vec-0.2.18
                                     rust-thiserror-1.0.69
                                     rust-thiserror-2.0.18
                                     rust-thiserror-impl-1.0.69
                                     rust-thiserror-impl-2.0.18
                                     rust-thread-local-1.1.9
                                     rust-tiff-0.11.3
                                     rust-time-0.3.47
                                     rust-time-core-0.1.8
                                     rust-time-macros-0.2.27
                                     rust-timezone-provider-0.2.3
                                     rust-tiny-keccak-2.0.2
                                     rust-tinystr-0.8.3
                                     rust-tinyvec-1.11.0
                                     rust-tinyvec-macros-0.1.1
                                     rust-tokio-1.52.3
                                     rust-tokio-macros-2.7.0
                                     rust-tokio-rustls-0.26.4
                                     rust-tokio-stream-0.1.18
                                     rust-tokio-util-0.7.18
                                     rust-toml-0.8.23
                                     rust-toml-0.9.12+spec-1.1.0
                                     rust-toml-1.1.2+spec-1.1.0
                                     rust-toml-datetime-0.6.11
                                     rust-toml-datetime-0.7.5+spec-1.1.0
                                     rust-toml-datetime-1.1.1+spec-1.1.0
                                     rust-toml-edit-0.19.15
                                     rust-toml-edit-0.22.27
                                     rust-toml-edit-0.25.12+spec-1.1.0
                                     rust-toml-parser-1.1.2+spec-1.1.0
                                     rust-toml-write-0.1.2
                                     rust-toml-writer-1.1.1+spec-1.1.0
                                     rust-tower-0.5.3
                                     rust-tower-http-0.6.11
                                     rust-tower-layer-0.3.3
                                     rust-tower-service-0.3.3
                                     rust-tracing-0.1.44
                                     rust-tracing-appender-0.2.5
                                     rust-tracing-attributes-0.1.31
                                     rust-tracing-core-0.1.36
                                     rust-tracing-log-0.2.0
                                     rust-tracing-serde-0.2.0
                                     rust-tracing-subscriber-0.3.23
                                     rust-tracing-test-0.2.6
                                     rust-tracing-test-macro-0.2.6
                                     rust-triomphe-0.1.15
                                     rust-try-lock-0.2.5
                                     rust-ttf-parser-0.25.1
                                     rust-turso-0.6.1
                                     rust-turso-core-0.6.1
                                     rust-turso-ext-0.6.1
                                     rust-turso-macros-0.6.1
                                     rust-turso-parser-0.6.1
                                     rust-turso-sdk-kit-macros-0.6.1
                                     rust-turso-sync-engine-0.6.1
                                     rust-turso-sync-sdk-kit-0.6.1
                                     rust-twox-hash-2.1.2
                                     rust-type-map-0.5.1
                                     rust-typed-arena-2.0.2
                                     rust-typeid-1.0.3
                                     rust-typenum-1.20.1
                                     rust-typetag-0.2.22
                                     rust-typetag-impl-0.2.22
                                     rust-ua-parser-0.2.2
                                     rust-ubyte-0.10.4
                                     rust-ucd-trie-0.1.7
                                     rust-uds-windows-1.2.1
                                     rust-uncased-0.9.10
                                     rust-unic-langid-0.9.6
                                     rust-unic-langid-impl-0.9.6
                                     rust-unic-langid-macros-0.9.6
                                     rust-unic-langid-macros-impl-0.9.6
                                     rust-unicase-2.8.1
                                     rust-unicode-bom-2.0.3
                                     rust-unicode-id-0.3.6
                                     rust-unicode-id-start-1.4.0
                                     rust-unicode-ident-1.0.24
                                     rust-unicode-normalization-0.1.25
                                     rust-unicode-segmentation-1.13.3
                                     rust-unicode-width-0.1.14
                                     rust-unicode-width-0.2.2
                                     rust-unicode-xid-0.2.6
                                     rust-universal-hash-0.5.1
                                     rust-universal-hash-0.6.1
                                     rust-unsafe-libyaml-0.2.11
                                     rust-untrusted-0.9.0
                                     rust-url-2.5.8
                                     rust-urlencoding-2.1.3
                                     rust-utf-8-0.7.6
                                     rust-utf16-iter-1.0.5
                                     rust-utf8-ranges-1.0.5
                                     rust-utf8-width-0.1.8
                                     rust-utf8-iter-1.0.4
                                     rust-utf8parse-0.2.2
                                     rust-uuid-1.23.2
                                     rust-v-frame-0.3.9
                                     rust-valuable-0.1.1
                                     rust-vergen-9.1.0
                                     rust-vergen-gix-9.1.0
                                     rust-vergen-lib-9.1.0
                                     rust-version-check-0.9.5
                                     rust-voprf-0.5.0
                                     rust-vsimd-0.8.0
                                     rust-walkdir-2.5.0
                                     rust-want-0.3.1
                                     rust-wasi-0.11.1+wasi-snapshot-preview1
                                     rust-wasip2-1.0.3+wasi-0.2.9
                                     rust-wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06
                                     rust-wasm-bindgen-0.2.122
                                     rust-wasm-bindgen-futures-0.4.72
                                     rust-wasm-bindgen-macro-0.2.122
                                     rust-wasm-bindgen-macro-support-0.2.122
                                     rust-wasm-bindgen-shared-0.2.122
                                     rust-wasm-encoder-0.244.0
                                     rust-wasm-metadata-0.244.0
                                     rust-wasm-streams-0.5.0
                                     rust-wasmparser-0.244.0
                                     rust-wayland-backend-0.3.15
                                     rust-wayland-client-0.31.14
                                     rust-wayland-csd-frame-0.3.0
                                     rust-wayland-cursor-0.31.14
                                     rust-wayland-protocols-0.32.12
                                     rust-wayland-protocols-experimental-20250721.0.1
                                     rust-wayland-protocols-misc-0.3.12
                                     rust-wayland-protocols-plasma-0.3.12
                                     rust-wayland-protocols-wlr-0.3.12
                                     rust-wayland-sys-0.31.11
                                     rust-web-sys-0.3.99
                                     rust-web-time-1.1.0
                                     rust-web-atoms-0.2.4
                                     rust-webbrowser-1.2.1
                                     rust-webpki-root-certs-1.0.7
                                     rust-weezl-0.1.12
                                     rust-which-4.4.2
                                     rust-wide-1.5.0
                                     rust-widestring-1.2.1
                                     rust-winapi-0.3.9
                                     rust-winapi-i686-pc-windows-gnu-0.4.0
                                     rust-winapi-util-0.1.11
                                     rust-winapi-x86-64-pc-windows-gnu-0.4.0
                                     rust-windows-0.61.3
                                     rust-windows-0.62.2
                                     rust-windows-collections-0.2.0
                                     rust-windows-collections-0.3.2
                                     rust-windows-core-0.61.2
                                     rust-windows-core-0.62.2
                                     rust-windows-future-0.2.1
                                     rust-windows-future-0.3.2
                                     rust-windows-implement-0.60.2
                                     rust-windows-interface-0.59.3
                                     rust-windows-link-0.1.3
                                     rust-windows-link-0.2.1
                                     rust-windows-numerics-0.2.0
                                     rust-windows-numerics-0.3.1
                                     rust-windows-registry-0.5.3
                                     rust-windows-result-0.3.4
                                     rust-windows-result-0.4.1
                                     rust-windows-strings-0.4.2
                                     rust-windows-strings-0.5.1
                                     rust-windows-sys-0.52.0
                                     rust-windows-sys-0.59.0
                                     rust-windows-sys-0.61.2
                                     rust-windows-targets-0.52.6
                                     rust-windows-threading-0.1.0
                                     rust-windows-threading-0.2.1
                                     rust-windows-aarch64-gnullvm-0.52.6
                                     rust-windows-aarch64-msvc-0.52.6
                                     rust-windows-i686-gnu-0.52.6
                                     rust-windows-i686-gnullvm-0.52.6
                                     rust-windows-i686-msvc-0.52.6
                                     rust-windows-x86-64-gnu-0.52.6
                                     rust-windows-x86-64-gnullvm-0.52.6
                                     rust-windows-x86-64-msvc-0.52.6
                                     rust-winnow-0.5.40
                                     rust-winnow-0.7.15
                                     rust-winnow-1.0.3
                                     rust-winreg-0.55.0
                                     rust-winresource-0.1.31
                                     rust-winstructs-0.3.2
                                     rust-wit-bindgen-0.51.0
                                     rust-wit-bindgen-0.57.1
                                     rust-wit-bindgen-core-0.51.0
                                     rust-wit-bindgen-rust-0.51.0
                                     rust-wit-bindgen-rust-macro-0.51.0
                                     rust-wit-component-0.244.0
                                     rust-wit-parser-0.244.0
                                     rust-workspace-filter-0.1.0
                                     rust-workspace-filter-build-0.1.0
                                     rust-write16-1.0.0
                                     rust-writeable-0.6.3
                                     rust-wyz-0.5.1
                                     rust-x11-dl-2.21.0
                                     rust-x11rb-protocol-0.13.2
                                     rust-x509-parser-0.16.0
                                     rust-xattr-1.6.1
                                     rust-xcursor-0.3.10
                                     rust-xi-unicode-0.3.0
                                     rust-xkbcommon-dl-0.4.2
                                     rust-xkbcommon-rs-codegen-0.1.1
                                     rust-xkeysym-0.2.1
                                     rust-xml-1.3.0
                                     rust-xml-rs-1.0.0
                                     rust-xmltree-0.12.0
                                     rust-xsum-0.1.6
                                     rust-xxhash-rust-0.8.15
                                     rust-y4m-0.8.0
                                     rust-yansi-1.0.1
                                     rust-yoke-0.8.3
                                     rust-yoke-derive-0.8.2
                                     rust-zbus-5.16.0
                                     rust-zbus-lockstep-0.5.2
                                     rust-zbus-lockstep-macros-0.5.2
                                     rust-zbus-macros-5.16.0
                                     rust-zbus-names-4.3.2
                                     rust-zerocopy-0.8.50
                                     rust-zerocopy-derive-0.8.50
                                     rust-zerofrom-0.1.8
                                     rust-zerofrom-derive-0.1.7
                                     rust-zeroize-1.8.2
                                     rust-zeroize-derive-1.4.3
                                     rust-zerotrie-0.2.4
                                     rust-zerovec-0.11.6
                                     rust-zerovec-derive-0.11.3
                                     rust-zlib-rs-0.5.5
                                     rust-zmij-1.0.21
                                     rust-zstd-0.13.3
                                     rust-zstd-safe-7.2.4
                                     rust-zstd-sys-2.0.16+zstd.1.5.7
                                     rust-zune-core-0.5.1
                                     rust-zune-inflate-0.2.54
                                     rust-zune-jpeg-0.5.15
                                     rust-zvariant-5.12.0
                                     rust-zvariant-derive-5.12.0
                                     rust-zvariant-utils-3.4.0)))

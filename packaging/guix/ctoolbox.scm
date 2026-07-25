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

;; CURRENT STATUS: Blocked by dependencies that require rustc 1.89, while Guix
;; only offers 1.85.1.

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

;; Building a package that's hidden (untested): guix build -e '(@ (gnu packages module-name) package-name)'
;; Building a package that's not exported: guix build --sources -e '(@@ (gnu packages rust-crates) rust-whatever-0.1.0)'

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
                (base32 "0placeholder1"))
                (modules '((guix build utils)))
                (snippet '(begin
                    (for-each delete-file-recursively '(
                        ".github"
                        ".vscode"
                        "built"
                        "packaging/guix/generated"
                        "ctoolbox/Cargo.lock"
                    ))
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
            (cargo-inputs 'ctoolbox #:module '(ctoolbox-rust-crates))
        ))
        (arguments
            (list
                #:phases
                #~(modify-phases %standard-phases
                    (add-before 'build 'pre-build
                        (lambda* (#:key inputs #:allow-other-keys)
                            (display (string-append "file://" (getcwd)))
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

ctoolbox

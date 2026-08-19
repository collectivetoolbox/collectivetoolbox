// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from from programatik29/axum-server/blob/master/src/tls_rustls/mod.rs: MIT
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// Parts derived from from programatik29/axum-server/blob/master/src/tls_rustls/mod.rs:

// Copyright 2021 Axum Server Contributors

// See additional licensing details at end of file.

//! Tls implementation using [`rustls`].
//! Vendored with changes from <https://github.com/programatik29/axum-server/blob/master/src/tls_rustls/mod.rs>
//!
//! # Example
//!
//! ```rust,no_run
//! use axum::{routing::get, Router};
//! use axum_server::tls_rustls::RustlsConfig;
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new().route("/", get(|| async { "Hello, world!" }));
//!
//!     let config = RustlsConfig::from_pem_file(
//!         "tls_rustls_vendored/fixtures/cert.pem",
//!         "tls_rustls_vendored/fixtures/key.pem",
//!     )
//!     .await
//!     .unwrap();
//!
//!     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//!     println!("listening on {}", addr);
//!     axum_server::bind_rustls(addr, config)
//!         .serve(app.into_make_service())
//!         .await
//!         .unwrap();
//! }
//! ```

use axum::BoxError;
use ctb_utilities::get_embedded_asset;
use include_dir::{Dir, include_dir};
use rustls::ServerConfig;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::{io, path::Path};

pub use axum_server::tls_rustls::RustlsConfig;

static TLS_RUSTLS_VENDORED_DATA_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/tls_rustls_vendored");

pub(crate) fn get_tls_rustls_vendored_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&TLS_RUSTLS_VENDORED_DATA_DIR, key)
}

fn io_other<E: Into<BoxError>>(error: E) -> io::Error {
    io::Error::other(error)
}

fn config_from_der(
    cert: Vec<Vec<u8>>,
    key: Vec<u8>,
) -> io::Result<ServerConfig> {
    let cert = cert.into_iter().map(CertificateDer::from).collect();
    let key = PrivateKeyDer::try_from(key).map_err(io_other)?;

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .map_err(io_other)?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(config)
}

fn config_from_pem(cert: Vec<u8>, key: Vec<u8>) -> io::Result<ServerConfig> {
    der_from_pem(cert, key)
        .and_then(|(cert_der, key_der)| config_from_der(cert_der, key_der))
}

pub(crate) fn typed_der_from_pem(
    cert: Vec<u8>,
    key: Vec<u8>,
) -> io::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert: Vec<CertificateDer<'static>> =
        CertificateDer::pem_slice_iter(&cert)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| io_other("failed to parse certificate"))?;

    let mut key_result: Result<PrivateKeyDer<'static>, io::Error> =
        Err(io_other("The private key file contained no keys"));

    // Check the entire PEM file for the key in case it is not first section
    for item in rustls_pki_types::pem::PemObject::pem_slice_iter(&key) {
        let key: Result<PrivateKeyDer<'static>, io::Error> =
            item.map_err(|_| io_other("failed to parse PEM"));

        match key_result {
            // if we already got a key, then...
            Ok(_) => {
                // ...if we get a key now, we know that there are multiple keys and that's not allowed
                if key.is_ok() {
                    return Err(io_other(
                        "The private key file containsed multiple keys (it must only contain one)",
                    ));
                }
            }
            // but if already have an error, just overwrite it with whatever we got this time. If
            // it's a good key, that's cool. If it's an error, then we're just ignoring the old
            // error in favor of this new one
            Err(_) => key_result = key,
        }
    }

    let key = key_result?;

    Ok((cert, key))
}

pub(crate) fn der_from_pem(
    cert: Vec<u8>,
    key: Vec<u8>,
) -> io::Result<(Vec<Vec<u8>>, Vec<u8>)> {
    let (cert, key) = typed_der_from_pem(cert, key)?;
    let cert_der: Vec<Vec<u8>> = cert.into_iter().map(|c| c.to_vec()).collect();
    let key_der = key.secret_der().to_vec();

    Ok((cert_der, key_der))
}

async fn config_from_pem_file(
    cert: impl AsRef<Path>,
    key: impl AsRef<Path>,
) -> io::Result<ServerConfig> {
    let cert = fs_err::tokio::read(cert.as_ref()).await?;
    let key = fs_err::tokio::read(key.as_ref()).await?;

    config_from_pem(cert, key)
}

async fn config_from_pem_chain_file(
    cert: impl AsRef<Path>,
    chain: impl AsRef<Path>,
) -> io::Result<ServerConfig> {
    let cert = fs_err::tokio::read(cert.as_ref()).await?;
    let cert = CertificateDer::pem_slice_iter(&cert)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| io_other("failed to parse certificate"))?;
    let key = fs_err::tokio::read(chain.as_ref()).await?;
    let key_cert: PrivateKeyDer = PrivateKeyDer::from_pem_slice(&key)
        .map_err(|_| io_other("could not parse pem file"))?;

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key_cert)
        .map_err(|_| io_other("invalid certificate"))
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use std::io;

    #[crate::ctb_test("tokio")]
    async fn from_pem_file_not_found() {
        let err = RustlsConfig::from_pem_file(
            "tls_rustls_vendored/fixtures/missing.pem",
            "tls_rustls_vendored/fixtures/key.pem",
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(
            err.to_string(),
            "failed to read from file `tls_rustls_vendored/fixtures/missing.pem`: No such file or directory (os error 2)"
        );

        let err = RustlsConfig::from_pem_file(
            "tls_rustls_vendored/fixtures/cert.pem",
            "tls_rustls_vendored/fixtures/missing.pem",
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(
            err.to_string(),
            "failed to read from file `tls_rustls_vendored/fixtures/missing.pem`: No such file or directory (os error 2)"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn from_pem_file_chain_file_not_found() {
        let err = RustlsConfig::from_pem_chain_file(
            "tls_rustls_vendored/fixtures/missing.pem",
            "tls_rustls_vendored/fixtures/key.pem",
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(
            err.to_string(),
            "failed to read from file `tls_rustls_vendored/fixtures/missing.pem`: No such file or directory (os error 2)"
        );

        let err = RustlsConfig::from_pem_chain_file(
            "tls_rustls_vendored/fixtures/cert.pem",
            "tls_rustls_vendored/fixtures/missing.pem",
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(
            err.to_string(),
            "failed to read from file `tls_rustls_vendored/fixtures/missing.pem`: No such file or directory (os error 2)"
        );
    }
}

/*

// From axum-server:

MIT License

Copyright 2021 Axum Server Contributors

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

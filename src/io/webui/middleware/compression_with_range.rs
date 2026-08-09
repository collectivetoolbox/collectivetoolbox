// Derived from tower-http (https://github.com/tower-rs/tower-http).
// SPDX-License-Identifier for parts derived from from tower-http: MIT

//! An edited version of parts of `tower_http::compression::CompressionLayer`
//! that supports range requests. Unfortunately this also required copying two
//! other enormous modules into the source because they're private in the
//! original.

use bytes::{Buf, Bytes};
use http::{Request, Response, header};
use http_body::Body;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, ready},
};
use tower::Layer;
use tower::Service;
use tower_http::BoxError;
use tower_http::compression::Predicate;
use tower_http::compression::predicate::DefaultPredicate;

pub mod compression_utils;
pub mod content_encoding;

use compression_utils::{
    AcceptEncoding, AsyncReadBody, CompressionLevel, DecorateAsyncRead,
    WrapBody,
};
use content_encoding::Encoding;

use async_compression::tokio::bufread::BrotliEncoder;
use async_compression::tokio::bufread::GzipEncoder;
use async_compression::tokio::bufread::ZlibEncoder;

/// Compress response bodies of the underlying service.
///
/// This uses the `Accept-Encoding` header to pick an appropriate encoding and adds the
/// `Content-Encoding` header to responses.
#[derive(Clone, Debug, Default)]
pub struct CompressionLayer<P = DefaultPredicate> {
    accept: AcceptEncoding,
    predicate: P,
    quality: CompressionLevel,
}

impl<S, P> Layer<S> for CompressionLayer<P>
where
    P: Predicate,
{
    type Service = Compression<S, P>;

    fn layer(&self, inner: S) -> Self::Service {
        Compression {
            inner,
            accept: self.accept,
            predicate: self.predicate.clone(),
            quality: self.quality,
        }
    }
}

impl CompressionLayer {
    /// Creates a new [`CompressionLayer`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to enable the gzip encoding.
    pub fn gzip(mut self, enable: bool) -> Self {
        self.accept.set_gzip(enable);
        self
    }

    /// Sets whether to enable the Deflate encoding.
    pub fn deflate(mut self, enable: bool) -> Self {
        self.accept.set_deflate(enable);
        self
    }

    /// Sets whether to enable the Brotli encoding.
    pub fn br(mut self, enable: bool) -> Self {
        self.accept.set_br(enable);
        self
    }

    /// Sets the compression quality.
    pub fn quality(mut self, quality: CompressionLevel) -> Self {
        self.quality = quality;
        self
    }

    /// Disables the gzip encoding.
    pub fn no_gzip(mut self) -> Self {
        self.accept.set_gzip(false);
        self
    }

    /// Disables the Deflate encoding.
    pub fn no_deflate(mut self) -> Self {
        self.accept.set_deflate(false);
        self
    }

    /// Disables the Brotli encoding.
    pub fn no_br(mut self) -> Self {
        self.accept.set_br(false);
        self
    }

    /// Disables the Zstd encoding.
    pub fn no_zstd(mut self) -> Self {
        self.accept.set_zstd(false);
        self
    }

    /// Replace the current compression predicate.
    pub fn compress_when<C>(self, predicate: C) -> CompressionLayer<C>
    where
        C: Predicate,
    {
        CompressionLayer {
            accept: self.accept,
            predicate,
            quality: self.quality,
        }
    }
}

/// Compress response bodies of the underlying service.
#[derive(Clone, Copy)]
pub struct Compression<S, P = DefaultPredicate> {
    inner: S,
    accept: AcceptEncoding,
    predicate: P,
    quality: CompressionLevel,
}

impl<S> Compression<S, DefaultPredicate> {
    /// Creates a new `Compression` wrapping the `service`.
    pub fn new(service: S) -> Compression<S, DefaultPredicate> {
        Self {
            inner: service,
            accept: AcceptEncoding::default(),
            predicate: DefaultPredicate::default(),
            quality: CompressionLevel::default(),
        }
    }
}

impl<S, P> Compression<S, P> {
    /// Sets whether to enable the gzip encoding.
    pub fn gzip(mut self, enable: bool) -> Self {
        self.accept.set_gzip(enable);
        self
    }

    /// Sets whether to enable the Deflate encoding.
    pub fn deflate(mut self, enable: bool) -> Self {
        self.accept.set_deflate(enable);
        self
    }

    /// Sets whether to enable the Brotli encoding.
    pub fn br(mut self, enable: bool) -> Self {
        self.accept.set_br(enable);
        self
    }

    /// Sets the compression quality.
    pub fn quality(mut self, quality: CompressionLevel) -> Self {
        self.quality = quality;
        self
    }

    /// Disables the gzip encoding.
    pub fn no_gzip(mut self) -> Self {
        self.accept.set_gzip(false);
        self
    }

    /// Disables the Deflate encoding.
    pub fn no_deflate(mut self) -> Self {
        self.accept.set_deflate(false);
        self
    }

    /// Disables the Brotli encoding.
    pub fn no_br(mut self) -> Self {
        self.accept.set_br(false);
        self
    }

    /// Disables the Zstd encoding.
    pub fn no_zstd(mut self) -> Self {
        self.accept.set_zstd(false);
        self
    }

    /// Replace the current compression predicate.
    pub fn compress_when<C>(self, predicate: C) -> Compression<S, C>
    where
        C: Predicate,
    {
        Compression {
            inner: self.inner,
            accept: self.accept,
            predicate,
            quality: self.quality,
        }
    }
}

impl<ReqBody, ResBody, S, P> Service<Request<ReqBody>> for Compression<S, P>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Body,
    P: Predicate,
{
    type Response = Response<CompressionBody<ResBody>>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future, P>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> ResponseFuture<S::Future, P> {
        let encoding = Encoding::from_headers(req.headers(), self.accept);

        ResponseFuture {
            inner: self.inner.call(req),
            encoding,
            predicate: self.predicate.clone(),
            quality: self.quality,
        }
    }
}

pin_project! {
    /// Response future of [`Compression`].
    #[derive(Debug)]
    pub struct ResponseFuture<F, P> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) encoding: Option<Encoding>,
        pub(crate) predicate: P,
        pub(crate) quality: CompressionLevel,
    }
}

impl<F, B, E, P> Future for ResponseFuture<F, P>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Body,
    P: Predicate,
{
    type Output = Result<Response<CompressionBody<B>>, E>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let res = ready!(self.as_mut().project().inner.poll(cx)?);

        // never recompress responses that are already compressed
        let should_compress =
            !res.headers().contains_key(header::CONTENT_ENCODING)
                && self.predicate.should_compress(&res);

        let (mut parts, body) = res.into_parts();

        if should_compress
            && !parts.headers.get_all(header::VARY).iter().any(|value| {
                contains_ignore_ascii_case(
                    value.as_bytes(),
                    header::ACCEPT_ENCODING.as_str().as_bytes(),
                )
            })
        {
            parts
                .headers
                .append(header::VARY, header::ACCEPT_ENCODING.into());
        }

        let body = match (should_compress, self.encoding) {
            (false, _)
            | (_, None | Some(Encoding::Identity | Encoding::Zstd)) => {
                CompressionBody::new(BodyInner::identity(body))
            }

            (_, Some(Encoding::Gzip)) => CompressionBody::new(BodyInner::gzip(
                WrapBody::new(body, self.quality),
            )),
            (_, Some(Encoding::Deflate)) => CompressionBody::new(
                BodyInner::deflate(WrapBody::new(body, self.quality)),
            ),
            (_, Some(Encoding::Brotli)) => CompressionBody::new(
                BodyInner::brotli(WrapBody::new(body, self.quality)),
            ),
        };

        // Note: Do NOT remove ACCEPT_RANGES header so ranges are advertised.
        parts.headers.remove(header::CONTENT_LENGTH);

        if should_compress {
            if let Some(enc) = self.encoding {
                if enc != Encoding::Identity && enc != Encoding::Zstd {
                    parts.headers.insert(
                        header::CONTENT_ENCODING,
                        enc.into_header_value(),
                    );
                }
            }
        }

        let res = Response::from_parts(parts, body);
        Poll::Ready(Ok(res))
    }
}

fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

pin_project! {
    /// Response body of [`Compression`].
    pub struct CompressionBody<B>
    where
        B: Body,
    {
        #[pin]
        pub(crate) inner: BodyInner<B>,
    }
}

impl<B> Default for CompressionBody<B>
where
    B: Body + Default,
{
    fn default() -> Self {
        Self {
            inner: BodyInner::Identity {
                inner: B::default(),
            },
        }
    }
}

impl<B> CompressionBody<B>
where
    B: Body,
{
    pub(crate) fn new(inner: BodyInner<B>) -> Self {
        Self { inner }
    }
}

pin_project! {
    #[project = BodyInnerProj]
    pub(crate) enum BodyInner<B>
    where
        B: Body,
    {
        Gzip {
            #[pin]
            inner: WrapBody<GzipEncoder<B>>,
        },
        Deflate {
            #[pin]
            inner: WrapBody<ZlibEncoder<B>>,
        },
        Brotli {
            #[pin]
            inner: WrapBody<BrotliEncoder<B>>,
        },
        Identity {
            #[pin]
            inner: B,
        },
    }
}

impl<B: Body> BodyInner<B> {
    pub(crate) fn gzip(inner: WrapBody<GzipEncoder<B>>) -> Self {
        Self::Gzip { inner }
    }

    pub(crate) fn deflate(inner: WrapBody<ZlibEncoder<B>>) -> Self {
        Self::Deflate { inner }
    }

    pub(crate) fn brotli(inner: WrapBody<BrotliEncoder<B>>) -> Self {
        Self::Brotli { inner }
    }

    pub(crate) fn identity(inner: B) -> Self {
        Self::Identity { inner }
    }
}

impl<B> Body for CompressionBody<B>
where
    B: Body,
    B::Error: Into<BoxError>,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.project().inner.project() {
            BodyInnerProj::Gzip { inner } => inner.poll_frame(cx),
            BodyInnerProj::Deflate { inner } => inner.poll_frame(cx),
            BodyInnerProj::Brotli { inner } => inner.poll_frame(cx),
            BodyInnerProj::Identity { inner } => {
                match ready!(inner.poll_frame(cx)) {
                    Some(Ok(frame)) => {
                        let frame = frame.map_data(|mut buf| {
                            buf.copy_to_bytes(buf.remaining())
                        });
                        Poll::Ready(Some(Ok(frame)))
                    }
                    Some(Err(err)) => Poll::Ready(Some(Err(err.into()))),
                    None => Poll::Ready(None),
                }
            }
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        if let BodyInner::Identity { inner } = &self.inner {
            inner.size_hint()
        } else {
            http_body::SizeHint::new()
        }
    }

    fn is_end_stream(&self) -> bool {
        if let BodyInner::Identity { inner } = &self.inner {
            inner.is_end_stream()
        } else {
            false
        }
    }
}

impl<B> DecorateAsyncRead for GzipEncoder<B>
where
    B: Body,
{
    type Input = AsyncReadBody<B>;
    type Output = GzipEncoder<Self::Input>;

    fn apply(input: Self::Input, quality: CompressionLevel) -> Self::Output {
        GzipEncoder::with_quality(input, quality.into_async_compression())
    }

    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input> {
        pinned.get_pin_mut()
    }
}

impl<B> DecorateAsyncRead for ZlibEncoder<B>
where
    B: Body,
{
    type Input = AsyncReadBody<B>;
    type Output = ZlibEncoder<Self::Input>;

    fn apply(input: Self::Input, quality: CompressionLevel) -> Self::Output {
        ZlibEncoder::with_quality(input, quality.into_async_compression())
    }

    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input> {
        pinned.get_pin_mut()
    }
}

impl<B> DecorateAsyncRead for BrotliEncoder<B>
where
    B: Body,
{
    type Input = AsyncReadBody<B>;
    type Output = BrotliEncoder<Self::Input>;

    fn apply(input: Self::Input, quality: CompressionLevel) -> Self::Output {
        let level = match quality {
            CompressionLevel::Default => async_compression::Level::Precise(4),
            other => other.into_async_compression(),
        };
        BrotliEncoder::with_quality(input, level)
    }

    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input> {
        pinned.get_pin_mut()
    }
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
    use axum::body::Body as AxumBody;
    use http::{Request, Response, header::ACCEPT_ENCODING};
    use http_body_util::BodyExt;
    use std::convert::Infallible;
    use tower::{Service, ServiceBuilder, ServiceExt};

    async fn handle(
        _req: Request<AxumBody>,
    ) -> Result<Response<AxumBody>, Infallible> {
        let file = tokio::fs::read("Cargo.toml").await.expect("file missing");
        Ok(Response::new(AxumBody::from(file)))
    }

    #[crate::ctb_test("tokio")]
    async fn accept_encoding_configuration_works()
    -> Result<(), tower_http::BoxError> {
        use std::io::Read;

        fn decode<R: Read>(mut r: R) -> std::io::Result<Vec<u8>> {
            let mut buf = Vec::new();
            r.read_to_end(&mut buf)?;
            Ok(buf)
        }

        // Read the source file once so we can verify each response round-trips to the same bytes.
        let expected = tokio::fs::read("Cargo.toml").await?;

        // Configure a layer that only offers deflate, then confirm the response is actually
        // deflate-encoded by decoding it and comparing to the original content.
        let deflate_only_layer = CompressionLayer::new()
            .quality(CompressionLevel::Best)
            .no_br()
            .no_gzip();

        let mut service = ServiceBuilder::new()
            .layer(deflate_only_layer)
            .service_fn(handle);

        let request = Request::builder()
            .header(ACCEPT_ENCODING, "gzip, deflate, br")
            .body(AxumBody::empty())?;

        let response = service.ready().await?.call(request).await?;

        assert_eq!(response.headers()["content-encoding"], "deflate");

        let deflate_body = response.into_body().collect().await?.to_bytes();

        // The "deflate" Content-Encoding is RFC 1950 zlib framing (2-byte header + Adler-32),
        // not raw RFC 1951 deflate, so use ZlibDecoder rather than DeflateDecoder.
        let decoded =
            decode(flate2::bufread::ZlibDecoder::new(&deflate_body[..]))?;
        assert_eq!(decoded, expected);

        // Same check for brotli.
        let br_only_layer = CompressionLayer::new()
            .quality(CompressionLevel::Best)
            .no_gzip()
            .no_deflate();

        let mut service = ServiceBuilder::new()
            .layer(br_only_layer)
            .service_fn(handle);

        let request = Request::builder()
            .header(ACCEPT_ENCODING, "gzip, deflate, br")
            .body(AxumBody::empty())?;

        let response = service.ready().await?.call(request).await?;

        assert_eq!(response.headers()["content-encoding"], "br");

        let br_body = response.into_body().collect().await?.to_bytes();

        // 4096 is the decoder's internal read-buffer size, not a content-length bound.
        let decoded = decode(brotli::Decompressor::new(&br_body[..], 4096))?;
        assert_eq!(decoded, expected);

        Ok(())
    }
}

/*
Code from tower-http is used under the following license:
======

Copyright (c) 2019-2021 Tower Contributors

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.


*/

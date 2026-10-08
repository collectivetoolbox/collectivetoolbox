//! Source annotations for typed interpretations of Dc strings.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{DcMixedDecode, DcMixedReader, DcMstr, DcMstring, DcStr, DcString};
use std::ops::Range;

/// Field encoding supplied by tagged-record derives, using inline bindings.
pub trait DcMixedFields: DcMixedDecode {
    /// Identifies the record owning the generated field annotations.
    fn dc_field_owner() -> &'static str;

    /// Encodes one named field as a new source fragment, including its tag.
    ///
    /// # Errors
    /// Returns an error for unknown field names or failed encoding.
    fn encode_dc_field(
        &self,
        field: &str,
        target: &mut DcMstring,
    ) -> Result<()>;
}

/// An interpretation failure at a byte position in the original string.
#[derive(Debug)]
pub struct DcDiagnostic {
    position: usize,
    error: anyhow::Error,
}

impl DcDiagnostic {
    /// Returns the source-byte position at which interpretation stopped.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the interpretation error, not a string-validity verdict.
    #[must_use]
    pub const fn error(&self) -> &anyhow::Error {
        &self.error
    }
}

/// Source representations supporting byte-preserving structured replacement.
pub trait DcSource {
    /// The corresponding owned string representation.
    type Owned;

    /// Returns the original encoded string, including its format header.
    fn source_bytes(&self) -> &[u8];

    /// Splices an encoded field after checking source and fragment boundaries.
    ///
    /// # Errors
    /// Returns an error for invalid ranges or ranges splitting encapsulations.
    fn replace_source_field(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
    ) -> Result<Self::Owned>;
}

fn splice(
    source: &[u8],
    range: Range<usize>,
    fragment: &[u8],
) -> Result<Vec<u8>> {
    ensure!(range.start <= range.end, "Reversed source range");
    let prefix = source
        .get(..range.start)
        .context("Source range starts outside string")?;
    let suffix = source
        .get(range.end..)
        .context("Source range ends outside string")?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(prefix);
    bytes.extend_from_slice(fragment);
    bytes.extend_from_slice(suffix);
    Ok(bytes)
}

impl DcSource for DcMstr {
    type Owned = DcMstring;

    fn source_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn replace_source_field(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
    ) -> Result<DcMstring> {
        ensure!(
            range.start >= crate::dc_mixed::DCMX_HEADER_LEN,
            "Cannot replace string header"
        );
        ensure!(
            range.start <= range.end && range.end <= self.len(),
            "Invalid source range"
        );
        let mut chunks = self.chunks();
        while !chunks.slice.is_empty() {
            let start = self.len().saturating_sub(chunks.slice.len());
            let chunk = chunks.next().context("Invalid mixed-mode chunk")?;
            let end = self.len().saturating_sub(chunks.slice.len());
            match chunk {
                crate::DcMixedChunk::Text(text) => {
                    for boundary in [range.start, range.end] {
                        if start <= boundary && boundary <= end {
                            ensure!(
                                text.is_char_boundary(
                                    boundary.saturating_sub(start)
                                ),
                                "Source range splits a Dc character"
                            );
                        }
                    }
                }
                _ => {
                    for boundary in [range.start, range.end] {
                        ensure!(
                            boundary <= start || boundary >= end,
                            "Source range splits an encapsulation"
                        );
                    }
                }
            }
        }
        DcMstring::from_bytes(splice(
            self.as_bytes(),
            range,
            fragment.payload_bytes(),
        )?)
        .map_err(Into::into)
    }
}

impl DcSource for DcStr {
    type Owned = DcString;

    fn source_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn replace_source_field(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
    ) -> Result<DcString> {
        ensure!(
            range.start <= range.end && range.end <= self.len(),
            "Invalid source range"
        );
        ensure!(
            self.is_char_boundary(range.start)
                && self.is_char_boundary(range.end),
            "Source range splits a Dc character"
        );
        let fragment = fragment.to_dc_string()?;
        DcString::from_dcutf(splice(
            self.as_bytes(),
            range,
            fragment.as_bytes(),
        )?)
        .map_err(Into::into)
    }
}

/// A borrowed source with a typed interpretation and successful annotations.
///
/// Failed interpretation retains the source and annotations before the error.
/// Inspection does not execute the string or gate runtime execution.
#[derive(Debug)]
pub struct DcInspection<'a, T, S: DcSource + ?Sized = DcMstr> {
    source: &'a S,
    value: Option<T>,
    fields: Vec<DcFieldAnnotation>,
    diagnostic: Option<DcDiagnostic>,
}

impl<'a, T: DcMixedDecode, S: DcSource + ?Sized> DcInspection<'a, T, S> {
    fn read(source: &'a S, mut reader: DcMixedReader<'a>) -> Self {
        reader.annotating = true;
        let result = T::decode_dc_mixed(&mut reader);
        let (value, error) = match result {
            Ok(value) => {
                let error = match reader.peek_char() {
                    Ok(None) => None,
                    Ok(Some(_)) => Some(anyhow::anyhow!(
                        "Trailing content after typed Dc value"
                    )),
                    Err(error) => Some(error),
                };
                (Some(value), error)
            }
            Err(error) => (None, Some(error)),
        };
        let diagnostic = if let Some(error) = error {
            Some(DcDiagnostic {
                position: reader.position(),
                error,
            })
        } else {
            None
        };
        let fields = std::mem::take(&mut reader.annotations);
        Self {
            source,
            value,
            fields,
            diagnostic,
        }
    }

    /// Returns the original bytes without reconstruction or normalization.
    #[must_use]
    pub fn source_bytes(&self) -> &[u8] {
        self.source.source_bytes()
    }

    /// Returns the typed value if record decoding succeeded.
    /// Trailing syntax may still be reported by `diagnostic`.
    #[must_use]
    pub const fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Returns successfully decoded fields, including nested record fields.
    #[must_use]
    pub fn fields(&self) -> &[DcFieldAnnotation] {
        &self.fields
    }

    /// Returns the first failure; inspection never guesses past unknown syntax.
    #[must_use]
    pub const fn diagnostic(&self) -> Option<&DcDiagnostic> {
        self.diagnostic.as_ref()
    }

    /// Replaces an explicitly selected field, including nested occurrences.
    ///
    /// The annotation must belong to this inspection and the supplied record
    /// must have the annotated type. Repeated fields encode the supplied field
    /// collection in place of the selected occurrence, leaving others intact.
    /// Bytes outside the selected range are preserved; dependent unknown syntax
    /// may change meaning. Edits inline reference values, not their declarations.
    ///
    /// # Errors
    /// Returns an error for foreign annotations, type mismatches, encoding
    /// failures, or ranges that split encapsulations.
    pub fn replace_annotation<C: DcMixedFields>(
        &self,
        annotation: &DcFieldAnnotation,
        updated: &C,
    ) -> Result<S::Owned> {
        ensure!(
            self.fields
                .iter()
                .any(|field| std::ptr::eq(field, annotation)),
            "Annotation does not belong to this source inspection"
        );
        ensure!(
            annotation.owner == C::dc_field_owner(),
            "Updated record does not own this annotated field"
        );
        let mut fragment = DcMstring::new();
        updated.encode_dc_field(annotation.field, &mut fragment)?;
        self.source
            .replace_source_field(annotation.range(), &fragment)
    }
}

impl<T: DcMixedFields, S: DcSource + ?Sized> DcInspection<'_, T, S> {
    /// Replaces one fully decoded outer field using the supplied typed record.
    ///
    /// Every byte outside the field remains unchanged. The new field is encoded
    /// with inline bindings. This does not guarantee unchanged semantics for
    /// unknown syntax that depends on the edited value. Reinspect after editing.
    ///
    /// # Errors
    /// Returns an error for missing or repeated fields, encoding failure, or
    /// source ranges that split encapsulations. Absent fields are not inserted.
    pub fn replace_field(&self, field: &str, updated: &T) -> Result<S::Owned> {
        let mut matching = self.fields.iter().filter(|annotation| {
            annotation.depth == 1
                && annotation.owner == T::dc_field_owner()
                && annotation.field == field
        });
        let annotation = matching
            .next()
            .context("No fully decoded outer field with this name")?;
        ensure!(
            matching.next().is_none(),
            "Repeated field is ambiguous; select a larger understood record instead"
        );
        self.replace_annotation(annotation, updated)
    }
}

impl DcMstr {
    /// Annotates an interpretation, preserving source even when decoding fails.
    #[must_use]
    pub fn inspect<T: DcMixedDecode>(&self) -> DcInspection<'_, T> {
        DcInspection::read(self, DcMixedReader::new(self))
    }

    /// Annotates with explicitly supplied outer reference bindings.
    #[must_use]
    pub fn inspect_with_context<'a, T: DcMixedDecode>(
        &'a self,
        context: &'a crate::SharedValueContext,
    ) -> DcInspection<'a, T> {
        DcInspection::read(self, DcMixedReader::with_context(self, context))
    }
}

impl DcStr {
    /// Interprets the entire DcUtf string without copying or changing it.
    ///
    /// # Errors
    /// Returns an error on unsupported syntax or trailing content.
    pub fn decode<T: DcMixedDecode>(&self) -> Result<T> {
        let mut reader = DcMixedReader::from_dc_str(self);
        let value = T::decode_dc_mixed(&mut reader)?;
        ensure!(
            reader.peek_char()?.is_none(),
            "Trailing content after typed Dc value"
        );
        Ok(value)
    }

    /// Interprets the whole DcUtf string with explicitly supplied bindings.
    ///
    /// # Errors
    /// Returns an error on unsupported syntax or trailing content.
    pub fn decode_with_context<T: DcMixedDecode>(
        &self,
        context: &crate::SharedValueContext,
    ) -> Result<T> {
        let mut reader = DcMixedReader::from_dc_str_with_context(self, context);
        let value = T::decode_dc_mixed(&mut reader)?;
        ensure!(
            reader.peek_char()?.is_none(),
            "Trailing content after typed Dc value"
        );
        Ok(value)
    }

    /// Annotates an interpretation without copying or changing the source.
    #[must_use]
    pub fn inspect<T: DcMixedDecode>(&self) -> DcInspection<'_, T, DcStr> {
        DcInspection::read(self, DcMixedReader::from_dc_str(self))
    }

    /// Annotates DcUtf with explicitly supplied outer reference bindings.
    #[must_use]
    pub fn inspect_with_context<'a, T: DcMixedDecode>(
        &'a self,
        context: &'a crate::SharedValueContext,
    ) -> DcInspection<'a, T, DcStr> {
        DcInspection::read(
            self,
            DcMixedReader::from_dc_str_with_context(self, context),
        )
    }
}

/// A successfully interpreted field and its original source-byte range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DcFieldAnnotation {
    owner: &'static str,
    field: &'static str,
    range: Range<usize>,
    depth: usize,
}

impl DcFieldAnnotation {
    pub(crate) const fn new(
        owner: &'static str,
        field: &'static str,
        range: Range<usize>,
        depth: usize,
    ) -> Self {
        Self {
            owner,
            field,
            range,
            depth,
        }
    }

    /// Returns the fully qualified Rust type responsible for interpretation.
    #[must_use]
    pub const fn owner(&self) -> &'static str {
        self.owner
    }

    /// Returns the Rust field name, not a Dc identifier.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns a byte range in the original string, including any field tag.
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Returns the typed record nesting depth, starting at one.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
}

# Security Architecture and Collaboration Boundaries

Planning note, 2026-10-02. Status: **Proposed / not security-qualified**.
This records requirements and architecture decisions for the
[MVP roadmap](mvp-roadmap.md), not a cryptographic protocol specification or an
audit. No cryptographic implementation or dependency was changed for this note.

## Remaining Gaps

- Private collaboration, encrypted recovery, identity verification, device
  revocation, and metadata-private synchronization are not established as secure.
  Existing code and [security notes](../INSECURITY.md) are not certification.
- The requirement to conceal customer access patterns from an actively untrusted
  server is substantially stronger than end-to-end encryption. Direct network
  connections and identifiable login/recovery requests disclose information.
  No design satisfying the full requirement has been selected or demonstrated.
- Recoverable history and key backups change the consequences of compromise.
  A forward-secure messaging protocol does not make retained document history
  forward-secret against compromise of its recovery/decryption keys.
- Independent security review is a prerequisite to a production/business release,
  not something a library name, test suite, or this assessment can substitute for.

## Confirmed Requirements

- Global graph content is public; authenticity, integrity, and availability still
  matter. Local and team content is private unless deliberately shared.
- Treat hosting, storage, and relay servers as untrusted, including active
  misbehavior. The intended disclosed account information is username,
  payment/subscription information, and storage usage, not document content,
  document metadata, or customer-correlated PUT/GET usage patterns.
- Other team members are not universally trusted. A new employee must not learn
  board documents or their metadata, and peers must not learn another member's
  IP address merely by collaborating. These requirements also constrain search,
  indexes, notifications, identifiers, and the network topology.
- Employers must be able to revoke work access. An organization may provision and
  recover work identities, but that must confer no access to personal identities,
  documents, or keys. Linking work and personal accounts is organization policy;
  shared billing/login is not shared authority.
- The subscription should restore a user's documents, configuration, and necessary
  durable keys onto another computer. The untrusted server must store protected
  recovery material, not possess a way to decrypt it.
- The first `ctoolbox archive` is local and unencrypted, a metadata-preserving
  tar-like operation. No server, sync service, MLS, or account is required.
  Plaintext filenames, metadata, logs, and streams require suitable local storage.

These are requirements, not evidence that the current design satisfies them.
If stronger privacy requires different infrastructure or substantial overhead,
present that tradeoff to the operator; do not silently lower the requirement.

## Different Security Properties

| Concept | What it provides | What it does not provide |
| --- | --- | --- |
| Checksum/content digest | Detects accidental corruption; identifies bytes | Authenticity against someone who can replace both data and checksum |
| Digital signature | Evidence that the corresponding private key signed particular bytes/context | Secrecy, truth of the statement, proof of a person's real-world identity, or permission to act |
| Authenticated encryption | Confidentiality and tamper detection for holders of the right keys | Hidden traffic patterns, access policy, or protection from an authorized reader copying plaintext |
| Forward secrecy (FS/PFS) | Later key compromise need not expose past communication protected by deleted earlier secrets | Protection for plaintext history or decryptable backups deliberately retained elsewhere |
| Post-compromise security (PCS) | Future communication can recover after compromise ends and the required fresh-key updates occur | Healing while malware remains active or an attacker remains an authorized device/member |
| Post-quantum cryptography (PQC) | Algorithms intended to resist known attacks using large quantum computers | FS, PCS, recovery, authorization, or metadata privacy by itself |

The public/private signing-key model remains useful. The changes are stronger
protocols, key lifecycle rules, and algorithm migration, not the disappearance
of signatures. Signature validation, identity authentication, authorization,
semantic trust, and runtime capabilities must remain distinct checks.

### Double Ratchet, MLS, and post-quantum algorithms

Double Ratchet is principally a protocol for evolving keys in two-party
communication. MLS is a group key-establishment/messaging protocol for changing
groups of clients. It is a plausible candidate for future team collaboration,
not a graph database, access-control policy, synchronization algorithm, or
encrypted-backup format. Neither requires a request/response or page-load UI.

RFC 9420's original cipher suites use classical public-key algorithms. "Uses
MLS" is not a claim of post-quantum security. Signal's current specification
also distinguishes the classical Double Ratchet from Sparse Post-Quantum and
Triple Ratchet designs; do not implement an old description assuming it provides
the properties of newer designs.

NIST standardized ML-KEM for key establishment and ML-DSA for signatures in
August 2024. They address different jobs. Evaluate established protocol profiles
and maintained implementations, including approved hybrid constructions where
appropriate. Do not invent a hybrid by concatenating keys or signatures, or
substitute a primitive into a protocol whose security analysis no longer applies.

Long-lived confidential data creates a "harvest now, decrypt later" concern:
recorded classical public-key exchanges may become decryptable in the future.
Upgrading after capture cannot undo that exposure. PQ confidentiality and PQ
signature migration need separate plans; passwords and symmetric encryption
also require their own strength and implementation review.

The `mls-rs` README checked on 2026-10-02 claims RFC 9420 conformance but says
it has not yet received a full third-party security audit. It is a candidate,
not the selected dependency. Recheck that status when evaluating it. Inspect
the exact version, crypto provider, audit coverage, protocol suites, persistence,
interoperability, maintenance, and static Linux/Windows build compatibility.
An AWS association or use of a FIPS-standard algorithm does not by itself confer
security assurance or FIPS module validation on this application.

## Boundaries to Establish Before Stable Formats

### Identity must outlive keys

Keep person/organization/scope identity separate from any one public key.
Represent authenticated key bindings, purpose, device, rotation, revocation,
and verification evidence without changing the entity's Dc identity. Different
devices should have independently revocable credentials; restoring a desktop
must not require cloning another device's live protocol state.

Separate document signing, device/group communication, stored-content encryption,
and account authentication/recovery. Use established key-management facilities;
do not reuse one private key for every purpose. Ordinary graph configuration can
refer to protected key handles and public credentials, not publish private keys.
PC settings are shared across users and must not become a shared private-key store.

The server must not be able to silently substitute its key for a colleague's.
Identity binding needs an independent trust anchor/verification design, such as
organization credentials or verifiable key-directory mechanisms. A signed graph
claim saying "this is the CEO's key" is not its own proof of that identity.

### Stable Dc bodies, replaceable security envelopes

Keep exact document content/identity distinct from ciphertext, signatures, and
transport state. Use established signed/encrypted containers and supported
versioned algorithm identifiers. Select the exact serialization/profile before
publishing a stable encrypted/signed format; this note does not define one.

Sign/authenticate an unambiguous context binding the bytes to the graph, node,
revision, and intended operation where applicable. Never verify a signature and
then silently rebind its data to another identity, scope, or operation. Preserve
exact source filename bytes; Unicode normalization is not cryptographic
canonicalization. If deterministic Dc encoding is needed, define it before
signing and preserve those signed bytes.

New attestations can add signatures or migration evidence without modifying old
global node bodies. Do not fix one signature/key size or algorithm forever in
the node model. Algorithm agility requires explicit acceptance policy and
downgrade protection, not accepting arbitrary algorithms supplied by a peer.
Historical signature verification also needs revocation/compromise policy and
trusted checkpoints where relevant; a signer's own timestamp cannot prove that
a signature predates compromise. Later re-signing cannot retroactively establish
authenticity that was already lost.

For encrypted objects, key/ciphertext rotation must not renumber logical nodes.
Public content hashes and stable identifiers exposed in private requests may
reveal equality, relationships, or enable guessing attacks. Keep private object
addressing and encryption behind a reviewed storage interface; do not introduce
cross-customer plaintext-hash deduplication as an unnoticed privacy dependency.

### Read permissions precede trust-weighted querying

The scope's trust filter determines which accessible assertions to believe; it
does not prevent decryption of data that should never have been accessible.
Authorization and key distribution must restrict document visibility first.
The board and the whole company cannot share a universal decryption key merely
because they share a team graph. Group boundaries must match actual readership
boundaries, with organization membership-change policy enforced by clients.

Indexes can reveal entire documents through triples, names, relationship edges,
search terms, counts, and provenance. Shared indexes must respect the same
confidentiality boundaries as their source content; unauthorized peers cannot
receive the full index and simply filter it in the UI. Keep indexing on
authorized clients for the initial model. Private server-side search would be
a separate, specialist-reviewed problem.

Treat MLS's ordered membership epochs as distinct from graph revision ordering
or merge/conflict rules. Specify device addition, offline return, removal,
membership forks, replay/rollback detection, and fresh state on recovery. MLS
does not resolve concurrent graph edits or validate business approvals.

### Retained history and recoverable backups are a separate layer

Use secure communication for delivery, then a separately protected retained
document store. Do not keep old messaging/ratchet secrets forever to implement
history. Deliberately retained document decryption/recovery keys permit history
access and therefore expose that retained history if compromised. State that
threat boundary honestly even when delivery traffic has FS.

Back up durable identity/recovery material and retained-content keys through a
reviewed encrypted-vault design. Do not indiscriminately back up expired message
keys, consumed one-time keys, or live ratchet snapshots as ordinary immutable
graph history. Local protocol persistence must be crash-safe, and restoration
must prevent rollback/key reuse; a restored device should enroll/rejoin using
fresh protocol state under an approved recovery flow.

Decide who gets historical documents on joining, recovering, or leaving a group.
Removing a member and rotating future keys can stop future access under the
protocol's assumptions. It cannot erase plaintext or old keys already copied
by that member. Rewrapping keys alone does not revoke copies of the same content
key. Stronger retroactive-erasure claims are not available from MLS or encryption.

### Password login is not encrypted-vault recovery

A server that stores a password hash and releases a backup after authentication
has an access-control mechanism, not proof that it cannot decrypt the backup.
Vault protection must be performed client-side with secrets the server does not
learn. Never derive recovery encryption from a verifier/hash already available
to the server, and do not send the vault-unlocking password to an actively
untrusted server as ordinary login data.

Evaluate a reviewed password-authentication and vault-recovery design, potentially
an augmented PAKE such as OPAQUE plus separately designed vault protection, rather
than constructing one from primitives. A PAKE is not a complete backup protocol.
Password-only recovery exposes password-guessing risks under relevant server or
vault compromise; password strength and a calibrated password KDF matter.
Offer a high-entropy recovery secret or trusted-device recovery where appropriate.

If no device, recovery secret, or authorized recovery party remains, a password
reset cannot both recover otherwise inaccessible data and preserve the promise
that the provider has no recovery authority. Make that product tradeoff explicit.
Protect against server rollback/substitution of recovery bundles as well as
theft; encryption alone does not prove the bundle is the latest version.

Organization recovery is an explicit additional authority for work data only.
If the employer holds an employee's signing private key, it can also sign as
that employee. Decide whether work recovery should instead recover data while
using separate administrator/device attestations for new signing authority.
Do not promise exclusive employee authorship while sharing that signing key.

### Metadata privacy is an unresolved architecture gate

Separate content metadata from traffic metadata. Encryption can protect names,
graph relationships, and search content, but a directly contacted server sees
connection addresses, timing, request sizes, and identifiers it handles. An
authenticated restore also reveals an account activity event unless the design
specifically hides it. Total storage/accounting disclosures may themselves be
correlatable; define exactly what is allowed and at what granularity.

Blind tokens can separate token issuance from redemption at the cryptographic
level. They do not, by themselves, unlink IP addresses, timing, batch sizes,
sessions, object access patterns, or distinguishable issuer behavior. Existing
[blind-token code](../src/utilities/blind_signatures.rs) and
[sync code](../src/storage/models/sync.rs) are exploratory, not an anonymous
storage protocol. The inspected sync path uses direct HTTP, authenticates token
issuance with a session cookie, and transmits graph IDs for ID reservation;
these are concrete inputs to a future privacy review, not proof of unlinkability.

Direct peer-to-peer connections also conflict with hiding participant IPs from
peers. Relays may hide addresses from peers while exposing them to the relay.
Stronger anonymity can require independent/non-colluding intermediaries,
anonymity networks, padding, batching/cover traffic, and access-pattern-hiding
storage techniques. Each has costs and residual leakage; none is prescribed as
an ad hoc construction here. Define attacker observation/collusion capabilities,
then get specialist feasibility/design review before implementing this service.
Do not claim zero metadata leakage against an unspecified or global observer.

An actively untrusted server can withhold or delete backups even if it cannot
read them. Integrity checks and independent replicas can help; confidentiality
does not provide availability. No subscription should imply otherwise.

### The executing client is part of the trust boundary

An untrusted hosting server must not supply unchecked JavaScript that receives
passwords or decrypts documents: it could simply change the code to steal them.
Use authenticated client distribution/update paths and a trusted local browser
adapter, with explicit isolation of untrusted document code and capabilities.
Signed updates still require rollback and key-compromise policies.

Keep personal/work secrets, indexes, caches, and runtime capabilities isolated,
including in simultaneous sessions. This does not protect against a compromised
OS or an employer controlling the whole workstation; separate scopes within one
compromised machine are not a defense against its administrator or malware.

## What Changes in the Roadmap

### Now: small contracts, not a crypto project

Within M0, record a short threat model and the identity, recovery, readership,
and leakage boundaries above. Reserve approximately 1-2 hours of the existing
M0 budget for this first decision record, not for resolving anonymity or
inventing a protocol. If that exceeds the timebox, expose the decision/blocker.

M1 must keep logical identity separate from keys; signed/unsigned provenance
must be distinguishable; scope, authorization, and trust must not collapse into
one global flag. Preserve exact Dc bodies and room for versioned external
attestations. Do not introduce cryptographic placeholders that claim successful
verification or encryption.

M2 remains a local plaintext archive milestone. Stable framing must not prevent
a future established encryption wrapper, but do not build encryption, key
recovery, ratchets, or remote storage to complete it. Retain the byte-exact
filename and preservation requirements unchanged.

### Before private cloud/collaboration

Complete threat-model and specialist design review, evaluate established
protocol/library profiles, then demonstrate device add/remove/recovery,
restricted-subgroup indexing, fresh-key updates, offline membership changes,
crash/rollback handling, and authenticated identity binding. Test what revoked
members can and cannot read. Evaluate the promised traffic privacy against an
instrumented malicious server/peer rather than checking only ciphertext contents.

Choose PQ protection before transmitting data whose confidentiality must outlast
classical key exchange. Obtain review of the complete integration, not only its
primitive library. No production/team-data claim until the relevant findings
are resolved; unsupported anonymity or recovery properties remain explicit gaps.

Enterprise readiness additionally requires dependable recovery/deletion policy,
patching and incident response, dependency maintenance, access administration,
and appropriate audit evidence. Customer/regulatory requirements determine any
additional certification; "enterprise-grade" is not a cipher suite.

## Primary References

Consulted 2026-10-02; library documentation is mutable and must be rechecked when
choosing a dependency. This review did not audit any protocol implementation.

- [RFC 9420: Messaging Layer Security](https://www.rfc-editor.org/rfc/rfc9420.html),
  especially groups/epochs and Section 16.6 on FS/PCS and secret deletion.
- [RFC 9750: MLS Architecture](https://www.rfc-editor.org/rfc/rfc9750.html),
  especially authentication, delivery, and application responsibilities.
- [Signal's ratchet specification](https://signal.org/docs/specifications/doubleratchet/),
  distinguishing Double Ratchet, Sparse Post-Quantum Ratchet, and Triple Ratchet.
- [NIST FIPS 203: ML-KEM](https://csrc.nist.gov/pubs/fips/203/final) and
  [FIPS 204: ML-DSA](https://csrc.nist.gov/pubs/fips/204/final).
- [mls-rs README and security notice](https://github.com/awslabs/mls-rs/blob/main/mls-rs/README.md).
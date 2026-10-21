# Longer-Term Projects

Priorities and accepted product decisions live in the
[personal MVP roadmap](../docs/mvp-roadmap.md). These are open workflow goals
and proposals, not claims that their underlying crates are absent. Distributed
collaboration and hosting remain outside the personal MVP.

## Workspace and Graph

- [ ] Qualify local document/node creation, typed relationships, revisions,
    attestations, lookup indices, and queries through the existing storage API.
    Reuse the current backend; the old Redb suggestion is not a migration plan.
- [ ] Connect editing and node history to the effective trusted revision view,
    including valid/withdrawn attestations and immediate relationships. Preserve
    provenance rather than overwriting the original document with its latest edit.
- [ ] Support multiple local graphs and copying nodes between them, with or
    without history. The earlier proposal defaults to copying effective trusted
    state without history; reconcile with the roadmap's trust/deletion contracts.
- [ ] Qualify graph-scoped, offline-safe node identity. Do not confuse a local
    document ID with a bare Dc behind an `L` prefix, or rely on small machine-ID
    spaces that collide across offline sessions. Preserve existing global IDs;
    do not introduce a competing encoding/allocation scheme without review.
- [ ] Demonstrate useful queries and graph-driven stemming/search on a real
    sample corpus, such as a subset of MusicBrainz.
- [ ] Resolve dependencies on shared-library and private/team documents through
    explicit trust and availability policy, not ambient network access.

## Desktop and Browser Workflows

- [ ] Make source downloads and node uploads usable without native-only file
    selection. Reuse the built-in browser/selection model and stream large imports
    without unnecessary temporary-file staging.
- [ ] Qualify executable, filename, and cache paths containing non-UTF-8 native
    bytes; do not silently replace them for display and then use the changed path.
- [ ] Connect the existing runtime and renderer into a self-sufficient GUI.
    Further graphics work may use statically linked LLVM/Mesa software rendering
    for OpenGL/Vulkan. The old Linux `dlopen` shim proposal is superseded by the
    static-musl, native-Wayland/X11 invariants; it is not an approved exception.
- [ ] Evaluate macOS cross-build options, including
    [Zig's cross-compilation support](https://actually.fyi/posts/zig-makes-rust-cross-compilation-just-work/),
    under the repository's source-build and licensing constraints.
- [ ] Identify remaining browser/v86 gaps for desktop-equivalent use, including
    authenticated access to external services. A tunneling service is a design
    possibility, not an approved dependency or hosted-service commitment.

Installer, signed chunk/manifest publication, automatic updates, safe binary
replacement, and canary rollback are tracked in [installation.md](installation.md).
Implementations already exist; end-to-end qualification remains the task.

## Security and Account Experience

- [ ] Add optional, recommended TOTP at login.
- [ ] Evaluate password-strength and compromised-password checks at registration,
    login, and password change. Cover old/new passwords on change and useful
    warnings, without sending secrets to an untrusted service or assuming a full
    Pwned Passwords database must be bundled.
- [ ] Evaluate OS keychain/credential-vault integration for user-controlled
    auto-unlock, without weakening the separate-identity password boundary.
- [ ] Audit private storage permissions: Unix user-only directories (`0700`)
    and Windows ACLs restricted to the intended user SID.
- [ ] Investigate whether sensitive rendered regions can be excluded from the
    application's own screenshots and applicable Wayland capture paths. Do not
    promise protection from arbitrary screen-scraping malware.

## Collaboration and Distribution

These require the separate [security design](../docs/security-architecture.md);
authentication or a cryptography dependency alone does not qualify them.

- [ ] Design server storage and object transfer with authenticated, user/team-
    scoped data access, explicit sharing/non-resharing requirements and feasibility,
    and low-latency asynchronous collaboration. Evaluate object storage such as
    R2, indexed direct reads, and narrowly scoped single-object upload grants;
    do not route all payload bandwidth through the application server by default.
- [ ] Keep public update retrieval separate from authenticated private-data
    APIs, while retaining signature/integrity verification for public content.
- [ ] Design graph replication and signed attestations across untrusted peers,
    including per-scope effective indices and dependency availability.
- [ ] Evaluate authenticated peer discovery and transport, team/service peer
    pools, and optional update sharing. Distinguish STUN-assisted direct access
    from TURN relay costs; bound abuse, resource use, and disruptive clients even
    when accounts are authenticated.

## Additional Tools

- [ ] Define the data matching/linking tool's input, matching rules, and output
    workflow before implementation.
- [ ] Scope a ddrescue-style recovery workflow separately from ordinary CSC
    copying, including read-error recovery and retained diagnostics.




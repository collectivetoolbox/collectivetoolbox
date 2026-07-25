## This isn't secure.

- I don't know what I'm doing, so there are probably lots worse problems than these!
- Provides multiple network services; unclear if those could be unintentionally visible if the relevant ports were open
  - Some of the services are *meant* to be visible, but not all of them
- Rust doesn't zero memory when it's freed, so secrets could linger in memory
  - Possibly using a custom allocator could help? https://ianbull.com/posts/rust-custom-allocators
- Similarly, Linux doesn't zero processes' memory when they exit
- If a process gets swapped to disk, secrets could be leaked
  - mlockall tries to help but isn't sufficient (e.g. laptop suspend; running in VMs)
- Dependency issues:
  - I used an LLM for quite a bit of the code, which are well-known to have higher rate of vulnerability than humans
    - I don't know if they'd have any higher rate of vulnerability than *this* human, though. Again, I don't know what I'm doing.
  - Security-related dependencies "ring" (provides cryptography for preventing man-in-the-middle attacks, etc) and "turso" (keeps your documents private from for instance a malicious server) are described as "an experiment" and "experimental" respectively.
  - Just dependencies in general. Supply chain attacks and all that.
- Security in the technical sense does nothing with shoulder surfers.
  - Digital "shoulder surfers" like screen-scraping malware also can't really be guarded against, at least when using an external web browser to render or when using X11.
  - Software can only be as secure as the OS and machine it's running on.
- The in-development backup/sync features have the client back up to the server in the traditional way, which likely would leak metadata at the very least.
- Panics could write user data or secrets to log files.
- ...

## Where keys and similar things are used

- Requests are transferred between server and client using TLS
- Application code is signed
- Users' databases use at-rest encryption: password encrypts the DEK which encrypts the database
- The server generates accounting tokens for future paid API requests
- Session tokens are generated when users log in

## Post-Quantum Cryptography Assessment

| Task | Current algorithm(s) | Affected by post-quantum? | Options for adding new post-quantum layer (if affected) |
| :--- | :--- | :--- | :--- |
| **Network traffic over TLS** (Client-Server connection - relevant to both requests going out from the application (the "HTTPS client") and to the Axum server) | TLS 1.2 / 1.3 via `rustls` (configured with `ring` provider). Key exchange uses ECDHE (e.g., X25519 or P-256); authentication uses ECDSA, RSA, or Ed25519. | **Yes** (Key exchange & signatures are vulnerable to Shor's algorithm). | Implement hybrid key exchange combining a classical algorithm (like X25519) with a post-quantum Key Encapsulation Mechanism (KEM) such as **ML-KEM-768** (Kyber). Migrate signatures to **ML-DSA** (Dilithium) or Falcon, or use hybrid signatures. |
| **Application code signing** (Release verification) | Ed25519 (via `ed25519_dalek` and `SHA-256`). | **Yes** (Ed25519 signatures are vulnerable to Shor's algorithm). SHA-256 is quantum-safe. | Transition to a post-quantum signature scheme like **ML-DSA** (Dilithium), **Falcon**, or a stateful hash-based signature scheme like **XMSS** or **LMS/HSS** (which are standard for code/firmware signing). Alternatively, implement a hybrid signature scheme (Ed25519 + ML-DSA). |
| **Users' databases at-rest encryption** (Local storage) | **Key Derivation (KEK):** Argon2id (via `argon2`).<br>**DEK Generation:** Concatenation of two random UUID v4s (stub).<br>**DEK Wrapping/Sealing:** None (no-op stub returning the raw DEK/plaintext).<br>**Database Cipher:** Aegis-256 (via `turso_sdk_kit`). | **No** (Symmetric cryptography and password hashing are not vulnerable to Shor's algorithm; Grover's algorithm only reduces effective symmetric security, leaving Aegis-256 and Argon2id highly secure). | Not affected by quantum threats. However, it has major classical vulnerabilities: DEK wrapping/sealing is a no-op stub, and DEK generation uses UUID v4. To secure this classically and quantum-resistantly, implement DEK generation with a CSPRNG (`rand::rng()`) and wrap/seal DEKs using a standard symmetric AEAD like **AES-256-GCM** or **ChaCha20-Poly1305**. **Update**: Filled these pieces in using LLM (Aes256Gcm). Haven't checked them for correctness. |
| **Accounting/Billing tokens** (Sync API requests) | Blind signatures using VOPRF (RFC 9497) over the Ristretto255 curve (via `voprf` crate). | **Yes** (Elliptic-curve-based VOPRF relies on discrete logarithms, which are vulnerable to Shor's algorithm). | Use a post-quantum blind signature or VOPRF scheme (e.g., lattice-based schemes like LaV). Alternatively, migrate to or wrap with a symmetric-key protocol (like HMAC-based tokens or keyed-hashes) if blind signatures are not strictly required. |
| **Session tokens** (User authentication login/session) | Cryptographically secure random 32-byte session tokens generated using `rand::rng()` (URL-safe base64 encoded). | **No** (Session tokens are random values checked against a server-side store; they do not use public-key cryptography and are quantum-safe). | Not affected. No changes needed. |

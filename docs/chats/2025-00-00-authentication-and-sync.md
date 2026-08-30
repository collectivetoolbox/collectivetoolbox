
### 🔐 Authentication: Password-Derived Key + SRP or PAKE

Use a **Password Authenticated Key Exchange (PAKE)** protocol like **SRP (Secure Remote Password)** or **OPAQUE**:

- The password is never sent to the server.
- The server stores a verifier, not the actual password.
- Authentication happens via a cryptographic exchange that proves knowledge of the password without revealing it.

---

### 🧳 Syncing from New Device

When the user logs in from a new device:

1. They enter their username and password.
2. The password is used to derive a master encryption key using a KDF like Argon2 or PBKDF2.
3. This key decrypts their data stored on the server.
4. The vault contains:
    - File encryption keys
    - Metadata
    - Possibly even device-specific sync tokens

Because the vault is encrypted with a key derived from the password, the server can’t read it. And since the vault is synced, the user gets access from any device.

---

### 🧪 Optional Enhancements

- **Yubikey or WebAuthn**: You could add this as a second factor, but not required for basic login.
- **Local key caching**: Once decrypted, keys can be cached locally for performance.
- **Forward secrecy**: Use per-file or per-session keys to limit blast radius of compromise.

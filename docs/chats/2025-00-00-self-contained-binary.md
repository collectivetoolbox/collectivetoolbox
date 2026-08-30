To ensure your Rust binary is 100% self-contained and compatible with legacy systems like

**Damn Small Linux (2003)**, you must avoid the default `glibc` linking. Modern `glibc` is notoriously difficult to link fully statically and often expects a specific kernel version. 

1\. Building a Fully Static Binary

The most reliable method for creating a "run-anywhere" Linux binary in Rust is to target **musl libc**, which is designed for static linking. 

*   **Add the musl target:**
    
    bash
    
    ```
    rustup target add x86_64-unknown-linux-musl
    ```
    
    Use code with caution.
    

*   **Compile your project:**
    
    bash
    
    ```
    cargo build --release --target x86_64-unknown-linux-musl
    ```
    
    Use code with caution.
    

*   **Static glibc (Alternative):** If you must use `glibc`, you can attempt to force static linking (provided `glibc-static` is installed on your host) using:  
    `RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu`. 

2\. Auditing the Compiled Binary

To verify that your binary does not depend on any external shared libraries (`.so` files), use these audit tools:

*   **`ldd` (Quick Check):**  
    Run `ldd <path-to-binary>`. A fully static binary will return **"not a dynamic executable"**. If it lists libraries like `libc.so.6`, it is still dynamically linked.
*   **`readelf` (Detailed Audit):**  
    Use this command to see if any libraries are marked as "NEEDED":
    
    bash
    
    ```
    readelf -d <path-to-binary> | grep NEEDED
    ```
    
    Use code with caution.
    

*   If this command returns **no output**, the binary has no dynamic dependencies.
*   **`file` (Metadata Check):**  
    Run `file <path-to-binary>`. Look for the string **"statically linked"** in the output. 

3\. Critical Compatibility Warning

Even if a binary is "statically linked," it still makes **system calls** to the Linux kernel. A modern Rust binary (compiled in 2026) might use syscalls or features that didn't exist in the **2.4.x kernel** used by Damn Small Linux in 2003. To maximize compatibility with such old systems: 

*   Ensure your dependencies do not use modern hardware features (like AVX instructions) or new kernel APIs (like `io_uring`).
*   Verify if your target architecture (e.g., 32-bit vs 64-bit) matches the legacy hardware. 

Would you like to see how to **verify if your binary uses specific modern kernel syscalls** that might break on a 2003-era Linux kernel?

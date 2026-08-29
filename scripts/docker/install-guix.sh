#!/usr/bin/env bash
#
# install-guix.sh: Install GNU Guix using pre-bundled keys to avoid network timeouts.
#

set -euo pipefail
IFS=$'\n\t'

KEYS_DIR="${1:-/tmp/guix-keys}"

echo "=== Installing GNU Guix ==="

# 1. Import pre-extracted OpenPGP signing keys into GPG keyring
# This satisfies `chk_gpg_keyring` in guix-install.sh so it does not query remote keyservers.
if [[ -d "${KEYS_DIR}/openpgp" ]]; then
    echo "Importing OpenPGP signing keys from ${KEYS_DIR}/openpgp..."
    for key_file in "${KEYS_DIR}/openpgp"/*; do
        if [[ -f "${key_file}" ]]; then
            echo "  Importing $(basename "${key_file}")..."
            gpg --batch --import "${key_file}" || true
        fi
    done
fi

# 2. Download and execute upstream guix-install.sh
INSTALL_SCRIPT="/tmp/guix-install.sh"
echo "Downloading guix-install.sh..."
curl -fsSL https://codeberg.org/guix/guix/raw/branch/master/etc/guix-install.sh -o "${INSTALL_SCRIPT}"
chmod +x "${INSTALL_SCRIPT}"

# 3. Find a fast, reachable GNU Guix binary mirror
# Default to ftp.gnu.org and fall back to alternative mirrors if unreachable or timed out
CANDIDATE_MIRRORS=(
    "https://ftp.gnu.org/gnu/guix/"
    "https://mirror.dogado.de/gnu/guix/"
    "https://mirror.accum.se/mirror/gnu.org/gnu/guix/"
    "https://mirror.freedif.org/GNU/guix/"
    "https://ftpmirror.gnu.org/gnu/guix/"
)

WORKING_MIRROR=""
echo "Selecting responsive GNU Guix mirror..."
for mirror in "${CANDIDATE_MIRRORS[@]}"; do
    if curl -sSL --connect-timeout 3 -m 5 -I "${mirror}" >/dev/null 2>&1; then
        echo "  Selected mirror: ${mirror}"
        WORKING_MIRROR="${mirror}"
        break
    fi
done

if [[ -z "${WORKING_MIRROR}" ]]; then
    echo "Warning: No candidate mirror passed quick check, falling back to ftpmirror.gnu.org"
    WORKING_MIRROR="https://ftpmirror.gnu.org/gnu/guix/"
fi

# Apply necessary patches for installation in container environment
sed -i "s|https://ftpmirror.gnu.org/gnu/guix/|${WORKING_MIRROR}|g" "${INSTALL_SCRIPT}"
sed -i 's/sys_maybe_setup_apparmor/true/g' "${INSTALL_SCRIPT}"

echo "Running guix-install.sh..."
# Temporarily disable pipefail so SIGPIPE on 'yes' (exit code 141) doesn't fail the pipeline
# after guix-install.sh completes and closes stdin.
set +o pipefail
yes '' | "${INSTALL_SCRIPT}"
set -o pipefail

if [[ -f "${INSTALL_SCRIPT}" ]]; then
    rm "${INSTALL_SCRIPT}"
fi

# 3. Clean up AppArmor profiles if present (unsupported / not needed in Docker)
for apparmor_file in /etc/apparmor.d/guix* /etc/apparmor.d/tunables/guix*; do
    if [[ -e "${apparmor_file}" ]]; then
        rm -r "${apparmor_file}"
    fi
done

# 4. Ensure required directories and build permissions
mkdir -p /etc/guix /var/log/guix/drvs
chown -R root:guixbuild /var/guix /var/log/guix /gnu/store
chmod 1775 /gnu/store /var/guix
chmod -R 1777 /var/log/guix

# 5. Restore ACL configuration if provided
if [[ -f "${KEYS_DIR}/acl" ]]; then
    echo "Restoring /etc/guix/acl..."
    cat "${KEYS_DIR}/acl" > /etc/guix/acl
fi

# 6. Authorize substitute server public keys
echo "Authorizing substitute server public keys..."
SEARCH_KEY_DIRS=(
    "/var/guix/profiles/per-user/root/current-profile/share/guix"
    "/root/.config/guix/current/share/guix"
    "/usr/local/share/guix"
    "${KEYS_DIR}/substitutes"
)

for dir in "${SEARCH_KEY_DIRS[@]}"; do
    if [[ -d "${dir}" ]]; then
        for pub in "${dir}"/*.pub; do
            if [[ -f "${pub}" ]]; then
                echo "  Authorizing $(basename "${pub}") from ${dir}..."
                guix archive --authorize < "${pub}" || true
            fi
        done
    fi
done

# 7. Clean up temporary keys directory if located in /tmp
if [[ "${KEYS_DIR}" == /tmp/* && -d "${KEYS_DIR}" ]]; then
    echo "Cleaning up temporary keys in ${KEYS_DIR}..."
    rm -r "${KEYS_DIR}"
fi

echo "=== GNU Guix installation completed successfully ==="

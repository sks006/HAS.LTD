#!/usr/bin/env bash
# scripts/init_secrets.sh
# Initializes local cryptographic key material for HAS.LTD platform.
# Enforces strict directory permissions and idempotency guards.

set -euo pipefail

SECRETS_DIR=".secrets"
JWT_DIR="${SECRETS_DIR}/jwt"
TOTP_KEY="${SECRETS_DIR}/totp.key"
GITIGNORE=".gitignore"

echo "=== HAS.LTD Cryptographic Secrets Initialization ==="

# 1. Refuse to run if .secrets/ directory already exists
if [ -d "${SECRETS_DIR}" ]; then
    echo "ERROR: Directory '${SECRETS_DIR}' already exists. Refusing to run to prevent accidental key overwrite." >&2
    exit 1
fi

# 2. Append .secrets/ to .gitignore BEFORE creating the directory
if [ -f "${GITIGNORE}" ]; then
    if ! grep -qxF "${SECRETS_DIR}/" "${GITIGNORE}" 2>/dev/null; then
        echo "Updating ${GITIGNORE} to ignore ${SECRETS_DIR}/..."
        echo "${SECRETS_DIR}/" >> "${GITIGNORE}"
    fi
else
    echo "Creating ${GITIGNORE} with ${SECRETS_DIR}/..."
    echo "${SECRETS_DIR}/" > "${GITIGNORE}"
fi

# 3. Create .secrets/ and .secrets/jwt/ with 0700 permissions
mkdir -p -m 0700 "${JWT_DIR}"
chmod 0700 "${SECRETS_DIR}"
chmod 0700 "${JWT_DIR}"

# 4. Generate RS256 keypairs using openssl
echo "Generating RS256 keypairs (2048-bit PKCS#8 private keys, SPKI public keys)..."

# Keypair 1: Retained / Previous verification key
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "${JWT_DIR}/1.pem" 2>/dev/null
openssl pkey -in "${JWT_DIR}/1.pem" -pubout -out "${JWT_DIR}/1.pub.pem" 2>/dev/null

# Keypair 2: Active signing key (highest kid selected deterministically)
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "${JWT_DIR}/2.pem" 2>/dev/null
openssl pkey -in "${JWT_DIR}/2.pem" -pubout -out "${JWT_DIR}/2.pub.pem" 2>/dev/null

# 5. Generate 32-byte master key for AES-256-GCM TOTP secret encryption at rest
echo "Generating 32-byte master key for TOTP encryption at rest..."
openssl rand -out "${TOTP_KEY}" 32

# 6. Apply strict file permissions
chmod 0600 "${JWT_DIR}/1.pem"
chmod 0644 "${JWT_DIR}/1.pub.pem"
chmod 0600 "${JWT_DIR}/2.pem"
chmod 0644 "${JWT_DIR}/2.pub.pem"
chmod 0600 "${TOTP_KEY}"

echo ""
echo "=== Secrets Successfully Initialized ==="
echo "Directory layout:"
echo "  ${SECRETS_DIR}/              [0700]"
echo "  ├── jwt/             [0700]"
echo "  │   ├── 1.pem        [0600] (RS256 PKCS#8 private key, kid=1)"
echo "  │   ├── 1.pub.pem    [0644] (RS256 SPKI public key, kid=1)"
echo "  │   ├── 2.pem        [0600] (RS256 PKCS#8 private key, kid=2, ACTIVE SIGNER)"
echo "  │   └── 2.pub.pem    [0644] (RS256 SPKI public key, kid=2)"
echo "  └── totp.key         [0600] (32 random bytes AES-256-GCM master key)"
echo ""
echo "Loaded Key IDs (KID list):"
echo "  - kid=1: Retained verification keypair"
echo "  - kid=2: Active signing keypair (highest numeric kid selected deterministically)"
echo ""
echo "SECURITY NOTICE:"
echo "  Ensure '${SECRETS_DIR}/' is backed up securely to offline vault / KMS before production use."
echo "  Never commit private keys or '${TOTP_KEY}' to version control."

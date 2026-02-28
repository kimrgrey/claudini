#!/bin/sh
set -eu

REPO="kimrgrey/claudini"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

arch=$(uname -m)
case "$arch" in
  arm64)  target="aarch64-apple-darwin" ;;
  x86_64) target="x86_64-apple-darwin" ;;
  *)
    echo "Error: unsupported architecture: $arch" >&2
    exit 1
    ;;
esac

echo "Detected architecture: $arch ($target)"

tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')
if [ -z "$tag" ]; then
  echo "Error: could not determine latest release" >&2
  exit 1
fi

echo "Latest release: $tag"

url="https://github.com/${REPO}/releases/download/${tag}/claudini-${target}"
echo "Downloading ${url}..."
curl -fsSL -o claudini "$url"
chmod +x claudini

echo "Installing to ${INSTALL_DIR}/claudini..."
if [ -w "$INSTALL_DIR" ]; then
  mv claudini "${INSTALL_DIR}/claudini"
else
  sudo mv claudini "${INSTALL_DIR}/claudini"
fi

echo "claudini ${tag} installed successfully"

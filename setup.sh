#!/usr/bin/env bash
set -euo pipefail

USER='foxyon'
APP_DIR="/usr/local/bin"
BIN_PATH="${APP_DIR}/foxyon"
BUILD='RUSTFLAGS="-Ctarget-cpu=native" cargo build --release'
SERVICE_PATH="/etc/systemd/system/foxyon.service"
SERVICE_FILE="systemd/foxyon.service"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[34m'
NC='\033[0m'

build() {
    if command -v cargo &>/dev/null; then
      echo -e "${BLUE}[i] Building foxyon...${NC}"
      eval "${BUILD}"
    else
      echo -e "${RED}[-] Cargo not found!${NC}"
      exit 1
    fi

    if [ ! -f "target/release/foxyon" ]; then
      echo -e "${RED}[-] Binary not found at target/release/foxyon.${NC}"
      exit 1
    fi

    echo -e "${GREEN}[+] Build completed!${NC}"
}

install_fox() {
  if [ "${EUID}" -ne 0 ]; then
    echo -e "${RED}[-] This script must be run as root.${NC}"
    exit 1
  fi

  echo -e "${BLUE}[i] Starting Foxyon installation!${NC}"

  if ! id "${USER}" &>/dev/null; then
    useradd -r -M -s /usr/sbin/nologin "${USER}"
    echo -e "${GREEN}[+] User '${USER}' created.${NC}"
  else
    echo -e "${BLUE}[i] User '${USER}' already exists.${NC}"
  fi

  if [ ! -f "target/release/foxyon" ]; then
    echo -e "${RED}[-] Binary not found at target/release/foxyon. Did you run the ‘build’?${NC}"
    exit 1
  fi

  install -Dm755 target/release/foxyon "${BIN_PATH}"

  chown root:root "${BIN_PATH}"
  echo -e "${GREEN}[+] Installed binary to ${BIN_PATH}${NC}"

  if [ -f "${SERVICE_FILE}" ]; then
    install -Dm644 "${SERVICE_FILE}" "${SERVICE_PATH}"
    systemctl daemon-reload
    systemctl enable foxyon.service
    systemctl restart foxyon.service || systemctl start foxyon.service
    echo -e "${GREEN}[+] Installed and started systemd unit${NC}"
  else
    echo -e "${RED} [-] Failure to install the systemd service${NC}"
    exit 1
  fi

  echo -e "${GREEN}[🦊] Foxyon installation complete.${NC}"
}

case "${1:-}" in
    "build")
        build
        ;;
    "install")
        install_fox
        ;;
esac
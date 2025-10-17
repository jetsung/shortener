#!/bin/bash
# Installation script for Shortener Server systemd service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root${NC}"
    exit 1
fi

echo -e "${GREEN}Installing Shortener Server systemd service...${NC}"

# Configuration
BINARY_PATH="/usr/local/bin/shortener-server"
CONFIG_DIR="/etc/shortener"
DATA_DIR="/var/lib/shortener"
LOG_DIR="/var/log/shortener"
SERVICE_FILE="shortener-server.service"
SERVICE_PATH="/etc/systemd/system/${SERVICE_FILE}"

# Create user if not exists
if ! id -u shortener > /dev/null 2>&1; then
    echo -e "${YELLOW}Creating shortener user...${NC}"
    useradd -r -s /bin/false -d ${DATA_DIR} shortener
else
    echo -e "${GREEN}User shortener already exists${NC}"
fi

# Create directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p ${CONFIG_DIR}
mkdir -p ${DATA_DIR}
mkdir -p ${LOG_DIR}
mkdir -p /opt/shortener

# Set ownership
chown -R shortener:shortener ${DATA_DIR}
chown -R shortener:shortener ${LOG_DIR}
chown -R shortener:shortener /opt/shortener

# Copy binary if provided
if [ -f "../../target/release/shortener-server" ]; then
    echo -e "${YELLOW}Copying binary...${NC}"
    cp ../../target/release/shortener-server ${BINARY_PATH}
    chmod +x ${BINARY_PATH}
    chown root:root ${BINARY_PATH}
else
    echo -e "${YELLOW}Warning: Binary not found at ../../target/release/shortener-server${NC}"
    echo -e "${YELLOW}Please copy the binary manually to ${BINARY_PATH}${NC}"
fi

# Copy configuration if provided
if [ -f "../../config/config.toml" ]; then
    if [ ! -f "${CONFIG_DIR}/config.toml" ]; then
        echo -e "${YELLOW}Copying configuration...${NC}"
        cp ../../config/config.toml ${CONFIG_DIR}/config.toml
        chown root:root ${CONFIG_DIR}/config.toml
        chmod 644 ${CONFIG_DIR}/config.toml
    else
        echo -e "${GREEN}Configuration already exists, skipping${NC}"
    fi
fi

# Copy GeoIP database if provided
if [ -f "../../data/ip2region.xdb" ]; then
    echo -e "${YELLOW}Copying GeoIP database...${NC}"
    cp ../../data/ip2region.xdb ${DATA_DIR}/ip2region.xdb
    chown shortener:shortener ${DATA_DIR}/ip2region.xdb
    chmod 644 ${DATA_DIR}/ip2region.xdb
fi

# Install systemd service
echo -e "${YELLOW}Installing systemd service...${NC}"
cp ${SERVICE_FILE} ${SERVICE_PATH}
chmod 644 ${SERVICE_PATH}

# Reload systemd
echo -e "${YELLOW}Reloading systemd...${NC}"
systemctl daemon-reload

# Enable service
echo -e "${YELLOW}Enabling service...${NC}"
systemctl enable ${SERVICE_FILE}

echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo -e "${GREEN}Next steps:${NC}"
echo -e "  1. Edit configuration: ${CONFIG_DIR}/config.toml"
echo -e "  2. Start service: systemctl start ${SERVICE_FILE}"
echo -e "  3. Check status: systemctl status ${SERVICE_FILE}"
echo -e "  4. View logs: journalctl -u ${SERVICE_FILE} -f"
echo ""
echo -e "${YELLOW}Important files:${NC}"
echo -e "  Binary: ${BINARY_PATH}"
echo -e "  Config: ${CONFIG_DIR}/config.toml"
echo -e "  Data: ${DATA_DIR}"
echo -e "  Logs: ${LOG_DIR}"
echo -e "  Service: ${SERVICE_PATH}"

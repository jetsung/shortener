#!/bin/bash
# Uninstallation script for Shortener Server systemd service

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

echo -e "${YELLOW}Uninstalling Shortener Server systemd service...${NC}"

# Configuration
SERVICE_FILE="shortener-server.service"
SERVICE_PATH="/etc/systemd/system/${SERVICE_FILE}"

# Stop service if running
if systemctl is-active --quiet ${SERVICE_FILE}; then
    echo -e "${YELLOW}Stopping service...${NC}"
    systemctl stop ${SERVICE_FILE}
fi

# Disable service
if systemctl is-enabled --quiet ${SERVICE_FILE}; then
    echo -e "${YELLOW}Disabling service...${NC}"
    systemctl disable ${SERVICE_FILE}
fi

# Remove service file
if [ -f "${SERVICE_PATH}" ]; then
    echo -e "${YELLOW}Removing service file...${NC}"
    rm -f ${SERVICE_PATH}
fi

# Reload systemd
echo -e "${YELLOW}Reloading systemd...${NC}"
systemctl daemon-reload
systemctl reset-failed

echo -e "${GREEN}Service uninstalled successfully!${NC}"
echo ""
echo -e "${YELLOW}Note: The following were NOT removed:${NC}"
echo -e "  - Binary: /usr/local/bin/shortener-server"
echo -e "  - Config: /etc/shortener/"
echo -e "  - Data: /var/lib/shortener/"
echo -e "  - Logs: /var/log/shortener/"
echo -e "  - User: shortener"
echo ""
echo -e "${YELLOW}To completely remove all files, run:${NC}"
echo -e "  rm -rf /usr/local/bin/shortener-server"
echo -e "  rm -rf /etc/shortener"
echo -e "  rm -rf /var/lib/shortener"
echo -e "  rm -rf /var/log/shortener"
echo -e "  userdel shortener"

#!/bin/bash

# Define ANSI color codes
BLUE='\033[0;34m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Clear the terminal screen
clear

# --- Configuration ---
REPO_API_URL="https://api.github.com/repos/zyit0000/PotassiumUi/releases/tags/Release"
APP_NAME="Potassium.app"
APPLICATIONS_DIR="/Applications"

echo -e "${BLUE}Starting Potassium installation...${NC}"

# Header UI
echo -e "${CYAN}=========================================${NC}"
echo -e "${CYAN}             Potassium                   ${NC}"
echo -e "${CYAN}=========================================${NC}"
echo ""

# 0. Check for existing installation
if [ -d "$APPLICATIONS_DIR/$APP_NAME" ]; then
    echo -e "${YELLOW}[?] Potassium is already installed.${NC}"
    read -p "Overwrite existing application? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}[-] Installation cancelled.${NC}"
        exit 0
    fi
    sudo rm -rf "$APPLICATIONS_DIR/$APP_NAME"
fi

# 1. Fetch Release Info
echo -e "${CYAN}[i] Checking for DMG in 'Release' tag...${NC}"
RELEASE_DATA=$(curl -s "$REPO_API_URL")
DOWNLOAD_URL=$(echo "$RELEASE_DATA" | grep -o 'https://[^"]*\.dmg' | head -n 1)

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}[!] Error: Could not find a .dmg file in the 'Release' tag.${NC}"
    exit 1
fi

DMG_FILENAME=$(basename "$DOWNLOAD_URL")
echo -e "${GREEN}[✓] Found: $DMG_FILENAME${NC}"

# 2. Download
echo -e "${CYAN}[i] Downloading installer...${NC}"
curl -L -s "$DOWNLOAD_URL" -o "$DMG_FILENAME"

if [ $? -ne 0 ]; then
    echo -e "${RED}[!] Error: Failed to download.${NC}"
    exit 1
fi
echo -e "${GREEN}[✓] Download complete.${NC}"

# 3. Mount
echo -e "${CYAN}[i] Mounting disk image...${NC}"
MOUNT_POINT=$(mktemp -d)
hdiutil attach "$DMG_FILENAME" -nobrowse -quiet -mountpoint "$MOUNT_POINT"

if [ $? -ne 0 ]; then
    echo -e "${RED}[!] Error: Failed to mount.${NC}"
    rm -f "$DMG_FILENAME"
    exit 1
fi
echo -e "${GREEN}[✓] DMG mounted successfully.${NC}"

# 4. Locate App
APP_PATH_IN_DMG=$(find "$MOUNT_POINT" -maxdepth 1 -type d -name "*.app" | head -n 1)

if [ -z "$APP_PATH_IN_DMG" ]; then
    echo -e "${RED}[!] Error: No .app found inside DMG.${NC}"
    hdiutil detach "$MOUNT_POINT" -force > /dev/null 2>&1
    exit 1
fi

# 5. Install
echo -e "${CYAN}[i] Installing to $APPLICATIONS_DIR...${NC}"
sudo cp -R "$APP_PATH_IN_DMG" "$APPLICATIONS_DIR/"

if [ $? -ne 0 ]; then
    echo -e "${RED}[!] Error: Copy failed.${NC}"
    hdiutil detach "$MOUNT_POINT" -force > /dev/null 2>&1
    exit 1
fi
echo -e "${GREEN}[✓] Application installed successfully.${NC}"

# 6. Cleanup
echo -e "${CYAN}[i] Cleaning up...${NC}"
hdiutil detach "$MOUNT_POINT" -force > /dev/null 2>&1
rm -rf "$MOUNT_POINT"
rm -f "$DMG_FILENAME"

echo -e "${GREEN}[✓] Setup complete! Potassium is ready.${NC}"
echo ""
echo -e "${CYAN}Ui by 7sleeps${NC}"
echo -e "${CYAN}Backend by ZYiT0${NC}"

exit 0
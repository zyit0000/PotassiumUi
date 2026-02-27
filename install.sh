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
# Targeting the specific 'Release' tag
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
    # Remove old version to ensure a clean install
    sudo rm -rf "$APPLICATIONS_DIR/$APP_NAME"
fi

# 1. Fetch Release Info and Find DMG
echo -e "${CYAN}[i] Checking for DMG in 'Release' tag...${NC}"

# API call to get release data
RELEASE_DATA=$(curl -s "$REPO_API_URL")

# Extract the browser_download_url for the asset ending in .dmg
DOWNLOAD_URL=$(echo "$RELEASE_DATA" | grep -o 'https://[^"]*\.dmg' | head -n 1)

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}[!] Error: Could not find a .dmg file in the 'Release' tag.${NC}"
    echo -e "${YELLOW}[?] Please ensure a .dmg is uploaded to the 'Release' tag assets.${NC}"
    exit 1
fi

DMG_FILENAME=$(basename "$DOWNLOAD_URL")
echo -e "${GREEN}[<checkmark>] Found: $DMG_FILENAME${NC}"

# 2. Download the DMG
echo -e "${CYAN}[i] Downloading installer...${NC}"
curl -L -s "$DOWNLOAD_URL" -o "$DMG_FILENAME"

if [ $? -ne 0 ]; then
    echo -e "${RED}[!] Error: Failed to download the DMG.${NC}"
    exit 1
fi
echo -e "${GREEN}[<checkmark>] Download complete.${NC}"

# 3. Mount the DMG
echo -e "${CYAN}[i] Mounting disk image...${NC}"
MOUNT_POINT=$(mktemp -d)
hdiutil attach "$DMG_FILENAME" -nobrowse -quiet -mountpoint "$MOUNT_POINT"

if [ $? -ne 0 ]; then
    echo -e "${RED}[!] Error: Failed to mount the DMG.${NC}"
    rm -f "$DMG_FILENAME"
    exit 1
fi
echo -e "${GREEN}[<checkmark>] DMG mounted successfully.${NC}"

# 4. Locate .app bundle inside DMG
APP_PATH_IN_DMG=$(find "$MOUNT_POINT" -maxdepth 1 -type d -name "*.app" | head -n 1)

if [ -z "$APP_PATH_IN_DMG" ]; then
    echo -e "${RED}[!] Error: No .app bundle found inside the DMG.${NC}"
    hdiutil detach "$MOUNT_POINT" -force > /dev/null 2>&1
    rm -rf "$MOUNT_POINT"
    rm -f "$DMG_FILENAME"
    exit 1
fi

# 5. Copy to Applications
echo -e "${CYAN}[i] Installing to $APPLICATIONS_DIR...${NC}"
sudo cp -R "$APP_PATH_IN_DMG" "$APPLICATIONS_DIR/"

if [ $? -ne 0 ]; then
    echo -e "${RED}[!] Error: Failed to copy files. Check sudo permissions.${NC}"
    hdiutil detach "$MOUNT_POINT" -force > /dev/null 2>&1
    exit 1
fi
echo -e "${GREEN}[<checkmark>] Application installed successfully.${NC}"

# 6. Cleanup
echo -e "${CYAN}[i] Cleaning up...${NC}"
hdiutil detach "$MOUNT_POINT" -force > /dev/null 2>&1
rm -rf "$MOUNT_POINT"
rm -f "$DMG_FILENAME"

echo -e "${GREEN}[<checkmark>] Setup complete! Potassium is ready.${NC}"
echo ""
echo -e "${CYAN}Ui by 7sleeps${NC}"
echo -e "${CYAN}Backend by ZYiT0${NC}"

exit 0
#!/bin/bash

# Script to insert CAMERA permission after INTERNET permission in AndroidManifest.xml

MANIFEST_FILE="src-tauri/gen/android/app/src/main/AndroidManifest.xml"

# Check if manifest file exists
if [ ! -f "$MANIFEST_FILE" ]; then
    echo "Error: $MANIFEST_FILE not found!"
    exit 1
fi

# Check if INTERNET permission exists
if ! grep -q 'android.permission.INTERNET' "$MANIFEST_FILE"; then
    echo "Error: INTERNET permission not found in $MANIFEST_FILE"
    exit 1
fi

# Check if CAMERA permission already exists
if grep -q 'android.permission.CAMERA' "$MANIFEST_FILE"; then
    echo "CAMERA permission already exists in $MANIFEST_FILE"
    exit 0
fi

# Create backup
cp "$MANIFEST_FILE" "${MANIFEST_FILE}.backup"
echo "Created backup: ${MANIFEST_FILE}.backup"

# Insert CAMERA permission after INTERNET permission
sed -i '' '/android.permission.INTERNET/a\
    <uses-permission android:name="android.permission.CAMERA" />
' "$MANIFEST_FILE"

# Verify the insertion was successful
if grep -q 'android.permission.CAMERA' "$MANIFEST_FILE"; then
    echo "Successfully inserted CAMERA permission after INTERNET permission"
else
    echo "Error: Failed to insert CAMERA permission"
    # Restore backup
    mv "${MANIFEST_FILE}.backup" "$MANIFEST_FILE"
    echo "Restored original file from backup"
    exit 1
fi

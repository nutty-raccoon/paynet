#!/bin/bash

# Check if correct number of arguments provided
if [ $# -ne 3 ]; then
    echo "Usage: $0 <storeFilePath> <password> <keyAlias>"
    echo "Example: $0 /path/to/keystore.jks mypassword mykey"
    exit 1
fi

# Assign arguments to variables
STORE_FILE_PATH="$1"
PASSWORD="$2"
KEY_ALIAS="$3"

MANIFEST_FILE="src-tauri/gen/android/app/src/main/AndroidManifest.xml"
KEYSTORE_PROPERTIES_FILE="src-tauri/gen/android/keystore.properties"

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
else
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
fi

# Create keystore.properties file
echo "Creating keystore.properties file at $KEYSTORE_PROPERTIES_FILE"

# Ensure the directory exists
mkdir -p "$(dirname "$KEYSTORE_PROPERTIES_FILE")"

# Write keystore properties
cat > "$KEYSTORE_PROPERTIES_FILE" << EOF
password=$PASSWORD
keyAlias=$KEY_ALIAS
storeFile=$STORE_FILE_PATH
EOF

# Verify the keystore.properties file was created successfully
if [ -f "$KEYSTORE_PROPERTIES_FILE" ]; then
    echo "Successfully created keystore.properties file"
else
    echo "Error: Failed to create keystore.properties file"
    exit 1
fi

echo "Script completed successfully!"

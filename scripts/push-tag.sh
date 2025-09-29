#!/bin/bash

# Check if correct number of arguments provided
if [ $# -ne 3 ]; then
    echo "Usage: $0 <add/remove> <service-name> <version-number>"
    echo "Example: $0 add my-service v1.2.3"
    exit 1
fi

ACTION=$1
SERVICE=$2
VERSION=$3
TAG_NAME="$SERVICE-$VERSION"

case $ACTION in
    "add")
        echo "Adding tag: $TAG_NAME"
        git tag "$TAG_NAME"
        if [ $? -eq 0 ]; then
            echo "Pushing tag to origin..."
            git push origin "$TAG_NAME" --no-verify
            if [ $? -eq 0 ]; then
                echo "Tag $TAG_NAME successfully added and pushed!"
            else
                echo "Error: Failed to push tag to origin"
                exit 1
            fi
        else
            echo "Error: Failed to create tag"
            exit 1
        fi
        ;;
    "remove")
        echo "Removing tag: $TAG_NAME"
        # Remove local tag
        git tag -d "$TAG_NAME"
        if [ $? -eq 0 ]; then
            echo "Local tag $TAG_NAME successfully removed!"
        else
            echo "Warning: Failed to delete local tag (it might not exist)"
        fi

        # Remove remote tag first
        git push origin --delete "$TAG_NAME" --no-verify
        if [ $? -eq 0 ]; then
            echo "Remote tag deleted successfully"
        else
            echo "Warning: Failed to delete remote tag (it might not exist)"
        fi       
        ;;
    *)
        echo "Error: Invalid action '$ACTION'. Use 'add' or 'remove'"
        echo "Usage: $0 <add/remove> <service-name> <version-number>"
        exit 1
        ;;
esac

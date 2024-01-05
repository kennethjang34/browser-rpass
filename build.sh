#!/bin/bash

rm -rf ./pkg
mkdir pkg
(cd ./native-client && cargo build --release)
(cd ./service-worker && source ./build.sh)
(cd ./content && source ./build.sh)
(cd ./popup && source ./build.sh)
mv ./service-worker/pkg/* ./pkg
mv ./popup/pkg/* ./pkg
mv ./content/pkg/* ./pkg
cp ./manifest_v3.json ./pkg/manifest.json

echo "Which browser version do you want? (chrome/chromium)"
read browser

echo "Enter your extension ID:"
read extension_id

manifest_path=""
manifest='
{
    "name": "rpass",
    "description": "rpass",
    "path": "'$HOME'/rpass/browser-rpass/target/release/native-client",
    "type": "stdio",
    "allowed_origins": ["chrome-extension://'$extension_id'/"]
}
'

if [ "$browser" = "chrome" ]; then
    if [ "$(uname)" = "Darwin" ]; then
        manifest_path="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts/com.rpass.json"
    elif [ "$(uname)" = "Linux" ]; then
        manifest_path="$HOME/.config/google-chrome/NativeMessagingHosts/com.rpass.json"
    else
        echo "Unsupported platform"
        exit 1
    fi
elif [ "$browser" = "chromium" ]; then
    if [ "$(uname)" = "Darwin" ]; then
        manifest_path="$HOME/Library/Application Support/Chromium/NativeMessagingHosts/com.rpass.json"
    elif [ "$(uname)" = "Linux" ]; then
        manifest_path="$HOME/.config/chromium/NativeMessagingHosts/com.rpass.json"
    else
        echo "Unsupported platform"
        exit 1
    fi
else
    echo "Unsupported browser"
    exit 1
fi

echo "Manifest path: $manifest_path"

touch "$manifest_path"
echo "$manifest" > "$manifest_path"





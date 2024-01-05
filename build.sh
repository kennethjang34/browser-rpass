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

cp ./manifest_v3_chrome.json ./pkg/manifest_v3_chrome.json
cp ./manifest_v3_firefox.json ./pkg/manifest_v3_firefox.json

echo "Which browser version do you want? (chrome/firefox)"
read browser


if [ "$browser" = "chrome" ]; then
	cp ./pkg/manifest_v3_chrome.json ./pkg/manifest.json
cat << EOF
The application is installed, but you need manifest file for native app.
EOF

	read -p "Do you want to open chrome? (y/n) " -n 1 -r
	if [[ $REPLY =~ ^[Yy]$ ]]; then
		if [ "$(uname)" = "Darwin" ]; then
			open -a "Google Chrome" "chrome://extensions"
		fi
	else
		echo "Open chrome and go to chrome://extensions"
	fi
cat << EOF
1. Enable developer mode 
2. Click on 'Load unpacked' and select the 'pkg' folder 
3. you can find your extension ID on the same page, or open the extension. it will also display the extension id 
4. enter the extension ID below
EOF

elif [ "$browser" = "firefox" ]; then
	cp ./manifest_v3_firefox.json ./pkg/manifest.json
cat << EOF
The application is installed, but you need manifest file for native app.
EOF
	read -p "Do you want to open firefox? (y/n) " -n 1 -r
	if [[ $REPLY =~ ^[Yy]$ ]]; then
		if [ "$(uname)" = "Darwin" ]; then
			open -a "Firefox"
		fi
	else
		echo "Open firefox and go to 'about:debugging'"
	fi

cat << EOF
1. Go to "about:debugging" page
2. Open 'This Firefox' section
3. Click on "Load Temporary Add-on" and select the 'pkg/manifest.json' file
4. you can find your extension ID on the same page, or open the extension. it will also display the extension id 
5. enter the extension ID below
EOF
fi

read extension_id

manifest_path=""
allowed_origins=""

if [ "$browser" = "chrome" ]; then
	cp ./manifest_v3_chrome.json ./pkg/manifest.json
	allowed_origins='"allowed_origins": ["chrome-extension://'$extension_id'/"]'
    if [ "$(uname)" = "Darwin" ]; then
        manifest_path="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts/rpass.json"
    elif [ "$(uname)" = "Linux" ]; then
        manifest_path="$HOME/.config/google-chrome/NativeMessagingHosts/rpass.json"
    else
        echo "Unsupported platform"
        exit 1
    fi
elif [ "$browser" = "firefox" ]; then
	allowed_origins='"allowed_extensions": ["'$extension_id'"]'
    if [ "$(uname)" = "Darwin" ]; then
        manifest_path="$HOME/Library/Application Support/Mozilla/NativeMessagingHosts/rpass.json"
    elif [ "$(uname)" = "Linux" ]; then
        manifest_path="$HOME/.mozilla/native-messaging-hosts/rpass.json"
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
manifest='
{
    "name": "rpass",
    "description": "rpass",
	"path": "'$(pwd)'/target/release/native-client",
    "type": "stdio",
	'$allowed_origins'
}
'
echo "Writing manifest"

echo "$manifest"
echo "$manifest" > "$manifest_path"

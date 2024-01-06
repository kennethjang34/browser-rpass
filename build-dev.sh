#!/bin/bash
rm -rf ./pkg
mkdir pkg
trunk build ./popup/index.html --dist ./pkg
(cd ./service-worker && wasm-pack build ./  --target web --out-dir ./pkg --dev)
(cd ./content && wasm-pack build ./  --target web --out-dir ./pkg --dev)
cargo build -p native-client
cp -r ./service-worker/pkg/* ./pkg
cp -r ./content/pkg/* ./pkg
cp ./run_service_worker.js ./pkg
cp ./run_content.js ./pkg
cp ./manifest_v3_chrome.json ./pkg/manifest_v3_chrome.json
cp ./manifest_v3_firefox.json ./pkg/manifest_v3_firefox.json

# Check if the files exist
if [ ! -f "./popup/assets/styles.css" ]; then
  echo "styles.css does not exist"
  exit 1
fi

if [ ! -f "./assets/popup_styles.css" ];  then
  echo "popup_styles.css does not exist. Creating it."
  npx tailwindcss -i "./popup/assets/styles.css" -o "./assets/popup_styles.css"
  exit 1
fi

# Get the modification times
styles_css_mod=$(stat -c %Y "./popup/assets/styles.css")
popup_styles_css_mod=$(stat -c %Y "./assets/popup_styles.css")

# Compare modification times and execute npx tailwindcss if styles.css is newer
if [ $styles_css_mod -gt $popup_styles_css_mod ]; then
	rm "./assets/popup_styles.css"
  npx tailwindcss -i "./popup/assets/styles.css" -o "./assets/popup_styles.css"
  echo "Changes detected. TailwindCSS has been executed."
else
  echo "No changes detected."
fi


# npx tailwindcss -i ./content/assets/styles.css -o ./assets/content_styles.css
if [ ! -f "./content/assets/styles.css" ]; then
  echo "styles.css does not exist"
  exit 1
fi

if [ ! -f "./assets/content_styles.css" ];  then
  npx tailwindcss -i "./content/assets/styles.css" -o "./assets/content_styles.css"
  exit 1
fi

# Get the modification times
styles_css_mod=$(stat -c %Y "./content/assets/styles.css")
content_styles_css_mod=$(stat -c %Y "./assets/content_styles.css")

# Compare modification times and execute npx tailwindcss if styles.css is newer
if [ $styles_css_mod -gt $content_styles_css_mod ]; then
	rm "./assets/content_styles.css"
  npx tailwindcss -i "./content/assets/styles.css" -o "./assets/content_styles.css"
  echo "TailwindCSS has been executed."
else
  echo "No changes detected."
fi
cp ./assets/* ./pkg


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
			open -a "Firefox" "about:debugging"
		fi
	else
		echo "Open firefox and go to 'about:debugging'"
	fi

cat << EOF
1. Go to 'This Firefox' section and click on 'Load Temporary Add-on'
2. Click on "Load Temporary Add-on" and select the 'pkg/manifest.json' file
3. you can find your extension ID on the same page, or open the extension. it will also display the extension id 
4. enter the extension ID below
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
	"path": "'$(pwd)'/target/debug/native-client",
    "type": "stdio",
	'$allowed_origins'
}
'
echo "Writing manifest"

echo "$manifest"
echo "$manifest" > "$manifest_path"

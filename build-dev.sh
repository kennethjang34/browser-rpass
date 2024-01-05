#!/bin/bash
rm -rf ./pkg
mkdir pkg
cargo build -p native-client
(cd ./popup && trunk build ./index.html --dist ./pkg)
wasm-pack build ./service-worker  --target web --out-dir ./pkg --dev
wasm-pack build ./content --target web --out-dir ./pkg --dev
cp -r ./service-worker/pkg/* ./pkg
cp -r ./content/pkg/* ./pkg
cp -r ./popup/pkg/* ./pkg
cp ./run_service_worker.js ./pkg
cp ./run_content.js ./pkg
cp ./manifest_v3.json ./pkg/manifest.json

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

echo "Which browser version do you want? (chrome/chromium)"
read browser

if [ "$browser" = "chrome" ]; then
cat << EOF
The application is installed, but you need manifest file for native app. 
1. If not opened automatically, open your browser and go to chrome://extensions
2. Enable developer mode 
3. Click on 'Load unpacked' and select the 'pkg' folder 
4. you can find your extension ID on the same page, or open the extension. it will also display the extension id 
5. enter the extension ID below
EOF

	if [ "$(uname)" = "Darwin" ]; then
		open -a "Google Chrome" "chrome://extensions"
	fi
fi

read extension_id

manifest_path=""
manifest='
{
    "name": "rpass",
    "description": "rpass",
    "path": "'$HOME'/rpass/browser-rpass/target/debug/native-client",
    "type": "stdio",
    "allowed_origins": ["chrome-extension://'$extension_id'/"]
}
'

if [ "$browser" = "chrome" ]; then
    if [ "$(uname)" = "Darwin" ]; then
        manifest_path="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts/rpass.json"
    elif [ "$(uname)" = "Linux" ]; then
        manifest_path="$HOME/.config/google-chrome/NativeMessagingHosts/rpass.json"
    else
        echo "Unsupported platform"
        exit 1
    fi
elif [ "$browser" = "chromium" ]; then
    if [ "$(uname)" = "Darwin" ]; then
        manifest_path="$HOME/Library/Application Support/Chromium/NativeMessagingHosts/rpass.json"
    elif [ "$(uname)" = "Linux" ]; then
        manifest_path="$HOME/.config/chromium/NativeMessagingHosts/rpass.json"
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

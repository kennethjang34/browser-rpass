#!/bin/bash
browser="chrome"

rpass_path=$(pwd)
profile="release"
overwrite_manifest=true
open_browser=false

########## OPTIONS
while getopts ":rb:p:dmos" flag; do
   case $flag in
      r)
         profile="release";;
      b) 
         browser="${OPTARG}";;
      p) 
         rpass_path="${OPTARG}";;
      d) 
         profile="debug";;
	  m) 
		 overwrite_manifest=true;;
	  s) 
		 overwrite_manifest=false;;
	  o) 
		 open_browser=true;;
	*)
		echo "Error: Invalid option"
		exit 1
		;;
   esac
done

rm -rf ./pkg
mkdir pkg


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

if [ "$profile" = "release" ]; then
	trunk build ./popup/index.html --dist ./pkg --release
	(cd ./service-worker && wasm-pack build ./  --target web --out-dir ./pkg --release)
	(cd ./content && wasm-pack build ./  --target web --out-dir ./pkg --release)
	cargo build -p native-client --release
else
	trunk build ./popup/index.html --dist ./pkg
	(cd ./service-worker && wasm-pack build ./  --target web --out-dir ./pkg --dev)
	(cd ./content && wasm-pack build ./  --target web --out-dir ./pkg --dev)
	cargo build -p native-client
fi

cp -r ./service-worker/pkg/* ./pkg
cp -r ./content/pkg/* ./pkg
cp ./run_service_worker.js ./pkg
cp ./run_content.js ./pkg
cp ./manifest_v3_chrome.json ./pkg/manifest_v3_chrome.json
cp ./manifest_v3_firefox.json ./pkg/manifest_v3_firefox.json

cp -n ./assets/* ./pkg

create_manifest() {
	browser=$1
	rpass_path=$2
	profile=$3
	manifest_path="$4"
	guide=""
	allowed_origins=""
	echo "Enter extension ID"
	read extension_id
	if [[ "$browser" = "chrome" ]]; then
	allowed_origins='"allowed_origins": ["chrome-extension://'$extension_id'/"]'
	guide="1. Enable developer mode 
2. Click on 'Load unpacked' and select the 'pkg' folder 
3. You can find your extension ID on the same page, or open the extension. It will also display the extension ID 
4. Enter the extension ID below"
	elif [[ "$browser" = "firefox" ]]; then
		allowed_origins='"allowed_extensions": ["'$extension_id'"]'
		guide="1. Go to 'This Firefox' section and click on 'Load Temporary Add-on'
2. Click on 'Load Temporary Add-on' and select the 'pkg/manifest.json' file
3. You can find your extension ID on the same page, or open the extension. It will also display the extension ID 
4. Enter the extension ID below"
	else
		echo "Error: Invalid browser"
		exit 1
	fi
	echo "$guide"
	touch "$manifest_path"
	manifest='
	{
		"name": "rpass",
		"description": "rpass",
		"path": "'${rpass_path}'/target/'${profile}'/native-client",
		"type": "stdio",
		'$allowed_origins'
	}
	'
	echo "$manifest"
	echo "$manifest" > "$manifest_path"
}

if [ "$browser" = "chrome" ]; then
	cp ./pkg/manifest_v3_chrome.json ./pkg/manifest.json
	if [[ "$open_browser" = true ]]; then
		if [ "$(uname)" = "Darwin" ]; then
			open -a "Google Chrome" "chrome://extensions"
		fi
	else
		echo "Open chrome and go to chrome://extensions"
	fi

elif [ "$browser" = "firefox" ]; then
	cp ./manifest_v3_firefox.json ./pkg/manifest.json
	if [[ $open_browser  ]]; then
		if [ "$(uname)" = "Darwin" ]; then
			open -a "Firefox" "about:debugging"
		fi
	else
		echo "Open firefox and go to 'about:debugging'"
	fi

fi



manifest_path=""

if [ "$browser" = "chrome" ]; then
	cp ./manifest_v3_chrome.json ./pkg/manifest.json
    if [ "$(uname)" = "Darwin" ]; then
        manifest_path="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts/rpass.json"
    elif [ "$(uname)" = "Linux" ]; then
        manifest_path="$HOME/.config/google-chrome/NativeMessagingHosts/rpass.json"
    else
        echo "Unsupported platform"
        exit 1
    fi
elif [ "$browser" = "firefox" ]; then
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
if [ ! -e "$manifest_path" ]; then
	echo "Creating new manifest"
	create_manifest $browser $rpass_path $profile "${manifest_path}"
elif [ "$overwrite_manifest" = true ]; then
	echo "Overwriting manifest"
	create_manifest $browser $rpass_path $profile "${manifest_path}"
else
	echo "Manifest already exists."
fi

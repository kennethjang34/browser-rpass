rm -rf ./pkg
mkdir pkg
cargo build -p native-client
wasm-pack build ./service-worker --target web --out-dir ./pkg --dev
wasm-pack build ./content --target web --out-dir ./pkg --dev
trunk build ./popup/index.html
cp -r ./popup/dist/* ./pkg
cp ./init_popup.js ./pkg
cp ./run_service_worker.js ./pkg
cp ./run_wasm_content.js ./pkg
cp ./run_content.js ./pkg
cp ./service-worker/pkg/service_worker.js ./pkg
cp ./service-worker/pkg/service_worker_bg.wasm ./pkg
cp ./content/pkg/content.js ./pkg 
cp ./content/pkg/content_bg.wasm ./pkg
cp ./manifest_v3.json ./pkg/manifest.json

# npx tailwindcss -i ./popup/assets/styles.css -o ./assets/popup_styles.css
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

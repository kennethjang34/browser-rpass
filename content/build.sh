wasm-pack build  --target web --out-dir ./pkg --release

cp ./run_content.js ./pkg

# npx tailwindcss -i ./content/assets/styles.css -o ./assets/content_styles.css
if [ ! -f "./assets/styles.css" ]; then
  echo "styles.css does not exist"
  exit 1
fi

if [ ! -f "./pkg/content_styles.css" ];  then
  npx tailwindcss -i "./assets/styles.css" -o "./pkg/content_styles.css"
  exit 1
fi

# Get the modification times
styles_css_mod=$(stat -c %Y "./assets/styles.css")
content_styles_css_mod=$(stat -c %Y "./pkg/content_styles.css")

# Compare modification times and execute npx tailwindcss if styles.css is newer
if [ $styles_css_mod -gt $content_styles_css_mod ]; then
	rm "./pkg/content_styles.css"
  npx tailwindcss -i "./assets/styles.css" -o "./pkg/content_styles.css"
  echo "TailwindCSS has been executed."
else
  echo "No changes detected."
fi

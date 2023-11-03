cargo build -p native-client
rm -rf ./pkg
mkdir pkg
wasm-pack build ./service-worker --target web --out-dir ./pkg
wasm-pack build ./content --target web --out-dir ./pkg
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

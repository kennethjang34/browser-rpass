rm -rf ./pkg
mkdir pkg
(cd ./native-client && cargo build)
(cd ./service-worker && source ./build.sh)
(cd ./content && source ./build.sh)
(cd ./popup && source ./build.sh)
mv ./service-worker/pkg/* ./pkg
mv ./popup/pkg/* ./pkg
mv ./content/pkg/* ./pkg
cp ./manifest_v3.json ./pkg/manifest.json

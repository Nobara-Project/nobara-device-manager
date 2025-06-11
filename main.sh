#! /bin/bash

set -e

VERSION="0.1.0"

source ./pika-build-config.sh

echo "$PIKA_BUILD_ARCH" > pika-build-arch

# Clone Upstream
git submodule update --init --force --remote
mkdir -p pika-device-manager
cp -rvf ./* ./pika-device-manager/ || true
cd ./pika-device-manager/

# Cut empty locales
apt-get install jq -y
for i in ./locales/*.json
do
    echo "Cutting down $i"
    jq 'del(.[] | select(. == ""))' $i > /tmp/tmp-locales.json && mv /tmp/tmp-locales.json $i
done

# Get build deps
apt-get build-dep ./ -y
apt-get install curl -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | CARGO_HOME=/root/.cargo sh -s -- -y

# Build package
LOGNAME=root dh_make --createorig -y -l -p pika-device-manager_"$VERSION" || echo "dh-make: Ignoring Last Error"
dpkg-buildpackage --no-sign

# Move the debs to output
cd ../
mkdir -p ./output
mv ./*.deb ./output/

#!/usr/bin/env bash
set -euxo pipefail

BRANCH=$1
NAME=orbital-decay-web

pushd web

npm install

NODE_OPTIONS=--openssl-legacy-provider npm run build-production

TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

rm -rf $NAME
mkdir $NAME

mv dist $NAME/$BRANCH

zip -r $TMP/$NAME.zip $NAME
rm -rf $NAME

popd

mv $TMP/$NAME.zip .

#!/usr/bin/env bash
set -euxo pipefail

BRANCH=$1
NAME=orbital-decay

pushd web

npm install

npm run build -- --mode production

TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

rm -rf $NAME
mkdir $NAME

mv dist $NAME/$BRANCH

zip -r $TMP/$NAME.zip $NAME
rm -rf $NAME

aws s3 cp $TMP/$NAME.zip s3://games.gridbugs.org/$NAME.zip

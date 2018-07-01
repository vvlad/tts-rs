#!/bin/bash

VERSION=$1

docker build -t tts:build dist/deb
ID=$(docker run -d -ti tts:build) 
docker wait $ID 
docker cp $ID:build/tts-$VERSION-amd64.deb dist/tts-$VERSION-amd64.deb

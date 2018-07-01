FROM ubuntu:latest

WORKDIR /build
ADD package /build/package

CMD dpkg-deb --build package tts-$VERSION-amd64.deb


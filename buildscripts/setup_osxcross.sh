#!/bin/bash

export UNATTENDED='yes'
export OSX_VERSION_MIN='10.7'

git clone https://github.com/tpoechtrager/osxcross
cd osxcross
git checkout 9498bfdc621716959e575bd6779c853a03cf5f8d
wget https://github.com/phracker/MacOSX-SDKs/releases/download/10.13/MacOSX10.11.sdk.tar.xz
# wget https://s3.dockerproject.org/darwin/v2/MacOSX10.11.sdk.tar.xz
mv MacOSX10.11.sdk.tar.xz tarballs/
./build.sh
mkdir -p /usr/local/osx-ndk-x86
mv target/* /usr/local/osx-ndk-x86

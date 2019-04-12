#!/bin/bash

export UNATTENDED='yes'
export OSX_VERSION_MIN='10.7'

git clone https://github.com/tpoechtrager/osxcross
cd osxcross
wget https://s3.dockerproject.org/darwin/v2/MacOSX10.11.sdk.tar.xz
mv MacOSX10.11.sdk.tar.xz tarballs/
./build.sh
mkdir -p /usr/local/osx-ndk-x86
mv target/* /usr/local/osx-ndk-x86

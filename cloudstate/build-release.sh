#!/bin/bash
#
# Usage: ./build-release <PROJECT> ${TRAVIS_TAG}-${TRAVIS_OS_NAME}
#
# The latest version of this script is available at
# https://github.com/emk/rust-musl-builder/blob/master/examples/build-release
#
# Called by `.travis.yml` to build release binaries.  We use
# ekidd/rust-musl-builder to make the Linux binaries so that we can run
# them unchanged on any distro, including tiny distros like Alpine (which
# is heavily used for Docker containers).  Other platforms get regular
# binaries, which will generally be dynamically linked against libc.
#
# If you have a platform which supports static linking of libc, and this
# would be generally useful, please feel free to submit patches.

set -euo pipefail

case `uname -s` in
    Linux)
        sudo apt-get update && sudo apt-get install -y upx-ucl
        echo "Building static binaries using ekidd/rust-musl-builder"
        docker build -t build-"$1"-image .
        docker run -it --name build-"$1" build-"$1"-image
        docker cp build-"$1":/home/rust/src/target/x86_64-unknown-linux-musl/release/cloudstate cloudstate
        docker rm build-"$1"
        docker rmi build-"$1"-image
        
        echo "Compress binary file"
        /usr/bin/upx --brute cloudstate

        echo "Validate binary file"
        file cloudstate
        
        echo "Generate relase file"
        tar vczf "$1"-"$2".tar.gz cloudstate
        ;;
    *)
        echo "Building standard release binaries"
        RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup" cargo build --release
        tar vczf "$1"-"$2".tar.gz target/release/cloudstate
        ;;
esac
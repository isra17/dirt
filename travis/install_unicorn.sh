#!/bin/sh
set -e
export LD_LIBRARY_PATH=`pwd`:$LD_LIBRARY_PATH
git clone --branch 0.9 https://github.com/unicorn-engine/unicorn.git
cd unicorn
UNICORN_ARCHS="x86" ./make.sh clang

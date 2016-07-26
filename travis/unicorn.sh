#!/bin/sh
set -e
if [ ! -d "unicorn" ]; then
  git clone --branch 0.9 https://github.com/unicorn-engine/unicorn.git
  cd unicorn
  UNICORN_ARCHS="x86" ./make.sh clang
fi

#!/bin/sh
set -e
if [ ! -d "unicorn/*.so" ]; then
  rm -rf unicorn
  git clone --branch 0.9 https://github.com/unicorn-engine/unicorn.git
fi
cd unicorn
./make.sh clang
sudo ./make.sh install

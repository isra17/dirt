#!/bin/sh
set -e
if [ ! -d "capstone/*.so" ]; then
  rm -rf capstone
  git clone --branch 3.0.4 https://github.com/aquynh/capstone.git
fi
cd capstone
./make.sh clang
sudo ./make.sh install

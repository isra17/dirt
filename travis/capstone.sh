#!/bin/sh
set -e
if [ ! -d "capstone" ]; then
  git clone --branch 3.0.4 https://github.com/aquynh/capstone.git
  cd capstone
  CAPSTONE_ARCHS="x86" ./make.sh clang
fi

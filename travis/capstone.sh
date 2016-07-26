#!/bin/sh
set -e
git clone --branch 3.0.4 https://github.com/aquynh/capstone.git
cd capstone
CAPSTONE_ARCHS="x86" ./make.sh clang

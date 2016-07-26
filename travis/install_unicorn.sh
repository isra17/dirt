#!/bin/sh
set -e
git clone --branch 0.9 git@github.com:unicorn-engine/unicorn.git
cd unicorn
UNICORN_ARCHS="x86" ./make.sh clang

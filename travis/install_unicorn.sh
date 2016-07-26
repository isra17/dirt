#!/bin/sh
set -e
export UNICORN_QEMU_FLAGS="--python=$(which python2)"
wget https://github.com/unicorn-engine/unicorn/archive/0.9.tar.gz
tar zxf 0.9.tar.gz
cd unicorn-0.9
sudo ./make.sh install

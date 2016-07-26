#!/bin/sh
set -e
wget https://github.com/unicorn-engine/unicorn/archive/0.9.tar.gz
tar zxf 0.9.tar.gz
cd unicorn-0.9
echo $CC
echo `which cc`
./make.sh
./make.sh install

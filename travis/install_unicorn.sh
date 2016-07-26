#!/bin/sh
wget https://github.com/unicorn-engine/unicorn/archive/0.9.tar.gz
tar zxvf 0.9.tar.gz
UNICORN_ARCHS="x86" ./make.sh
sudo ./make.sh install

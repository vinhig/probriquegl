#!/bin/sh
cargo build --release
cp target/debug/libGL.so target/debug/libGL.so.1
CODE=$(pwd)
CIV6="/home/vincent/.local/share/Steam/steamapps/common/Sid Meier's Civilization VI"
cd "$CIV6"
LIBS=$CODE/target/release/:$LD_LIBRARY_PATH
LD_LIBRARY_PATH=$LIBS ./Civ6Sub
cd $CODE
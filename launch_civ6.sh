cargo build
rm target/debug/libGL.so.1
mv target/debug/libGL.so target/debug/libGL.so.1
rm shaders/*
CODE=$(pwd)
#CIV6="/home/vincent/.local/share/Steam/steamapps/common/Sid Meier's Civilization VI"
CIV6="/run/media/vincent/PlifPlaf/steam-apps-linux/steamapps/common/Sid Meier's Civilization VI"
cd "$CIV6"
LIBS=$CODE/target/debug/:$LD_LIBRARY_PATH
LD_LIBRARY_PATH=$LIBS "$CIV6/Civ6Sub"
cd $CODE
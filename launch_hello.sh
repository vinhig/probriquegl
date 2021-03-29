cargo build
rm target/debug/libGL.so.1
mv target/debug/libGL.so target/debug/libGL.so.1
gcc tests/main.c -ggdb -lGL -lglfw -lm
LD_LIBRARY_PATH=$(pwd)/target/debug:/home/vincent/Projects/glfw/build/src/:$LD_LIBRARY_PATH ./a.out
# LD_LIBRARY_PATH=$(pwd)/target/debug:$LD_LIBRARY_PATH ./a.out
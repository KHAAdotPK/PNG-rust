git clone https://github.com/madler/zlib.git
cd zlib
cmake ./
msbuild /p:Configuration=Debug /p:Platform=x64 zlib.sln
cd ..


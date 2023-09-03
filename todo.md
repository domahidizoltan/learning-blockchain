#install Ganace
download
sudo apt-get install fuse libfuse2
chmod +x *.AppImage
./*.AppImage

New Workspace: learning-blockchain

#install web3-cli
curl -LSs https://raw.githubusercontent.com/gochain/web3/master/install.sh | sh

web3 --rpc-url=http://127.0.0.1:7545 address --private-key 0xd355b3c36ed96e85aa90650e3f7819cc6740ddff91d41a1e2195c2daa5e59778

web3 --rpc-url=http://127.0.0.1:7545 block --input 0x31f3a720a28772ec4d196044da21211aac1ae686b91cf122667405776ec3cfd2

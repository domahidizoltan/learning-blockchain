//SPDX-License-Identifier: MIT

pragma solidity ^0.8.15;

// import "forge-std/console.sol";

contract TheBlockchainMessenger {

    uint public changeCounter;

    address public owner;

    string public theMessage;

    constructor() {
        owner = msg.sender;
    }

    // function ttt() public view {
    //     console.log("Hello from SharedWallet");
    // }

    function updateTheMessage(string memory _newMessage) public {
        if(msg.sender == owner) {
            theMessage = _newMessage;
            changeCounter++;
        }
    }
}
//SPDX-License-Identifier: MIT

pragma solidity ^0.8.15;

import "forge-std/console.sol";
import "openzeppelin-contracts/contracts/utils/Strings.sol";

contract SharedWallet {

    address payable public owner;

    address[] allowanceKeys;
    mapping(address => uint) public allowance;
    mapping(address => bool) public isAllowedToSend;

    address[] guardianKeys;
    mapping(address => bool) public guardian;
    address payable public nextOwner;
    uint public guardiansResetCount;
    uint public constant confirmationsFromGuardiansForReset = 3;

    constructor() {
        console.log("Deploying a SharedWallet with owner:", msg.sender);
        owner = payable(msg.sender);
    }

    function proposeNewOwner(address payable newOwner) public {
        require(guardian[msg.sender], "You are no guardian, aborting");
        if(nextOwner != newOwner) {
            nextOwner = newOwner;
            guardiansResetCount = 0;
        }

        guardiansResetCount++;

        if(guardiansResetCount >= confirmationsFromGuardiansForReset) {
            owner = nextOwner;
            nextOwner = payable(address(0));
        }
    }

    function setAllowance(address _from, uint _amount) public {
        require(msg.sender == owner, "You are not the owner, aborting!");
        allowance[_from] = _amount;
        isAllowedToSend[_from] = true;
        addAllowanceKey(_from);
    }

    function denySending(address _from) public {
        require(msg.sender == owner, "You are not the owner, aborting!");
        isAllowedToSend[_from] = false;
    }

    function transfer(address payable _to, uint _amount, bytes memory payload) public returns (bytes memory) {
        require(_amount <= address(this).balance, "Can't send more than the contract owns, aborting.");
        if(msg.sender != owner) {
            require(isAllowedToSend[msg.sender], "You are not allowed to send any transactions, aborting");
            require(allowance[msg.sender] >= _amount, "You are trying to send more than you are allowed to, aborting");
            allowance[msg.sender] -= _amount;

        }

        (bool success, bytes memory returnData) = _to.call{value: _amount}(payload);
        require(success, "Transaction failed, aborting");
        return returnData;
    }

    receive() external payable {}

    //------------------ Helper functions ------------------//

        function addAllowanceKey(address _from) private {
        for (uint256 index = 0; index < allowanceKeys.length; index++)
        {
            if (allowanceKeys[index] == _from) {
                return;
            }
        }
        allowanceKeys.push(_from);
    }

    function getAllowanceMapAsString() public returns (string memory) {
        string memory allowanceMapAsString = "";
        for (uint256 index = 0; index < allowanceKeys.length; index++)
        {
            address adr = allowanceKeys[index];
            string memory adrString = Strings.toHexString(uint160(adr), 20);
            allowanceMapAsString = string(abi.encodePacked(allowanceMapAsString, adrString, " => ", Strings.toString(allowance[adr]), "\n"));
        }
        return allowanceMapAsString;
    }

    function addGuardianKey(address _from) private {
        for (uint256 index = 0; index < guardianKeys.length; index++)
        {
            if (guardianKeys[index] == _from) {
                return;
            }
        }
        guardianKeys.push(_from);
    }

    function getIsAllowedToSendMapAsString() public returns (string memory) {
        string memory mapAsString = "";
        for (uint256 index = 0; index < allowanceKeys.length; index++)
        {
            address adr = allowanceKeys[index];
            string memory adrString = Strings.toHexString(uint160(adr), 20);
            string memory val = isAllowedToSend[adr] ? "true" : "false";
            mapAsString = string(abi.encodePacked(mapAsString, adrString, " => ", val, "\n"));
        }
        return mapAsString;
    }

    function getGuardianMapAsString() public returns (string memory) {
        string memory mapAsString = "";
        for (uint256 index = 0; index < guardianKeys.length; index++)
        {
            address adr = guardianKeys[index];
            string memory adrString = Strings.toHexString(uint160(adr), 20);
            mapAsString = string(abi.encodePacked(mapAsString, adrString, " => ", string(abi.encodePacked(guardian[adr])), "\n"));
        }
        return mapAsString;
    }
}
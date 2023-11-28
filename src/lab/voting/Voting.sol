// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.15;

import "forge-std/console.sol";
import "openzeppelin-contracts/utils/Strings.sol";

library helpers {
    function getProposalsAsString(Ballot.Proposal[] memory proposals) internal pure returns (string memory) {
        string memory result = "";
        for (uint i = 0; i < proposals.length; i++) {
            result = string(abi.encodePacked(result, proposals[i].name, " => ", Strings.toString(proposals[i].voteCount), "\n"));
        }
        return result;
    }
}

/// @title Voting with delegation.
contract Ballot {
    struct Voter {
        uint weight;
        bool voted;
        address delegate;
        uint vote;
    }

    struct Proposal {
        bytes32 name;
        uint voteCount;
    }

    address public chairperson;

    mapping(address => Voter) public voters;

    Proposal[] public proposals;

    event BallotCreated(address indexed _chairperson, string _proposals);
    event GotRightToVote(address indexed _voter);
    event RightDelegated(address indexed _from, address indexed _to);
    event Voted(address indexed _voter, uint indexed _proposal, uint _weight);

    modifier isValidVote(Ballot.Voter memory voter){
        require(voter.weight != 0, "No right to vote");
        require(!voter.voted, "Already voted.");
        _;
    }

    modifier onlyChairperson() {
        require(
            msg.sender == chairperson,
            "Only chairperson have right to this action."
        );
        _;
    }

    constructor(bytes32[] memory proposalNames) {
        console.log("Deploying a Ballot with chairperson", msg.sender);

        chairperson = msg.sender;
        voters[chairperson].weight = 1;

        string memory props;
        for (uint i = 0; i < proposalNames.length; i++) {
            proposals.push(Proposal({
                name: proposalNames[i],
                voteCount: 0
            }));
            string memory propName = string(abi.encodePacked(proposalNames[i]));
            props = string.concat(props, propName);
            props = string.concat(props, ",");
        }
        console.log("Added proposals:", props);
        emit BallotCreated(chairperson, props);
    }

    function giveRightToVote(address voter) external onlyChairperson() {
        require(
            !voters[voter].voted,
            "The voter already voted."
        );
        require(voters[voter].weight == 0);
        voters[voter].weight = 1;

        console.log("Got right to vote:", voter);
        emit GotRightToVote(voter);
    }

    function delegate(address to) external isValidVote(voters[msg.sender]) {
        Voter storage sender = voters[msg.sender];
        require(to != msg.sender, "Self-delegation is disallowed.");

        while (voters[to].delegate != address(0)) {
            to = voters[to].delegate;

            require(to != msg.sender, "Found loop in delegation.");
        }

        Voter storage delegate_ = voters[to];

        require(delegate_.weight >= 1);

        sender.voted = true;
        sender.delegate = to;

        if (delegate_.voted) {
            proposals[delegate_.vote].voteCount += sender.weight;
        } else {
            delegate_.weight += sender.weight;
        }

        console.log("Voter delegated voting right", msg.sender, to);
        emit RightDelegated(msg.sender, to);
    }

    function vote(uint proposal) external isValidVote(voters[msg.sender]) {
        Voter storage sender = voters[msg.sender];
        sender.voted = true;
        sender.vote = proposal;

        proposals[proposal].voteCount += sender.weight;
        console.log("%s for proposal %s with weight %s", msg.sender, string(abi.encodePacked(proposal)), string(abi.encodePacked(sender.weight)));
        emit Voted(msg.sender, proposal, sender.weight);
    }

    function winningProposal() public view
            returns (uint winningProposal_)
    {
        uint winningVoteCount = 0;
        for (uint p = 0; p < proposals.length; p++) {
            if (proposals[p].voteCount > winningVoteCount) {
                winningVoteCount = proposals[p].voteCount;
                winningProposal_ = p;
            }
        }
    }

    function winnerName() external view
            returns (bytes32 winnerName_)
    {
        winnerName_ = proposals[winningProposal()].name;
    }

    function deleteBallot() external onlyChairperson() {
        selfdestruct(payable(msg.sender));
    }

    function getProposalsAsString() external view returns (string memory) {
        return helpers.getProposalsAsString(proposals);
    }

}

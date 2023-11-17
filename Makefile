export ETH_RPC_URL=http://localhost:8545

install-foundry:
	curl -L https://foundry.paradigm.xyz | bash

install-tailwind:
	npm install tailwindcss daisyui

install-solidity:
	asdf plugin add solidity
	asdf install solidity 0.8.21
	asdf global solidity 0.8.21

install-direnv:
	asdf plugin add direnv
	asdf install direnv 2.32.2
	asdf global direnv 2.32.2

install-web3cli:
	curl -LSs https://raw.githubusercontent.com/gochain/web3/master/install.sh | sh

remap:
# forge install OpenZeppelin/openzeppelin-contracts
	forge remappings > remappings.txt

prepare-env: install-foundry install-tailwind install-solidity install-direnv remap

start-testnet:
	anvil

watch-tailwind:
	rm templates/static/output.css || true
	npx tailwindcss -i ./templates/input.css -o ./templates/static/output.css --watch --minify

check:
	# cargo fix --allow-dirty
	cargo fmt
	cargo clippy
	cargo check

build: check
	cargo clean
	cargo build --release

run:
	direnv allow
	forge build
	cargo run

BLOCK_NR=0
web3-get-block:
	# web3 --rpc-url=$(ENDPOINT) block --input $(ACCOUNT)
	web3 --rpc-url=$(ENDPOINT) block --input $(BLOCK_NR)

web3-get-balance:
	web3 --rpc-url=$(ENDPOINT) address $(ACCOUNT)

BLOCK_NR=0
get-block:
	@cast block $(BLOCK_NR)

TX_HASH=0x0
get-tx:
	@cast tx $(TX_HASH)

get-balance:
	@cast balance $(ACCOUNT)

decode:
	@cast 4byte-decode $(HEX)

to-bytes32:
	@echo -n $(STR)|xxd -p|cast to-bytes32

# lab1: the_blockchain_messenger

# export CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER=$(make lab1-deploy)
lab1-deploy:
	@forge create --private-key=$(PRIVATE_KEY) src/lab/the_blockchain_messenger/TheBlockchainMessenger.sol:TheBlockchainMessenger | grep "Deployed to:" | cut -d' ' -f3

lab1-updateTheMessage:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "updateTheMessage(string)" "$(MSG)"

lab1-get-data:
	@echo 'changeCounter:'
	@cast call --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "changeCounter()(uint)"
	@echo 'theMessage:'
	@cast call --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "theMessage()(string)"

lab1-get-data-at-block:
	@echo 'changeCounter:'
	@cast call --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "changeCounter()(uint)" --block=$(BLOCK_NR)
	@echo 'theMessage:'
	@cast call --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "theMessage()(string)" --block=$(BLOCK_NR)


# lab2: smart_money

# export CONTRACT_ADDRESS_SMARTMONEY=$(make lab2-deploy)
lab2-deploy:
	@forge create --private-key=$(PRIVATE_KEY) src/lab/smart_money/SmartMoney.sol:SmartMoney | grep "Deployed to:" | cut -d' ' -f3

lab2-deposit:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SMARTMONEY) "deposit()" --value="$(AMOUNT)"

lab2-getContractBalance:
	@cast call $(CONTRACT_ADDRESS_SMARTMONEY) "getContractBalance()(uint)"

lab2-withdrawAll:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SMARTMONEY) "withdrawAll()"

lab2-withdrawToAddress:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SMARTMONEY) "withdrawToAddress(address)" "$(TO_ACC)"

lab2-get-data:
	@echo 'balanceReceived:'
	@cast call $(CONTRACT_ADDRESS_SMARTMONEY) "balanceReceived()(uint)"

lab2-get-data-at-block:
	@echo 'balanceReceived:'
	@cast call $(CONTRACT_ADDRESS_SMARTMONEY) "balanceReceived()(uint)" --block=$(BLOCK_NR)


# lab3: shared_wallet

# export CONTRACT_ADDRESS_SHAREDWALLET=$(make lab3-deploy)
lab3-deploy:
	@forge create --private-key=$(PRIVATE_KEY) src/lab/shared_wallet/SharedWallet.sol:SharedWallet | grep "Deployed to:" | cut -d' ' -f3

lab3-fund-contract:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SHAREDWALLET) --value="$(AMOUNT)"

lab3-proposeNewOwner:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SHAREDWALLET) "proposeNewOwner(address)" "$(ACC)"

lab3-setAllowance:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SHAREDWALLET) "setAllowance(address, uint)" "$(ACC)" "$(AMOUNT)"

lab3-denySending:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SHAREDWALLET) "denySending(address)" "$(ACC)"

lab3-transfer:
	@cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_SHAREDWALLET) "transfer(address, uint, bytes)(bytes)" "$(ACC)" "$(AMOUNT)" "$(shell echo $(PAYLOAD)|xxd -p)"

lab3-get-data:
	@echo 'contractBalance:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getContractBalance()(uint)"
	@echo 'owner:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "owner()(address)"
	@echo 'allowance:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getAllowanceMapAsString()(string)"
	@echo 'isAllowedToSend:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getIsAllowedToSendMapAsString()(string)"
	@echo 'guardian:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getGuardianMapAsString()(string)"
	@echo 'nextOwner:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "nextOwner()(address)"
	@echo 'guardiansResetCount:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "guardiansResetCount()(uint)"

lab3-get-data-at-block:
	@echo 'contractBalance:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getContractBalance()(uint)" --block=$(BLOCK_NR)
	@echo 'owner:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "owner()(address)" --block=$(BLOCK_NR)
	@echo 'allowance:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getAllowanceMapAsString()(string)" --block=$(BLOCK_NR)
	@echo 'isAllowedToSend:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getIsAllowedToSendMapAsString()(string)" --block=$(BLOCK_NR)
	@echo 'guardian:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "getGuardianMapAsString()(string)" --block=$(BLOCK_NR)
	@echo 'nextOwner:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "nextOwner()(address)" --block=$(BLOCK_NR)
	@echo 'guardiansResetCount:'
	@cast call $(CONTRACT_ADDRESS_SHAREDWALLET) "guardiansResetCount()(uint)" --block=$(BLOCK_NR)


# lab4: voting

# export CONTRACT_ADDRESS_VOTING=$(make lab4-deploy)
# ARGS=$(patsubst %,[$(shell echo -n %|xxd -p|cast to-bytes32)], $(BALLOT_PROPOSAL_NAMES))
ARGS=[0x7465737431000000000000000000000000000000000000000000000000000000,0x7465737432000000000000000000000000000000000000000000000000000000]
lab4-deploy:
	@forge create --private-key=$(PRIVATE_KEY) src/lab/voting/Voting.sol:Ballot --constructor-args $(ARGS) | grep "Deployed to:" | cut -d' ' -f3

lab4-giveRightToVote:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_VOTING) "giveRightToVote(address)" "$(TO)"

lab4-delegate:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_VOTING) "delegate(address)" "$(TO)"

lab4-vote:
	cast send --private-key=$(PRIVATE_KEY) $(CONTRACT_ADDRESS_VOTING) "vote(uint)" $(PROPOSAL)

lab4-get-data:
	@echo 'chairperson:'
	@cast call $(CONTRACT_ADDRESS_VOTING) "chairperson()(address)"
	@echo 'proposals:'
	@cast call $(CONTRACT_ADDRESS_VOTING) "getProposalsAsString()(string)"
	@echo 'winner name:'
	@cast to-ascii $(shell cast call $(CONTRACT_ADDRESS_VOTING) "winnerName()(bytes32)")

lab4-get-data-at-block:
	@echo 'chairperson:'
	@cast call $(CONTRACT_ADDRESS_VOTING) "chairperson()(address)" --block=$(BLOCK_NR)
	@echo 'proposals:'
	@cast call $(CONTRACT_ADDRESS_VOTING) "getProposalsAsString()(string)" --block=$(BLOCK_NR)
	@echo 'winner name:'
	@cast to-ascii $(shell cast call $(CONTRACT_ADDRESS_VOTING) "winnerName()(bytes32)") --block=$(BLOCK_NR)

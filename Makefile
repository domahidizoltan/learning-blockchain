install-foundry:
	curl -L https://foundry.paradigm.xyz | bash

install-tailwind:
	npm install tailwindcss daisyui

install-solidity:
	asdf plugin add solidity
	asdf plugin install solidity
	asdf install solidity 0.8.21
	asdf global solidity 0.8.21

install-direnv:
	asdf plugin add direnv
	asdf plugin install direnv
	asdf install direnv 2.32.2
	asdf global direnv 2.32.2

install-web3cli:
	curl -LSs https://raw.githubusercontent.com/gochain/web3/master/install.sh | sh

remap:
	forge remappings

prepare-env: install-foundry install-tailwind install-solidity install-direnv remap

start-testnet:
	anvil

watch-tailwind:
	rm templates/static/output.css
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
	cargo run

BLOCK_NR=0
web3-get-block:
	# web3 --rpc-url=$(ENDPOINT) block --input $(ACCOUNT)
	web3 --rpc-url=$(ENDPOINT) block --input $(BLOCK_NR)

web3-get-balance:
	web3 --rpc-url=$(ENDPOINT) address $(ACCOUNT)

BLOCK_NR=0
get-block:
	cast block $(BLOCK_NR)

TX_HASH=0x0
get-tx:
	cast tx $(TX_HASH)

deploy-lab1:
	forge create --rpc-url $(ENDPOINT) src/lab/the_blockchain_messenger/TheBlockchainMessenger.sol:TheBlockchainMessenger

send-lab1:
	cast send $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "updateTheMessage(string)" "$(MSG)"

get-lab1-data:
	@echo 'changeCounter:'
	@cast call $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "changeCounter()(uint)"
	@echo 'theMessage:'
	@cast call $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "theMessage()(string)"

get-lab1-data-at-block:
	@echo 'changeCounter:'
	@cast call $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "changeCounter()(uint)" --block=$(BLOCK_NR)
	@echo 'theMessage:'
	@cast call $(CONTRACT_ADDRESS_THEBLOCKCHAINMESSENGER) "theMessage()(string)" --block=$(BLOCK_NR)

get-balance:
	cast balance $(ACCOUNT)

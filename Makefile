# include .env file and export its env vars
# (-include to ignore error if it does not exist)
-include .env

.PHONY: build clean publish test

# Variables
DOCKER_IMAGE_NAME ?= gallynaut/solana-simple-randomness-function

check_docker_env:
ifeq ($(strip $(DOCKER_IMAGE_NAME)),)
	$(error DOCKER_IMAGE_NAME is not set)
else
	@echo DOCKER_IMAGE_NAME: ${DOCKER_IMAGE_NAME}
endif

# Default make task
all: anchor_sync build

anchor_sync :; anchor keys sync
anchor_build :; anchor build
anchor_publish:; make -j 2 simple-flip-deploy callback-flip-deploy

docker_build: 
	docker buildx build --platform linux/amd64 --pull -f ./token-manager/Dockerfile -t ${DOCKER_IMAGE_NAME} --load ./token-manager
docker_publish: 
	docker buildx build --platform linux/amd64 --pull -f ./token-manager/Dockerfile -t ${DOCKER_IMAGE_NAME} --push ./token-manager

build: anchor_build docker_build measurement

dev: dev_docker_build measurement

publish: anchor_publish docker_publish measurement

measurement: check_docker_env
	@docker run -d --platform=linux/amd64 -q --name=my-token-manager ${DOCKER_IMAGE_NAME}:latest
	@docker cp my-token-manager:/measurement.txt ./measurement.txt
	@echo -n 'MrEnclve: '
	@cat measurement.txt
	@docker stop my-token-manager > /dev/null
	@docker rm my-token-manager > /dev/null

simple-flip:
	anchor run simple-flip
simple-flip-deploy:
	anchor build -p super_simple_randomness
	anchor deploy --provider.cluster devnet -p super_simple_randomness

callback-flip:
	anchor run callback-flip
callback-flip-deploy:
	anchor build -p switchboard_randomness_callback
	anchor deploy --provider.cluster devnet -p switchboard_randomness_callback

# Task to clean up the compiled rust application
clean:
	cargo clean

	

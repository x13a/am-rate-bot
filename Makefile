NAME := am-rate-bot
all: build

build:
	cargo build --locked --release --bins

test:
	cargo test $(args)

clean:
	cargo clean

docker:
	docker build . -t $(NAME)

docker-clean:
	docker rmi $(NAME)

fix-config:
	chmod 600 ./config/*.env

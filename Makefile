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

fix-perm:
	chmod 600 ./config/*.env
	chmod 600 ./certs/*.key

.DEFAULT: build run

tag = 0.0.1-alpha.6

build:
	docker build --tag loggy-rs:$(tag) .

deploy:
	docker buildx build --push --platform linux/arm64 --tag bloveless/loggy-rs:$(tag) .

clean:
	docker container stop loggy
	docker container rm loggy

run:
	docker run --name loggy loggy-rs:$(tag)

exec:
	docker exec -it loggy bash

shell:
	docker run --rm -it --entrypoint bash loggy-rs:$(tag)

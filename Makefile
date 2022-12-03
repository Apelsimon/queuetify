
.PHONY: default docker down serve

default: docker

docker:
	docker build --tag queuetify -f Dockerfile .

serve: docker
	docker-compose up

down:
	docker-compose down
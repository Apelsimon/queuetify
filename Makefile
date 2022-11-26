
.PHONY: default docker down postgres redis serve

default: docker

postgres:
	./scripts/init_db.sh

redis:
	./scripts/init_redis.sh

docker:
	docker build --tag queuetify -f Dockerfile .

serve: docker
	docker-compose up

down:
	docker-compose down
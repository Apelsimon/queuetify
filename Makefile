ROOT=$(shell git rev-parse --show-toplevel)
OUTPUT_DIR=${ROOT}/build
TEMPLATE_DIR=${OUTPUT_DIR}/templates

.PHONY: all clean init build client run server docker

all: clean init build

clean:
	@echo "Clean output directory"
	rm -rf ${OUTPUT_DIR}
	rm -rf server/target
	rm -rf client/dist

init:
	@echo "Clean build directories"
	mkdir -p "${TEMPLATE_DIR}"; \
	cd server; cargo sqlx prepare -- --lib

build: client server

client:
	@echo "Build client"
	cd client; \
	npm run build; \
	cp dist/*.js ${OUTPUT_DIR}

server:
	@echo "Build server"
	cd ${ROOT}/server; \
	cargo build --release; \
	cp target/release/queuetify ${OUTPUT_DIR}; \
	cp -r templates ${OUTPUT_DIR}; \
	cp -r configuration ${OUTPUT_DIR}; \
	cp .env* ${OUTPUT_DIR}

postgres:
	./scripts/init_db.sh

redis:
	./scripts/init_redis.sh

run:
	cd build; ./queuetify

serve: docker
	docker-compose up

down:
	docker-compose down

docker:
	docker build --tag queuetify -f Dockerfile .
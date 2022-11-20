

ROOT=$(shell git rev-parse --show-toplevel)
OUTPUT_DIR=${ROOT}/build
TEMPLATE_DIR=${OUTPUT_DIR}/templates

.PHONY: all clean init build client server

all: clean init build

clean:
	@echo "Clean output directory"
	rm -rf ${OUTPUT_DIR}
	rm -rf server/target
	rm -rf client/dist

init:
	@echo "Clean build directories"
	mkdir -p "${TEMPLATE_DIR}"

build: client server

client:
	@echo "Build client"
	cd client; \
	npm run build; \
	cp dist/*.js ${OUTPUT_DIR}

server:
	@echo "Build server"
	cd ${ROOT}/server; \
	cargo build; \
	cp target/debug/server ${OUTPUT_DIR}; \
	cp -r templates ${OUTPUT_DIR}; \
	cp -r configuration ${OUTPUT_DIR}; \
	cp .env* ${OUTPUT_DIR}

postgres:
	./init_db.sh

redis:
	./init_redis.sh

serve:
	cd build; ./server
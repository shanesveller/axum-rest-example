export DOCKERBUILD_KIT := "1"
export DOCKER_DATABASE_URL := "postgresql://postgres:postgres@localhost:15432/axum_rest_example"

build:
  cargo build
docker-migrate:
  docker-compose up -d db
  sqlx database create -D $DOCKER_DATABASE_URL
  sqlx migrate run -D $DOCKER_DATABASE_URL
lint:
  cargo clippy --lib
migrate:
  sqlx database create
  sqlx migrate run
release: migrate
  cargo sqlx prepare -- --lib
  docker-compose build --progress plain -- app
reset:
  sqlx database reset -y
run:
  cargo run
run-docker-all: release docker-migrate
  docker-compose up -d
run-docker: release docker-migrate
  docker-compose up -d app
  docker-compose logs -f
sweep:
  cargo sweep -s -v
  cargo build
  cargo build --release
  cargo test
  cargo sweep -f -v
test: test-migrate
  cargo test
test-migrate:
  sqlx migrate run -D $TEST_DATABASE_URL
test-reset:
  sqlx database reset -y -D $TEST_DATABASE_URL
watch:
  cargo watch --clear --postpone -x 'clippy --lib' -x 'test --lib -- --show-output' -x 'run'

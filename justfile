export DOCKERBUILD_KIT := "1"
export DOCKER_DATABASE_URL := "postgresql://postgres:postgres@localhost:15432/axum_rest_example"

build:
  cargo build
docker-migrate:
  docker-compose up -d db
  sqlx -D $DOCKER_DATABASE_URL database create
  sqlx -D $DOCKER_DATABASE_URL migrate run
lint:
  cargo clippy --lib
migrate:
  sqlx database create
  sqlx migrate run
release: migrate
  cargo sqlx prepare -- --lib
  docker build -t axum-rest-example:latest .
reset:
  sqlx database reset -y
run:
  cargo run
run-docker-all: release docker-migrate
  docker-compose up -d
run-docker: release docker-migrate
  docker-compose up -d app
  docker-compose logs -f
test: test-migrate
  cargo test
test-migrate:
  sqlx migrate run -D $TEST_DATABASE_URL
test-reset:
  sqlx database reset -y -D $TEST_DATABASE_URL
watch:
  cargo watch --clear --postpone -x 'clippy --lib' -x 'test --lib -- --show-output' -x 'run'

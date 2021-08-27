build:
  cargo build
lint:
  cargo clippy --lib
migrate:
  sqlx database create
  sqlx migrate run
release: migrate
  cargo sqlx prepare -- --lib
  git rm -f Cargo.nix
  nix run .#cargo2nix -- -f
  git add Cargo.nix
  nix -vL build .#docker --no-link
  ./result | docker load
reset:
  sqlx database reset -y
run:
  cargo run
test: test-migrate
  cargo test
test-migrate:
  sqlx -D $TEST_DATABASE_URL migrate run
test-reset:
  sqlx -D $TEST_DATABASE_URL database reset -y
watch:
  cargo watch --clear --postpone -x 'clippy --lib' -x 'test --lib -- --show-output' -x 'run'

default: ## Build app in debug mode
	cargo build

check: ## Check for syntax errors on all workspaces
	cargo check --workspace --tests --all-features

clean: ## Delete build artifacts
	@cargo clean

coverage: migrate ## Generate code coverage report in HTML format
	$(call cache_bust)
	cargo tarpaulin -t 1200 --out Html

doc: ## Generate documentation
	#yarn doc
	cargo doc --no-deps --workspace --all-features

docker: ## Build Docker image
	docker build -t realaravinth/dumbserve:master -t realaravinth/dumbserve:latest .

docker-publish: docker ## Build and publish Docker image
	docker push realaravinth/dumbserve:master
	docker push realaravinth/dumbserve:latest

env: ## Setup development environtment
	cargo fetch

lint: ## Lint codebase
	cargo fmt -v --all -- --emit files
	cargo clippy --workspace --tests --all-features

release: ## Build app with release optimizations
	cargo build --release

run: ## Run app in debug mode
	cargo run


test: ## Run all available tests
	cargo test --no-fail-fast
	
xml-test-coverage: ## Generate code coverage report in XML format
	cargo tarpaulin -t 1200 --out Xml

help: ## Prints help for targets with comments
	@cat $(MAKEFILE_LIST) | grep -E '^[a-zA-Z_-]+:.*?## .*$$' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

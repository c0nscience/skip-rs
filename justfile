default:
	@just --list
docker VERSION:
	@docker buildx build --platform linux/arm64 -t bherzig/rskip:{{VERSION}} --push .
sloc:
	@tokei src templates

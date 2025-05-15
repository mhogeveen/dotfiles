MAKEFLAGS += --jobs
SHELL := /bin/zsh

all:
	@stow --verbose --restow */

delete:
	@stow --verbose --delete */

init:
	@stow --verbose --adopt */

up: brew zap pnpm rust nvim

brew:
	@brew upgrade
	@brew bundle
	@cd ~/.dotfiles/homebrew/ && brew bundle dump -f

zap:
	@source $(ZAP_DIR)/zap.zsh && zap update self && zap update all

pnpm:
	@pnpm up --global --latest

rust:
	@rustup update

nvim:
	@nvim --headless "+Lazy! sync" +qa && echo "Updated Neovim plugins"
	@nvim --headless "+MasonUpdateAll" +qa && echo "Updated Mason packages"

paru:
	@./scripts/paru.zsh

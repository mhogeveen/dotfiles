MAKEFLAGS += --jobs
SHELL := /bin/zsh

all:
	@stow --verbose --restow */

delete:
	@stow --verbose --delete */

init:
	@stow --verbose --adopt */

stow-host:
	@./scripts/stow-host.sh $(args)

up: brew zap pnpm rust nvim

brew:
	@brew upgrade -yg
	@brew bundle -g
	@cd ~/.dotfiles/homebrew/ && brew bundle dump -g --force --no-vscode

zap:
	@source $(ZAP_DIR)/zap.zsh && zap update self && zap update all

pnpm:
	@pnpm up --global --latest

rust:
	@rustup update

nvim:
	@nvim --headless "+Lazy! sync" +qa && echo "\nUpdated Neovim plugins"
	@nvim --headless "+MasonUpdateAll" +qa && echo "\nUpdated Mason packages"

paru:
	@./scripts/paru.zsh

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

zap:
	@source $(ZAP_DIR)/zap.zsh && zap update self && zap update all

pnpm:
	@pnpm up --global --latest

rust:
	@rustup update

nvim:
	@nvim --headless "+Lazy! sync" +qa && echo "Updated Neovim plugins"
	@nvim --headless "+MasonUpdate" +qa

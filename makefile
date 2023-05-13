SHELL := /bin/zsh

all:
	@stow --verbose --restow */

delete:
	@stow --verbose --delete */

init:
	@stow --verbose --adopt */

up: brew zap pnpm

brew:
	@brew upgrade
	@brew bundle

zap:
	@source $(ZAP_DIR)/zap.zsh && zap update self && zap update all

pnpm:
	@pnpm up --global --latest

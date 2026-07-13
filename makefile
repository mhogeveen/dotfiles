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

up: brew shell pnpm rust nvim

brew:
	@brew upgrade -y
	@brew bundle
	@cd ~/.dotfiles/homebrew/.config/homebrew/ && brew bundle dump --force --no-vscode

shell:
	@source ~/.dotfiles/zsh/.config/zsh/plugins.zsh && zplugin-update

pnpm:
	@pnpm up --global --latest

rust:
	@rustup update

nvim:
	@nvim --headless "+Lazy! sync" +qa && echo "\nUpdated Neovim plugins"
	@nvim --headless "+MasonUpdateAll" +qa && echo "\nUpdated Mason packages"

paru:
	@./scripts/paru.zsh

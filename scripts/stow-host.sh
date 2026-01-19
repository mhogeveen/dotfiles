#! /usr/bin/env zsh

# Check dependencies before running script
if ! type stow &> /dev/null; then
  echo "This script requires stow to be installed"
  exit 1
fi

# Save script args in variable for use in function scope
stow_args=$@

## Device hostnames
work='MacBook-Pro-van-Maarten-Hogeveen'
personal='mbp.local'
server='proxmox'
nas='nas'

# To stow packages per hostname
work_packages=(btop homebrew karabiner lazygit neovim wezterm yazi zsh)
personal_packages=$work_packages
server_packages=(lazygit neovim zsh)
nas_packages=$server_packages

# Find all valid stow packages as defined by the directories
# with a couple of exceptions (root dir, git dir, scripts dir)
# and parse to usable format
get_valid_stow_packages() {
  find . -maxdepth 1 -type d ! -name '.' ! -name '.git' ! -name 'scripts' | sed 's/\.\///'
}

# Stow the list of packages received as args.
# Check if the package is valid before stowing
# and exit early if stow receive invalid flags.
stow_packages() {
  for package in $@; do
    if echo $(get_valid_stow_packages) | grep -w -q "$package"; then
      echo "Valid package: ${package}. Stowing..."
      stow --verbose --dir=$HOME/.dotfiles $stow_args  $package || exit 1
    else
      echo "Invalid package: ${package}. Continuing with next.."
    fi
  done
}

# Check which host the script is run on and stow the packages
# specific to that host.
case $(hostname) in;
  $work) echo 'Work laptop'; stow_packages $work_packages ;;
  $personal) echo 'Personal laptop'; stow_packages $personal_packages ;;
  $server) echo 'Beelink'; stow_packages $server_packages ;;
  $nas) echo 'NAS'; stow_packages $nas_packages ;;
  *) echo 'Unsupported host'; exit 1 ;;
esac

#!/usr/bin/env zsh

# Install paru if it isn't already
remote_repo=https://aur.archlinux.org/paru-bin.git
local_repo=~/git/paru-bin

if [ -d $local_repo/.git ]; then
  pushd $local_repo;
  git pull;
  popd;
else
  git clone $remote_repo $local_repo;
fi

makepkg -sirc --needed --noconfirm --dir $local_repo

# Install packages
paru -S --needed --noconfirm \
  amd-ucode \
  base \
  base-devel \
  bluetuith-bin \
  bluez \
  bluez-deprecated-tools \
  bluez-utils \
  btop \
  btrfs-progs \
  catppuccin-cursors-mocha \
  clipse-bin \
  ddcutil \
  dolphin \
  dunst-git \
  efibootmgr \
  eza \
  fastfetch \
  fzf \
  git \
  git-delta \
  gnome-font-viewer \
  go \
  grim \
  gst-plugin-pipewire \
  hypridle \
  hyprland \
  hyprlock \
  hyprpaper \
  hyprpolkitagent-git \
  iwd \
  keyd \
  kitty \
  lazygit \
  libpulse \
  linux \
  linux-firmware \
  ly \
  man-db \
  man-pages \
  neovim \
  networkmanager \
  noto-fonts \
  noto-fonts-emoji \
  npm \
  openssh \
  paru-bin \
  pavucontrol \
  pipewire \
  pipewire-alsa \
  pipewire-jack \
  pipewire-pulse \
  qt5-wayland \
  rofi-wayland \
  slurp \
  stow \
  ttf-nonicons-bin-git \
  unzip \
  uwsm \
  waybar \
  wezterm-git \
  wireplumber \
  wl-clipboard \
  wlogout \
  xdg-desktop-portal \
  xdg-desktop-portal-gtk \
  xdg-desktop-portal-hyprland-git \
  yazi \
  zen-browser-bin \
  zoxide \
  zram-generator \
  zsh

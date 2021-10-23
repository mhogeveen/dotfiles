# ⚙️ Dotfiles Repository

This repo serves as a easy starting point for use of my own workspace configuration on other systems.

## 💾 Usage

1. Clone the dotfiles into a bare repo:

   ```shell
   git clone --bare <git-repo-url> $HOME/.cfg
   ```

2. Define the alias in the current shell scope:

   ```shell
   alias config='/usr/bin/git --git-dir=$HOME/.cfg/ --work-tree=$HOME'
   ```

3. Checkout the actual content from the bare repository to your $HOME:

   ```shell
   config checkout
   ```

4. If the checkout command produces errors because files will be overwritten, either back them up in a seperate folder or deleted them.

5. Rerun the checkout command if there were problems the first time.

6. Set the flag showUntrackedFiles to no on this specific (local) repository:

   ```shell
   config config --local status.showUntrackedFiles no
   ```

7. You are now setup to use the config repo.

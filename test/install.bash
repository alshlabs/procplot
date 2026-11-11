cargo build --release
rm -rf $HOME/.local/procplot
mkdir -p $HOME/.local/procplot
cp ./target/release/procplot ~/.local/procplot

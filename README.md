# Process Monitor

A command line utility that spawns a process and produces a `csv` report on its resource usage.

```
Usage: procplot <COMMAND>

Commands:
  exec     Spawn a process and record its resource usage [alias: e]
  upgrade  Update the binary to the latest release [aliases: up, u]
  version  Print the current version
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

# Usage

Run a command and record its resource usage with `exec`:

```bash
procplot exec -- node -e "setTimeout(() => console.log('Sup'), 4000)"
```

This will produce a file named `procplot.csv`

```bash
procplot exec --report something.csv -- node -e "setTimeout(() => console.log('Sup'), 4000)"
```

This will produce a report called `something.csv`

The csv file looks like:
```javascript
time_s,cpu,memory_mb,disk_read,disk_write
0.000,0,0,0,0
0.065,0,16,5976064,0
0.567,0,37,16556032,0
1.068,0,37,0,0
1.569,0,37,0,0
2.069,0,37,0,0
2.570,0,37,0,0
3.070,0,37,0,0
3.571,0,37,0,0
4.073,0,37,0,0
4.145,0,0,0,0
```

# Installation

## MacOS

### Upgrading

```bash
procplot upgrade
```

### Install script

```bash
mkdir -p $HOME/.local/procplot
curl -L --url https://github.com/alshlabs/procplot/releases/latest/download/procplot-macos-arm64.tar.gz | tar -xvzf - -C $HOME/.local/procplot
echo "\nexport \PATH=\$PATH:\$HOME/.local/procplot\n" >> $HOME/.zshrc
source $HOME/.zshrc
```

## Linux

### Upgrading

```bash
procplot upgrade
```

### Install script

```bash
mkdir -p $HOME/.local/procplot
curl -L --url https://github.com/alshlabs/procplot/releases/latest/download/procplot-linux-amd64.tar.gz | tar -xvzf - -C $HOME/.local/procplot
echo "\nexport \PATH=\$PATH:\$HOME/.local/procplot\n" >> $HOME/.zshrc
source $HOME/.zshrc
```

## Credit

[Matt Jones](https://github.com/mattcompiles) 
[David Alsh](https://github.com/alshdavid) 

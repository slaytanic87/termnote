# Termnote

Simple Rust based terminal CLI tool to manage your commands which you save as notes

![Termnote](Termnote_list_view.png)

## Usage

```bash
Usage: termnote [COMMAND]

Commands:
  add     Add a new command to the list
  update  Update a noted command
  remove  Remove a noted command by title or index
  list    List all noted commands
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Development

**Compilation**:

```bash
cargo build --release
```

**Test your build**:

```bash
./target/debug/termnote list
```

On default termnote is creating a ~/.termnote/db.json file in your home directory if not exists

**Installation**:

after the compilation move the binary ./target/release/termnote to /bin folder of your OS

**Supported OS**:

* MAC OSX
* Linux

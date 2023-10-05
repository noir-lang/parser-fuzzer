# Noir Parser Fuzzer

A parser fuzzer for the [**Noir** language](https://github.com/noir-lang/noir).

## Installation

To install the dependency required by the `bnf-to-pest.py` script:

```bash
pip install -r requirements.txt
```

## Usage

### BNF to Pest grammar

To convert the `grammar.bnf` file into the `grammar.pest` file, run:

```bash
python bnf-to-pest.py
```

### Pest setup

To boostrap Pest:

```bash
cd generator_as_pest_fork
cargo build --package pest_bootstrap
```

### Fuzzing setup

To setup afl:
```bash
cargo install cargo-afl
```

You need `make` installed.

### Fuzzing

To fuzz:
```bash
cd tester_for_pest

# make sure you run "afl build" instead of "build", and rebuild every time
# so that the debug binary is updated 
cargo afl build

cargo afl fuzz -i in -o out target/debug/tester_for_pest
```

To list 10 first crashes:
```bash
cd tester_for_pest
ls -U out/default/crashes/ | head -10
```

To get detailed information for a crash, provide the tester with the crash input file name:
```bash
cd tester_for_pest
mkdir debug
cargo afl run -- out/default/crashes/id\:000000\,sig\:06\,src\:000000+000084\,time\:15815\,execs\:14618\,op\:splice\,rep\:16
```
Debug information about the case will be included in the `debug` directory.

To get information for every crash in a directory, provide the tester with the `--all` option and the directory's path:
```bash
cd tester_for_pest
mkdir debug
cargo afl run -- --all out/default/crashes/
ls debug
```

## Progress

* [x] BNF formal grammar
* [x] converting BNF into Pest
* [x] parsing
* [x] generation
* [x] fuzzing
* [ ] grammar compliance

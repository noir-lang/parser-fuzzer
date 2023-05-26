# parser-fuzzer

A parser fuzzer for the **Noir** language.

## installation

To install the dependency required by the `bnf-to-pest.py` script:

```bash
pip install -r requirements.txt
```

## usage

To convert the `grammar.bnf` file into the `grammar.pest` file, run:

```bash
python bnf-to-pest.py
```

## progress

* [x] BNF formal grammar
* [x] converting BNF into Pest
* [ ] parsing
* [ ] generation
* [ ] fuzzing

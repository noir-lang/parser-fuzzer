import re
import regex
from dataclasses import dataclass

@dataclass
class Symbol:
    name: str

@dataclass
class Rule:
    lhs: Symbol
    rhs: list[Symbol]

@dataclass
class Grammar:
    rules: list[Rule]

    def __init__(self, grammar_str):
        self.rules = get_rules(grammar_str)

@dataclass
class TokenRegexp:
    symbol: Symbol
    regexp: re.Pattern

@dataclass
class TokenString:
    symbol: Symbol
    string: str

rule_regexp = re.compile("""
    (?P<lhs>\w+)     \s* # lhs
    # (?:
    #     ->
    #     (?<type>.*)
    # )?
    ::=             \s*
    (?P<rhs>    # rhs
        .+
        (\n\s*\|.+)*
    )
    \n
""", re.VERBOSE)

# replace with ' ~ '
concat_regexp = re.compile("""
    (?<=
        [\w\)'"\+\*\?]
    )
    [ ]  # space
    (?=
        [\w\('"&!]
    )
""", re.VERBOSE)

separator_regexp = regex.compile("""
    (?<string>
        [a-zA-Z_][a-zA-Z0-9_]* |
        (?<rec> #capturing group rec
            \\( #open parenthesis
            (?: #non-capturing group
                [^()]++ #anyting but parenthesis one or more times without backtracking
                | #or
                (?&rec) #recursive substitute of group rec
            )*
            \\) #close parenthesis
        )
    )
    % (?<separator>
        "[^"]+"
    )
    (?<operator>\+|\*)
""", re.VERBOSE)

token_regexp = re.compile("""
    '(.+)'  \s*
    ->      \s*
    (\w+)   \s*
    (?:
        :       \s*
        (\w+)   \s*
        {(.*)}
    )?
""", re.VERBOSE)

token_regexp_regexp = re.compile("""
    \/(.+)\/    \s*
    ->          \s*
    (\w+)       \s*
    (?:
        :       \s*
        (\w+)   \s*
        {(.*)}
    )?
""", re.VERBOSE)

start_decl_regexp = re.compile("""
    ^ \s* [#] start \s+ ([a-zA-Z_][a-zA-Z0-9_]*)
""", re.VERBOSE | re.MULTILINE)

def modify_seq(match):
    string = match.group('string')
    separator = match.group('separator')
    operator = match.group('operator')
    if operator == '*':
        return f'(({string} ~ {separator})* ~ {string})?'
    elif operator == '+':
        return f'({string} ~ {separator})* ~ {string}'
    else:
        raise RuntimeError(f'Invalid seq operator: {operator}, expected: + or *')

def modify_rule(match):
    lhs = match.group('lhs')
    rhs = match.group('rhs')
    rhs = re.sub(concat_regexp, ' ~ ', rhs)
    rhs = regex.sub(separator_regexp, modify_seq, rhs)
    return f"{lhs} ::= {rhs}"

with open('grammar.bnf', 'r') as grammar_file:
    grammar_str = grammar_file.read()
    grammar_str = re.sub(rule_regexp, modify_rule, grammar_str)
    grammar_str = re.sub(rule_regexp, r'\g<1> = { \g<2> }', grammar_str)
    grammar_str = re.sub(start_decl_regexp, r'start = { SOI ~ \g<1> ~ EOI }', grammar_str)
    with open('grammar.pest', 'w') as pest_file:
        pest_file.write(grammar_str)



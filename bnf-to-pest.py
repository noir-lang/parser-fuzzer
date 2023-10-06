#!/usr/bin/python3

import regex

rule_regexp = regex.compile("""
    (?P<atomic>
        \s*\#atomic\s*
    )?
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
""", regex.VERBOSE)

# replace with ' ~ '
concat_regexp = regex.compile("""
    (?<=
        [\w\)'"\+\*\?]
    )
    [ ]  # space
    (?=
        [\w\('"&!]
    )
""", regex.VERBOSE)

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
    (?<operator>
        (\.\.\.)?
        (\+|\*)
    )
""", regex.VERBOSE)

token_regexp = regex.compile("""
    '(.+)'  \s*
    ->      \s*
    (\w+)   \s*
    (?:
        :       \s*
        (\w+)   \s*
        {(.*)}
    )?
""", regex.VERBOSE)

token_regexp_regexp = regex.compile("""
    \/(.+)\/    \s*
    ->          \s*
    (\w+)       \s*
    (?:
        :       \s*
        (\w+)   \s*
        {(.*)}
    )?
""", regex.VERBOSE)

start_decl_regexp = regex.compile("""
    ^ \s* [#] start \s+ ([a-zA-Z_][a-zA-Z0-9_]*)
""", regex.VERBOSE | regex.MULTILINE)

def modify_seq(match):
    string = match.group('string')
    separator = match.group('separator')
    operator = match.group('operator')
    if operator == '*':
        return f'(({string} ~ {separator})* ~ {string})?'
    elif operator == '+':
        return f'({string} ~ {separator})* ~ {string}'
    elif operator == '...*':
        return f'({string} ~ {separator})* ~ ({string} ~ {separator}?)?'
    elif operator == '...+':
        return f'({string} ~ {separator})* ~ {string} ~ {separator}?'
    else:
        raise RuntimeError(f'Invalid seq operator: {operator}, expected: + or *')

def modify_rule(match):
    lhs = match.group('lhs')
    rhs = match.group('rhs')
    atomic = match.group('atomic')
    if atomic is None:
        atomic = ''
    rhs = regex.sub(concat_regexp, ' ~ ', rhs)
    rhs = regex.sub(separator_regexp, modify_seq, rhs)
    return f"{atomic}{lhs} ::= {rhs}"

def modify_syntax_of_rule(match):
    lhs = match.group('lhs')
    rhs = match.group('rhs')
    atomic = '' if match.group('atomic') is None else '@'
    atomic_replacement = '' if match.group('atomic') is None else "\n"
    return f'{atomic_replacement}{lhs} = {atomic}{{ {rhs} }}'

with open('grammar.bnf', 'r') as grammar_file:
    grammar_str = grammar_file.read()
    grammar_str = regex.sub(rule_regexp, modify_rule, grammar_str)
    grammar_str = regex.sub(rule_regexp, modify_syntax_of_rule, grammar_str)
    grammar_str = regex.sub(start_decl_regexp, r'start = { SOI ~ \g<1> ~ EOI }', grammar_str)
    with open('grammar.pest', 'w') as pest_file:
        pest_file.write(grammar_str)



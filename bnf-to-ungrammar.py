#!/usr/bin/python3

import regex

rule_regexp = regex.compile("""
    (?<! //[^\n]*)
    (?P<lhs>[a-zA-Z_][a-zA-Z0-9_]*)     \s* # lhs
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

# camelcase-ify
named_symbol_regexp = regex.compile("""
    (?<string>
        '[^']+'
    )
    |
    (?<=
        (\A | [\(\)'"\+\*\?\|])
        [ ]?  # space
        |# replace with: *
        \w
        [ ]
    )
    (?<ident>
        [a-zA-Z_][a-zA-Z0-9_]*
    )
""", regex.VERBOSE)

# replace with: '
doublequote_regexp = regex.compile("""
    "
""", regex.VERBOSE)

# replace with: *
repetition_regexp = regex.compile("""
    \{\d*,\d*\}
""", regex.VERBOSE)

# replace with: nothing
lookahead_regexp = regex.compile("""
    \s
    \&
    (?<rec> #capturing group rec
        \\( #open parenthesis
        (?: #non-capturing group
            [^()]++ #anyting but parenthesis one or more times without backtracking
            | #or
            (?&rec) #recursive substitute of group rec
        )*
        \\) #close parenthesis
    )
""", regex.VERBOSE)

# replace with: ... ...*
kleene_plus_regexp = regex.compile("""
    (?<tokens>
        [a-zA-Z_][a-zA-Z0-9_]*
        |
        (?<rec> #capturing group rec
            \\( #open parenthesis
            (?: #non-capturing group
                [^()]++ #anyting but parenthesis one or more times without backtracking
                | #or
                (?&rec) #recursive substitute of group rec
            )*
            \\) #close parenthesis
        )
        |
        '[^']+'
    )
    \s*
    [+]
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
        '[^']+'
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

def camelcaseify(our_str):
    return ''.join(elem.title() for elem in our_str.split('_'))

def camelcaseify_match(match):
    if match.group('string'):
        return match.group('string')
    elif match.group('ident'):
        return camelcaseify(match.group('ident'))
    else:
        raise Exception("invalid match string or ident")

def modify_seq(match):
    string = match.group('string')
    separator = match.group('separator')
    operator = match.group('operator')
    if operator == '*':
        return f'(({string} {separator})* {string})?'
    elif operator == '+':
        return f'({string} {separator})* {string}'
    elif operator == '...*':
        return f'({string} {separator})* ({string} {separator}?)?'
    elif operator == '...+':
        return f'({string} {separator})* {string} {separator}?'
    else:
        raise RuntimeError(f'Invalid seq operator: {operator}, expected: + or *')

def modify_kleene(match):
    s = match.group('tokens')
    return f'{s} {s}*'

def source_file_rule(match):
    return f'SourceFile = {camelcaseify(match[1])}'

def modify_rule(match):
    lhs = camelcaseify(match.group('lhs'))
    rhs = match.group('rhs')
    rhs = regex.sub(doublequote_regexp, """'""", rhs)
    rhs = regex.sub(named_symbol_regexp, camelcaseify_match, rhs)
    rhs = regex.sub(separator_regexp, modify_seq, rhs)
    rhs = regex.sub(kleene_plus_regexp, modify_kleene, rhs)
    rhs = regex.sub(repetition_regexp, '*', rhs)
    rhs = regex.sub(lookahead_regexp, '', rhs)
    return f"{lhs} =\n  {rhs}"

with open('grammar.bnf', 'r') as grammar_file:
    grammar_str = grammar_file.read()
    grammar_str = regex.sub(rule_regexp, modify_rule, grammar_str)
    # grammar_str = regex.sub(rule_regexp, r'\g<1> = { \g<2> }', grammar_str)
    grammar_str = regex.sub(start_decl_regexp, source_file_rule, grammar_str)
    grammar_str = grammar_str.replace(
        "Str =\n  '\\'' (!'\\'' ~ ANY)* '\\''",
        """Str = 'string'
AsciiDigit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
AsciiAlpha = 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'w' | 'x' | 'y' | 'z' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'W' | 'X' | 'Y' | 'Z'
AsciiAlphanumeric = AsciiDigit | AsciiAlpha""")
    # grammar_str = grammar_str.replace("Whitespace =\n  ' ' | '\\t' | '\\n'", '')
    grammar_str = grammar_str.replace(" !ASCII_ALPHA", '')
    grammar_str = grammar_str.replace("\n#atomic\n", "\n")
    grammar_str = grammar_str.replace("""' ' | '\\t' | '\\n'""", """' '""")
    with open('grammar.ungram', 'w') as ungram_file:
        ungram_file.write(grammar_str)

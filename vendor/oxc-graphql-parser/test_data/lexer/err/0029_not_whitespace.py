assert __file__.endswith('.py')
with open(__file__[:-3] + '.graphql', 'wb') as f:
    for c in '\uFEFF\u000B\u000C\u0085\u00A0\u200E\u200F\u2028\u2029':
        f.write(f'{c}# U+{ord(c):04X}\n'.encode('utf8'))
import sys

src = open(sys.argv[1]).read()


def strip_comments_and_strings(s):
    out = []
    i = 0
    n = len(s)
    while i < n:
        c = s[i]
        if s.startswith('//', i):
            j = s.find('\n', i)
            i = n if j < 0 else j
            continue
        if s.startswith('/*', i):
            j = s.find('*/', i + 2)
            i = n if j < 0 else j + 2
            continue
        if c == '"':
            out.append(c)
            i += 1
            while i < n and s[i] not in '"\n':
                if s[i] == '\\':
                    out.append(s[i])
                    out.append(s[i + 1])
                    i += 2
                else:
                    out.append(s[i])
                    i += 1
            if i < n:
                out.append(s[i])
                i += 1
            continue
        if c == "'":
            out.append(c)
            i += 1
            while i < n and s[i] not in "'\n":
                if s[i] == '\\':
                    out.append(s[i])
                    out.append(s[i + 1])
                    i += 2
                else:
                    out.append(s[i])
                    i += 1
            if i < n:
                out.append(s[i])
                i += 1
            continue
        out.append(c)
        i += 1
    return ''.join(out)


s = strip_comments_and_strings(src)
stk = []
line = 1
depth = 0
for ch in s:
    if ch == '\n':
        line += 1
        continue
    if ch == '{':
        depth += 1
        stk.append(line)
    elif ch == '}':
        depth -= 1
        if stk:
            stk.pop()
print("depth at EOF:", depth)
if stk:
    print("unbalanced opens (lines where still open at EOF):")
    for ln in sorted(set(stk)):
        print("  line", ln)
    lines = src.split('\n')
    first = stk[0]
    print("--- context around first unbalanced open (line", first, ") ---")
    lo = max(1, first - 6)
    for j in range(lo - 1, min(first + 3, len(lines))):
        print("%5d: %s" % (j + 1, lines[j]))

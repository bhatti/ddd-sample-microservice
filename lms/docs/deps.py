import glob
import re

def print_graph(include):
    out = open(include + '.dot', 'w')
    out.write('digraph {\n')
    out.write('  node [shape=component];\n')
    dups = {}
    for k, vv in deps.items():
        for v in vv:
            if k != v and ('::' in v or 'main' in k or 'main' in v) and include in k:
                line = '  "{}" -> "{}";\n'.format(k, v)
                if dups.get(line) == None:
                    out.write(line)
                    dups[line] = True
    out.write('}\n')
    out.close()

file_names = glob.glob('**/*.rs', recursive=True)
deps = {}
for file_name in file_names:
    mod = file_name.replace('/', '::').replace('.rs', '')
    if '_cmd' in mod:
        mod = re.sub(r'[a-z_]+_cmd', '*_cmd', mod)
    file = open(file_name, 'r')
    lines = file.readlines()
    for line in lines:
        if 'use crate::' in line:
            parts = line.strip().split(' ')[1].split('::')
            sub_mod = ''
            last_range = len(parts)
            if '{' in parts[-1] or parts[-1][0].isupper():
                last_range = len(parts)-1

            if last_range > 3:
                last_range = 3
            for x in range(1, last_range):
                if x > 1:
                    sub_mod = sub_mod + '::'
                if '_cmd' in parts[x]:
                    parts[x] = '*_cmd'
                sub_mod = sub_mod + parts[x]
            mod_deps = deps.get(mod, [])
            mod_deps.append(sub_mod)
            deps[mod] = mod_deps
print_graph('catalog')
print_graph('patrons')
print_graph('checkout')
print_graph('hold')


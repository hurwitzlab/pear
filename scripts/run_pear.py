#!/usr/bin/env python3
"""Run PEAR on input directory"""

import argparse
import os
import re
import sys

# --------------------------------------------------
def stderr(msg):
    print(msg, file=sys.stderr)

# --------------------------------------------------
def main():
    """main"""
    args = get_args()
    file_group = group_files(args.dir)
    out_dir = args.outdir or os.path.join(os.getcwd(), 'pear-out')
    pear = args.pear

    if not os.path.isdir(out_dir):
        os.makedirs(out_dir)

    for parent, files in file_group.items():
        if not ('for' in files and 'rev' in files):
            stderr('Error: "{}" missing for or rev'.format(parent))
            continue

        forward, reverse = files['for'], files['rev']
        out_file = os.path.join(os.path.abspath(out_dir), parent)
        print('{} -f {} -r {} -o {}'.format(pear, forward, reverse, out_file))

    stderr('Done')

# --------------------------------------------------
def group_files(in_dir):
    """Group all the R1/2 files"""

    if not os.path.isdir(in_dir):
        stderr('--dir "{}" is not a directory'.format(in_dir))
        sys.exit(1)

    files = os.listdir(in_dir)
    num_files = len(files)
    if num_files > 0:
        stderr('Processing {} file{} in --dir "{}"'.format(
            num_files, '' if num_files == 1 else 's', in_dir))
    else:
        stderr('No files in --dir "{}"'.format(in_dir))
        sys.exit(1)

    regex = '([a-zA-Z0-9_-]+)[_.](?:[rR])?([12])([_.].*)?'
    file_group = {}
    for file in files:
        base, _ = os.path.splitext(file)
        match = re.match(regex, base)
        if not match:
            stderr('{} does not look like R1/2'.format(file))
            continue

        parent = ''.join([match.group(1), match.group(3) or ''])

        if parent not in file_group:
            file_group[parent] = {}

        direction = 'for' if match.group(2) == '1' else 'rev'
        file_group[parent][direction] = os.path.join(os.path.abspath(in_dir), file)

    if file_group.keys() == 0:
        stderr('Found no usable files in --dir "{}"'.format(dir))
        sys.exit(1)

    return file_group

# --------------------------------------------------
def get_args():
    """get args"""
    parser = argparse.ArgumentParser()
    parser.add_argument('-d', '--dir', type=str, help='Input directory',
                        required=True)
    parser.add_argument('-o', '--outdir', type=str, help='Output directory',
                        default='')
    parser.add_argument('-p', '--pear', type=str, help='Path to pear',
                        default='pear')
    return parser.parse_args()

# --------------------------------------------------
if __name__ == '__main__':
    main()

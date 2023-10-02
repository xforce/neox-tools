#!/usr/bin/env python

# NOTE(alexander): DO NOT REORDER THIS!
import sys
import os
dir_path = os.path.dirname(os.path.realpath(__file__))

try:
    import uncompyle6
except ImportError as e:
    sys.path.insert(0, os.path.join(dir_path, "python-uncompyle6"))

from uncompyle6.bin.uncompile import main_bin

try:
    main_bin("utf-8")
except TypeError:
    # New version doesn't have the
    main_bin()

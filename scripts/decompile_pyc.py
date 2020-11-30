#!/usr/bin/env python

# NOTE(alexander): DO NOT REORDER THIS!
import sys
import os
dir_path = os.path.dirname(os.path.realpath(__file__))

sys.path.insert(0, os.path.join(dir_path, "python-uncompyle6"))
sys.path.insert(0, os.path.join(dir_path, "python-xdis"))

from uncompyle6.bin.uncompile import main_bin

main_bin("utf-8")

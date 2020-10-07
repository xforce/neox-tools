import os


def find_file(sub_path, root_path):
    return False


def rreplace(s, old, new, occurrence):
    li = s.rsplit(old, occurrence)
    return new.join(li)


def get_file(sub_path, root_path):
    input_file = os.path.join(root_path, sub_path)
    input_file = rreplace(input_file, '/', '.', 1)
    return open(input_file, "rb").read()


def new_module(unk1, data, unk2):
    return data

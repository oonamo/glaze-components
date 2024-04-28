#!/usr/bin/env python3
import os

# This script is independent of any Rust components
# therefore, this script can be extended without affecting
# the watcher

# This is the script that moves the list for the click command
userprofile = os.environ["USERPROFILE"]
list_path = userprofile + "\\.glaze-wm\\components\\task_list.log"
state_path = userprofile + "\\.state\\daily_note.log"

buffer = []


def read_to_buffer():
    with open(list_path, "r") as file:
        for line in file:
            buffer.append(line)


# the first_element becomes the last
# the second becomes the first, third is now the second, ect...
def rotate():
    if buffer:
        first_element = buffer.pop(0)
        buffer.append(first_element)


def write_buffer():
    with open(list_path, "w") as file:
        for line in buffer:
            file.write(line)


def write_first_entry_to_glaze():
    with open(state_path, "w") as file:
        file.write("\n" + buffer[0])


read_to_buffer()
rotate()
if buffer:
    write_buffer()
    write_first_entry_to_glaze()

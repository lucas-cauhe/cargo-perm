#!/bin/bash
# Program usage: compile_mock_integration.sh <crate_local_path> <src_program_path>
# <mock_program_name>

# Where crate_local_path is the path to the target crate in the local archive
# src_program_path is the path to the target file where to inject the payload
# from the crate directory
# mock_program_name is the path to the mock program resulting from the
# injection, its name needn't to be the same as the target file from the
# archive's crate

# clone crate to new space
crate_path="$1"
src_program_path="$2"
mock_program_name="$3"
crate_name="$(echo "$crate_path" | rev | cut -d '/' -f1-2 | rev)"
mkdir /tmp/$(echo "$crate_name" | cut -d '/' -f1)
cp -rf "$HOME/.cargo/registry/src/$crate_path" /tmp/"$crate_name"
   # delete original program
rm /tmp/"$crate_name"/"$src_program_path"
   # create new file with content <src_program>
cp "$mock_program_name" /tmp/"$crate_name"/"$src_program_path" 
   # compile crate with new file and skip compiling original program
curr_path="$(pwd)"
cd /tmp/"$crate_name" 
   # get compilation result
if cargo build &>/dev/null; then
  echo "0"
else
  echo "FAILED COMPILATION" >&2
fi
# Remove temp crate
rm -rf /tmp/$( echo "$crate_name" | cut -d '/' -f1)
cd "$curr_path"

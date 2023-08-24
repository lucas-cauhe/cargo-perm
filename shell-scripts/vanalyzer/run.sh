#!/bin/bash
# Output format
#
# <n> : <crate_name> <file_path_from_src_crate>
#       <k> <method_name> <line_no>
#
# Where <n> is the file number from the selected files
# <crate_name> is the crate's path in the local archive 
# <file_path_from_src_crate> is the path to the vulnerable file from the root
# of the crate
# <k> is the method number from the vulnerable file
# <method_name> is the method name so that the attacker can identify whether it
# is being used in the target code base
# <line_no> is the line number of where the method <method_name> is defined in
# the target file

read_cargo_file () {
  list_deps=""
  last_found=0
  in_deps=0
  while read line && [ $last_found -eq 0 ]; do 
    if [ "$line" == "[dependencies]" ]; then
      in_deps=1
    elif [[ $in_deps -eq 1 ]] && [[ $line =~ ^\[.*\]$ ]]; then
      last_found=1
    elif [ $in_deps -eq 1 ]; then
      list_deps="${list_deps}&${line}"
    fi
  done < "$1"
  echo $list_deps | tr '&' '\n' 
}

join_deps_version () {
  result_list=""
  while read dep; do
    dep_name=$(echo $dep | cut -d '=' -f1 | tr '_' '-')
    version=""
    if [[ $(echo $dep | cut -d '=' -f2- <<< "$dep") =~ \{.*\} ]]; then
      version=$(echo $dep | egrep -o "version\ *=\ *\".*\..*\..*\"," | cut -d ',' -f1) 
    else
      version=$dep
    fi
    version=$(echo $version | cut -d '=' -f2 | tr '\"' " ")
    if [ "$version" != "" ]; then
      result_list="${result_list}${dep_name}-$version&" 
    fi
  done <<< "$1" 
  echo $result_list | tr '&' '\n'| sed 's/\ //g' 
}

match_crates () {
   matched_crates=""
   while read line; do
      ls /home/${target_user}/.cargo/registry/src/* | grep "$line" &>/dev/null 
      [ $? -eq 0 ] && matched_crates="${matched_crates}${line}&"
   done <<< "$1"
   echo $matched_crates | tr '&' '\n'
}

# parse clargs
target_project="$1" 
target_user="$2"
# Check target project is readable for current user
[ ! -r "$target_project" ] && echo "target project is not readable" && exit 1
# list deps in target cargo project inside Cargo.toml file
[ ! -f "${target_project}/Cargo.toml"  ] && echo "Target project specified is not a cargo project or Cargo.toml
file was not found" && exit 1

all_writable_files="$(find /home/${target_user}/.cargo/registry/src/* -type f -writable -print | egrep "*\.rs")"

list_deps=$(read_cargo_file "${target_project}/Cargo.toml")
deps_versioned=$(join_deps_version "$list_deps")
# match exact version
matching_crates=$(match_crates "$deps_versioned") 

[ "$matching_crates" == "" ] && echo "No suitable crates have been found in the archive" && exit 0

# get matching writable files
vulnerable_crate_files=""
while read line; do
  vuln_files="$(echo "$all_writable_files" | grep $line)"
  vulnerable_crate_files="${vulnerable_crate_files}${vuln_files}&"
done <<< "$matching_crates"

all_target_files="$(echo $vulnerable_crate_files | tr '&' '\n' | tr ' ' '\n')"
# find crate methods used in cargo project
current_matched_file=0
current_matched_method=0
# For each method in each target file
while read file; do 
  file_output=""
  while read method line; do
    # see if it is included in any file of the target project
    if [ $(./method_is_included.sh $method $file $target_project) -eq 0 ]; then
      # if it is, check it is not unused code/tests/devdependencies (TODO)
     file_output="${file_output}\t$current_matched_method $method $line\n" 
     current_matched_method=$(($current_matched_method+1))
  done <<< "$(./list_methods.sh $file)" 
  if [ $current_matched_method -eq 0 ]; then
    common=$(echo "$file" | sed 's/src/\ /' | cut -d ' ' -f2)
    crate_name=$(echo "$common" | cut -d '/' -f1-2)
    file_path=$(echo "$common" | cut -d '/' -f3-)
    echo "$current_matched_file : $crate_name ${file_path}\n$file_output"
    current_matched_file=$(($current_matched_file+1))
    current_matched_method=0
  fi
done <<< $(echo "$all_target_files")

# for now it will only print all vulnerable files


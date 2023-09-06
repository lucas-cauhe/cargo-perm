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

# Reads a Cargo.toml file and extracts the lines defining all the dependencies
# Takes the path to the Cargo.toml file
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

# Takes a string with lines defining cargo dependencies and converts the lines
# as they are stored in the local archive 
join_deps_version () {
  result_list=""
  while read dep; do
    dep_name=$(echo $dep | cut -d '=' -f1 | tr '_' '-')
    version="$dep"
    if [[ $(echo $dep | cut -d '=' -f2- <<< "$dep") =~ \{.*\} ]]; then
      version=$(echo $dep | egrep -o "version\ *=\ *\".*\..*\..*\"," | cut -d ',' -f1) 
    fi
    version=$(echo $version | cut -d '=' -f2 | tr '\"' " ")
    if [ "$version" != "" ]; then
      result_list="${result_list}${dep_name}-$version&" 
    fi
  done <<< "$1" 
  echo $result_list | tr '&' '\n'| sed 's/\ //g' 
}

# Retrieves the crates found in the local archive
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

all_writable_files="$(find /home/${target_user}/.cargo/registry/src/* -type f -writable -print | egrep "*\.rs$")"

list_deps=$(read_cargo_file "${target_project}/Cargo.toml")
deps_versioned=$(join_deps_version "$list_deps")
# match exact version
matching_crates=$(match_crates "$deps_versioned") 

[ "$matching_crates" == "" ] && echo "No suitable crates have been found in the archive" && exit 0

# For each dep-crate in Cargo.toml of target_project
# get all the defined methods from all the writable files
# For each method found, check if it is used in target_project
#
# For each dep-crate in Cargo.toml of target_project
# get all the public methods included in target_project
# For each method found, check if it is defined in a writable file

# get matching writable files
vulnerable_crate_files=""
while read line; do
  vuln_files="$(echo "$all_writable_files" | grep $line)"
  vulnerable_crate_files="${vulnerable_crate_files}${vuln_files}"$'\n'

  # generate docs required in method_is_included
  crate_path=$(find $HOME/.cargo/registry/src/* -maxdepth 1 -type d -print | grep $line) 
  crate_name=$(echo "$crate_path" | rev | cut -d '/' -f1 | rev | tr '-' '_')
  api_file="/tmp/${crate_name}-public-api"
  cargo public-api --manifest-path "${crate_path}/Cargo.toml" | grep "pub fn " > "$api_file"
  cargo doc -q --no-deps --manifest-path "${crate_path}/Cargo.toml" 
done <<< "$matching_crates"

# Generate invalid lines files
while read file; do
  file_name=$(echo $file | tr '/' '_')
  ./invalid_lines.sh $file > "/tmp/$file_name-inv_lines"
done <<< "$(find "$target_project" -type f -print | grep "\.rs$")"

all_target_files="$(echo $vulnerable_crate_files | tr ' ' '\n')"
# find crate methods used in cargo project
current_matched_file=0
final_output=""
# For each method in each target file
while read file; do 
  file_output=""
  current_matched_method=0
  while read method line; do
    # see if it is included in any file of the target project
    included_result=$(./method_is_included.sh "$method" "$file" "$target_project" )
    if [ $included_result -eq 2 ]; then
      echo "Fatal error occurred, solve it and restart" >&2
      exit 1
    elif [ $included_result -eq 0 ]; then
      # if it is, check it is not unused code/tests/devdependencies/commentary (TODO)
     file_output="${file_output}"$'\t'"$current_matched_method $method $line"$'\n' 
     current_matched_method=$(($current_matched_method+1))
    fi
  done <<< "$(./list_methods.sh $file)" 
  if [ $current_matched_method -gt 0 ]; then
    crate_name=$(echo "$file" | grep -oE "/[a-zA-Z-]*[0-9\.]+/" | tr '-' '_' | sed 's/\///g')
    file_path="src$(echo "$file" | sed 's/src/ /g' | rev | cut -d ' ' -f1 | rev)"
    final_output="$final_output$current_matched_file : $crate_name ${file_path}"$'\n'"$file_output"
    current_matched_file=$(($current_matched_file+1))
  fi
done <<< $(echo "$all_target_files")
rm /tmp/*-public-api
rm /tmp/*-inv_lines
echo "$final_output" >&2

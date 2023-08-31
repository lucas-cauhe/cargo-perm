#!/bin/bash
# method file target_project
# Checks whether <method> is used in <target_project>
# The essence of it is to find the module that exposes <method> in the archive
# crate which, to make it simple, rarely aren't either the file or the file
# folder the method is defined in.
# The way it is going to be done is by trailing the path that exposes <method>
# in the crate from the given source file
# If it is not exposed or not used in <target_project> then it should echo 1,
# otherwise 0
# cargo-public-api crate will take care of that
# It can be either included with use statement so that <file> must appear in
# the file in target_project as well as method
# or the whole path to the method is used
# in any case, both <file> and <method> must appear in any file in
# <target_project>, hence searching for the path to the file first and the name
# of the method next should be enough??, nope since the file could be included
# but not the method and there could be another method with the same name
# so search by use statement and whole path

# the search must be semantic-scope-based
# since use statements can be used in specific scopes


# all scopes, except for global scope are held inside curly braces
# perform n searches accross the n scopes you find

# first scope is global scope

# Get the public-api result
# Select every function with the same name as method
# If there is more than one result
# For each result, check if in the docs built exists a file with the same name (starting from next for slash from src)
# as the given one and the path to it from the src of the docs is the same path
# that exposes the method in the API
find_method () {
  pub_api="$(cargo public-api --manifest-path "$crate_path/Cargo.toml" | grep "pub fn " | grep "::$method(" )"
  cargo doc --no-deps --manifest-path "$crate_path/Cargo.toml" &>/dev/null
  found_method=""
  while read result; do
   # Take the exposing path of the method previous to it
   prev_path=$(echo "$result" | grep -o "\(\w*::\)*$method(" | sed "s/::$method(//g")
   exposing_item=$(echo "$prev_path" | sed 's/::/\ /g' | rev | cut -d ' ' -f1 | rev)
   # Find the html file where the struct/trait/enum is documented
   html_half_path="${crate_path}target/doc/""$(echo "$prev_path" | sed 's/::/\//g' | rev | cut -d '/' -f2- | rev)"
   file_doc=$(ls $html_half_path | grep "\.$exposing_item\.")
   html_path="$html_half_path/$file_doc"
   # Scrap the html file found, to hit the method's name
   grep -oE "<a class=\"srclink rightside\" href=\"../src/${crate_name}${file_in_crate}\.html#[0-9]*-[0-9]*\">source</a><a href=\"#method.$method\" class=\"anchor\">" "$html_path" &>/dev/null

   [ $? -eq 0 ] && found_method="$(echo $result | grep -o "\(\w*::\)*$method(" | sed 's/(//g' )"
  done <<< $(echo "$pub_api")
  echo "$found_method" 
}

uses_method () {
  # Pattern 1: use .*<mod>::<method> + <method>(.*)
  # Pattern 2: .*<mod>::<method>(.*)
  # Pattern 3: ::<mod> + .<method>(.*) 
  # Pattern 3 and every other possible combination can be reduced to the first
  # two.
  mod=$(echo "$method_exposure" | sed 's/::/:/g' | rev | cut -d ':' -f2 | rev)
  grep "use\ *$method_exposure\ *$" "$1" &>/dev/null && grep "$method\(" "$1" &>/dev/null && echo 0 && exit # Pattern 1 matches 

  grep "^.*$mod::$method\(.*\).*$" "$1" &>/dev/null && echo 0 && exit # Pattern 2 mathces
  
  grep "^use.*::$mod" "$1" &>/dev/null && grep "\.$method\(" "$1" &>/dev/null && echo 0 && exit 
  }
# parse cl args
export method="$1"
file_in_crate=$(echo "$2" | sed 's/src/ /g' | rev | cut -d ' ' -f1 | rev)
# This is wrong, if there is a folder of depth 3 doesn't work
crate_path=$(echo "$2" | grep -oE "^.*/[a-zA-Z-]*[0-9\.]+/")
crate_name=$(echo "$2" | grep -oE "/[a-zA-Z-]*[0-9\.]+/" | cut -d '-' -f1-2 | tr '-' '_' | sed 's/\///g')
target_project="$3"

# Check cargo-public-api is installed
[ ! -f "$HOME/.cargo/bin/cargo-public-api" ] && echo "You must install cargo-public-api subcommand, please read the requirements" && exit 1
# More than one method with the same name may appear in public API
export method_exposure=$(find_method)
[ "$method_exposure" == "" ] && echo "method has not been found in crate's public API" && exit 1

method_is_used=$(find "$target_project" -type f -print | grep "\.rs$" | xargs -I% bash -c "$(declare -f uses_method) ; uses_method %" ) 
[ "$method_is_used" == "" ] && echo 1 && exit
echo 0
# rest of scopes are left for future versions


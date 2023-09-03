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
  api_file="/tmp/${crate_name_version}-public-api"
  pub_api="$(grep "::$method(" "$api_file")"
  found_method=""
  while read result; do
   # Take the exposing path of the method previous to it
   prev_path=$(echo "$result" | grep -o "\(\w*::\)*$method(" | sed "s/::$method(//g")
   exposing_item=$(echo "$prev_path" | sed 's/::/\ /g' | rev | cut -d ' ' -f1 | rev | sed 's/<.*>//g')
   # Find the html file where the struct/trait/enum is documented
   if [[ "$exposing_item" =~ [a-zA-Z][a-zA-Z]+ ]]; then
     html_half_path="${crate_path}target/doc/""$(echo "$prev_path" | sed 's/::/\//g' | rev | cut -d '/' -f2- | rev)"
     if [ -d $html_half_path ]; then
       file_doc=$(ls $html_half_path | grep "\.$exposing_item\.")
       html_path="$html_half_path/$file_doc"
       # Scrap the html file found, to hit the method's name
       grep -oE "<a class=\"srclink rightside\" href=\"../src/${crate_name}${file_in_crate}.html#[0-9]*-[0-9]*\">source</a><a href=\"#method.$method\" class=\"anchor\">" "$html_path" 

       [ $? -eq 0 ] && found_method="$(echo $result | grep -o "\(\w*::\)*$method(" | sed 's/(//g' )"
     fi
   fi
  done <<< $(echo "$pub_api")
  echo "$found_method" 
}

is_valid () {
   line=$(echo "$1" | cut -d ' ' -f1)
   cat $2 | grep -w -q $line
}

uses_method () {
  # Pattern 1: use .*<mod>::<method> + <method>(.*)
  # Pattern 2: .*<mod>::<method>(.*)
  # Pattern 3: ::<mod> + .<method>(.*) 
  # Pattern 3 and every other possible combination can be reduced to the first two.
  # Once matched, make sure the line is not commented (line or block)
  file_name=$(echo $1 | tr '/' '_')
  inv_file="/tmp/$file_name-inv_lines"
  mod=$(echo "$method_exposure" | sed 's/::/:/g' | rev | cut -d ':' -f2 | rev)
  cat -n "$1" | echo "$(grep "use\ *$method_exposure\ *\$")" $inv_file | is_valid && grep "$method(" "$1" &>/dev/null && echo 0 && exit # Pattern 1 matches

  cat -n "$1" | echo "$(grep "^.*$mod::$method(.*).*\$")" $inv_file | is_valid && echo 0 && exit # Pattern 2 matches
  
  cat -n "$1" | echo "$(grep "^use.*::$mod")" $inv_file | is_valid && grep "\.$method(" "$1" &>/dev/null && echo 0 && exit # Pattern 3 matches 
  }
# parse cl args
export method="$1"
file_in_crate=$(echo "$2" | sed 's/src/ /g' | rev | cut -d ' ' -f1 | rev)
crate_path=$(echo "$2" | grep -oE "^.*/[a-zA-Z-]*[0-9\.]+/")
crate_name_version=$(echo "$2" | grep -oE "/[a-zA-Z-]*[0-9\.]+/" | tr '-' '_' | sed 's/\///g')
crate_name=$(echo "$2" | grep -oE "/[a-zA-Z-]*[0-9\.]+/" | grep -oE "[a-zA-Z-]*[a-zA-Z]" | tr '-' '_' )
export target_project="$3"

# Check cargo-public-api is installed
[ ! -f "$HOME/.cargo/bin/cargo-public-api" ] && echo "You must install cargo-public-api subcommand, please read the requirements" >&2 && echo 2 && exit 1
# More than one method with the same name may appear in public API
export method_exposure=$(find_method)
[ "$method_exposure" == "" ] && echo 1 && exit 1

method_is_used=$(find "$target_project" -type f -print | grep "\.rs$" | xargs -I% bash -c "$(declare -f uses_method) ; uses_method %" ) 

[ "$method_is_used" == "" ] && echo 1 && exit
echo 0 && exit

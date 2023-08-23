#!/bin/bash
# method file target_project
# Checks whether <method> is used in <target_project>
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

# list all files from the target_project and get those matching the pattern

uses_method () {
  # Pattern 1: use .*<file>::<method> + <method>(.*)
  # Pattern 2: .*<file>::<method>(.*)
  # Pattern 3: ::<file> + .<method>(.*)
  grep "use\ *.*$file::\{?[\w_,\ ]*$method[\w_,\ ]*\}?.*$" "$1" && grep "$method\(" "$1"  && echo 1 && exit 0 # Pattern 1 matches 
  grep && echo 1 && exit 0 # Pattern 2 mathces
}

method="$1"
file="$2"
target_project="$3"

method_is_used=$(find "$target_project" -type f -print | grep "\.rs$" | xargs -I '{}' uses_method '{}') 
[ "$method_is_used" == "" ] && exit 1
exit 0

# rest of scopes are left for future versions


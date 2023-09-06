#!/bin/bash
# This script determines which lines of a given file are invalid
# It detects commented lines, whether the file is a tests file or the file has a test module which lines will be voided

# Takes in the file path and returns a list 
# of all line numbers that are invalid
get_commented_lines () {
  simples=$(cat -n "$1" | awk '$1=$1' | grep -o "^[0-9]* *//" | cut -d ' ' -f1)
  blocks=""
  paired=0
  line_no=1
  while read line; do
    [ $paired -ne 0 ] && blocks="$blocks$line_no"$'\n'
    echo "$line" | grep -q "/\*"
    if [ $? -eq 0 ]; then
      [ $paired -eq 0 ] && blocks="$blocks$line_no"$'\n'
      paired=$(($paired+1))
    fi
    echo "$line" | grep -q "\*/" 
    [ $? -eq 0 ] && paired=$(($paired-1))
    line_no=$(($line_no+1))
  done < $1
  two="$simples"$'\n'"$blocks"$'\n'
  echo "$two" | sort | uniq 
}


comm_lines=$(get_commented_lines "$1")
#test_lines=$(get_tests_lines "$1")
echo "$comm_lines" 

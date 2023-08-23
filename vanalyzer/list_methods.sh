#!/bin/bash
# find for fn <method_name>(.*) pattern in the given file
raw_methods=$(cat -n "$1" | grep "fn\ *\w[\w\d_]*\ *\(.*\).*$")
groomed_methods=""
while read method; do
  line_no=$(echo "$method" | awk '$1=$1' | cut -d ' ' -f1)
  m_name=$(echo "$method" | tr '\(' ' ' | tr -s " " | sed 's/^.*fn\ //'  | cut -d ' ' -f1 | cut -d '<' -f1 )
  groomed_methods="${groomed_methods}$m_name ${line_no}"$'\n'
done <<< $(echo "$raw_methods")
echo "$groomed_methods"

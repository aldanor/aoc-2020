#!/usr/bin/env bash
IFS='-: '; n=0; while read a b c p; do
    [ -z $p ] && break || set -- ${p//$c/} && k=$[${#p}-${#1}]
    [ $k -ge $a ] && [ $k -le $b ] && let n++
done < input.txt; echo $n

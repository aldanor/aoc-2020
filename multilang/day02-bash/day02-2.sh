#!/usr/bin/env bash
IFS='-: '; n=0; while read a b c p; do
    ! [ "${p:$a-1:1}" == "$c" ]; a=$?
    ! [ "${p:$b-1:1}" == "$c" ]; b=$?
    [ $a != $b ] && let 'n++'
done < input.txt; echo $n

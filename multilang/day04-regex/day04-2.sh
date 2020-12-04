#!/usr/bin/env sh
byr='(byr:(19[2-9][0-9]|200[012]))'
iyr='(iyr:(201[0-9]|2020))'
eyr='(eyr:(202[0-9]|2030))'
hgt='(hgt:(1([5-8][0-9]|9[0-3])cm|(59|6[0-9]|7[0-6])in))'
hcl='(hcl:(#[0-9a-f]{6}))'
ecl='(ecl:(amb|blu|brn|gry|grn|hzl|oth))'
pid='(pid:([0-9]{9}))'
cid='(cid:[0-9]+)'
sep="\s"
s="($sep($cid$sep)?)"
field="($byr$s|$iyr$s|$eyr$s|$hgt$s|$hcl$s|$ecl$s|$pid$s)"
match="($field){7}\n"
# printf "%s" "$match"
rg --multiline --count-matches $@ "$match" input.txt

#!/usr/bin/env sh
any="[^\s]+"
byr="(byr:$any)"
iyr="(iyr:$any)"
eyr="(eyr:$any)"
hgt="(hgt:$any)"
hcl="(hcl:$any)"
ecl="(ecl:$any)"
pid="(pid:$any)"
cid="c$any\s"
field="(($byr|$iyr|$eyr|$hgt|$hcl|$ecl|$pid)\s)"
match="(($cid)?$field($cid)?){7}\n"
printf "%s" "$match"
rg --multiline $@ "$match" input.txt

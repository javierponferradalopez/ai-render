#!/bin/bash
# look.sh <regex-literal> [before] [after] [nth]
pat="$1"; before="${2:-800}"; after="${3:-2500}"; nth="${4:-0}"
PAT="$pat" BEF="$before" AFT="$after" NTH="$nth" perl -0777 -ne '
  my $p = quotemeta($ENV{PAT}); my $b=$ENV{BEF}; my $a=$ENV{AFT}; my $n=$ENV{NTH};
  my $i=0;
  while (/$p/g) {
    my $s = pos($_) - length($ENV{PAT});
    $i++;
    next if ($n>0 && $i != $n);
    my $st = $s-$b; $st=0 if $st<0;
    print "=== match $i \@ $s ===\n", substr($_, $st, $b+$a+length($ENV{PAT})), "\n\n";
  }
' js.txt

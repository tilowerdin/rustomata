
mkdir res/

while read -r n; do
  make benchmark -B NAME=corp/$n PTK=5 WORDS=3 NFA=true
  cp benchmark-results.txt res/$n-5-results.txt

  make benchmark -B NAME=corp/$n PTK=10 WORDS=3 NFA=true
  cp benchmark-results.txt res/$n-10-results.txt

  make benchmark -B NAME=corp/$n PTK=15 WORDS=3 NFA=true
  cp benchmark-results.txt res/$n-15-results.txt

  make benchmark -B NAME=corp/$n PTK=20 WORDS=3 NFA=true
  cp benchmark-results.txt res/$n-20-results.txt
done < corp/bench_no_nfa.list

while read -r n; do
  make benchmark -B NAME=corp/$n PTK=5 WORDS=3
  cp benchmark-results.txt res/$n-5-nfa-results.txt

  make benchmark -B NAME=corp/$n PTK=10 WORDS=3
  cp benchmark-results.txt res/$n-10-nfa-results.txt
done < corp/bench-nfa.list
GRAMMAR ?= example/example_mcfg.gr
PTK ?= 5
EQC ?= example/example_equivalence.classes
WORDS ?= example/example_word.txt
WORDLIMIT ?= 1
NFA ?= false

RUSTOMATA_RELEASE := target/release/rustomata
default: rustomata
.PHONY: default rustomata $(RUSTOMATA_RELEASE)

bench: benchmark-results.txt

benchmark-results.txt:
	target/release/rustomata coarse-to-fine benchmark ${GRAMMAR} ${EQC} ${WORDS} ${PTK} -w ${WORDLIMIT} --nfabool ${NFA} 2> benchmark.log

rustomata: $(RUSTOMATA_RELEASE)

$(RUSTOMATA_RELEASE):
	cargo +nightly build --release

.PHONY: clean clean-all
clean:
	rm -fv benchmark-results.txt benchmark.log

clean-all: clean
	rm -fv target/release/rustomata

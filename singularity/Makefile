.PHONY = run clean img shell

SINGULARITY = /usr/local/bin/singularity
IMG = pear-0.9.11.img
DEF = image.def

run:
	sudo $(SINGULARITY) run $(IMG)

clean:
	rm -f $(IMG)

img: clean
	sudo $(SINGULARITY) build $(IMG) $(DEF)

shell:
	sudo $(SINGULARITY) shell --writable -B $(shell pwd):/tmp $(IMG)

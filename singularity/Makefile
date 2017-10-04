.PHONY = run clean img shell scp

SINGULARITY = /usr/local/bin/singularity
IMG_NAME = pear-0.9.10.img
DEF_NAME = image.def
SIZE = 1024

run:
	sudo $(SINGULARITY) run $(IMG_NAME)

clean:
	#[[ -e $(IMG_NAME) ]] && rm -f $(IMG_NAME)
	find . -name $(IMG_NAME) -exec rm {} \;

img: clean
	sudo $(SINGULARITY) create --size $(SIZE) $(IMG_NAME)
	sudo $(SINGULARITY) bootstrap $(IMG_NAME) $(DEF_NAME)

shell:
	sudo $(SINGULARITY) shell --writable -B $(shell pwd):/tmp $(IMG_NAME)

scp:
	scp $(IMG_NAME) stampede:/work/03137/kyclark/pear-0.0.1/stampede

TAG = hurwitzlab/pear:0.9.11

img:
	docker build --tag=$(TAG) .

run:
	docker run --rm $(TAG)

shell:
	docker run -it $(TAG) bash

test:
	docker run --rm -v /data:/data $(TAG) run_centrifuge.py -q /data/kyclark/dolphin/test/Dolphin_1.fasta /data/kyclark/dolphin/test/Dolphin_2.fasta -u -o /data/kyclark/centrifuge-out -I /data/centrifuge-indexes -H 1

push: 
	docker push $(TAG)

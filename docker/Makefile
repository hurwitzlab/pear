TAG = hurwitzlab/pear:0.9.11

img:
	docker build --no-cache --tag=$(TAG) .

run:
	docker run --rm $(TAG)

shell:
	docker run -it $(TAG) bash

test:
	docker run --rm -v /data:/data $(TAG) run_pear -q /data/kyclark/planetmicrobe/sra/

push: 
	docker push $(TAG)

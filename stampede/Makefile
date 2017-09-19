APP = pear-0.0.1
EMAIL = kyclark@email.arizona.edu

clean:
	find . \( -name \*.out -o -name \*.param -o -name pear-out \) -exec rm -rf {} \;

up: 
	copy-from-manifest.pl6 $(WORK)/$(APP)

template:
	jobs-template $(APP) > job.json

job:
	jobs-submit -F job.json

app:
	apps-addupdate -F app.json

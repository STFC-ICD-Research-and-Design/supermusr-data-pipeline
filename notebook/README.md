To run the jupyter notebook for plotting, you can either load the .ipynb file into an already running instance of jupyter notebook, or run the script "run-jupyter.sh" it will install a new conda environment, and start a jupyter notebook instance with the .ipynb file available. The .sh script is written to be run on IDAaaS, without any changes.

To run the notebook without downloading the files manually, run this command:
```shell
wget -O - https://raw.githubusercontent.com/STFC-ICD-Research-and-Design/supermusr-data-pipeline/main/notebook/run-jupyter.sh | bash
```
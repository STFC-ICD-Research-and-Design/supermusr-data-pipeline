# trace-archiver-hdf5

This tool listens on a given trace topic and saves each received message inside an HDF5 file.
If *no* control topic is provided,
messages are continuously saved to the file specified by the `--file` command line argument,
until the program terminates.
If a control topic is provided,
a new HDF5 file is created for each run with the file name `<timestamp>.h5`.

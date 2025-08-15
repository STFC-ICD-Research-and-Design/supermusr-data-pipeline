# Trace Viewer

This diagnostic tool allows you to search for and view detector traces and eventlists superimposed upon one another.

Given Kafka broker and topic specification, the tool searches for digitiser trace messages according to a given criteria.
Upon finding the trace messages, it then searches for digitiser eventlists corresponding to the found messages.
The results of these searches are matched up, and displayed as a list, from which the user can select to display as a plotly graph.
The resulting graph can then be saved as an image for more detailed inspection.

You specify parameters through the command line, and the Web UI.

## Sections

At the top of the Web UI is the *Broker* pane.
This allows the user to discover how many trace and eventlist messages there are, as well as the timestamps of the earliest and latest ones.

Below the *Broker* section is the *Search* section. This contains the search parameters as well as the button to begin a search.
When a search is in progress this button is replaced with a status bar showing the progress of the search, and the cancel search button.

Once a search has completed, the *Results* section displays its results.

Use `<Up>/<Down>` to select from the resulting digitiser messages, and `<Left>/<Right>` to select the channel of of that message. Press `<Enter>` to plot the selected message and channel in the *Graph* pane.

The *Graph* pane shows a terminal plot of the selected message and channel. Use the Arrow keys to pan the image, and `<+>/<->` to zoom in and out respectively. Details of the currently selected trace/eventlist and the pan location/zoom factor are shown in a Bar above the plot.

## Search Parameters

The following parameters are found in the *Setup* pane, and control how traces and eventlists are searched for. See [Search Modes](#search-modes) for more description of how the searches work.

| Parameter | Description |
|---|---|
|Search Mode|Select the type of search to perform. See [Search Modes](#search-modes).|
|Number|The maximum number of digitiser messages to collect, this is used in every search mode.|
|Date|The date of the timestamp to use in the `From Timestamp` and `From End` [Search mode](#search-modes). This must be in the format `YY-MM-DD`|
|Time|The time of the timestamp to use in the `From Timestamp` and `From End` [Search mode](#search-modes). This must be in the format `hh:mm:ss.f`|
|Search Criteria|Can be either `By Channels` or `By Digitiser Ids`. See [Search Criteria](#search-criteria).|
|Channels|Matched digitiser messages must contain at least one of these channels. This must be a comma separated list.|
|Digitiser Ids|Matched digitiser messages must have as `Id` one element of this list. This must be a comma separated list.|

## Search Modes

| Mode | Description |
|---|---|
|From Timestamp|Collect up to `Number` digitiser trace messages occuring no earlier than the timestamp specified in the setup pane.|
|From End|Collect `Number` digitiser messages occuring just before the end of the digitiser topic.|
|Capture in Realtime|Collect `Number` digitiser messages as they are captured in realtime.|

Internally, the tool uses three types of search: binary tree, backward linear search, forward linear search. Binary tree is efficient for searching through the entire topic for a specified timestamp, whereas forward/backward linear search is for finding and gathering contiguous messages which satisfy a given criteria. Forward linear search is much more efficent than backward linear, so the backward search tends to jump back in large steps then hones in on the target using forward search.

The `From Timestamp` mode uses the binary tree search to find the specified timestamp. Once found, the searcher jumps back a sepecified number of messages then searches forwards until the timestamp is found again. This is because the binary tree search is not guaranteed to find the first digitiser message with the specified timestamp.

The `From End` mode uses the backward search to jump back an estimated number of messages, followed by the forward search to find the required number of messages. This process is repeated if the first iteration doesn't find the required number.

The `Capture in Realtime` mode waits for the required number of messages satisfying the crieteria in realtime, this should only be used when the detectors are in use.

## Search Criteria

Although the Date and Time fields are only used in the `From Timestamp` mode, all modes use one of the two search criteria below.

| Criteria | Description |
|---|---|
|By Digitisers|Only match messages whose digitiser id is in this list.|
|By Channels|Only match messages whose channel list contains at least one value in this list.|

## Save Parameters

These parameters are found in the *Setup* pane to the right of the search parameters.

When the *Graph* pane is selected, pressing `<Enter>` saves the currently selected message and channel to an image file.
The file is saved at

```shell
[Save Path]/[Status Packet Timestamp]/[Channel].[Format]
```

e.g.

```shell
Saves/2025-06-06T09:34:00.026677500+00:00/31.svg
```

| Parameter | Description |
|---|---|
|Format|The image format to save, currently "svg"|
|Save Path|The subdirectory in the working directory to store the images.|
|Width|The width of the image.|
|Height|The height of the image.|

## Poll Broker

Pressing `<Home>` will cause the tool to retrieve the number of traces and eventlists from the broker, as well as the range of timestamps available on each topic.

This operation may take a few seconds and cannot be done whilst a search is in progress.

## Issues

Has to run `cargo install --force cargo-leptos` to upgrade cargo-leptos from 0.2.22 to 2.2.40, needed to use wasm-bindgen 2.1.100

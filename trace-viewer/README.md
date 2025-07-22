Has to run `cargo install --force cargo-leptos` to upgrade cargo-leptos from 0.2.22 to 2.2.40, needed to use wasm-bindgen 2.1.100



# Trace Viewer

This diagnostic tool allows you to search for and view detector traces and eventlists superimposed upon one another.

Given Kafka broker and topic specification, the tool searches for digitiser trace messages according to a given criteria.
Upon finding the trace messages, it then searches for digitiser eventlists corresponding to the found messages.
The results of these searches are matched up, and displayed as a list, from which the user can select to display as a terminal graph.
The resulting graph can then be saved as an image for more detailed inspection.

You specify parameters through the command line, and the terminal UI.
The terminal UI is controlled through the keyboard. Uses the `<Tab>` and Arrow keys to navigate through the interface.
Specific instructions for each control are displayed in the help bar at the bottom of the screen.

Note that the terminal UI is best viewed as full screen.

## Panes

At the top of the terminal UI is the *Setup* pane. When selected, navigate using the `<Left>/<Right>` arrow keys to select each control. Use `<Up>/<Down>` to edit list controls, and type/delete to edit text/numerical controls. Press `<Enter>` to begin a search using the current search parameters.

Below the *Setup* pane is the *Status Bar*, this shows the results of searches both in progress and completed.
Below the *Status Bar* are the *Results* and *Graph* pane, to the left and right respectively.

The *Results* pane shows the results of the last search. Use `<Up>/<Down>` to select from the resulting digitiser messages, and `<Left>/<Right>` to select the channel of of that message. Press `<Enter>` to plot the selected message and channel in the *Graph* pane.

The *Graph* pane shows a terminal plot of the selected message and channel. Use the Arrow keys to pan the image, and `<+>/<->` to zoom in and out respectively. Details of the currently selected trace/eventlist and the pan location/zoom factor are shown in a Bar above the plot.

Below the *Results* and *Graph* pane is the *Help Bar*, which shows tool tips pertaining to the currently selected pane and control.

## Controls

- `Tab`: Cycle through each pane in the terminal: *Setup*, *Results*, and *Graph*.
- `Home`: Run the [Poll Broker](#poll-broker) function, to discover details about the traces and eventlists available.
- `Enter`: Depends on which pane is selected:
   - Setup: Begin a search using the current settings.
   - Results: Graph the currently selected digitiser message and channel to the *Graph* pane.
   - Graph: Save the currently graphed image using the settings in the *Setup* pane.
- `Arrow Keys`: dependent on which pane and control is selected, see the tooltips for specifics.
- `Escape`: Quit the terminal app.

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

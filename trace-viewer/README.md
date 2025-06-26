# Trace Viewer

This diagnostic tool allows users to search for and view detector traces and eventlists superimposed upon one another.

Given Kafka broker and topic specification, the tool searches for digitiser trace messages according to a given criteria.
Upon finding the trace messages, it then searches for digitiser eventlists corresponding to the found messages.
The results of these searches are matched up, and displayed as a list, from which the user can select to display as a terminal graph.
The resulting graph can then be saved as an image for more detailed inspection.

The user specifies parameters through the command line, and the Terminal UI.

## Controls

- `Tab`: Cycle through each pane in the terminal: `Setup`, `Results`, and `Graph`.
- `Enter`: Depends on which pane is selected:
   - Setup: Begin a search using the current settings.
   - Results: Graph the currently selected digitiser message and channel to the `Graph` pane.
   - Graph: Save the currently graphed image using the settings in the `Setup` pane.
- `Arrow Keys`: dependent on which pane and control is selected, see the tooltips for specifics.
- `Escape`: Quit the terminal app.

## Search Parameters

| Parameter | Description |
|---|---|
|Search Mode|Select the type of search to perform. See [Search Modes](#search-modes).|
|Number|The maximum number of digitiser messages to collect, this is used in every search mode.|
|Date|The date of the timestamp to use in the `From Timestamp` and `From End` [Search mode](#search-modes). This must be in the format `YY-MM-DD`|
|Time|The time of the timestamp to use in the `From Timestamp` and `From End` [Search mode](#search-modes). This must be in the format `hh:mm:ss.f`|
|Search Criteria|Can be either `By Channels` or `By Digitiser Ids`. See [Search Criteria](#search-criteria).|
|Channels|Matched digitiser messages must contain at least one of these channels. This must be a comma separated list of channels.|
|Digitiser Ids|Matched digitiser messages must have as `Id` one element of this list. This must be a comma separated list of Digitiser Ids.|

## Search Modes



| Mode | Description |
|---|---|
|From Timestamp|Collect up to `Number` digitiser trace messages occuring no earlier than the timestamp specified in the setup pane.|
|From End|Collect the `Number` digitiser messages occuring just before the timestamp  |
|Capture in Realtime|Collect `Number` digitiser messages as they are captured in realtime.|

## Search Criteria

| Criteria | Description |
|---|---|
|By Digitisers| |
|By Channels| |

## Save Parameters

| Parameter | Description |
|---|---|
|Image Format| |
|Save Path| |
|Image Width| |
|Image Height| |

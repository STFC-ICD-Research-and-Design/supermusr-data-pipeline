# Simulation Class Diagram

```mermaid
classDiagram
    class Simulation {
        Transformation<int> voltage_transformation
        int time_bins
        int sample_rate
        ChannelConfig : channel_config 
        Vec~EventListTemplate~ event_lists
        Vec~PulseAttributes~ pulses
        Vec~Action~ schedule
    }
    class ChannelConfig{
        <<Enumeration>>
    }
    ChannelConfig ..|> Auto
    class Auto {
        int num_digitisers
        int channels_per_digitisers
    }
    ChannelConfig ..|> Digitisers
    class Digitisers {
        Vec~Digitiser~ digitisers
    }
    Digitisers "1" --> "*" Digitiser 
    class Digitiser {
        int id
        interval channels
    }
    Simulation "1" --> "1" ChannelConfig
    class EventListTemplate {
        int time_bins
        int sample_rate
        RandomDistribution num_pulses
        Vec~Pulse~ pulses
        Vec~Noise~ noises
    }
    class Pulse {
        float weight
        int index
    }
    EventListTemplate "1" --> "*" Pulse

    Simulation "1" --> "*" EventListTemplate
    Simulation "1" --> "*" Action
    Simulation "1" --> "*" PulseAttributes
    class PulseAttributes ["PulseAttributes"] {
        <<enumeration>>
    }
    class Action ["Action"] {
        <<Enumeration>>
    }
    Action ..|> WaitMs
    Action ..|> RunMessages
    class Loop {
        LoopVariable variable
        int start
        int end
        Vec~Action~ schedule
    }
    Action ..|> Loop
    class GenerateTrace {
        selection_mode
    }
    Action ..|> GenerateTrace
    class GenerateEventList {
        int event_list_template_index
    }
    Action ..|> GenerateEventList
    class EmitDigitiserTrace {
        Source trace_source
    }
    Action ..|> EmitDigitiserTrace
    class EmitDigitiserEventList {
        Source trace_source
    }
    Action ..|> EmitDigitiserEventList
    class EmitFrameEventList {
        Source trace_source
    }
    Action ..|> EmitFrameEventList
    Action ..|> SetProperty
```

```mermaid
    SOURCE_TYPE "0..1" --> "*" DIGITIZER
    class DIGITIZER ["Digitizer"] {
        int id
        interval channels
    }
    SOURCE_TYPE "0..1" --> "1" CHANNELS_BY_DIGITIZER
    class CHANNELS_BY_DIGITIZER ["ChannelsByDigitisers"] {
        int num_digitisers
        int channels_per_digitiser
    }
    TRACE_MESSAGE "1" --> "1" SOURCE_TYPE
    class FRAMES ["Frames"] {
        <<Enumeration>>
        Vec~int~
        interval

    }
    class PULSE ["Pulse"] {

    }
    class NOISE ["Noise"] {

    }
```

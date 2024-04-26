# Super MuRS Data Pipeline

Each message is uniquely identified by the following:
- Digitiser Trace: (Dig. ID, Frame Metadata)
- Digitiser Event List: (Dig. ID, Frame Metadata)
- Frame Event List: (Frame Metadata)

```mermaid
sequenceDiagram
participant C as EPICS/IBEX
participant T1 as Digitiser 1
participant T2 as Digitiser 2
box rgba(255,255,192,.5) <br/>Data Pipeline<br/>
    participant E as Event Formation
    participant A as Frame Aggregator
    participant W as Nexus Writer
end
participant X as Nexus File

rect rgba(224,240,255,0.75)
    critical Run Commands
        C ->> W: Run Start Command
        W -->> W: New Run
        W -->> X: Create
    end
end

loop Detector Data
    rect rgba(224,255,224,.75)
        rect rgba(255,255,255,0.65)
            T1 ->> E: Trace
            E -->> A: DAT Event List
            T2 ->> E: Trace
            E -->> A: DAT Event List
        end
        note over A: User Defined Delay<br/>to allow slow digitiser<br/>data to arrive.
        A ->> W: Frame Event List
        A -->> A: Frame cache expires
        W -->> X: Write
    end
    rect rgba(224,255,224,.75)
        rect rgba(255,255,255,0.65)
            T1 ->> E: Trace
            E -->> A: DAT Event List
            T2 ->> E: Trace
            E -->> A: DAT Event List
        end
        note over A: User defined delay<br/>to allow slow digitiser<br/>data to arrive.
        A ->> W: Frame Event List
        A -->> A: Frame cache expires
        W -->> X: Write
    end
end

rect rgba(256,224,224,.75)
    loop Metadata
        C ->> W: Run Log/SE Log/Alert
        W -->> X: Write
    end
end

rect rgba(224,240,255,0.75)
    critical Run Commands
        W -->> X: Write
        C ->> W: Run Stop Command
        note over W: User Defined Delay<br/>to allow slow frame<br/>data to arrive.
        W -->> W: Run cache expires
    end
end
```
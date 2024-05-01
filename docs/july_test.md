# July test

## Components under test

- Single (8 channel) digitiser
- Kafka broker
- `stream-to-file` in trace mode
- `frame-assembler` (incomplete, rewrite probably needed)
- `stream-to-file` in event mode
- `live-plot`

## Broker configuration

The following commands will create the topics, users and ACL rules required for the test.
Note that the ACL rules are more permissive that necessary and should be refined.

```sh
rpk topic create events-in
rpk topic create traces-in
rpk topic create events-preproc-1
rpk topic list

rpk acl user create digitiser --password '<digitiser password>'
rpk acl user create processing --password '<processing password>'
rpk acl user list

rpk acl create --allow-principal digitiser --operation read,write --topic '*'
rpk acl create --allow-principal processing --operation read,write --topic '*'
rpk acl create --allow-principal digitiser --operation read --group '*'
rpk acl create --allow-principal processing --operation read --group '*'
rpk acl list
```

## Results

- No significant issues.
   - File identifiers missing from messages from digitiser. Trivial change to digitiser firmware fixed this.
- Could stream both trace and event data at 50Hz when in R2 sample lab to server in MuSR cabin (~16 MBs).
   - Processing pipeline kept up with incoming data rate on server.
   - External client could view data (albeit limited by Wifi).

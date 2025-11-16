# Benchmark results without schema evolution

This benchmark serializes/deserializes 10000 "oplog entries", where oplog entries is a big enum type,
taken from an early prototype of [Golem](https://github.com/golemcloud/golem). Some of the cases have
an arbitrary dynamic 'Value' payloads in them, which the benchmark sets to various sizes to see the effect on
serialization speed.

Generating data set
## Benchmarking case with 10000 entries, payload size 16 bytes

...JSON...
...bincode...
...bincode without serde...
...MessagePack (rmp-serde)...
...postcard...
...BARE...
...bitcode...
...desert...
JSON
- total size:               21897680 bytes
- serialization duration:   2.419896ms
- deserialization duration: 3.874654ms

bincode
- total size:               4523670 bytes
- serialization duration:   1.143416ms
- deserialization duration: 848.595µs

bincode without serde
- total size:               4523230 bytes
- serialization duration:   693.162µs
- deserialization duration: 712.804µs

MessagePack (rmp-serde)
- total size:               8954490 bytes
- serialization duration:   2.471866ms
- deserialization duration: 2.001346ms

postcard
- total size:               4254760 bytes
- serialization duration:   436.383µs
- deserialization duration: 709.345µs

BARE
- total size:               4381520 bytes
- serialization duration:   600.162µs
- deserialization duration: 1.296845ms

bitcode
- total size:               4239540 bytes
- serialization duration:   5.02062ms
- deserialization duration: 6.345041ms

**desert**
- total size:               5196810 bytes
- serialization duration:   1.075125ms
- deserialization duration: 1.171066ms

## Benchmarking case with 10000 entries, payload size 256 bytes

...JSON...
...bincode...
...bincode without serde...
...MessagePack (rmp-serde)...
...postcard...
...BARE...
...bitcode...
...desert...
JSON
- total size:               130653630 bytes
- serialization duration:   14.857854ms
- deserialization duration: 30.555216ms

bincode
- total size:               28540050 bytes
- serialization duration:   3.866045ms
- deserialization duration: 5.976562ms

bincode without serde
- total size:               28539850 bytes
- serialization duration:   3.343825ms
- deserialization duration: 3.853312ms

MessagePack (rmp-serde)
- total size:               61880330 bytes
- serialization duration:   15.699295ms
- deserialization duration: 18.298858ms

postcard
- total size:               28201370 bytes
- serialization duration:   2.794162ms
- deserialization duration: 3.494741ms

BARE
- total size:               28326690 bytes
- serialization duration:   5.386591ms
- deserialization duration: 9.593004ms

bitcode
- total size:               22328210 bytes
- serialization duration:   8.83335ms
- deserialization duration: 9.648112ms

**desert**
- total size:               35996790 bytes
- serialization duration:   6.6763ms
- deserialization duration: 5.822154ms

## Benchmarking case with 10000 entries, payload size 1024 bytes

...JSON...
...bincode...
...bincode without serde...
...MessagePack (rmp-serde)...
...postcard...
...BARE...
...bitcode...
...desert...
JSON
- total size:               480290950 bytes
- serialization duration:   53.205616ms
- deserialization duration: 109.613875ms

bincode
- total size:               105508720 bytes
- serialization duration:   10.722729ms
- deserialization duration: 22.111728ms

bincode without serde
- total size:               105508320 bytes
- serialization duration:   11.654062ms
- deserialization duration: 14.060412ms

MessagePack (rmp-serde)
- total size:               231889990 bytes
- serialization duration:   55.562737ms
- deserialization duration: 70.155358ms

postcard
- total size:               105169120 bytes
- serialization duration:   9.322028ms
- deserialization duration: 12.22675ms

BARE
- total size:               105295060 bytes
- serialization duration:   20.050962ms
- deserialization duration: 34.204704ms

bitcode
- total size:               80199150 bytes
- serialization duration:   18.484208ms
- deserialization duration: 19.140212ms

**desert**
- total size:               134792130 bytes
- serialization duration:   23.074337ms
- deserialization duration: 20.605966ms

## Benchmarking case with 10000 entries, payload size 16384 bytes

...JSON...
...bincode...
...bincode without serde...
...MessagePack (rmp-serde)...
...postcard...
...BARE...
...bitcode...
...desert...
JSON
- total size:               7582570100 bytes
- serialization duration:   796.629195ms
- deserialization duration: 1.714430683s

bincode
- total size:               1661931150 bytes
- serialization duration:   141.912479ms
- deserialization duration: 340.191175ms

bincode without serde
- total size:               1661930830 bytes
- serialization duration:   176.784912ms
- deserialization duration: 210.847895ms

MessagePack (rmp-serde)
- total size:               3693009640 bytes
- serialization duration:   843.058737ms
- deserialization duration: 1.108289866s

postcard
- total size:               1661663730 bytes
- serialization duration:   125.145387ms
- deserialization duration: 187.903329ms

BARE
- total size:               1661788490 bytes
- serialization duration:   296.375849ms
- deserialization duration: 513.484474ms

bitcode
- total size:               1242374290 bytes
- serialization duration:   203.318737ms
- deserialization duration: 206.068116ms

**desert**
- total size:               2141855520 bytes
- serialization duration:   361.245366ms
- deserialization duration: 312.288645ms
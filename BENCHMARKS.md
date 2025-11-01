# Benchmark results without schema evolution

This benchmark serializes/deserializes 10000 "oplog entries", where oplog entries is a big enum type, 
taken from an early prototype of [Golem](https://github.com/golemcloud/golem). Some of the cases have
an arbitrary byte array payload in them, which the benchmark sets to various sizes to see the effect on
serialization speed.

## Benchmarking case with 10000 entries, payload size 16 bytes

JSON
- total size:               18270410 bytes
- serialization duration:   3.534995ms
- deserialization duration: 5.546094ms

bincode
- total size:               3992850 bytes
- serialization duration:   1.890258ms
- deserialization duration: 1.152444ms

bincode without serde
- total size:               3992410 bytes
- serialization duration:   1.273413ms
- deserialization duration: 840.346µs

MessagePack (rmp-serde)
- total size:               6860700 bytes
- serialization duration:   1.461062ms
- deserialization duration: 2.929389ms
  
postcard
- total size:               3723940 bytes
- serialization duration:   699.054µs
- deserialization duration: 861.619µs
  
BARE
- total size:               3850700 bytes
- serialization duration:   858.038µs
- deserialization duration: 1.554844ms

bitcode
- total size:               4092090 bytes
- serialization duration:   8.064358ms
- deserialization duration: 6.0171ms

**desert**
- total size:               4164660 bytes
- serialization duration:   2.338951ms
- deserialization duration: 3.013326ms 
 
## Benchmarking case with 10000 entries, payload size 256 bytes

JSON
- total size:               79051770 bytes
- serialization duration:   21.282349ms
- deserialization duration: 31.889273ms

bincode
- total size:               21156090 bytes
- serialization duration:   5.358322ms
- deserialization duration: 4.496157ms

bincode without serde
- total size:               21155890 bytes
- serialization duration:   1.485321ms
- deserialization duration: 1.062149ms

MessagePack (rmp-serde)
- total size:               32373110 bytes
- serialization duration:   12.363401ms
- deserialization duration: 17.009083ms

postcard
- total size:               20817410 bytes
- serialization duration:   4.762567ms
- deserialization duration: 3.42866ms

BARE
- total size:               20942730 bytes
- serialization duration:   6.417973ms
- deserialization duration: 11.144555ms

bitcode
- total size:               21326510 bytes
- serialization duration:   10.377369ms
- deserialization duration: 8.463068ms 

**desert**
- total size:               21257490 bytes
- serialization duration:   3.684346ms
- deserialization duration: 3.659971ms
  

## Benchmarking case with 10000 entries, payload size 1024 bytes
  
JSON
- total size:               275904820 bytes
- serialization duration:   66.386483ms
- deserialization duration: 102.119753ms

bincode
- total size:               76298500 bytes
- serialization duration:   12.598484ms
- deserialization duration: 14.708719ms

bincode without serde (current implementation)
- total size:               76298100 bytes
- serialization duration:   3.110397ms
- deserialization duration: 2.029679ms

MessagePack (rmp-serde)
- total size:               115077580 bytes
- serialization duration:   46.694929ms
- deserialization duration: 63.136818ms

postcard
- total size:               75958900 bytes
- serialization duration:   15.382506ms
- deserialization duration: 12.341794ms

BARE
- total size:               76084840 bytes
- serialization duration:   21.800745ms
- deserialization duration: 35.825761ms

bitcode
- total size:               76469580 bytes
- serialization duration:   18.171212ms
- deserialization duration: 16.978511ms
  
**desert**
- total size:               76400160 bytes
- serialization duration:   5.10305ms
- deserialization duration: 4.503051ms
  
## Benchmarking case with 10000 entries, payload size 16384 bytes

JSON
- total size:               4224183380 bytes
- serialization duration:   1.082191248s
- deserialization duration: 1.611288103s

bincode
- total size:               1182149070 bytes
- serialization duration:   138.413387ms
- deserialization duration: 227.466057ms

bincode without serde
- total size:               1182148750 bytes
- serialization duration:   20.931162ms
- deserialization duration: 18.64979ms

MessagePack (rmp-serde)
- total size:               1773910600 bytes
- serialization duration:   738.76901ms
- deserialization duration: 981.761867ms 

postcard
- total size:               1181881650 bytes
- serialization duration:   200.451978ms
- deserialization duration: 201.557102ms

BARE
- total size:               1182006410 bytes
- serialization duration:   361.714387ms
- deserialization duration: 524.595809ms

bitcode
- total size:               1182321010 bytes
- serialization duration:   145.3623ms
- deserialization duration: 190.639901ms

**desert**
- total size:               1182320640 bytes
- serialization duration:   22.305921ms
- deserialization duration: 19.189082ms

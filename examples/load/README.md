# examples use to

Not an example, but a "load application" used to measure memory usage (+/-)

```sh
> bash -c "cargo run --release 2>/dev/null"
...
13s Current memory usage: Some(MemoryStats { physical_mem: 22814720, virtual_mem: 1123610624 })
13s Current memory usage: Some(MemoryStats { physical_mem: 22835200, virtual_mem: 1123610624 })
13s Current memory usage: Some(MemoryStats { physical_mem: 22859776, virtual_mem: 1123610624 })
13s Current memory usage: Some(MemoryStats { physical_mem: 22863872, virtual_mem: 1123610624 })
14s Current memory usage: Some(MemoryStats { physical_mem: 22867968, virtual_mem: 1123610624 })
14s Current memory usage: Some(MemoryStats { physical_mem: 22876160, virtual_mem: 1123610624 })
14s Current memory usage: Some(MemoryStats { physical_mem: 22888448, virtual_mem: 1123610624 })
15s Current memory usage: Some(MemoryStats { physical_mem: 22892544, virtual_mem: 1123610624 })
15s Current memory usage: Some(MemoryStats { physical_mem: 22896640, virtual_mem: 1123610624 })
15s Current memory usage: Some(MemoryStats { physical_mem: 22904832, virtual_mem: 1123610624 })
16s Current memory usage: Some(MemoryStats { physical_mem: 22921216, virtual_mem: 1123610624 })
16s Current memory usage: Some(MemoryStats { physical_mem: 22933504, virtual_mem: 1123610624 })
16s Current memory usage: Some(MemoryStats { physical_mem: 22937600, virtual_mem: 1123610624 })
16s Current memory usage: Some(MemoryStats { physical_mem: 22941696, virtual_mem: 1123610624 })
22s Current memory usage: Some(MemoryStats { physical_mem: 22945792, virtual_mem: 1123610624 })
22s Current memory usage: Some(MemoryStats { physical_mem: 22949888, virtual_mem: 1123610624 })
28s Current memory usage: Some(MemoryStats { physical_mem: 22970368, virtual_mem: 1123610624 })
36s Current memory usage: Some(MemoryStats { physical_mem: 22999040, virtual_mem: 1123815424 })
36s Current memory usage: Some(MemoryStats { physical_mem: 23003136, virtual_mem: 1123815424 })
36s Current memory usage: Some(MemoryStats { physical_mem: 23007232, virtual_mem: 1123815424 })
36s Current memory usage: Some(MemoryStats { physical_mem: 23011328, virtual_mem: 1123815424 })
37s Current memory usage: Some(MemoryStats { physical_mem: 23015424, virtual_mem: 1123815424 })
38s Current memory usage: Some(MemoryStats { physical_mem: 23207936, virtual_mem: 1123815424 })
38s Current memory usage: Some(MemoryStats { physical_mem: 22712320, virtual_mem: 1123299328 })
38s Current memory usage: Some(MemoryStats { physical_mem: 22786048, virtual_mem: 1123459072 })
38s Current memory usage: Some(MemoryStats { physical_mem: 22872064, virtual_mem: 1123688448 })
38s Current memory usage: Some(MemoryStats { physical_mem: 22876160, virtual_mem: 1123688448 })
39s Current memory usage: Some(MemoryStats { physical_mem: 22880256, virtual_mem: 1123688448 })
40s Current memory usage: Some(MemoryStats { physical_mem: 22888448, virtual_mem: 1123688448 })
40s Current memory usage: Some(MemoryStats { physical_mem: 22904832, virtual_mem: 1123688448 })
40s Current memory usage: Some(MemoryStats { physical_mem: 22921216, virtual_mem: 1123688448 })
...

```

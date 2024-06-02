<!-- AUTOMATICALLY GENERATED, DO NOT EDIT -->
<!-- edit README.md.template instead -->

# Rust serialization benchmark

The goal of these benchmarks is to provide thorough and complete benchmarks for various rust
serialization frameworks.

## These benchmarks are a work in progress

These benchmarks are still being developed and pull requests to improve benchmarks are welcome.

## [Interactive site](https://djkoloski.github.io/rust_serialization_benchmark/)

Calculate the number of messages per second that can be sent/received with various rust serialization frameworks and compression libraries.
[Documentation](pages/README.md)

## Format

All tests benchmark the following properties (time or size):

* **Serialize**: serialize data into a buffer
* **Deserialize**: deserializes a buffer into a normal rust object
* **Size**: the size of the buffer when serialized
* **Zlib**: the size of the buffer after zlib compression
* **Zstd**: the size of the buffer after zstd compression
* **Zstd Time**: the time taken to compress the serialized buffer with zstd

Zero-copy deserialization libraries have an additional set of benchmarks:

* **Access**: accesses a buffer as structured data
* **Read**: runs through a buffer and reads fields out of it
* **Update**: updates a buffer as structured data

Some benchmark results may be italicized and followed by an asterisk. Mouse over these for more details on what situation was benchmarked. Other footnotes are located at the bottom.

## Last updated: 2024-6-2 20:36:21

<details><summary>Runtime info</summary>

### `rustc` version

```
rustc 1.77.2 (25ef9e3d8 2024-04-09)
binary: rustc
commit-hash: 25ef9e3d85d934b27d9dada2f9dd52b1dc63bb04
commit-date: 2024-04-09
host: aarch64-apple-darwin
release: 1.77.2
LLVM version: 17.0.6
```

</details>

## `log`

This data set is composed of HTTP request logs that are small and contain many strings.

### Raw data

For operations, time per iteration; for size, bytes. Lower is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 384.31 µs | 1.2600 ms | 724953 | 302399 | 253747 | 2.4216 ms |
| [postcard_forth 0.1.0][postcard-forth] | 265.66 µs | 1.1349 ms | 724953 | 302399 | 253747 | 2.5798 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 69.13% | 90.07% | 100.00% | 100.00% | 100.00% | 100.00% |
| [postcard_forth 0.1.0][postcard-forth] | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% | 93.87% |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

## `mesh`

This data set is a single mesh. The mesh contains an array of triangles, each of which has three vertices and a normal vector.

### Raw data

For operations, time per iteration; for size, bytes. Lower is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 335.42 µs | 1.2317 ms | 6000003 | 5378495 | 5345900 | 7.2406 ms |
| [postcard_forth 0.1.0][postcard-forth] | 393.28 µs | 737.69 µs | 6000003 | 5378495 | 5345900 | 7.2910 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 100.00% | 59.89% | 100.00% | 100.00% | 100.00% | 100.00% |
| [postcard_forth 0.1.0][postcard-forth] | 85.29% | 100.00% | 100.00% | 100.00% | 100.00% | 99.31% |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

## `minecraft_savedata`

This data set is composed of Minecraft player saves that contain highly structured data.

### Raw data

For operations, time per iteration; for size, bytes. Lower is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 375.38 µs | 1.1119 ms | 367489 | 221913 | 207344 | 1.7428 ms |
| [postcard_forth 0.1.0][postcard-forth] | 257.64 µs | 950.03 µs | 367489 | 221913 | 207344 | 1.8475 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 68.63% | 85.44% | 100.00% | 100.00% | 100.00% | 100.00% |
| [postcard_forth 0.1.0][postcard-forth] | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% | 94.33% |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

## `mk48`

This data set is composed of mk48.io game updates that contain data with many exploitable patterns and invariants.

### Raw data

For operations, time per iteration; for size, bytes. Lower is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 1.2014 ms | 2.4270 ms | 1279599 | 1058243 | 1016738 | 5.2347 ms |
| [postcard_forth 0.1.0][postcard-forth] | 691.92 µs | 1.6302 ms | 1279599 | 1058243 | 1016738 | 5.2679 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 57.59% | 67.17% | 100.00% | 100.00% | 100.00% | 100.00% |
| [postcard_forth 0.1.0][postcard-forth] | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% | 99.37% |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

[postcard]: https://crates.io/crates/postcard/1.0.8
[postcard-forth]: https://crates.io/crates/postcard_forth/0.1.0


## Footnotes:

\* *mouse over for situational details*

† *do not provide deserialization capabilities, but the user can write their own*

‡ *do not support buffer mutation (`capnp` and `flatbuffers` may but not for rust)*

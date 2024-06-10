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

## Last updated: 2024-6-10 9:0:50

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
| [postcard 1.0.8][postcard] | 370.34 µs | 1.1861 ms | 724953 | 302399 | 253747 | 2.3519 ms |
| [postcard_forth 0.1.0][postcard-forth] | 250.77 µs | 1.0742 ms | 724953 | 302399 | 253747 | 2.4021 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 67.71% | 90.57% | 100.00% | 100.00% | 100.00% | 100.00% |
| [postcard_forth 0.1.0][postcard-forth] | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% | 97.91% |

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
| [postcard 1.0.8][postcard] | 328.78 µs | 1.2508 ms | 6000003 | 5378495 | 5345900 | 6.4919 ms |
| [postcard_forth 0.1.0][postcard-forth] | 391.09 µs | 705.90 µs | 6000003 | 5378495 | 5345900 | 6.4544 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 100.00% | 56.44% | 100.00% | 100.00% | 100.00% | 99.42% |
| [postcard_forth 0.1.0][postcard-forth] | 84.07% | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% |

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
| [postcard 1.0.8][postcard] | 363.80 µs | 1.0675 ms | 367489 | 221913 | 207344 | 1.7195 ms |
| [postcard_forth 0.1.0][postcard-forth] | 244.14 µs | 920.09 µs | 367489 | 221913 | 207344 | 1.7180 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 67.11% | 86.19% | 100.00% | 100.00% | 100.00% | 99.91% |
| [postcard_forth 0.1.0][postcard-forth] | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% |

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
| [postcard 1.0.8][postcard] | 1.1626 ms | 2.3392 ms | 1279599 | 1058243 | 1016738 | 5.1470 ms |
| [postcard_forth 0.1.0][postcard-forth] | 674.91 µs | 1.5785 ms | 1279599 | 1058243 | 1016738 | 5.1563 ms |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

### Comparison

Relative to best. Higher is better.

#### Serialize / deserialize speed and size

| Crate | Serialize | Deserialize | Size | Zlib | Zstd | Zstd Time |
|---|--:|--:|--:|--:|--:|--:|
| [postcard 1.0.8][postcard] | 58.05% | 67.48% | 100.00% | 100.00% | 100.00% | 100.00% |
| [postcard_forth 0.1.0][postcard-forth] | 100.00% | 100.00% | 100.00% | 100.00% | 100.00% | 99.82% |

#### Zero-copy deserialization speed

| Crate | Access | Read | Update |
|---|--:|--:|--:|

[postcard]: https://crates.io/crates/postcard/1.0.8
[postcard-forth]: https://crates.io/crates/postcard_forth/0.1.0


## Footnotes:

\* *mouse over for situational details*

† *do not provide deserialization capabilities, but the user can write their own*

‡ *do not support buffer mutation (`capnp` and `flatbuffers` may but not for rust)*

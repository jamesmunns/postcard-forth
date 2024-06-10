# Results so far

| case                          | types | text size | input lines | expanded lines | ttl time | crate time |
| :---                          | :---- | :-------- | :---------- | :------------- | :------- | :--------- |
| baseline                      | 0     | 10584     | 0           | 53             | 13.3s    | 0.33s      |
| postcard-serde                | 128   | 164000    | 2664        | 47240          | 20.1s    | 7.03s      |
| postcard-forth                | 128   | 111220    | 2664        | 15931          | 16.4s    | 3.14s      |
| postcard-serde                | 512   | 640456    | 10244       | 181583         | 39.7s    | 25.95s     |
| postcard-forth                | 512   | 395712    | 10244       | 60232          | 24.6s    | 11.82s     |
| postcard-forth (inlined)      | 512   | 389944    | 10244       | 79471          | 24.8s    | 11.78s     |
| postcard-serde (no enums)     | 512   | 550012    | 8248        | 72610          | 33.1s    | 19.7s      |
| postcard-forth (no enums)     | 512   | 223492    | 8248        | 20594          | 19.6s    | 6.82s      |
| postcard-serde (onlyprims)    | 512   | 610800    | 10248       | 177647         | 45.6s    | 32.4s      |
| postcard-forth (onlyprims)    | 512   | 295704    | 10248       | 59645          | 22.3s    | 9.63s      |


## Steps

* regen stress-gen code with `cargo run > ../rp2040-demo/src/gen.rs`
* Timings
    * baseline: `cargo build --release --no-default-features --timings`
    * serde: `cargo build --release --timings`
    * postcard-forth: `cargo build --release --no-default-features --features=postcard-forth --timings`
    * manually copy HTML file to results folder
* Size
    * baseline: `cargo size --release --no-default-features > results-xxx/baseline/size.txt`
    * serde: `cargo size --release > results-xxx/serde/size.txt`
    * postcard-forth: `cargo size --release --no-default-features --features=postcard-forth > results-xxx/postcard-forth/size.txt`
* Generated code size:
    * `cat src/gen.rs | wc -l > results-xxx/yyy/gen-lines.txt`
* Expanded code size:
    * baseline: `cargo expand --release --no-default-features | wc -l > results-xxx/baseline/lines.txt`
    * serde: `cargo expand --release | wc -l > results-xxx/serde/lines.txt`
    * postcard-forth: `cargo expand --release --no-default-features --features=postcard-forth | wc -l > results-xxx/postcard-forth/lines.txt`

## Special tests:

* "no enums": Disabled `enum` generation in stress-gen
* "onlyprims": Don't nest generated types - all generated types only include primitives
* "inlined": After adding `ser_inliner` and `deser_inliner` functions and tweaking derive

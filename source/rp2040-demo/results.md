# Results so far

| case           | types | text size | input lines | expanded lines | ttl time | crate time |
| :---           | :---- | :-------- | :---------- | :------------- | :------- | :--------- |
| baseline       | 0     | 10584     | 0           | 53             | 13.3s    | 0.33s      |
| postcard-serde | 128   | 164000    | 2664        | 47240          | 20.1s    | 7.03s      |
| postcard-forth | 128   | 111220    | 2664        | 15931          | 16.4s    | 3.14s      |
| postcard-serde | 512   | 640456    | 10244       | 181583         | 39.7s    | 25.95s     |
| postcard-forth | 512   | 395712    | 10244       | 60232          | 24.6s    | 11.82s     |

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

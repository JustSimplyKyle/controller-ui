#!/bin/bash
cd "/home/kyle/coding/controller-ui" || exit
cargo b -r
dx bundle --web -r

rm include.rs
touch include.rs

echo "impl AppBuilder for Application {" >> include.rs
echo "type PathRouter = impl routing::PathRouter;" >> include.rs
echo "fn build_app(self) -> picoserve::Router<Self::PathRouter> {" >> include.rs

jq -r '
  .assets.assets 
  | map(.[0]) 
  | {
      # Extract bundled filenames
      css: (map(select(.bundled_path | endswith(".css"))) | .[0].bundled_path),
      js:  (map(select(.bundled_path | endswith(".js")))  | .[0].bundled_path),
      wasm:(map(select(.bundled_path | endswith(".wasm")))| .[0].bundled_path),
      
      # Calculate base path: Take JS absolute path -> split -> remove last 2 parts (/wasm/file.js) -> join
      base:(map(select(.bundled_path | endswith(".js")))  | .[0].absolute_source_path | split("/") | .[:-2] | join("/"))
    } 
  | "static_routes!(
    \"\(.base)\",
    \"index.html\",
    \"assets/\(.css)\",
    \"assets/\(.js)\",
    \"assets/\(.wasm)\"
)"' /home/kyle/coding/controller-ui/target/dx/controller-ui/release/web/.manifest.json >> include.rs


echo ".route(\"/controller\", post(handle_command))" >> include.rs
echo "}}" >> include.rs
mv include.rs "/home/kyle/coding/no-std-esp32"

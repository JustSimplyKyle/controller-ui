cargo b -r
dx bundle --web -r

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
)"' /home/kyle/coding/controller-ui/target/dx/controller-ui/release/web/.manifest.json

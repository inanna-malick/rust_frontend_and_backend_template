

Example project with a rust webserver (warp) serving a rust frontend (yew) as wasm included in the webserver binary using include_bytes.

This is motivated by a desire to share code and types between the frontend (wasm) and backend (x86) code, so it's all in a single workspace.


TODO:
- revise web server section - mildly janky, more of a proof of concept

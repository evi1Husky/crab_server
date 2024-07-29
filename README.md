# Crab Server

Simple dev server with tab reload.

## Features

- single binary with zero config
- uses recursive file system watcher for live reload 
  with simple long polling client and a debouncing mechanism
- serves CORS headers
- support for all common mime types
- multithreaded and efficient
- automatically adds a favicon to the served html
- no caching header

## How To Use

1. Install rust toolchain and run cargo build --release in the repository root.
2. Put the compiled exe in your web project directory and launch it from there.
3. Go to http://[::1]:8080/index.html or whatever your project entry is.
4. You can run tsc --watch or other process along with the server and pipe its output 
   to the main process like this ./crab_server --run="npx tsc --watch" --reloadDelay=300.
5. The file watcher listens on :8087 and sends updates to the client using long polling.

#!/bin/bash

wasm-pack build --target bundler --scope mrcoolthecucumber --release
cp .npmrc pkg/npmrc
npm publish --access public --dry-run 
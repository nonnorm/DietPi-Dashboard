#!/bin/bash -ex

asset_path='crates/server/assets'
dist_path='crates/server/dist'

js_assets=(
  "$asset_path/js/xterm-5.5.0.js"
  "$asset_path/js/microlight-0.0.7.js"
  "$asset_path/js/components.js"
)

css_assets=(
  "$asset_path/css/vars-clean.css"
  "$asset_path/css/global.css"
  "$asset_path/css/system.css"
  "$asset_path/css/process.css"
  "$asset_path/css/management.css"
  "$asset_path/css/software.css"
  "$asset_path/css/xterm-5.5.0.css"
)

svg_src="$asset_path/icons.svg"

js_out="$dist_path/main.js"
css_out="$dist_path/main.css"
svg_out="$dist_path/icons.svg"

mkdir "$dist_path"

./scripts/clean-css.bash "${css_assets[@]:1}" > "${css_assets[0]}"

cat "${js_assets[@]}" | gzip -9 > "$js_out"
cat "${css_assets[@]}" | gzip -9 > "$css_out"
cat "$svg_src" | gzip -9 > "$svg_out"

rm "${css_assets[0]}"

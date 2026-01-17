#!/usr/bin/env bash

# Based on this comment:
# https://github.com/TechnikTobi/little_exif/issues/83#issuecomment-3763256142

# Exit the script immediately in case any command 
# returns a non-zero (i.e. error) status
set -e

# Check that an argument was provided
if [ $# -ne 1 ]; then
  echo "Usage: $0 /path/to/image"
  exit 1
fi

input="$1"

# Extract directory, filename, base name, and extension
dir="$(dirname "$input")"
filename="$(basename "$input")"
base="${filename%.*}"
ext="${filename##*.}"

# Construct output path
output="${dir}/${base}_resized.${ext}"

# Perform resize
magick "$input" -resize '64x64>' "$output"

echo "Resized image written to: $output"

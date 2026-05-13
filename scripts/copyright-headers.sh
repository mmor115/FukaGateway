#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(git -C "$script_dir" rev-parse --show-toplevel)"
header_file="$script_dir/copyright_header.txt"

if [[ ! -f "$header_file" ]]; then
  echo "Missing header file: $header_file" >&2
  exit 1
fi

header_comment_tmp="$(mktemp)"
prefix_tmp="$(mktemp)"

cleanup() {
  rm -f "$header_comment_tmp" "$prefix_tmp"
}
trap cleanup EXIT

while IFS= read -r line || [[ -n "$line" ]]; do
  printf '// %s\n' "$line" >> "$header_comment_tmp"
done < "$header_file"

cat "$header_comment_tmp" > "$prefix_tmp"
printf '\n' >> "$prefix_tmp"

header_comment_size="$(wc -c < "$header_comment_tmp")"
prefix_size="$(wc -c < "$prefix_tmp")"

while IFS= read -r -d '' rel_path; do
  file_path="$repo_root/$rel_path"
  [[ -f "$file_path" ]] || continue

  file_size="$(wc -c < "$file_path")"

  if (( file_size >= prefix_size )) && head -c "$prefix_size" "$file_path" | cmp -s - "$prefix_tmp"; then
    continue
  fi

  tmp_out="$(mktemp)"

  if (( file_size >= header_comment_size )) && head -c "$header_comment_size" "$file_path" | cmp -s - "$header_comment_tmp"; then
    {
      cat "$prefix_tmp"
      if (( file_size > header_comment_size )); then
        tail -c "+$((header_comment_size + 1))" "$file_path" | perl -0pe 's/\A\n+//'
      fi
    } > "$tmp_out"
  else
    {
      cat "$prefix_tmp"
      cat "$file_path"
    } > "$tmp_out"
  fi

  mv "$tmp_out" "$file_path"
done < <(git -C "$repo_root" ls-files -z -- '*.rs')

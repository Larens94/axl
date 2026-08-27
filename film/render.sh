#!/bin/sh
set -eu

film_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
root_dir=$(dirname "$film_dir")
render_dir="$film_dir/rendered"
concat_list=$(mktemp /tmp/axl-film-concat.XXXXXX)
trap 'rm -f "$concat_list"' EXIT

mkdir -p "$render_dir"

while IFS='|' read -r scene narration; do
  [ -n "$scene" ] || continue
  number=$(printf '%02d' "$scene")
  image="$render_dir/scene-$number.png"
  audio="$render_dir/scene-$number.aiff"
  segment="$render_dir/scene-$number.mp4"

  if [ -f "$segment" ] && [ -f "$image" ] && [ -f "$audio" ]; then
    printf "Reuse scene %s\n" "$number"
  else
    playwright screenshot \
      --viewport-size="1920,1080" \
      --wait-for-timeout=900 \
      "file://$root_dir/film.html?scene=$scene&capture=1" \
      "$image" < /dev/null

    say -v Alice -r 176 -o "$audio" "$narration" < /dev/null
    audio_duration=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$audio" < /dev/null)
    total_duration=$(awk -v value="$audio_duration" 'BEGIN { printf "%.3f", value + 1.20 }')
    fade_out=$(awk -v value="$total_duration" 'BEGIN { printf "%.3f", value - 0.55 }')
    audio_fade=$(awk -v value="$audio_duration" 'BEGIN { printf "%.3f", value - 0.45 }')

    ffmpeg -nostdin -y -loglevel error \
      -loop 1 -framerate 30 -i "$image" -i "$audio" \
      -t "$total_duration" \
      -vf "scale=1920:1080:flags=lanczos,format=yuv420p,fade=t=in:st=0:d=0.45,fade=t=out:st=$fade_out:d=0.55" \
      -af "afade=t=in:st=0:d=0.25,afade=t=out:st=$audio_fade:d=0.45,apad=pad_dur=1.2" \
      -c:v libx264 -preset medium -crf 19 -r 30 -c:a aac -b:a 160k \
      "$segment" < /dev/null
  fi

  printf "file '%s'\n" "$segment" >> "$concat_list"
done < "$film_dir/scenes.txt"

ffmpeg -nostdin -y -loglevel error -f concat -safe 0 -i "$concat_list" \
  -c copy -movflags +faststart \
  -metadata title="AXL — Il piano completo" \
  -metadata language="ita" \
  "$film_dir/axl-plan-film.mp4"

cp "$render_dir/scene-00.png" "$film_dir/poster.png"
printf 'Creati %s e %s\n' "$film_dir/axl-plan-film.mp4" "$film_dir/poster.png"

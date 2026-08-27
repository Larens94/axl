# AXL project film

The film has two synchronized deliverables:

- `../film.html`: responsive, interactive and automatically advancing browser film;
- `axl-plan-film.mp4`: narrated Italian 1080p export.

`scenes.txt` is the narration source. Scene order matches `film.html`. Rebuild
the binary assets on macOS with:

```sh
./film/render.sh
```

Requirements: Playwright with Chromium, `say`, FFmpeg and FFprobe. Generated
intermediate frames, audio and scene segments live under ignored
`film/rendered/`. The final MP4 and poster are versioned deliverables.
`render.sh` reuses complete scene segments, so an interrupted export can be
resumed without rebuilding earlier scenes.

The film deliberately distinguishes implemented capabilities from planned
gates. Update it whenever a gate changes status, together with the main
presentation and documentation. After editing `film.html` or `scenes.txt`,
delete the affected `film/rendered/scene-NN.*` files before re-running
`./film/render.sh` so those scenes are regenerated.

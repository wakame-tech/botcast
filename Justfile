
run:
  deno run -A --location http://localhost src/index.ts

script:
  deno run -A --location http://localhost test/script.ts

moviefy mp3:
  ffmpeg \
    -loop 1 \
    -r 30000/1001 \
    -i 500x500.png -i {{mp3}} \
    -vcodec libx264 \
    -acodec aac -strict experimental -ab 320k -ac 2 -ar 48000 \
    -pix_fmt yuv420p \
    -shortest \
    output.mp4
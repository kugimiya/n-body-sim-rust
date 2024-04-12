cd output;
ffmpeg -framerate 30 -pattern_type glob -i '*.png'  -c:v libx264 -pix_fmt yuv420p -y out.mp4;
ffmpeg -i out.mp4 -c:v libx264 -b:v 50000 -aspect 1:1 -crf 23 -movflags faststart -vf "scale=2560:-1:flags=lanczos" -c:a copy -y out2.mp4;
cd ..;

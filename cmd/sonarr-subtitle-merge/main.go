package main

import (
	"log/slog"
	"os"
	"os/exec"
)

func main() {
	slog.Info("starting sonarr-subtitle-merge")

	ffprobeBin, err := exec.LookPath("ffprobe")
	if err != nil {
		slog.Error("error looking for ffprobe", slog.String("error", err.Error()))
		os.Exit(1)
	}
	slog.Info("ffprobe found", slog.String("path", ffprobeBin))

	ffmpegBin, err := exec.LookPath("ffmpeg")
	if err != nil {
		slog.Error("error looking for ffmpeg", slog.String("error", err.Error()))
		os.Exit(1)
	}
	slog.Info("ffmpeg found", slog.String("path", ffmpegBin))
}

package main

import (
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
)

var languageFilter = map[string]struct{}{
	"chi": {},
	"en":  {},
	"eng": {},
	"zh":  {},
	"zho": {},
}
var ReSubStreams = *regexp.MustCompile(`Stream #(\d+:\d+).*?\((\w+)\).*?Subtitle: (\w+)`)

func main() {
	slog.Info("starting sonarr-subtitle-merge")

	// check event type

	eventtype := os.Getenv("sonarr_eventtype")
	if eventtype == "" {
		slog.Error("eventtype cannot be empty")
		os.Exit(1)
	}
	slog.Info("got event type", slog.String("eventtype", eventtype))
	if eventtype == "Test" {
		slog.Info("test event, exiting")
		os.Exit(0)
	}
	if eventtype != "Download" {
		slog.Info("only download events supported")
		os.Exit(1)
	}

	// look for binaries

	ffprobeBin, err := exec.LookPath("ffprobe")
	if err != nil {
		slog.Error("error looking for ffprobe", slog.String("error", err.Error()))
		os.Exit(1)
	}
	slog.Info("found ffprobe", slog.String("path", ffprobeBin))

	ffmpegBin, err := exec.LookPath("ffmpeg")
	if err != nil {
		slog.Error("error looking for ffmpeg", slog.String("error", err.Error()))
		os.Exit(1)
	}
	slog.Info("found ffmpeg", slog.String("path", ffmpegBin))

	// ensure file

	episodefile := os.Getenv("sonarr_episodefile_path")
	if episodefile == "" {
		slog.Error("episode file cannot be empty")
		os.Exit(1)
	}
	slog.Info("got episode file", slog.String("episodefile", episodefile))

	if _, err := os.Stat(episodefile); err != nil {
		slog.Error("episode file may not exist", slog.String("error", err.Error()))
		os.Exit(1)
	}

	// find and dump subtitles

	streams, err := getSubtitleStreams(episodefile)
	if err != nil {
		slog.Error("error getting subtitle streams", slog.String("error", err.Error()))
		os.Exit(1)
	}

	for _, stream := range streams {
		slog.Info("got subtitle stream", slog.String("id", stream.StreamID), slog.String("language", stream.Language), slog.String("format", stream.Format))
		if _, found := languageFilter[stream.Language]; !found {
			continue
		}
		dumpSubtitleFile(stream)
	}
	slog.Info("dumped all subtitles")
}

func fileNameWithoutExtension(fileName string) string {
	return strings.TrimSuffix(fileName, filepath.Ext(fileName))
}

func getSubtitleStreams(file string) ([]SubtitleStream, error) {
	cmd := exec.Command("ffprobe", "-i", file)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return nil, err
	}

	matches := ReSubStreams.FindAllStringSubmatch(string(output), -1)
	if matches == nil {
		return nil, fmt.Errorf("subtitle stream regex had no matches")
	}

	var streams = make([]SubtitleStream, len(matches))
	for i, match := range matches {
		streams[i] = SubtitleStream{
			SourceFile: file,
			StreamID:   match[1],
			Language:   match[2],
			Format:     match[3],
		}
	}

	return streams, nil
}

func dumpSubtitleFile(stream SubtitleStream) error {
	fileStem := fileNameWithoutExtension(stream.SourceFile)
	idClean := strings.ReplaceAll(stream.StreamID, ":", "_")
	subFormat := "ass" // currently assume all can be converted into ass
	subExt := fmt.Sprintf(".%s.%s.%s", idClean, stream.Language, subFormat)
	subFile := fmt.Sprintf("%s%s", fileStem, subExt)

	cmd := exec.Command("ffmpeg", "-i", stream.SourceFile, "-map", stream.StreamID, "-c:s", subFormat, subFile)
	if err := cmd.Run(); err != nil {
		return err
	}

	return nil
}

type SubtitleStream struct {
	SourceFile string
	StreamID   string
	Language   string
	Format     string
}

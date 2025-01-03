package main
import (
  "fmt"
  "net/url"
  "os"
  "os/exec"
  "regexp"
  "strings"
  "syscall"
)
var (
  tagFmt = regexp.MustCompile("^[a-zA-Z0-9_\\-.]+$")
)
func main() {
  args := os.Args
  kanikoBin := "/kaniko/executor"
  if path, ok := os.LookupEnv("KANIKO_BIN"); ok {
      _ = os.Unsetenv("KANIKO_BIN")
      kanikoBin = path
  }
  repository := ""
  if found, ok := os.LookupEnv("KANIKO_IMAGE_REPOSITORY"); ok {
      _ = os.Unsetenv("KANIKO_IMAGE_REPOSITORY")
      repository = strings.TrimSpace(found)
  }
  name := ""
  if found, ok := os.LookupEnv("KANIKO_IMAGE_NAME"); ok {
      _ = os.Unsetenv("KANIKO_IMAGE_NAME")
      name = strings.TrimSpace(found)
  }
  tags := []string{"latest"}
  if found, ok := os.LookupEnv("KANIKO_IMAGE_TAGS"); ok {
      _ = os.Unsetenv("KANIKO_IMAGE_TAGS")
      for _, rawTag := range strings.Split(found, ",") {
          tag := strings.TrimSpace(rawTag)
          if !tagFmt.MatchString(tag) {
              continue
          } else {
              tags = append(tags, tag)
          }
      }
  }
  if image, _ := url.JoinPath(repository, name); len(image) > 0 {
      for _, tag := range tags {
          args = append(args, fmt.Sprintf("--destination=%s:%s", image, tag))
      }
  }
  if path, ok := os.LookupEnv(kanikoBin); ok {
      _ = os.Unsetenv("KANIKO_BIN")
      kanikoBin = path
  }
  if foundPath, err := exec.LookPath(kanikoBin); err != nil {
      panic(fmt.Sprintf("failed to find path %s, %s", kanikoBin, err))
  } else {
      kanikoBin = foundPath
  }
  args[0] = kanikoBin
  fmt.Println(strings.Join(args, " "))
  if err := syscall.Exec(kanikoBin, args, os.Environ()); err != nil {
      panic(fmt.Sprintf("kaniko exec failed: %s", err))
  }
}

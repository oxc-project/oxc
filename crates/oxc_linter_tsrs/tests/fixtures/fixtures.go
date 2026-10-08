package fixtures

import (
	"path"
	"runtime"
)

func GetRootDir() string {
	_, filename, _, _ := runtime.Caller(0)
	return path.Dir(filename)
}

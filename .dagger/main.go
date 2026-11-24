package main

import (
	"context"
	"fmt"
	"strings"

	"dagger/lesscode-ci/internal/dagger"
)

// LesscodeCi runs the same native checks locally and in GitHub Actions.
type LesscodeCi struct{}

// Evidence exports logs and exit codes even when a correctness check fails.
// Use Check to enforce the recorded result.
func (m *LesscodeCi) Evidence(
	// +defaultPath="/"
	// +ignore=["**", "!native/**", "!model/**", "!tools/check_native.sh", "!tools/install_nextest.sh", "!tools/guide-optimizer/**", "!docs/experiments/annotation-driver/pack-guide-v3.md", "native/target/**", "model/target/**", "tools/guide-optimizer/target/**"]
	source *dagger.Directory,
	revision string,
) *dagger.Directory {
	return dag.Container().
		From("rust:1.98.1-bookworm").
		WithExec([]string{"rustup", "component", "add", "rustfmt", "clippy"}).
		WithFile("/workspace/tools/install_nextest.sh", source.File("tools/install_nextest.sh")).
		WithExec([]string{"bash", "/workspace/tools/install_nextest.sh"}).
		WithMountedCache("/usr/local/cargo/registry", dag.CacheVolume("lesscode-cargo-registry-1.98.1")).
		WithDirectory("/workspace/native", source.Directory("native")).
		// The seam: its source scan, and the guide optimiser's dependency on it.
		WithDirectory("/workspace/model", source.Directory("model")).
		WithFile("/workspace/tools/check_native.sh", source.File("tools/check_native.sh")).
		WithDirectory("/workspace/tools/guide-optimizer", source.Directory("tools/guide-optimizer")).
		// The one document the optimiser's dry run reads.
		WithFile("/workspace/docs/experiments/annotation-driver/pack-guide-v3.md", source.File("docs/experiments/annotation-driver/pack-guide-v3.md")).
		WithMountedCache("/workspace/native/target", dag.CacheVolume("lesscode-native-target-1.98.1")).
		WithEnvVariable("GITHUB_SHA", revision).
		// Cargo decides by a file's time whether to build again, and the files of a source directory
		// arrive here without the time of their change. With the kept target directory a changed
		// test or source file was then not built, and the binaries of an earlier run were tested
		// (seen 2026-11-24: "Finished in 0.10s", and two new tests missing from the log). The
		// core's own files get the time of this run, so the core is built in every run; its
		// dependencies stay built.
		WithExec([]string{"find", "/workspace/native", "-path", "/workspace/native/target", "-prune", "-o", "-type", "f", "-exec", "touch", "{}", "+"}).
		// The seam's and the guide optimiser's checks run in this job only; no other platform compiles the client library.
		WithEnvVariable("LESSCODE_CHECK_SEAM", "1").
		WithWorkdir("/workspace").
		WithExec([]string{"bash", "tools/check_native.sh", "/evidence"}, dagger.ContainerWithExecOpts{Expect: dagger.ReturnTypeAny}).
		Directory("/evidence")
}

// Check fails when any native check failed. Evidence remains separately exportable.
func (m *LesscodeCi) Check(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["**", "!native/**", "!model/**", "!tools/check_native.sh", "!tools/install_nextest.sh", "!tools/guide-optimizer/**", "!docs/experiments/annotation-driver/pack-guide-v3.md", "native/target/**", "model/target/**", "tools/guide-optimizer/target/**"]
	source *dagger.Directory,
	revision string,
) (string, error) {
	code, err := m.Evidence(source, revision).File("overall.exit-code").Contents(ctx)
	if err != nil {
		return "", err
	}
	if strings.TrimSpace(code) != "0" {
		return "", fmt.Errorf("native checks failed; export evidence for individual logs and exit codes")
	}
	return "All native checks passed", nil
}

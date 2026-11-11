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
	// +ignore=["**", "!native/**", "!tools/check_native.sh", "!tools/install_nextest.sh", "!tools/guide-optimizer/**", "native/target/**", "tools/guide-optimizer/target/**"]
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
		WithFile("/workspace/tools/check_native.sh", source.File("tools/check_native.sh")).
		WithDirectory("/workspace/tools/guide-optimizer", source.Directory("tools/guide-optimizer")).
		WithMountedCache("/workspace/native/target", dag.CacheVolume("lesscode-native-target-1.98.1")).
		WithEnvVariable("GITHUB_SHA", revision).
		WithWorkdir("/workspace").
		WithExec([]string{"bash", "tools/check_native.sh", "/evidence"}, dagger.ContainerWithExecOpts{Expect: dagger.ReturnTypeAny}).
		Directory("/evidence")
}

// Check fails when any native check failed. Evidence remains separately exportable.
func (m *LesscodeCi) Check(
	ctx context.Context,
	// +defaultPath="/"
	// +ignore=["**", "!native/**", "!tools/check_native.sh", "!tools/install_nextest.sh", "!tools/guide-optimizer/**", "native/target/**", "tools/guide-optimizer/target/**"]
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

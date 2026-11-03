package command

import (
	"bytes"
	"strings"
	"testing"

	"github.com/Javier-GarciaP/sprixel/cli/internal/testapp"
)

// Checks that "sprixel mcp" with no arguments connects to the app with the
// token file and answers on stdout (scenario "No token argument").
func TestMCPBridgeNeedsNoArguments(t *testing.T) {
	app := testapp.Start(t, t.TempDir()+"/.sprixel")
	home := strings.TrimSuffix(app.Dir, "/.sprixel")
	input := `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"claude-code","version":"1"}}}` + "\n" +
		`{"jsonrpc":"2.0","id":2,"method":"tools/list"}` + "\n"

	var out, errOut bytes.Buffer
	code := Run([]string{"mcp"}, Env{Stdin: strings.NewReader(input), Stdout: &out, Stderr: &errOut, Home: home})

	if code != ExitOK {
		t.Fatalf("exit code = %d, stderr = %q", code, errOut.String())
	}
	lines := strings.Split(strings.TrimSpace(out.String()), "\n")
	if len(lines) != 2 || !strings.Contains(lines[1], "list_sprites") {
		t.Errorf("stdout = %q", out.String())
	}
}

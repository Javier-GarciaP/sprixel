package command

import (
	"encoding/json"
	"fmt"
	"io"
)

// printer writes the result of a command, as text or as JSON (--json).
// In JSON mode stdout holds one JSON value and nothing else, also for errors.
type printer struct {
	out, err io.Writer
	json     bool
}

// Print writes a result. data is used in JSON mode, text otherwise.
func (p *printer) Print(text string, data any) {
	if p.json {
		p.writeJSON(data)
		return
	}
	io.WriteString(p.out, text)
}

// Fail writes an error and returns its exit code. kind is a short stable
// word for scripts, message says what went wrong and hint how to fix it.
func (p *printer) Fail(code int, kind, message, hint string) int {
	if p.json {
		e := map[string]string{"code": kind, "message": message}
		if hint != "" {
			e["hint"] = hint
		}
		p.writeJSON(map[string]any{"error": e})
		return code
	}
	fmt.Fprintf(p.err, "error: %s\n", message)
	if hint != "" {
		fmt.Fprintf(p.err, "hint: %s\n", hint)
	}
	return code
}

func (p *printer) writeJSON(data any) {
	bytes, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		bytes = []byte(`{"error":{"code":"internal","message":"cannot encode the result"}}`)
	}
	p.out.Write(append(bytes, '\n'))
}

// splitFlags removes the global flags --json and --help (or -h) from args,
// wherever they are, and returns the rest in the same order.
func splitFlags(args []string) (rest []string, asJSON, help bool) {
	for _, a := range args {
		switch a {
		case "--json":
			asJSON = true
		case "--help", "-h":
			help = true
		default:
			rest = append(rest, a)
		}
	}
	return rest, asJSON, help
}

package command

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/Javier-GarciaP/sprixel/cli/internal/appclient"
	"github.com/Javier-GarciaP/sprixel/cli/internal/launcher"
	"github.com/Javier-GarciaP/sprixel/cli/internal/runtime"
	"github.com/Javier-GarciaP/sprixel/cli/internal/version"
)

func init() {
	commands = append(commands,
		command{name: "start", summary: "Start the app (it does not open a project yet)", usage: "sprixel start [path] [--json]", run: runStart},
		command{name: "status", summary: "Say whether the app is running", usage: "sprixel status [--json]", run: runStatus},
		command{name: "tools", summary: "List the tools an AI can use", usage: "sprixel tools [--json]", run: runTools},
		command{name: "tool", summary: "Run one tool: sprixel tool <name> key=value ...", usage: "sprixel tool <name> [key=value ...] [--json]", run: runTool},
	)
}

const notRunningHint = "start it with `sprixel start`"

func (c *invocation) appClient() *appclient.Client {
	return &appclient.Client{Dir: c.dir, Version: version.Version}
}

// notRunning prints the "app closed" answer with the right exit code.
func (c *invocation) notRunning() int {
	return c.p.Fail(ExitNotRunning, "not_running", "the Sprixel app is not running", notRunningHint)
}

// appFailure turns an error of the app client into an exit code.
func (c *invocation) appFailure(err error) int {
	if errors.Is(err, appclient.ErrNotRunning) {
		return c.notRunning()
	}
	hint := ""
	var refused *appclient.ServerError
	if errors.As(err, &refused) && refused.Code == "unauthorized" {
		hint = "close the app and start it again, then run `sprixel doctor`"
	}
	return c.p.Fail(ExitFailure, "app", err.Error(), hint)
}

func runStatus(c *invocation, args []string) int {
	if code, bad := noArgs(c, "sprixel status [--json]", args); bad {
		return code
	}
	info, running := runtime.Running(c.dir)
	if !running {
		c.p.Print("Sprixel is not running.\n", map[string]any{"running": false})
		return ExitNotRunning
	}
	v := info.Version
	if v == "" {
		v = "unknown"
	}
	c.p.Print(fmt.Sprintf("Sprixel is running (port %d, version %s).\n", info.Port, v),
		map[string]any{"running": true, "port": info.Port, "pid": info.PID, "version": info.Version})
	return ExitOK
}

func runStart(c *invocation, args []string) int {
	if len(args) > 1 {
		return c.p.Fail(ExitUsage, "usage", "unexpected argument: "+args[1], "usage: sprixel start [path]")
	}
	// The app does not read a project path from its command line yet, so the
	// path is checked but not passed: saying "opened" would be a lie. The
	// person opens it in the app, or an agent calls `open_project`.
	ignored := ""
	if len(args) == 1 {
		path, err := projectPath(args[0])
		if err != nil {
			return c.p.Fail(ExitUsage, "usage", err.Error(), "give the folder of a Sprixel project (it has a sprixel.json file)")
		}
		ignored = path
	}
	if info, ok := runtime.Running(c.dir); ok {
		return c.printStart("Sprixel is already running", false, info, ignored)
	}
	exited, err := c.startApp()
	if err != nil {
		return c.p.Fail(ExitFailure, "start", err.Error(), "")
	}
	info, err := runtime.WaitReady(context.Background(), c.dir, exited, 20*time.Second, 100*time.Millisecond)
	if err != nil {
		return c.p.Fail(ExitFailure, "start", err.Error(), "")
	}
	return c.printStart("Sprixel started", true, info, ignored)
}

// printStart tells which build runs: the path of the app (when this command
// can find it), its version, and a warning when the version is not the one of
// this command. An old build left in another place is the usual reason.
func (c *invocation) printStart(headline string, started bool, info runtime.Info, ignoredProject string) int {
	app, err := launcher.Find(launcher.System(c.env.Home))
	if err != nil {
		app = ""
	}
	v := info.Version
	if v == "" {
		v = "unknown"
	}
	text := fmt.Sprintf("%s (port %d, version %s).\n", headline, info.Port, v)
	data := map[string]any{"running": true, "started": started, "port": info.Port, "version": info.Version}
	if app != "" {
		text += "  app: " + app + "\n"
		data["path"] = app
	}
	if info.Version != "" && info.Version != version.Version {
		warning := fmt.Sprintf("the app is version %s but this command is %s. It may be an old build: install the same version of both, or set SPRIXEL_APP to the path of the app you want", info.Version, version.Version)
		text += "warning: " + warning + "\n"
		data["warning"] = warning
	}
	if ignoredProject != "" {
		note := "the app does not open a project from the command line yet, so " + ignoredProject + " was not opened. Open it in the app (Open project), or let an AI agent call open_project"
		text += "note: " + note + "\n"
		data["note"] = note
		data["project_opened"] = false
	}
	c.p.Print(text, data)
	return ExitOK
}

// projectPath checks that a path is a Sprixel project folder and makes it
// absolute (the app starts in another folder).
func projectPath(path string) (string, error) {
	abs, err := filepath.Abs(path)
	if err != nil {
		return "", err
	}
	info, err := os.Stat(abs)
	if err != nil || !info.IsDir() {
		return "", errors.New("this folder does not exist: " + path)
	}
	if _, err := os.Stat(filepath.Join(abs, "sprixel.json")); err != nil {
		return "", errors.New("this folder is not a Sprixel project (no sprixel.json): " + path)
	}
	return abs, nil
}

func runTools(c *invocation, args []string) int {
	if code, bad := noArgs(c, "sprixel tools [--json]", args); bad {
		return code
	}
	client := c.appClient()
	defer client.Close()
	tools, err := client.ListTools(context.Background())
	if err != nil {
		return c.appFailure(err)
	}
	if len(tools) == 0 {
		return c.p.Fail(ExitFailure, "awaiting_approval", "the app lists no tools: it has not allowed this command yet",
			"press Allow for \""+appclient.ClientName+"\" in the Agents tab of the Sprixel app, then try again")
	}
	var text strings.Builder
	width := 0
	for _, t := range tools {
		width = max(width, len(t.Name))
	}
	for _, t := range tools {
		text.WriteString(pad(t.Name, width+2) + firstSentence(t.Description) + "\n")
	}
	c.p.Print(text.String(), map[string]any{"tools": tools})
	return ExitOK
}

func firstSentence(text string) string {
	if i := strings.Index(text, ". "); i >= 0 {
		return text[:i+1]
	}
	return strings.TrimSpace(strings.SplitN(text, "\n", 2)[0])
}

func runTool(c *invocation, args []string) int {
	const usage = "usage: sprixel tool <name> [key=value ...]"
	if len(args) == 0 {
		return c.p.Fail(ExitUsage, "usage", "say which tool to run", usage)
	}
	name, pairs := args[0], args[1:]
	client := c.appClient()
	defer client.Close()
	ctx := context.Background()

	tools, err := client.ListTools(ctx)
	if err != nil {
		return c.appFailure(err)
	}
	var schema json.RawMessage
	found := false
	for _, t := range tools {
		if t.Name == name {
			schema, found = t.InputSchema, true
		}
	}
	// A client that is not allowed sees no tools: the call itself gives the
	// app's own "awaiting_approval" answer, so it is not blocked here.
	if len(tools) > 0 && !found {
		names := make([]string, len(tools))
		for i, t := range tools {
			names[i] = t.Name
		}
		sort.Strings(names)
		return c.p.Fail(ExitUsage, "usage", "there is no tool "+strconv.Quote(name), "the tools are: "+strings.Join(names, ", "))
	}
	arguments, err := parseArguments(pairs, schema)
	if err != nil {
		return c.p.Fail(ExitUsage, "usage", err.Error(), usage)
	}
	result, err := client.CallTool(ctx, name, arguments)
	if err != nil {
		return c.appFailure(err)
	}
	return c.printToolResult(result)
}

// parseArguments turns key=value words into the arguments of a tool. The type
// of each value comes from the schema of the tool: a string stays text (so a
// sprite called "123" is not a number); numbers, booleans, lists and objects
// are read as JSON. Without a schema a value is JSON when it can be, else text.
func parseArguments(pairs []string, schema json.RawMessage) (map[string]any, error) {
	var parsed struct {
		Properties map[string]struct {
			Type any `json:"type"`
		} `json:"properties"`
	}
	json.Unmarshal(schema, &parsed)
	arguments := map[string]any{}
	for _, pair := range pairs {
		key, value, ok := strings.Cut(pair, "=")
		if !ok || key == "" {
			return nil, errors.New("write each argument as key=value, not " + strconv.Quote(pair))
		}
		kind, _ := parsed.Properties[key].Type.(string)
		switch kind {
		case "string":
			arguments[key] = value
		case "integer", "number", "boolean", "array", "object":
			var v any
			if err := json.Unmarshal([]byte(value), &v); err != nil {
				return nil, fmt.Errorf("%s must be %s, for example %s", key, kind, example(kind))
			}
			arguments[key] = v
		default:
			var v any
			if json.Unmarshal([]byte(value), &v) == nil {
				arguments[key] = v
			} else {
				arguments[key] = value
			}
		}
	}
	return arguments, nil
}

func example(kind string) string {
	return map[string]string{"integer": "3", "number": "1.5", "boolean": "true", "array": `'[1,2]'`, "object": `'{"a":1}'`}[kind]
}

// printToolResult prints what a tool returned. A tool error (isError) is exit
// code 4. JSON output is {"is_error", "result", "images"}: result is the JSON
// the tool printed, or its text.
func (c *invocation) printToolResult(result appclient.Result) int {
	var texts []string
	var images []map[string]any
	for _, part := range result.Content {
		switch part.Type {
		case "text":
			texts = append(texts, part.Text)
		case "image":
			images = append(images, map[string]any{"mime_type": part.MimeType, "data": part.Data})
		}
	}
	joined := strings.Join(texts, "\n")
	// Images become files, so an AI that only has a shell can open them. The
	// text of the same result (the info of the preview) names the file.
	var saveErrors []string
	for _, image := range images {
		if image["mime_type"] != "image/png" {
			continue
		}
		if path, err := c.savePreview(image["data"].(string), joined); err != nil {
			saveErrors = append(saveErrors, err.Error())
		} else {
			image["path"] = path
		}
	}
	var value any = joined
	var decoded any
	if json.Unmarshal([]byte(joined), &decoded) == nil {
		value = decoded
	}
	code := ExitOK
	if result.IsError {
		code = ExitToolError
	}
	if c.p.json {
		out := map[string]any{"is_error": result.IsError, "result": value}
		if len(images) > 0 {
			out["images"] = images
		}
		c.p.writeJSON(out)
		return code
	}
	pretty := joined
	if decoded != nil {
		if b, err := json.MarshalIndent(decoded, "", "  "); err == nil {
			pretty = string(b)
		}
	}
	out := c.env.Stdout
	if result.IsError {
		out = c.env.Stderr
	}
	if pretty != "" {
		fmt.Fprintln(out, pretty)
	}
	for _, image := range images {
		if path, ok := image["path"]; ok {
			fmt.Fprintf(out, "image saved: %s\n", path)
			continue
		}
		fmt.Fprintf(out, "[image %s, %d base64 characters: use --json to get it]\n", image["mime_type"], len(image["data"].(string)))
	}
	for _, problem := range saveErrors {
		fmt.Fprintln(c.env.Stderr, "warning: could not save the image: "+problem)
	}
	return code
}

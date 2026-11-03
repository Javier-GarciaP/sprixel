package launcher

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

// Only one process may start the app: the second one must be told to wait.
func TestClaimStartAllowsOnlyOneProcess(t *testing.T) {
	dir := t.TempDir()
	release, ok := ClaimStart(dir, time.Minute)
	if !ok {
		t.Fatal("the first claim must succeed")
	}
	if _, second := ClaimStart(dir, time.Minute); second {
		t.Fatal("the second claim must fail while the first is fresh")
	}
	release()
	if _, again := ClaimStart(dir, time.Minute); !again {
		t.Fatal("after release the claim must work again")
	}
}

// A claim nobody released (the process died) must not block forever.
func TestClaimStartIgnoresAnOldClaim(t *testing.T) {
	dir := t.TempDir()
	if _, ok := ClaimStart(dir, time.Minute); !ok {
		t.Fatal("first claim")
	}
	old := time.Now().Add(-2 * time.Minute)
	if err := os.Chtimes(filepath.Join(dir, lockName), old, old); err != nil {
		t.Fatal(err)
	}
	if _, ok := ClaimStart(dir, time.Minute); !ok {
		t.Fatal("an old claim must be taken over")
	}
}

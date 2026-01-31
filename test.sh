#!/bin/bash
# Comprehensive test script for gitnu

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

echo_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

# Setup
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TEST_DIR="/tmp/gitnu-test-$(date +%s)"
GNU_BIN="$SCRIPT_DIR/target/release/gnu"

if [ ! -f "$GNU_BIN" ]; then
    echo "Building gnu binary..."
    cd "$SCRIPT_DIR"
    cargo build --release
fi

mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Test 1: Initialize vault
echo_test "Initializing vault"
$GNU_BIN init --name test-project
echo_pass "Vault initialized"

# Test 2: Check status
echo_test "Checking status"
$GNU_BIN status
echo_pass "Status displayed"

# Test 3: Add content and commit
echo_test "Adding content and committing"
echo "# Feature: Authentication" >> domains/test-project/spec.md
$GNU_BIN commit "Added auth feature" --author human
echo_pass "Committed successfully"

# Test 4: View log
echo_test "Viewing commit log"
$GNU_BIN log --oneline
echo_pass "Log displayed"

# Test 5: Create branch
echo_test "Creating branch"
$GNU_BIN branch feature-api --describe "API development branch"
echo_pass "Branch created"

# Test 6: List branches
echo_test "Listing branches"
$GNU_BIN branch
echo_pass "Branches listed"

# Test 7: Checkout branch
echo_test "Checking out branch"
$GNU_BIN checkout feature-api
echo_pass "Branch checked out"

# Test 8: Make changes and commit on branch
echo_test "Making changes on branch"
echo "## API Design Decision" >> domains/test-project/decisions.md
$GNU_BIN commit "Added API design decision" --author agent
echo_pass "Branch commit successful"

# Test 9: Diff between branches
echo_test "Comparing branches"
$GNU_BIN diff main feature-api
echo_pass "Diff displayed"

# Test 10: Merge branch
echo_test "Merging branch"
$GNU_BIN checkout main
$GNU_BIN merge feature-api
echo_pass "Branch merged"

# Test 11: Resolve wikilink
echo_test "Resolving wikilink"
$GNU_BIN resolve "[[spec]]"
echo_pass "Wikilink resolved"

# Test 12: Load and pin
echo_test "Loading and pinning files"
$GNU_BIN load "[[spec]]" --pin
$GNU_BIN load --list
echo_pass "Files loaded and listed"

# Test 13: Generate summary
echo_test "Generating summary"
$GNU_BIN summary
echo_pass "Summary generated"

# Test 14: Context export
echo_test "Exporting context"
$GNU_BIN context > /tmp/context-export.txt
echo_pass "Context exported"

# Test 15: Rewind
echo_test "Rewinding to previous commit"
SECOND_COMMIT=$($GNU_BIN log --oneline | head -2 | tail -1 | awk '{print $1}')
$GNU_BIN rewind $SECOND_COMMIT
echo_pass "Rewind successful"

# Cleanup
cd /
rm -rf "$TEST_DIR"

echo ""
echo -e "${GREEN}All tests passed!${NC}"
echo "gitnu is working correctly."

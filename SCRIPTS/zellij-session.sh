#!/bin/bash
# Launch zellij dev session
# Usage: ./SCRIPTS/zellij-session.sh [toofoo|full|custom]

LAYOUT="${1:-toofoo}"
SESSION="s3m2p-dev"

# Check if session exists
if zellij list-sessions 2>/dev/null | grep -q "$SESSION"; then
    echo "Session '$SESSION' already exists."
    echo "Attach with: zellij attach $SESSION"
    exit 0
fi

case $LAYOUT in
    toofoo)
        echo "🚀 Starting too.foo dev session..."
        echo "Tab 1: Run 'run toofoo' to start server"
        echo "Tab 2: Editor ready"
        echo "Tab 3: Run tests"
        echo ""
        zellij attach "$SESSION" --create --layout ~/.config/zellij/layouts/toofoo.kdl
        ;;

    full)
        echo "🚀 Starting full stack dev session..."
        echo "Tabs ready for: too.foo, Website, MCAD, Editor"
        echo ""
        zellij attach "$SESSION" --create --layout ~/.config/zellij/layouts/full.kdl
        ;;

    custom)
        echo "🚀 Starting custom dev session..."
        zellij attach "$SESSION" --create
        ;;


    *)
        echo "Usage: $0 [toofoo|full|custom]"
        echo ""
        echo "  toofoo  - too.foo server + editor + git + tests"
        echo "  full    - All servers in separate tabs"
        echo "  custom  - Blank session"
        echo ""
        echo "Quick reference:"
        echo "  Alt+n          New pane"
        echo "  Alt+h/j/k/l    Navigate panes"
        echo "  Alt+t          New tab"
        echo "  Alt+1/2/3      Switch tabs"
        echo "  Alt+d          Detach (keeps running)"
        echo "  Alt+q          Quit session"
        exit 1
        ;;
esac

#!/bin/bash
# Create tmux dev session with pre-configured layout
# Usage: ./SCRIPTS/dev-session.sh [toofoo|full|custom]

SESSION="s3m2p-dev"
MODE="${1:-toofoo}"

# Check if session exists
tmux has-session -t $SESSION 2>/dev/null

if [ $? == 0 ]; then
    echo "Session '$SESSION' already exists."
    echo "Attach with: tmux attach -t $SESSION"
    echo "Or kill it with: tmux kill-session -t $SESSION"
    exit 1
fi

case $MODE in
    toofoo)
        echo "Creating session: too.foo development"
        tmux new-session -d -s $SESSION -n main

        # Window 1: too.foo server
        tmux send-keys -t $SESSION:main "cd ~/S3M2P" C-m
        tmux send-keys -t $SESSION:main "run toofoo" C-m

        # Window 2: editor/commands
        tmux new-window -t $SESSION -n editor
        tmux send-keys -t $SESSION:editor "cd ~/S3M2P" C-m

        # Window 3: git/testing
        tmux new-window -t $SESSION -n git
        tmux send-keys -t $SESSION:git "cd ~/S3M2P" C-m
        ;;

    full)
        echo "Creating session: Full stack development"
        tmux new-session -d -s $SESSION -n toofoo

        # Window 1: too.foo
        tmux send-keys -t $SESSION:toofoo "cd ~/S3M2P && run toofoo" C-m

        # Window 2: website
        tmux new-window -t $SESSION -n website
        tmux send-keys -t $SESSION:website "run website" C-m

        # Window 3: MCAD
        tmux new-window -t $SESSION -n mcad
        tmux send-keys -t $SESSION:mcad "cd ~/S3M2P" C-m

        # Window 4: editor
        tmux new-window -t $SESSION -n editor
        tmux send-keys -t $SESSION:editor "cd ~/S3M2P" C-m
        ;;

    custom)
        echo "Creating session: Custom layout"
        tmux new-session -d -s $SESSION -n main
        tmux send-keys -t $SESSION:main "cd ~/S3M2P" C-m

        # Split into 3 panes
        tmux split-window -h -t $SESSION:main
        tmux split-window -v -t $SESSION:main

        # Pane 0 (left): server
        tmux send-keys -t $SESSION:main.0 "run list" C-m

        # Pane 1 (top-right): editor
        tmux send-keys -t $SESSION:main.1 "cd ~/S3M2P" C-m

        # Pane 2 (bottom-right): git
        tmux send-keys -t $SESSION:main.2 "cd ~/S3M2P && git status" C-m
        ;;

    *)
        echo "Usage: $0 [toofoo|full|custom]"
        echo ""
        echo "  toofoo  - Single server + editor + git"
        echo "  full    - All servers in separate windows"
        echo "  custom  - 3-pane layout"
        exit 1
        ;;
esac

echo ""
echo "✓ Session created: $SESSION"
echo ""
echo "Attach with: tmux attach -t $SESSION"
echo "Or just: tmux a"
echo ""
echo "Quick reference:"
echo "  Ctrl+a |    Split vertical"
echo "  Ctrl+a -    Split horizontal"
echo "  Ctrl+a h/j/k/l  Navigate panes"
echo "  Ctrl+a c    New window"
echo "  Ctrl+a d    Detach (servers keep running)"

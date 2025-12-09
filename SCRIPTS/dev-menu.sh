#!/bin/bash
# Interactive developer menu for project selection and actions

# Ensure we're in the project root
cd "$(dirname "$0")/.."

echo "========================================"
echo "   🚀 too.foo Developer Menu 🚀"
echo "========================================"
echo ""

PS3="👉 Select a project: "
projects=("welcome" "helios" "mcad" "ecad" "simulations" "blog" "learn" "website" "Quit")

select project in "${projects[@]}"; do
    case $project in
        "Quit")
            echo "Bye! 👋"
            exit 0
            ;;
        *)
            if [[ -n "$project" ]]; then
                echo ""
                echo "✅ Selected: $project"
                echo ""
                break
            else
                echo "Invalid selection. Try again."
            fi
            ;;
    esac
done

PS3="👉 Select action for $project: "
actions=("Start Dev Server" "Run Tests (Package)" "Run All Tests (Workspace)" "Quit")

select action in "${actions[@]}"; do
    case $action in
        "Start Dev Server")
            echo ""
            echo "🚀 Launching dev server..."
            ./SCRIPTS/run.sh "$project"
            break
            ;;
        "Run Tests (Package)")
            echo ""
            echo "🧪 Running tests for package '$project'..."
            # Handle website special case if needed, but standard cargo -p usually works if in workspace
            cargo test -p "$project"
            break
            ;;
        "Run All Tests (Workspace)")
            echo ""
            echo "🧪 Running full workspace tests..."
            cargo test --workspace
            break
            ;;
        "Quit")
            echo "Bye! 👋"
            exit 0
            ;;
        *)
            echo "Invalid selection."
            ;;
    esac
done

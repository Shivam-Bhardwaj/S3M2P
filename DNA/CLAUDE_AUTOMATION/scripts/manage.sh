#!/bin/bash
# Claude Automation Daemon Management Script

SERVICE="claude-automation.service"
LOG="/home/curious/.claude/automation-daemon.log"
BASE_URL="http://localhost:4242"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

case "$1" in
    start)
        echo -e "${BLUE}Starting Claude Automation Daemon...${NC}"
        systemctl --user start $SERVICE
        sleep 2
        systemctl --user status $SERVICE --no-pager
        ;;
    stop)
        echo -e "${YELLOW}Stopping Claude Automation Daemon...${NC}"
        systemctl --user stop $SERVICE
        ;;
    restart)
        echo -e "${BLUE}Restarting Claude Automation Daemon...${NC}"
        systemctl --user restart $SERVICE
        sleep 2
        systemctl --user status $SERVICE --no-pager
        ;;
    status|s)
        echo -e "${BLUE}═══════════════════════════════════════════${NC}"
        echo -e "${BLUE}     Claude Automation Status${NC}"
        echo -e "${BLUE}═══════════════════════════════════════════${NC}"
        echo ""

        # Health check
        HEALTH=$(curl -s $BASE_URL/health 2>/dev/null)
        if [ "$HEALTH" = "OK" ]; then
            echo -e "🟢 ${GREEN}Daemon: Running${NC}"
        else
            echo -e "🔴 ${RED}Daemon: Not responding${NC}"
            exit 1
        fi

        # Get API status
        STATUS=$(curl -s $BASE_URL/status 2>/dev/null)
        if [ -n "$STATUS" ]; then
            ACTIVE=$(echo "$STATUS" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('active_sessions',[])))" 2>/dev/null)
            echo -e "📊 Active sessions: ${YELLOW}${ACTIVE:-0}${NC}"

            if [ "$ACTIVE" != "0" ] && [ -n "$ACTIVE" ]; then
                echo ""
                echo "$STATUS" | python3 -c "
import sys, json
d = json.load(sys.stdin)
for s in d.get('active_sessions', []):
    alive = '🟢' if s.get('alive') else '🔴'
    print(f\"   {alive} Issue #{s['issue']}: PID {s['pid']}\")
" 2>/dev/null
            fi
        fi

        # Show DB state
        echo ""
        echo -e "${BLUE}📋 Automation Records:${NC}"
        python3 -c "
import sqlite3
conn = sqlite3.connect('/home/curious/.claude/automation.db')
for row in conn.execute('SELECT issue_number, status, pid, has_plan FROM automations ORDER BY issue_number'):
    status_icon = {'running': '🔄', 'waiting_for_user': '⏳', 'completed': '✅', 'triggered': '🚀'}.get(row[1], '❓')
    pid_str = f'PID {row[2]}' if row[2] else 'no PID'
    plan_str = '📋' if row[3] else ''
    print(f'   {status_icon} Issue #{row[0]}: {row[1]} ({pid_str}) {plan_str}')
conn.close()
" 2>/dev/null

        # Recent log activity
        echo ""
        echo -e "${BLUE}📝 Recent Activity:${NC}"
        tail -5 $LOG 2>/dev/null | grep -E "INFO|WARN|ERROR" | sed 's/^/   /'
        ;;
    poll|p)
        echo -e "${BLUE}🔄 Forcing poll for issues needing action...${NC}"
        RESULT=$(curl -s -X POST $BASE_URL/poll 2>/dev/null)
        if [ -n "$RESULT" ]; then
            echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
        else
            echo -e "${RED}Failed to contact daemon${NC}"
        fi
        ;;
    health|h)
        HEALTH=$(curl -s $BASE_URL/health 2>/dev/null)
        if [ "$HEALTH" = "OK" ]; then
            echo -e "🟢 ${GREEN}OK${NC}"
        else
            echo -e "🔴 ${RED}FAIL - Service not responding${NC}"
            exit 1
        fi
        ;;
    logs|l)
        echo -e "${BLUE}Showing last 30 lines (Ctrl+C to exit live mode)...${NC}"
        tail -30 $LOG
        echo ""
        echo -e "${YELLOW}--- Live logs ---${NC}"
        tail -f $LOG
        ;;
    logs-recent|lr)
        tail -100 $LOG
        ;;
    clear-stale|cs)
        echo -e "${YELLOW}Clearing stale PIDs from database...${NC}"
        python3 -c "
import sqlite3
conn = sqlite3.connect('/home/curious/.claude/automation.db')
cursor = conn.cursor()
cursor.execute('UPDATE automations SET pid = NULL, status = \"waiting_for_user\" WHERE pid IS NOT NULL')
conn.commit()
print(f'Cleared {cursor.rowcount} stale PIDs')
conn.close()
"
        ;;
    sessions|ss)
        echo -e "${BLUE}Automation sessions:${NC}"
        python3 -c "
import sqlite3
conn = sqlite3.connect('/home/curious/.claude/automation.db')
for row in conn.execute('SELECT issue_number, status, pid, has_plan FROM automations ORDER BY issue_number'):
    print(f'  Issue #{row[0]}: status={row[1]}, pid={row[2]}, has_plan={row[3]}')
conn.close()
"
        ;;
    *)
        echo -e "${BLUE}Claude Automation Daemon Management${NC}"
        echo ""
        echo "Usage: $0 {command}"
        echo ""
        echo -e "${GREEN}Commands:${NC}"
        echo "  status, s     - Show full status dashboard"
        echo "  poll, p       - Force immediate poll for issues"
        echo "  health, h     - Quick health check"
        echo "  logs, l       - Show logs (live tail)"
        echo "  logs-recent   - Show last 100 log lines"
        echo "  sessions, ss  - Show all automation sessions"
        echo "  start         - Start the daemon"
        echo "  stop          - Stop the daemon"
        echo "  restart       - Restart the daemon"
        echo "  clear-stale   - Clear stale PIDs"
        ;;
esac

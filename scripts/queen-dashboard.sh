#!/bin/bash

# Real-time Queen Orchestration Dashboard

watch -n 5 -c '
clear
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║       SAFLA QUEEN ORCHESTRATION - LIVE DASHBOARD                 ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "👑 QUEEN STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
claude-flow queen status --format compact
echo ""
echo "🧠 SAFLA NEURAL MEMORY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
claude-flow memory stats --metrics "utilization,patterns,learning_rate"
echo ""
echo "👥 WORKER SWARM STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
claude-flow swarm status --workers --progress-bars
echo ""
echo "🔍 RECENT FINDINGS (Last 5 minutes)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
tail -n 10 ./logs/worker-progress.log | grep -E "CRITICAL|HIGH|PATTERN"
echo ""
echo "⚡ QUEEN ADAPTIVE ACTIONS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
tail -n 5 ./logs/queen-orchestration.log
echo ""
echo "Press Ctrl+C to exit dashboard"
'
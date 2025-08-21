# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a Container-Use (cu) demonstration repository that showcases how AI coding agents can work in isolated container environments. The project contains demo applications, documentation, and scripts for testing the Container-Use MCP server.

## Common Development Commands

### Container-Use (cu) Commands
```bash
# List all active environments
cu list

# Create and work in a new environment
cu open --source . --name <environment-name>

# Monitor environment activity in real-time
cu watch

# Check logs for specific environment
cu log <environment-name>

# Access container terminal
cu terminal <environment-name>

# Merge changes from environment branch
cu merge <environment-name>

# Clean up environment when done
cu delete <environment-name>
```

### Running Demo Applications
```bash
# Flask application (runs on port 5000)
python3 test-hello-world.py

# FastAPI application (runs on port 8000)
python3 test-fastapi-app.py

# Install dependencies for Flask apps
pip install -r requirements.txt

# Install dependencies for FastAPI apps
pip install -r requirements-fastapi.txt
```

### Demo Scripts
```bash
# Run the main Container-Use demo
./demo-container-use.sh

# Test parallel execution capabilities
./parallel-test.sh

# Verify container isolation
./test-container-isolation.sh
```

### Monitoring with tmux
```bash
# Start tmux session with cu watch
tmux new-session -d -s cu-monitor 'cu watch'

# View the monitoring session
tmux attach -t cu-monitor

# Kill monitoring session
tmux kill-session -t cu-monitor
```

## Architecture and Key Concepts

### Container-Use Integration
The repository demonstrates how Container-Use provides isolated environments for AI agents:
- Each environment runs in its own Docker container
- Changes are tracked in separate Git branches (`container-use/<env-name>`)
- Multiple agents can work in parallel without conflicts
- All actions are auditable through Git history and cu logs

### Project Structure
- **Documentation/**: Guides and documentation about Container-Use (primarily in Japanese)
- **test-*.py/js/sh**: Various test applications demonstrating different aspects
- **demo-*.sh**: Shell scripts that automate common demonstration scenarios
- **requirements*.txt**: Python dependencies for different demo applications

### Key Integration Points
1. **MCP Server**: Container-Use acts as an MCP (Model Context Protocol) server
2. **Git Branching**: Each container environment gets its own Git branch
3. **Port Management**: Applications in containers can expose ports (5000, 8000, etc.)
4. **File Isolation**: Each container has its own filesystem, preventing conflicts

### Testing Workflow
When testing changes in this repository:
1. Create a new environment with `cu open`
2. Make changes and test within the container
3. Use `cu watch` to monitor activity
4. Merge successful changes with `cu merge`
5. Clean up with `cu delete`

## Important Notes

- The repository uses Colima for Docker on macOS (check with `colima status`)
- Container-Use requires the MCP server to be running (check `.claude/settings.local.json`)
- All demo scripts should be run from the repository root
- Python applications may need their dependencies installed within each container environment
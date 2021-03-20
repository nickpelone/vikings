#!/bin/bash
START="/path/to/Valheim dedicated server/start_server.sh"
CHANNEL="DISCORD_CHANNEL_ID_HERE"
KEY="DISCORD_BOT_KEY_HERE"

env VALHEIM_START_SCRIPT="$START" CHANNEL_ID="$CHANNEL" DISCORD_KEY="$KEY" path/to/valheim-watcher

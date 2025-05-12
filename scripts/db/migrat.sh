#!/bin/sh
echo "$(pwd)"

# sea-orm-cli migrate generate --help
sea-orm-cli migrate generate -d ./landscape-database/migration $1
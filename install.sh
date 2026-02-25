#!/bin/bash
set -e
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/takumi3488/gqlforge/releases/latest/download/gqlforge-installer.sh | sh

#!/bin/bash

export DEBIAN_FRONTEND=noninteractive

echo "Installing common tools..."
apt-get update
apt-get install -y \
    build-essential \
    curl \
    git


echo "Installing mise..."
curl https://mise.run | sh

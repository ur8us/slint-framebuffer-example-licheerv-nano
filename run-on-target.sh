#!/bin/bash
TARGET_IP=10.221.160.1
TARGET_USER=root
TARGET_PATH=/opt/slint-framebuffer-example

# Copy your public key to the host with a ssh-copy-id command, or you will have to enter password manually each time you connect

# Delete old file

ssh ${TARGET_USER}@${TARGET_IP} rm ${TARGET_PATH}

# Copy the binary to target
scp $1  ${TARGET_USER}@${TARGET_IP}:${TARGET_PATH}

# Run it remotely
ssh ${TARGET_USER}@${TARGET_IP} ${TARGET_PATH}

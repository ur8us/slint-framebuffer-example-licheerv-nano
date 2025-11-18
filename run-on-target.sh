#!/bin/bash
# TARGET_IP=10.221.160.1
TARGET_IP=10.100.85.1
TARGET_USER=root
TARGET_PATH=/opt/$(basename "$1")

# Copy your public key to the host with a ssh-copy-id command, or you will have to enter password manually each time you connect
#
# ssh-keygen -t ed25519 -C "zalizyaka@host"
# ssh-copy-id root@10.100.85.1
# ( keys are in ./ssh )

# Delete old file

ssh ${TARGET_USER}@${TARGET_IP} rm ${TARGET_PATH}

# Copy the binary to target
scp $1  ${TARGET_USER}@${TARGET_IP}:${TARGET_PATH}

# Run it remotely
ssh ${TARGET_USER}@${TARGET_IP} "killall $(basename "$1") ; ${TARGET_PATH}"

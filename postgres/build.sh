#!/bin/bash

# Set variables
CONTAINER_NAME="postgres_builder"
IMAGE_NAME="postgres-builder:latest"

# Check if the container is already running
if [ "$(docker ps -q -f name=$CONTAINER_NAME)" ]; then
    echo "Stopping and removing existing container $CONTAINER_NAME..."
    #docker stop $CONTAINER_NAME
    docker rm $CONTAINER_NAME --force
fi

# Build the preparation Docker image
docker build -t $IMAGE_NAME .

# Run the container with Docker socket
docker run --name $CONTAINER_NAME --rm -v /var/run/docker.sock:/var/run/docker.sock $IMAGE_NAME

echo "Docker image built successfully."
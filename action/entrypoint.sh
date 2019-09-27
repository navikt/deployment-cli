#!/bin/sh -l

SHORT_SHA=$(echo "$GITHUB_SHA" | cut -c -6)
echo "{}" > .empty


if [ "$INPUT_REF" = "GITHUB_SHA" ]
then
    export INPUT_REF=$SHORT_SHA
fi

if [ "$INPUT_IMAGE" = "FROM_FILE" ] && [ -f ".docker_image" ]
then
    export INPUT_IMAGE=$(cat .docker_image)
fi
if [ "$INPUT_TAG" = "FROM_FILE" ] && [ -f ".docker_tag" ]
then
    export INPUT_TAG=$(cat .docker_tag)
fi
if [ -z "$INPUT_VARS" ]
then
    export INPUT_VARS=".empty"
fi

if [ -z "$INPUT_RESOURCES" ] && [ -z "$INPUT_RAWRESOURCES" ]
then
    deployment-cli deploy create \
        --cluster="$INPUT_CLUSTER" \
        --team="$INPUT_TEAM" \
        --resource="nais.yaml" \
        --repository="$GITHUB_REPOSITORY" \
        --token="$GITHUB_TOKEN" \
        --var image="$INPUT_IMAGE" \
        --var tag="$INPUT_TAG" \
        --ref="$INPUT_REF" \
        --vars="$INPUT_VARS"
elif [ -z "$INPUT_RESOURCES" ]
then
    deployment-cli deploy create \
        --cluster="$INPUT_CLUSTER" \
        --team="$INPUT_TEAM" \
        --raw-resource="$INPUT_RAWRESOURCES"
        --repository="$GITHUB_REPOSITORY" \
        --token="$GITHUB_TOKEN" \
        --var image="$INPUT_IMAGE" \
        --var tag="$INPUT_TAG" \
        --ref="$INPUT_REF" \
        --vars="$INPUT_VARS"
elif [ -z "$INPUT_RAWRESOURCES" ]
then
    deployment-cli deploy create \
        --cluster="$INPUT_CLUSTER" \
        --team="$INPUT_TEAM" \
        --resource="$INPUT_RESOURCES" \
        --repository="$GITHUB_REPOSITORY" \
        --token="$GITHUB_TOKEN" \
        --var image="$INPUT_IMAGE" \
        --var tag="$INPUT_TAG" \
        --ref="$INPUT_REF" \
        --vars="$INPUT_VARS"
else
    deployment-cli deploy create \
        --cluster="$INPUT_CLUSTER" \
        --team="$INPUT_TEAM" \
        --resource="$INPUT_RESOURCES"
        --raw-resource="$INPUT_RAWRESOURCES"
        --repository="$GITHUB_REPOSITORY" \
        --token="$GITHUB_TOKEN" \
        --var image="$INPUT_IMAGE" \
        --var tag="$INPUT_TAG" \
        --ref="$INPUT_REF" \
        --vars="$INPUT_VARS"
fi

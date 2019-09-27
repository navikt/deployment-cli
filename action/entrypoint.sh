#!/bin/sh -l

SHORT_SHA=$(echo "$GITHUB_SHA" | cut -c -6)

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
    echo "{}" > .empty
    export INPUT_VARS=".empty"
fi
if [ -z "$INPUT_RAWRESOURCES" ] && [ -z "$INPUT_RESOURCES" ]
then
    echo "Defaulting to --resource=nais.yaml"
    deployment-cli deploy create \
	      --cluster="$INPUT_CLUSTER" \
	      --team="$INPUT_TEAM" \
	      --resource="nais.yaml" \ # default to nais.yaml if neither resource fields are set
	      --repository="$GITHUB_REPOSITORY" \
	      --token="$GITHUB_TOKEN" \
	      --var image="$INPUT_IMAGE" \
	      --var tag="$INPUT_TAG" \
	      --ref="$INPUT_REF" \
	      --vars="$INPUT_VARS"
elif [ -z "$INPUT_RAWRESOURCES" ] # use resources if rawresources is not set
then
    echo "Using provided resource $INPUT_RESOURCES"
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
elif [ -z "$INPUT_RESOURCES" ] # use rawresources if resources is not set
then
    echo "Using provided raw resources $INPUT_RAWRESOURCES"
    deployment-cli deploy create \
        --cluster="$INPUT_CLUSTER" \
        --team="$INPUT_TEAM" \
        --raw-resource="$INPUT_RAWRESOURCES" \
        --repository="$GITHUB_REPOSITORY" \
        --token="$GITHUB_TOKEN" \
        --var image="$INPUT_IMAGE" \
        --var tag="$INPUT_TAG" \
        --ref="$INPUT_REF" \
        --vars="$INPUT_VARS"
else                           # use both fields if they are set
    echo "Using both raw resources $INPUT_RAWRESOUCES and resources $INPUT_RESOURCES" 
    deployment-cli deploy create \
        --cluster="$INPUT_CLUSTER" \
        --team="$INPUT_TEAM" \
        --resource="$INPUT_RESOURCES"
        --raw-resource="$INPUT_RAWRESOURCES" \
        --repository="$GITHUB_REPOSITORY" \
        --token="$GITHUB_TOKEN" \
        --var image="$INPUT_IMAGE" \
        --var tag="$INPUT_TAG" \
        --ref="$INPUT_REF" \
        --vars="$INPUT_VARS"
fi

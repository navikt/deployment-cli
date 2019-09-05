#!/bin/sh -l

SHORT_SHA=$(echo "$GITHUB_SHA" | cut -c -6)

if [ "$INPUT_REF" = "GITHUB_SHA" ]
then
    export INPUT_REF=$SHORT_SHA
fi

if [ "$INPUT_IMAGE" = "FROM_FILE" ]
then
    export INPUT_IMAGE=$(cat .docker_image)
fi
if [ "$INPUT_TAG" = "FROM_FILE" ]
then
    export INPUT_TAG=$(cat .docker_tag)
fi

deployment-cli deploy create \
	       --cluster="$INPUT_CLUSTER" \
	       --team="$INPUT_TEAM" \
	       --resource="$INPUT_RESOURCE" \
	       --repository="$GITHUB_REPOSITORY" \
	       --token="$GITHUB_TOKEN" \
	       --var image="$INPUT_IMAGE" \
	       --var tag="$INPUT_TAG" \
	       --ref="$INPUT_REF"

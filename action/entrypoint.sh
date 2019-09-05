#!/bin/sh -l

SHORT_SHA=$(echo "$GITHUB_REF" | cut -c -6)
if [ "$INPUT_REF" = "GITHUB_SHA" ]
then
    export INPUT_REF=$SHORT_SHA
fi
if [ "$INPUT_TAG" = "GITHUB_SHA" ]
then
    export INPUT_TAG=$SHORT_SHA
fi

/deployment-cli deploy create \
		--cluster="$INPUT_CLUSTER" \
		--team="$INPUT_TEAM" \
		--repository="$INPUT_REPOSITORY" \
		--token="$GITHUB_TOKEN" \
		--var tag="$INPUT_TAG" \
		--ref="$INPUT_REF"
#		--resource="$GITHUB_WORKSPACE"/"$INPUT_RESOURCE" \
#		--vars="$GITHUB_WORKSPACE"/"$INPUT_VARS" \

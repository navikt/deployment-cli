#!/bin/sh -l
# shellcheck disable=SC2153

CMD="deployment-cli deploy create --cluster=$INPUT_CLUSTER --team=$INPUT_TEAM --repository=$GITHUB_REPOSITORY --token=$GITHUB_TOKEN"

if [ "$INPUT_REF" = "GITHUB_SHA" ]
then
    INPUT_REF="$(echo "$GITHUB_SHA" | cut -c -6)"
    export INPUT_REF
fi
CMD="$CMD --ref=$INPUT_REF"

if [ "$INPUT_IMAGE" = "FROM_FILE" ] && [ -f ".docker_image" ]
then
    INPUT_IMAGE="$(cat .docker_image)"
    export INPUT_IMAGE
fi
CMD="$CMD --var image=$INPUT_IMAGE"

if [ "$INPUT_TAG" = "FROM_FILE" ] && [ -f ".docker_tag" ]
then
    INPUT_TAG="$(cat .docker_tag)"
    export INPUT_TAG
fi
CMD="$CMD --var tag=$INPUT_TAG"

if [ -z "$INPUT_VARS" ]
then
    echo "{}" > .empty
    export INPUT_VARS=".empty"
fi
CMD="$CMD --vars=$INPUT_VARS"

if [ -n "$INPUT_RESOURCES" ]
then
    CMD="$CMD --resource=$(echo "$INPUT_RESOURCES" | sed -e 's/,/ --resource /g')"
fi

if [ -n "$INPUT_RAWRESOURCES" ]
then
    CMD="$CMD --raw-resource=$(echo "$INPUT_RAWRESOURCES" | sed -e 's/,/ --raw-resource /g')"
fi

eval "$CMD"

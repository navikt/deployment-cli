Github Action
=============

> Github Action used for performing NAIS deployments.

Take a look at [action.yml](action.yml) for a complete list of inputs and descriptions.

```
name: NAIS deploy
on: push
jobs:
  deploy:
    name: NAIS deployment
	runs-on: ubuntu-latest
	steps:
      - name: deploy
        uses: navikt/deployment-cli/action@v0.x.0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          cluster: <cluster>
          team: <team-name>
```


## Example workflow

The following example will build and push your Docker image to Github Package Registry, and then deploy that image to `dev-fss`.

This example also expects that your `spec.image` in your `nais.yaml` is set to `{{ image }}:{{ tag }}`. See [example nais.yaml](#example-nais.yaml) belowe if you're insure.

```
name: Deploy to NAIS
on: push
jobs:
  deploy-to-dev:
    name: Build, push and deploy to dev-fss
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v1
      - name: Build code
        run: <put in your build-script here> # or remove if it's done in your Dockerfile
      - name: Create Docker tag
        env:
          NAME: <app-name>
        run: |
          echo "docker.pkg.github.com"/"$GITHUB_REPOSITORY"/"$NAME" > .docker_image
          echo "$(date "+%Y.%m.%d")-$(git rev-parse --short HEAD)" > .docker_tag
      - name: Build Docker image
        run: |
          docker build -t $(cat .docker_image):$(cat .docker_tag) .
      - name: Login to Github Package Registry
        env:
          DOCKER_USERNAME: x-access-token
          DOCKER_PASSWORD: ${{ secrets.GITHUB_ACCESS_TOKEN }}
        run: |
          echo "$DOCKER_PASSWORD" | docker login --username "$DOCKER_USERNAME" --password-stdin docker.pkg.github.com
      - name: Push Docker image
        run: "docker push $(cat .docker_image):$(cat .docker_tag)"
      - name: deploy to dev-fss
        uses: navikt/deployment-cli/action@b60ef91
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          cluster: dev-fss
          team: <team-name>
          resource: nais/dev-fss.yaml
```

Remember to specify `app-name` and `team-name`!

### Example nais.yaml

```
apiVersion: "nais.io/v1alpha1"
kind: "Application"
metadata:
  name: <app-name>
  namespace: default
  labels:
    team: <team-name>
spec:
  image: {{ image }}:{{ tag }}
```

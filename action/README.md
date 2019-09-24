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

## Examples and documentation

Examples and more documentation can be found in [doc.nais.io/deploy](https://doc.nais.io/deploy#using-github-actions).

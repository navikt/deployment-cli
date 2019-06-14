# deployment-cli
A command line utility for deploying applications to the Nais platform. It is a companion tool for https://github.com/navikt/deployment 

The use case is in the deploy step of a build & deploy pipeline implemented in an as-a-service product such as Circle CI og Travis CI. It is supposed to be a human-friendly and standardized alternative to do-it-yourself bash scripting with the curl command etc.

Application program code is assumed to reside in Github. Deployment to the Nais Kubernetes clusters is done by creating Github Deployments via the Github API. The build & deploy pipeline process will typically authenticate itself to the Github API by use of a [Github App](https://lab.github.com/githubtraining/getting-started-with-github-apps). This CLI helps with that by hiding some stuff from the user.

This CLI provides templating of Kubernetes resources, i.e. it lets you configure the configuration file 'nais.yaml'. Currently it only supports Kubernetes resource files in the YAML format and properties/config/variable files in the JSON format. This will be improved on in the future.

## General usage
To see a list of usable commands you can start with using `deployment-cli --help`.

The `--help` flag is available for all subcommands.

To see the help info for a subcommand of a subcommand, type either one of
* `deployment-cli --help <subcommand> <subsubcommand>` e.g. `deployment-cli --help deploy create`
* `deployment-cli <subcommand> <subsubcommand> --help` e.g. `deployment-cli deploy create --help`

## Deployments
`deployment-cli deploy` contains a few subcommands for easily creating Github Deployments using the Github API:

### Creating a deployment
Creating a deployment is done using the `deployment-cli deploy create` or  command. It will also do templating using
handlebars ib the resource files specified with `-r/--resource`. The values used for templating can be specified using
the config file specified by `-v/--vars`. `deployment-cli` will also inject a few values regardless of specifying a
config file:
* ref: Git reference, i.e. branch name, tag name or commit SHA. Specified by the `--ref` flag. Default: `master`
* cluster: which Nais Kubernetes cluster the deploy is for. Specified by the `--cluster/-c` flag. Default: `dev-fss`
* team: the Github team this deployment is for. Specified by the `--team/t` flag.
* version: the version, this is strictly for templating and should be used to specify which version of the Docker file
should be pulled. Specified using the `--version` flag


The most basic deployment should look something like:
                                                                                   
`deployment-cli deploy create --cluster=dev-fss --repository=navikt/deployment --team=<team> --version=1.0.0 --appid=1234 --key=/path/to/private-key.pem --resource=nais.yaml --vars=placeholders.json`
                                                                                   
Note: For deployments using [Github Apps](https://lab.github.com/githubtraining/getting-started-with-github-apps) the private key has to be [PEM encoded](https://support.quovadisglobal.com/kb/a37/what-is-pem-format.aspx). Use the flag `--key-base64` instead of `--key`.

### Dumping the payload to stdout
If you want to just dump the payload to stdout you can do deployment-cli deploy payload with the same flags used in
creating a deploy except the ones used for authentication.

## Token
This application also supports dumping Github App installation tokens to stdout which can be useful for non-open
repository cloning or doing the deployment in two stages with `--username x-access-token --password <token>`

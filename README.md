# deployment-cli
Simple utility for templating kubernetes resources, authenticating as a github app and creating a deployment through
the github api. Currently it only supports yaml resource files and json config files. This will be improved on in the
future. Its a companion tool for https://github.com/navikt/deployment

## General usage
To see a list of usable commands you can start with using:

`deployment-cli --help`

The `--help` flag is available for all subcommands.

To see the help info for a subcommand of a subcommand, type `deployment-cli --help <subcommand> <subsubcommand>` e.g. `deployment-cli --help deploy create`

## Deployments
`deployment-cli create` contains a few subcommands for easily creating deployments using the github api

### Creating a deployment
Creating a deployment is done using the `deployment-cli deploy create` or  command. It will also do templating using
handlebars ib the resource files specified with `-r/--resource`. The values used for templating can be specified using
the config file specified by `-v/--vars`. `deployment-cli` will also inject a few values regardless of specifying a
config file:
* ref: git reference. Specified by the `--ref` flag. Default: `master`
* cluster: which cluster the deploy is for. Specified by the `--cluster/-c` flag. Default: `dev-fss`
* team: the team this deployment is for. Specified by the `--team/t` flag.
* version: the version, this is strictly for templating and should be used to specify which version of the Docker file
should be pulled. Specified using the `--version` flag


The most basic deployment should look something like:
                                                                                   
`deployment-cli create --cluster=dev-fss --repository=navikt/deployment --team=<team> --version=1.0.0 --appid=1234 --key=/path/to/private-key.pem --resource=nais.yaml --vars=placeholders.json`
                                                                                   
Note: For deployments using github apps the private key has to be pem encoded.

### Dumping the payload to stdout
If you want to just dump the payload to stdout you can do deployment-cli deploy payload with the same flags used in
creating a deploy except the ones used for authentication.

## Token
This application also supports dumping github app installation tokens to stdout which can be useful for non-open
repository cloning or doing the deployment in two stages with `--username x-access-token --password <token>`

# deployment-cli
Simple utility for templating kubernetes resources, authenticating as a github app and creating a deployment through
the github api. Currently it only supports yaml resource files and json config files. This will be improved on in the
future. Its a companion tool for https://github.com/navikt/deployment

## General usage
To see a list of usable commands you can start with using `deployment-cli --help`.

The `--help` flag is available for all subcommands.

To see the help info for a subcommand of a subcommand, type either one of
* `deployment-cli --help <subcommand> <subsubcommand>` e.g. `deployment-cli --help deploy create`
* `deployment-cli <subcommand> <subsubcommand> --help` e.g. `deployment-cli deploy create --help`

## Deployments
`deployment-cli deploy` contains a few subcommands for easily creating deployments using the github api

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
                                                                                   
`deployment-cli deploy create --cluster=dev-fss --repository=navikt/deployment --team=<team> --version=1.0.0 --appid=1234 --key=/path/to/private-key.pem --resource=nais.yaml --vars=placeholders.json`
                                                                                   
Note: For deployments using [Github Apps](https://lab.github.com/githubtraining/getting-started-with-github-apps) the private key has to be [PEM encoded](https://support.quovadisglobal.com/kb/a37/what-is-pem-format.aspx). Use the flag `--key-base64` instead of `--key`.

### Dumping the payload to stdout
If you want to just dump the payload to stdout you can do deployment-cli deploy payload with the same flags used in
creating a deploy except the ones used for authentication.

## Token
This application also supports dumping github app installation tokens to stdout which can be useful for non-open
repository cloning or doing the deployment in two stages with `--username x-access-token --password <token>`

## For MS Windows users
If you use this script in the form of the precompiled, binary file: To run the file in Linux - e.g. on Circle CI og Travis CI - it must be executable. If your local developer machine with your local Git repository runs Linux you can make it executable by running the command `chmod +x deployment-cli-v*-x86_64-unknown-linux-musl` before the usual `git commit`. This is not possible on Windows, not even in WSL (Windows Subsystem for Linux) per May 2019. A solution is to run the following command on Windows inside Git Bash or WSL: `git update-index --chmod=+x deployment-cli-v*-x86_64-unknown-linux-musl` and commit.

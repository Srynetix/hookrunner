# hookrunner - Execute actions on Git hosting webhooks

## What?

Simple solution to, for example, updating a static site when a commit is pushed.

It serves as a webhook server, waiting for a 'push' event from GitHub, then it will automatically clone or checkout/pull repositories.

Today, it focuses only on that feature, but in the future, multiple Git providers can be supported, and more actions will also be supported (like executing a custom command).

## Installation

If you develop in Rust, you can install the runner using `cargo install --git https://github.com/Srynetix/hookrunner` to have a bleeding-edge version.

## Configuration

You can configure the tool using an environment file (based on the [.env.dist](./.env.dist) file), or by passing command-line arguments.

### Webhook registration

To receive webhooks, you need to register a webhook URL on your GitHub project settings.  
Two ways are available:
- by hand and manually edit the project configuration,
- or if you have a personal GitHub API token, you can let **hookrunner** configure it by itself.

You can also optionally (but recommended) set a **webhook secret**.

To register a webhook through **hookrunner**, run:

```bash
hookrunner install --repository <your-repository> --url <your-url> --token <your-token>

# If you want to specify a secret, use:
# hookrunner --webhook-secret <my-secret> install --repository <your-repository> --url <your-url> --token <your-token>
```

It will scan existing webhooks, and will create a new webhook only if the **target url** is not already present in another webhook configuration.

To unregister the webhook, the command is:

```bash
hookrunner uninstall --repository <your-repository> --url <your-url> --token <your-token>
```

## Sample walkthrough

Here, we will see how you can setup **hookrunner** for a sample project.

You will need:
- **hookrunner** installed and available (of course),
- a target working directory, the location where you will clone your projects (let's say it will be named `./_work`),
- one or more GitHub target project(s), for this example we can use this one: `Srynetix/hookrunner`,
- the right webhook configuration on your project settings (refer to the [Webhook registration](#webhook-registration) section),
- the right networking configuration, you machine needs to accept incoming connections on the server port, or you can use something like **ngrok**.

Once you have everything, start the server:

```bash
hookrunner --working-dir ./_work serve

# If you have a webhook secret configured, use:
# hookrunner --working-dir ./_work --webhook-secret <my-secret> serve

# If you want to configure a specific bind ip, use:
# hookrunner --working-dir ./_work serve --bind-ip 127.0.0.1:8888
```

It will defaults to listening on ```0.0.0.0:3000```.

Then, when something will be pushed on the repository, **hookrunner** will automatically clone/checkout/pull the project on the right branch.

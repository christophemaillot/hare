
# hare 

A tool that listens to a RabbitMQ queue, and runs a external executable (like a bash script) for each messages fetched from the queue.

## configuration

"Hare" uses a set of environment variables for it configuration :

- HARE_AMQP_URL : the url of the RabbitMQ server to connect to (default value : "amqp://guest:guest@localhost:5672"),
- HARE_AMQP_QUEUE : the name of the queue to listen to,
- HARE_SCRIPT_ROOT : the root directory of the script to run,
- HARE_LOG_DESTINATION : the path of the log file, if not set, no log will be written to stdout,
- HARE_HANDLER_KEY : the name of the key to use in the message to identify the handler to run (see below),


## handler

The handler is a script that will be executed for each message fetched from the queue.

'Hare' parses the message headers to find the handler to run (and ignores the body of the message). 
The handler is identified by the value of the header named in the HARE_HANDLER_KEY environment variable.
The value of the header is expected to be a string that is the name of the script to run inside the HARE_SCRIPT_ROOT directory.
For security reasons, this value must be a alphanumeric string.

If the handler is not found, the message is ignored.

### Passing  header values to the handler

The handler script gets all the headers values as environment variables. The variables are uppercased,
and prefixed with HARE_VAR.

### for instance

if the message has the following headers :

```
type: deploy
app: myapp
env: dev
```

and HARE_HANDLER_KEY is set to "type",
and HARE_SCRIPT_ROOT is set to "/etc/hare/scripts",

the handler script is : /etc/hare/scripts/deploy  
and will be called with the following environment variables :

```
HARE_VAR_TYPE=deploy
HARE_VAR_APP=myapp
HARE_VAR_ENV=dev
```

## Project status

This project is in development, and is not ready for production use.
The project has not been audited for security vulnerabilities ; use at your own risk.

The project is my very first rust project, and I'm still learning rust.
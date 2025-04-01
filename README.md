# aws-init-container

`aws-init-container` is a single binary which will:
* Accept configuration from an environment variable
* Pull the specified secrets from AWS Secrets Manager
* Write them to nominated locations on the filesystem
* Auto-create any intermediate directories required

This is intended to run in an initialization container (running to completion before the main workload container
starts up), to prepare a shared volume with configuration files, certificates, etc.

This fills a common need when working with Amazon ECS, which is only capable of injecting runtime configuration using
environment variables, not files (until Amazon implements [#56](https://github.com/aws/containers-roadmap/issues/56)).

## Usage

Fill environment variable `CONTAINER_INIT_CONF` with YAML data, then invoke the program.

Example:

```shell
CONTAINER_INIT_CONF="
files:
  /config/db.conf:
    source_arn: arn:aws:secretsmanager:us-west-2:111111222222:secret:dbconf-gsjaUR
  /config/certificate.crt:
    source_arn: arn:aws:secretsmanager:us-west-2:111111222222:secret:certificate-pZrKNu
"

docker run --env CONTAINER_INIT_CONF="$CONTAINER_INIT_CONF" glenn/aws-init-container
```

ECS Example:
```json

```

## Configuration

### Target Files

The YAML document should contain a `files` map, keyed by the absolute path where the file should be written. Any parent
directories that don't exist will be created prior to writing the file.

For each file, you provide an object specifying the data source. Currently, only a `source_arn` parameter is supported, and
only AWS Secrets Manager ARNs are accepted. Both text and binary secrets are supported.

```yaml
files:
  /path/for/destination/file:
    source_arn: <Secrets Manager secret ARN>
```

### API Authentication

This tool uses the AWS SDK which supports the same environment variables as the AWS CLI (such as `AWS_DEFAULT_REGION`,
`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`).

When running under ECS, it will assume your ECS Task Role by default, so you probably won't need to provide any SDK
configuration.

You may optionally provide an `aws` section in your YAML document if you want to assume a different role prior to
accessing the API.

```yaml
aws:
  assume_role_arn: <role ARN>
  assume_role_external_id: <external ID> #optional
```

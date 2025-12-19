# Terraform Module for Hardware API

This folder contains a base [Terraform] module for the
[Hardware API charm][hardware-api-charm].

The module uses the [Terraform Juju provider][juju-provider] to model the charm
deployment onto any Kubernetes environment managed by [Juju].

## Module Structure

- [`main.tf`](main.tf): Defines the Juju application to be deployed.
- [`variables.tf`](variables.tf): Allows customization of the deployment.
  Also models the charm configuration.
- [`outputs.tf`](outputs.tf): Integrates the module with other Terraform modules,
  primarily by defining potential integration endpoints (charm integrations),
  but also by exposing the Juju application name
- [`versions.tf`](versions.tf): Defines the Terraform provider versions.

## Using `hardware-api`

If you want to use `hardware-api` base module as part of your Terraform module,
import it like shown below:

```hcl
data "juju_model" "my_model" {
  name = var.model
}

module "hardware_api" {
 source = "git::https://github.com/canonical/hardware-api//server/terraform/charm
 model  = juju_model.my_model.name
 # Customize configuration variables if needed
}
```

Create integrations, for instance:

```hcl
resource "juju_integration" "hardware_api_nginx" {
  model = juju_model.my_model.name
  application {
    name     = module.hardware_api.app_name
    endpoint = module.hardware_api.endpoints.nginx_route
  }
  application {
    name     = "ingress"
    endpoint = "nginx-route"
  }
}
```

The complete list of available integrations can be found
[in the Integrations tab][hardware-api-integrations].

## Requirements

| Name                  | Version   |
| --------------------- | --------- |
| [juju][juju-provider] | ~> 0.17.0 |

## Providers

| Name                  | Version   |
| --------------------- | --------- |
| [juju][juju-provider] | ~> 0.17.0 |

## Modules

No modules.

## Resources

| Name                                              | Type     |
| ------------------------------------------------- | -------- |
| [juju_application.hardware-api][juju_application] | resource |

## Inputs

| Name        | Type        | Description                                                                       | Default     |
| ----------- | ----------- | --------------------------------------------------------------------------------- | ----------- |
| app_name    | string      | Name to give the deployed application                                             | api         |
| base        | string      | The operating system on which to deploy                                           | null        |
| channel     | string      | Channel of the charm                                                              | latest/edge |
| config      | map(string) | Map for config options                                                            | {}          |
| constraints | string      | String listing constraints for this application                                   | arch=amd64  |
| model       | string      | Reference to an existing model resource or data source for the model to deploy to | null        |
| revision    | number      | Revision number of the charm                                                      | null        |
| units       | number      | Unit count/scale                                                                  | 1           |

## Outputs

| Name      | Type        | Description                      |
| --------- | ----------- | -------------------------------- |
| app_name  | string      | Name of the deployed application |
| endpoints | map(string) | Map of all endpoints             |

[terraform]: https://terraform.io
[hardware-api-charm]: https://charmhub.io/hardware-api
[juju-provider]: https://registry.terraform.io/providers/juju/juju/latest
[juju]: https://canonical.com/juju
[juju_application]: https://registry.terraform.io/providers/juju/juju/latest/docs/resources/application
[hardware-api-integrations]: https://charmhub.io/hardware-api/integrations

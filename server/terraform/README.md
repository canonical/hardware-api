# Terraform Module for Hardware API

This folder contains a base [Terraform](https://terraform.io) module for the
[Hardware API charm](https://charmhub.io/hardware-api).

The module uses the
[Terraform Juju provider](https://registry.terraform.io/providers/juju/juju/latest)
to model the charm deployment onto any Kubernetes environment managed by
[Juju](https://canonical.com/juju).

## Module Structure

- [`main.tf`](main.tf): Defines the Juju application to be deployed.
- [`variables.tf`](variables.tf): Allows customization of the deployment.
  Also models the charm configuration.
- [`outputs.tf`](outputs.tf): Integrates the module with other Terraform modules,
  primarily by defining potential integration endpoints (charm integrations),
  but also by exposing the Juju application name
- [`terraform.tf`](terraform.tf): Defines the Terraform provider versions.

## Using `hardware-api`

If you want to use `hardware-api` base module as part of your Terraform module,
import it like shown below:

```hcl
data "juju_model" "my_model" {
  name = "model"
  owner = "admin"
}

module "hardware_api" {
 source = "git::https://github.com/canonical/hardware-api//server/terraform
 model_uuid  = juju_model.my_model.uuid
 # Customize configuration variables if needed
}
```

Create integrations, for instance:

```hcl
resource "juju_integration" "hardware_api_ingress" {
  model_uuid = juju_model.my_model.uuid
  application {
    name     = module.hardware_api.application.name
    endpoint = module.hardware_api.requires.ingress
  }
  application {
    name     = "traefik-k8s"
    endpoint = "ingress"
  }
}
```

The complete list of available integrations can be found
[in the Integrations tab](https://charmhub.io/hardware-api/integrations).

<!-- BEGIN_TF_DOCS -->
## Requirements

| Name | Version |
|------|---------|
| <a name="requirement_juju"></a> [juju](#requirement\_juju) | ~> 2.0 |

## Providers

| Name | Version |
|------|---------|
| <a name="provider_juju"></a> [juju](#provider\_juju) | ~> 2.0 |

## Modules

No modules.

## Resources

| Name | Type |
|------|------|
| [juju_application.hardware_api](https://registry.terraform.io/providers/juju/juju/latest/docs/resources/application) | resource |

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| <a name="input_app_name"></a> [app\_name](#input\_app\_name) | Name to give the deployed application | `string` | `"api"` | no |
| <a name="input_base"></a> [base](#input\_base) | The operating system on which to deploy | `string` | `null` | no |
| <a name="input_channel"></a> [channel](#input\_channel) | Channel of the charm | `string` | `"latest/edge"` | no |
| <a name="input_config"></a> [config](#input\_config) | Map for config options | `map(string)` | `{}` | no |
| <a name="input_constraints"></a> [constraints](#input\_constraints) | String listing constraints for this application | `string` | `null` | no |
| <a name="input_model_uuid"></a> [model\_uuid](#input\_model\_uuid) | Reference to an existing model uuid | `string` | n/a | yes |
| <a name="input_revision"></a> [revision](#input\_revision) | Revision number of the charm | `number` | `null` | no |
| <a name="input_units"></a> [units](#input\_units) | Unit count/scale | `number` | `1` | no |

## Outputs

| Name | Description |
|------|-------------|
| <a name="output_application"></a> [application](#output\_application) | Object representing the deployed application |
| <a name="output_requires"></a> [requires](#output\_requires) | Map of requires integration endpoints |
<!-- END_TF_DOCS -->
